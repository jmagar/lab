#[cfg(test)]
use std::mem::size_of;

#[cfg(test)]
#[test]
fn lab_auth_crate_exports_router_and_signing_keys() {
    let _router_fn = lab_auth::routes::router;
    let _signing_keys_type = size_of::<lab_auth::jwt::SigningKeys>();
}
