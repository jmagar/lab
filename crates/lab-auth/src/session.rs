use axum::http::header::{COOKIE, HeaderMap, SET_COOKIE};
use axum::response::Response;

use crate::error::AuthError;
use crate::state::AuthState;
use crate::types::BrowserSessionRow;
use crate::util::{expires_at, now_unix, random_token};

pub const BROWSER_SESSION_COOKIE_NAME: &str = "lab_session";
pub const BROWSER_CSRF_HEADER_NAME: &str = "x-csrf-token";

pub fn read_cookie(headers: &HeaderMap, name: &str) -> Option<String> {
    headers
        .get(COOKIE)
        .and_then(|value| value.to_str().ok())
        .and_then(|raw| {
            raw.split(';').map(str::trim).find_map(|cookie| {
                let (key, value) = cookie.split_once('=')?;
                (key.trim() == name).then(|| value.trim().to_string())
            })
        })
}

pub fn append_set_cookie(response: &mut Response, cookie: &str) {
    if let Ok(value) = cookie.parse() {
        response.headers_mut().append(SET_COOKIE, value);
    }
}

pub async fn create_browser_session(
    state: &AuthState,
    subject: String,
    email: Option<String>,
) -> Result<BrowserSessionRow, AuthError> {
    let created_at = now_unix();
    let session = BrowserSessionRow {
        session_id: random_token(24)?,
        subject,
        email,
        csrf_token: random_token(18)?,
        created_at,
        expires_at: expires_at(
            created_at,
            state.config.refresh_token_ttl,
            "LAB_AUTH_REFRESH_TOKEN_TTL_SECS",
        )?,
    };
    state.store.upsert_browser_session(session.clone()).await?;
    Ok(session)
}

fn secure_cookie_attr(state: &AuthState) -> &'static str {
    if state
        .config
        .public_url
        .as_ref()
        .is_none_or(|url| url.scheme() == "https")
    {
        "; Secure"
    } else {
        ""
    }
}

pub fn build_browser_session_cookie(state: &AuthState, session_id: &str) -> String {
    format!(
        "{name}={value}; Path=/; HttpOnly; SameSite=Lax; Max-Age={max_age}{secure}",
        name = BROWSER_SESSION_COOKIE_NAME,
        value = session_id,
        max_age = state.config.refresh_token_ttl.as_secs(),
        secure = secure_cookie_attr(state),
    )
}

pub fn clear_browser_session_cookie(state: &AuthState) -> String {
    format!(
        "{name}=; Path=/; HttpOnly; SameSite=Lax; Max-Age=0; Expires=Thu, 01 Jan 1970 00:00:00 GMT{secure}",
        name = BROWSER_SESSION_COOKIE_NAME,
        secure = secure_cookie_attr(state),
    )
}

#[cfg(test)]
mod tests {
    use axum::http::HeaderMap;

    use super::read_cookie;

    #[test]
    fn read_cookie_finds_named_cookie_even_when_it_is_not_first() {
        let mut headers = HeaderMap::new();
        headers.insert(
            axum::http::header::COOKIE,
            "theme=dark; lab_session=session-123; other=value"
                .parse()
                .unwrap(),
        );

        assert_eq!(
            read_cookie(&headers, super::BROWSER_SESSION_COOKIE_NAME),
            Some("session-123".to_string())
        );
    }

    #[test]
    fn read_cookie_ignores_unrelated_cookies() {
        let mut headers = HeaderMap::new();
        headers.insert(
            axum::http::header::COOKIE,
            "theme=dark; other=value".parse().unwrap(),
        );

        assert_eq!(
            read_cookie(&headers, super::BROWSER_SESSION_COOKIE_NAME),
            None
        );
    }
}
