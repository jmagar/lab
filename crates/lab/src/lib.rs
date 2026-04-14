#[cfg(test)]
#[test]
fn lab_auth_crate_exports_router_and_verifier() {
    let _router_fn = lab_auth::routes::router;
    let _verify_fn = lab_auth::jwt::validate_access_token;
}
