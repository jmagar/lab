use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{Context, Result};
use lab_apis::core::{EnvVar, PluginMeta};
use serde::Deserialize;

use super::routes::{build_route_docs, service_has_action_api_route};
use super::types::{
    DocsProjection, EnvDoc, FeatureClass, FeatureDoc, FeatureMatrix, FeatureMismatch, ServiceDoc,
    ServiceExposure, SurfaceAvailability,
};
use crate::api::openapi::build_openapi_spec;
use crate::audit::{AuditReport, CheckResult, ServiceReport, audit_services};
use crate::catalog::build_catalog;
use crate::registry::{RegisteredService, build_docs_registry};

pub fn build_docs_projection(repo_root: &Path) -> Result<DocsProjection> {
    let registry = build_docs_registry();
    let mcp_help = build_catalog(&registry);
    let services = registry.services();
    let feature_matrix = build_feature_matrix(repo_root)?;
    let service_catalog = build_service_catalog(services, &feature_matrix, repo_root);
    let env_reference = build_env_reference(&service_catalog);
    let action_catalog = super::action_catalog::build_action_catalog(services);
    let api_route_services = service_catalog
        .iter()
        .filter(|service| service.surfaces.api && service_has_action_api_route(&service.name))
        .map(|service| service.name.clone())
        .collect::<Vec<_>>();
    let api_routes = build_route_docs(&api_route_services);
    let openapi_json =
        Arc::unwrap_or_clone(build_openapi_spec(services).context("failed to build OpenAPI spec")?);
    let mut onboarding_audit = build_onboarding_audit(&service_catalog, repo_root);
    normalize_audit_paths(&mut onboarding_audit, repo_root);

    Ok(DocsProjection {
        mcp_help,
        service_catalog,
        env_reference,
        action_catalog,
        feature_matrix,
        api_routes,
        onboarding_audit,
        openapi_json,
    })
}

fn build_service_catalog(
    services: &[RegisteredService],
    feature_matrix: &FeatureMatrix,
    repo_root: &Path,
) -> Vec<ServiceDoc> {
    let mut docs = services
        .iter()
        .map(|service| service_doc(service, feature_matrix, repo_root))
        .collect::<Vec<_>>();

    for meta in sdk_only_metas() {
        if docs.iter().any(|service| service.name == meta.name) {
            continue;
        }
        docs.push(ServiceDoc {
            name: meta.name.to_string(),
            display_name: meta.display_name.to_string(),
            description: meta.description.to_string(),
            category: meta.category.as_str().to_string(),
            status: "sdk_only".to_string(),
            feature: Some(meta.name.to_string()),
            exposure: ServiceExposure::SdkOnly,
            surfaces: SurfaceAvailability::none(),
            default_port: meta.default_port,
            docs_url: non_empty(meta.docs_url),
            coverage_doc: doc_exists(repo_root, &format!("docs/coverage/{}.md", meta.name)),
            upstream_doc: doc_exists(repo_root, &format!("docs/upstream-api/{}.md", meta.name)),
            supports_multi_instance: meta.supports_multi_instance,
            metadata_source: "PluginMeta only".to_string(),
        });
    }

    docs.sort_by(|a, b| a.name.cmp(&b.name));
    docs
}

fn service_doc(
    service: &RegisteredService,
    feature_matrix: &FeatureMatrix,
    repo_root: &Path,
) -> ServiceDoc {
    let meta = meta_for(service.name);
    let feature = service_feature(service.name, feature_matrix);
    let exposure = if service.name == "lab_admin" {
        ServiceExposure::RuntimeConditional
    } else if feature.is_some() {
        ServiceExposure::FeatureGated
    } else {
        ServiceExposure::AlwaysOn
    };
    let display_name = meta.map_or_else(
        || service.name.to_string(),
        |meta| meta.display_name.to_string(),
    );
    let description = meta.map_or(service.description, |meta| meta.description);
    let category = meta.map_or(service.category, |meta| meta.category.as_str());

    ServiceDoc {
        name: service.name.to_string(),
        display_name,
        description: description.to_string(),
        category: category.to_string(),
        status: service.status.to_string(),
        feature,
        exposure,
        surfaces: service_surfaces(service.name),
        default_port: meta.and_then(|meta| meta.default_port),
        docs_url: meta.and_then(|meta| non_empty(meta.docs_url)),
        coverage_doc: doc_exists(repo_root, &format!("docs/coverage/{}.md", service.name)),
        upstream_doc: doc_exists(repo_root, &format!("docs/upstream-api/{}.md", service.name)),
        supports_multi_instance: meta.is_some_and(|meta| meta.supports_multi_instance),
        metadata_source: if meta.is_some() {
            "registry + PluginMeta".to_string()
        } else {
            "registry synthetic metadata".to_string()
        },
    }
}

fn build_env_reference(services: &[ServiceDoc]) -> Vec<EnvDoc> {
    let mut vars = Vec::new();
    for service in services {
        let Some(meta) = meta_for(&service.name) else {
            continue;
        };
        vars.extend(env_docs(
            service,
            meta.required_env,
            true,
            meta.default_port,
        ));
        vars.extend(env_docs(
            service,
            meta.optional_env,
            false,
            meta.default_port,
        ));
    }
    vars.sort_by(|a, b| {
        (a.service.as_str(), a.env_var.as_str()).cmp(&(b.service.as_str(), b.env_var.as_str()))
    });
    vars
}

fn env_docs(
    service: &ServiceDoc,
    envs: &[EnvVar],
    required: bool,
    default_port: Option<u16>,
) -> Vec<EnvDoc> {
    envs.iter()
        .map(|env| EnvDoc {
            service: service.name.clone(),
            env_var: env.name.to_string(),
            required,
            secret: env.secret,
            description: env.description.to_string(),
            example: sanitized_example(env),
            default_port,
        })
        .collect()
}

fn build_feature_matrix(repo_root: &Path) -> Result<FeatureMatrix> {
    let lab = read_manifest(&repo_root.join("crates/lab/Cargo.toml"))?;
    let apis = read_manifest(&repo_root.join("crates/lab-apis/Cargo.toml"))?;
    let lab_features = lab.features;
    let api_features = apis.features;
    let lab_all = feature_set(&lab_features, "all");
    let api_all = feature_set(&api_features, "all");
    let lab_default = feature_set(&lab_features, "default");
    let api_default = feature_set(&api_features, "default");
    let mut features = Vec::new();
    let mut mismatches = Vec::new();

    for (feature, deps) in &lab_features {
        let classification = classify_lab_feature(feature, deps, &api_features);
        let mapped = mapped_lab_feature(feature, deps, &api_features);
        if classification == FeatureClass::ServicePassthrough {
            if !api_features.contains_key(feature.as_str()) {
                mismatches.push(FeatureMismatch {
                    feature: feature.clone(),
                    message: "service passthrough missing matching lab-apis feature".to_string(),
                });
            }
            if !lab_all.contains(feature.as_str()) {
                mismatches.push(FeatureMismatch {
                    feature: feature.clone(),
                    message: "service feature missing from lab all".to_string(),
                });
            }
            if !api_all.contains(feature.as_str()) {
                mismatches.push(FeatureMismatch {
                    feature: feature.clone(),
                    message: "service feature missing from lab-apis all".to_string(),
                });
            }
        }
        features.push(FeatureDoc {
            crate_name: "lab".to_string(),
            feature: feature.clone(),
            dependencies: deps.clone(),
            included_in_default: lab_default.contains(feature.as_str()),
            included_in_all: lab_all.contains(feature.as_str()),
            classification,
            mapped_crate_feature: mapped,
            exception_reason: exception_reason(classification).map(str::to_string),
        });
    }

    for (feature, deps) in &api_features {
        let classification = classify_api_feature(feature, &lab_features);
        if classification == FeatureClass::SdkOnly && !api_all.contains(feature.as_str()) {
            mismatches.push(FeatureMismatch {
                feature: feature.clone(),
                message: "SDK-only service feature missing from lab-apis all".to_string(),
            });
        }
        features.push(FeatureDoc {
            crate_name: "lab-apis".to_string(),
            feature: feature.clone(),
            dependencies: deps.clone(),
            included_in_default: api_default.contains(feature.as_str()),
            included_in_all: api_all.contains(feature.as_str()),
            classification,
            mapped_crate_feature: lab_features
                .contains_key(feature.as_str())
                .then(|| format!("lab/{feature}")),
            exception_reason: exception_reason(classification).map(str::to_string),
        });
    }

    features.sort_by(|a, b| {
        (a.crate_name.as_str(), a.feature.as_str())
            .cmp(&(b.crate_name.as_str(), b.feature.as_str()))
    });
    mismatches.sort_by(|a, b| a.feature.cmp(&b.feature));
    Ok(FeatureMatrix {
        features,
        mismatches,
    })
}

fn normalize_audit_paths(report: &mut AuditReport, repo_root: &Path) {
    let roots = path_prefixes(repo_root);
    for service in &mut report.services {
        for (_, result) in &mut service.checks {
            match result {
                CheckResult::Fail(message) | CheckResult::Skip(message) => {
                    for root in &roots {
                        *message = message.replace(root, ".");
                    }
                    *message = scrub_absolute_paths(message);
                }
                CheckResult::Pass => {}
            }
        }
    }
}

fn build_onboarding_audit(services: &[ServiceDoc], repo_root: &Path) -> AuditReport {
    let audited = services
        .iter()
        .filter(|service| should_run_onboarding_audit(service))
        .map(|service| service.name.clone())
        .collect::<Vec<_>>();
    let raw = audit_services(&audited, repo_root);
    let mut raw_by_service = raw
        .services
        .into_iter()
        .map(|report| (report.service.clone(), report))
        .collect::<BTreeMap<_, _>>();

    let services = services
        .iter()
        .map(|service| {
            raw_by_service
                .remove(&service.name)
                .unwrap_or_else(|| skipped_onboarding_report(service))
        })
        .collect();

    AuditReport { services }
}

fn should_run_onboarding_audit(service: &ServiceDoc) -> bool {
    matches!(service.exposure, ServiceExposure::FeatureGated)
        && service.surfaces.cli
        && service.surfaces.mcp
        && service.surfaces.api
        && !matches!(service.name.as_str(), "deploy")
}

fn skipped_onboarding_report(service: &ServiceDoc) -> ServiceReport {
    let reason = match service.exposure {
        ServiceExposure::SdkOnly => "sdk-only service; no lab binary surface expected",
        ServiceExposure::RuntimeConditional => {
            "runtime-conditional service; scaffold audit is not applicable"
        }
        ServiceExposure::AlwaysOn => {
            "always-on synthetic/internal service; scaffold audit is not applicable"
        }
        ServiceExposure::FeatureGated => {
            "non-standard service surface; scaffold audit is not applicable"
        }
    };
    ServiceReport {
        service: service.name.clone(),
        checks: vec![(
            "onboarding.scope".to_string(),
            CheckResult::Skip(reason.to_string()),
        )],
    }
}

fn path_prefixes(repo_root: &Path) -> Vec<String> {
    let mut roots = vec![repo_root.display().to_string()];
    if let Ok(canonical) = repo_root.canonicalize() {
        let canonical = canonical.display().to_string();
        if !roots.contains(&canonical) {
            roots.push(canonical);
        }
    }
    roots
}

fn scrub_absolute_paths(message: &str) -> String {
    message
        .split_whitespace()
        .map(scrub_path_token)
        .collect::<Vec<_>>()
        .join(" ")
}

fn scrub_path_token(token: &str) -> String {
    let trimmed = token.trim_matches(|ch: char| matches!(ch, '"' | '\'' | '`' | ',' | ';' | ':'));
    if is_absolute_path_like(trimmed) {
        token.replace(trimmed, "<absolute-path>")
    } else {
        token.to_string()
    }
}

fn is_absolute_path_like(value: &str) -> bool {
    value.starts_with("/home/")
        || value.starts_with("/Users/")
        || value.starts_with("/tmp/")
        || value.starts_with("/build/")
        || value.starts_with("\\Users\\")
        || (value.len() > 2
            && value.as_bytes()[1] == b':'
            && value.as_bytes()[2] == b'\\'
            && value.as_bytes()[0].is_ascii_alphabetic())
}

fn read_manifest(path: &Path) -> Result<CargoManifest> {
    let text = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read {}", path.display()))?;
    toml::from_str(&text).with_context(|| format!("failed to parse {}", path.display()))
}

#[derive(Debug, Deserialize)]
struct CargoManifest {
    #[serde(default)]
    features: BTreeMap<String, Vec<String>>,
}

fn feature_set(features: &BTreeMap<String, Vec<String>>, name: &str) -> BTreeSet<String> {
    features
        .get(name)
        .into_iter()
        .flatten()
        .map(|dep| dep.strip_prefix("lab-apis/").unwrap_or(dep).to_string())
        .collect()
}

fn classify_lab_feature(
    feature: &str,
    deps: &[String],
    api_features: &BTreeMap<String, Vec<String>>,
) -> FeatureClass {
    if matches!(feature, "all" | "default") {
        FeatureClass::AggregateDefault
    } else if matches!(feature, "fs" | "lab-admin") {
        FeatureClass::BinaryOnly
    } else if deps.iter().any(|dep| dep == &format!("lab-apis/{feature}"))
        && api_features.contains_key(feature)
    {
        FeatureClass::ServicePassthrough
    } else if deps.iter().any(|dep| dep.starts_with("dep:")) {
        FeatureClass::HelperInternal
    } else {
        FeatureClass::IntentionalException
    }
}

fn classify_api_feature(
    feature: &str,
    lab_features: &BTreeMap<String, Vec<String>>,
) -> FeatureClass {
    if matches!(feature, "all" | "default") {
        FeatureClass::AggregateDefault
    } else if matches!(feature, "servarr" | "test-utils") {
        FeatureClass::HelperInternal
    } else if lab_features.contains_key(feature) {
        FeatureClass::ServicePassthrough
    } else {
        FeatureClass::SdkOnly
    }
}

fn mapped_lab_feature(
    feature: &str,
    deps: &[String],
    api_features: &BTreeMap<String, Vec<String>>,
) -> Option<String> {
    deps.iter()
        .filter_map(|dep| dep.strip_prefix("lab-apis/"))
        .find(|dep| api_features.contains_key(*dep))
        .map(|dep| format!("lab-apis/{dep}"))
        .or_else(|| {
            api_features
                .contains_key(feature)
                .then(|| format!("lab-apis/{feature}"))
        })
}

fn exception_reason(classification: FeatureClass) -> Option<&'static str> {
    match classification {
        FeatureClass::BinaryOnly => Some("binary-only Lab feature"),
        FeatureClass::HelperInternal => Some("helper/internal feature"),
        FeatureClass::AggregateDefault => Some("aggregate/default feature"),
        FeatureClass::IntentionalException => Some("intentional crate-local exception"),
        FeatureClass::ServicePassthrough | FeatureClass::SdkOnly => None,
    }
}

fn service_feature(service: &str, matrix: &FeatureMatrix) -> Option<String> {
    matrix
        .features
        .iter()
        .find(|feature| {
            feature.crate_name == "lab"
                && feature.feature == service
                && matches!(
                    feature.classification,
                    FeatureClass::ServicePassthrough | FeatureClass::BinaryOnly
                )
        })
        .map(|feature| feature.feature.clone())
}

pub(super) fn service_surfaces(service: &str) -> SurfaceAvailability {
    SurfaceAvailability {
        cli: !matches!(service, "device" | "fs"),
        mcp: true,
        api: service_has_action_api_route(service)
            || matches!(
                service,
                "device" | "extract" | "marketplace" | "doctor" | "setup"
            ),
        web_ui: matches!(
            service,
            "gateway" | "marketplace" | "logs" | "setup" | "device" | "fs"
        ),
    }
}

impl SurfaceAvailability {
    fn none() -> Self {
        Self {
            cli: false,
            mcp: false,
            api: false,
            web_ui: false,
        }
    }
}

fn doc_exists(repo_root: &Path, rel: &str) -> Option<String> {
    repo_root.join(rel).exists().then(|| rel.to_string())
}

fn non_empty(value: &str) -> Option<String> {
    (!value.is_empty()).then(|| value.to_string())
}

fn sanitized_example(env: &EnvVar) -> String {
    if env.secret {
        format!("<{}>", env.name.to_ascii_lowercase())
    } else {
        env.example.to_string()
    }
}

#[cfg(test)]
pub(crate) fn secret_example_is_suspicious(value: &str) -> bool {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed.starts_with('<') {
        return false;
    }
    let lower = trimmed.to_ascii_lowercase();
    lower.contains("token")
        || lower.contains("secret")
        || lower.contains("password")
        || lower.contains("cookie")
        || lower.starts_with("sk-")
        || lower.starts_with("eyj")
        || lower.contains("-----begin ")
        || trimmed.len() >= 20
}

fn sdk_only_metas() -> Vec<&'static PluginMeta> {
    vec![
        #[cfg(feature = "acp_registry")]
        &lab_apis::acp_registry::META,
        #[cfg(feature = "mcpregistry")]
        &lab_apis::mcpregistry::META,
    ]
}

#[allow(clippy::too_many_lines)]
fn meta_for(name: &str) -> Option<&'static PluginMeta> {
    match name {
        "extract" => Some(&lab_apis::extract::META),
        "marketplace" => Some(&lab_apis::marketplace::META),
        "doctor" => Some(&lab_apis::doctor::META),
        "setup" => Some(&lab_apis::setup::META),
        "stash" => Some(&lab_apis::stash::META),
        "acp" => Some(&lab_apis::acp::META),
        "device_runtime" => Some(&lab_apis::device_runtime::META),
        #[cfg(feature = "adguard")]
        "adguard" => Some(&lab_apis::adguard::META),
        #[cfg(feature = "apprise")]
        "apprise" => Some(&lab_apis::apprise::META),
        #[cfg(feature = "arcane")]
        "arcane" => Some(&lab_apis::arcane::META),
        #[cfg(feature = "beads")]
        "beads" => Some(&lab_apis::beads::META),
        #[cfg(feature = "bytestash")]
        "bytestash" => Some(&lab_apis::bytestash::META),
        #[cfg(feature = "deploy")]
        "deploy" => Some(&lab_apis::deploy::META),
        #[cfg(feature = "dozzle")]
        "dozzle" => Some(&lab_apis::dozzle::META),
        #[cfg(feature = "freshrss")]
        "freshrss" => Some(&lab_apis::freshrss::META),
        #[cfg(feature = "glances")]
        "glances" => Some(&lab_apis::glances::META),
        #[cfg(feature = "gotify")]
        "gotify" => Some(&lab_apis::gotify::META),
        #[cfg(feature = "immich")]
        "immich" => Some(&lab_apis::immich::META),
        #[cfg(feature = "jellyfin")]
        "jellyfin" => Some(&lab_apis::jellyfin::META),
        #[cfg(feature = "linkding")]
        "linkding" => Some(&lab_apis::linkding::META),
        #[cfg(feature = "loggifly")]
        "loggifly" => Some(&lab_apis::loggifly::META),
        #[cfg(feature = "memos")]
        "memos" => Some(&lab_apis::memos::META),
        #[cfg(feature = "navidrome")]
        "navidrome" => Some(&lab_apis::navidrome::META),
        #[cfg(feature = "neo4j")]
        "neo4j" => Some(&lab_apis::neo4j::META),
        #[cfg(feature = "notebooklm")]
        "notebooklm" => Some(&lab_apis::notebooklm::META),
        #[cfg(feature = "openacp")]
        "openacp" => Some(&lab_apis::openacp::META),
        #[cfg(feature = "openai")]
        "openai" => Some(&lab_apis::openai::META),
        #[cfg(feature = "overseerr")]
        "overseerr" => Some(&lab_apis::overseerr::META),
        #[cfg(feature = "paperless")]
        "paperless" => Some(&lab_apis::paperless::META),
        #[cfg(feature = "pihole")]
        "pihole" => Some(&lab_apis::pihole::META),
        #[cfg(feature = "plex")]
        "plex" => Some(&lab_apis::plex::META),
        #[cfg(feature = "prowlarr")]
        "prowlarr" => Some(&lab_apis::prowlarr::META),
        #[cfg(feature = "qbittorrent")]
        "qbittorrent" => Some(&lab_apis::qbittorrent::META),
        #[cfg(feature = "qdrant")]
        "qdrant" => Some(&lab_apis::qdrant::META),
        #[cfg(feature = "radarr")]
        "radarr" => Some(&lab_apis::radarr::META),
        #[cfg(feature = "sabnzbd")]
        "sabnzbd" => Some(&lab_apis::sabnzbd::META),
        #[cfg(feature = "scrutiny")]
        "scrutiny" => Some(&lab_apis::scrutiny::META),
        #[cfg(feature = "sonarr")]
        "sonarr" => Some(&lab_apis::sonarr::META),
        #[cfg(feature = "tailscale")]
        "tailscale" => Some(&lab_apis::tailscale::META),
        #[cfg(feature = "tautulli")]
        "tautulli" => Some(&lab_apis::tautulli::META),
        #[cfg(feature = "tei")]
        "tei" => Some(&lab_apis::tei::META),
        #[cfg(feature = "unifi")]
        "unifi" => Some(&lab_apis::unifi::META),
        #[cfg(feature = "unraid")]
        "unraid" => Some(&lab_apis::unraid::META),
        #[cfg(feature = "uptime_kuma")]
        "uptime_kuma" => Some(&lab_apis::uptime_kuma::META),
        _ => None,
    }
}

#[allow(dead_code)]
pub fn workspace_root() -> Result<PathBuf> {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .map(Path::to_path_buf)
        .ok_or_else(|| anyhow::anyhow!("cannot determine workspace root from CARGO_MANIFEST_DIR"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn secret_examples_are_always_placeholdered() {
        let env = EnvVar {
            name: "SERVICE_API_KEY",
            description: "API key",
            example: "demo-key",
            secret: true,
            ui: None,
        };
        assert_eq!(sanitized_example(&env), "<service_api_key>");
    }

    #[test]
    fn action_catalog_exposes_fs_preview_as_http_only() {
        let projection = build_docs_projection(&workspace_root().unwrap()).unwrap();
        let preview = projection
            .action_catalog
            .iter()
            .find(|action| action.service == "fs" && action.action == "fs.preview")
            .unwrap();
        assert!(preview.surface_availability.api);
        assert!(preview.surface_availability.web_ui);
        assert!(!preview.surface_availability.mcp);
        assert!(preview.requires_http_subject);
    }

    #[test]
    fn mcp_help_is_equivalent_to_mcp_action_projection() {
        let projection = build_docs_projection(&workspace_root().unwrap()).unwrap();
        let help_actions = projection
            .mcp_help
            .services
            .iter()
            .flat_map(|service| {
                service
                    .actions
                    .iter()
                    .map(|action| (service.name.as_str(), action.name.as_str()))
            })
            .collect::<BTreeSet<_>>();
        let projected_mcp_actions = projection
            .action_catalog
            .iter()
            .filter(|action| action.surface_availability.mcp && !action.builtin)
            .map(|action| (action.service.as_str(), action.action.as_str()))
            .collect::<BTreeSet<_>>();
        assert_eq!(help_actions, projected_mcp_actions);
    }
}
