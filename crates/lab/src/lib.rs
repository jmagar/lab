#[cfg(test)]
#[test]
fn lab_auth_crate_exports_router_and_signing_keys() {
    let _router_fn = lab_auth::routes::router;
    let _signing_keys_type = std::mem::size_of::<lab_auth::jwt::SigningKeys>();
}
