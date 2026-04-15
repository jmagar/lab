# Environment Variables

This document lists the `lab` environment variables that matter for transport and auth setup.

## HTTP Auth

Bearer mode:

```env
LAB_AUTH_MODE=bearer
LAB_MCP_HTTP_TOKEN=replace-me
```

OAuth mode:

```env
LAB_AUTH_MODE=oauth
LAB_PUBLIC_URL=https://lab.example.com
LAB_GOOGLE_CLIENT_ID=google-client-id
LAB_GOOGLE_CLIENT_SECRET=google-client-secret
```

Optional auth overrides:

```env
LAB_AUTH_SQLITE_PATH=/var/lib/lab/auth.db
LAB_AUTH_KEY_PATH=/var/lib/lab/auth-jwt.pem
LAB_AUTH_ALLOWED_REDIRECT_URIS=https://callback.tootie.tv/callback/*
LAB_GOOGLE_CALLBACK_PATH=/auth/google/callback
LAB_GOOGLE_SCOPES=openid,email,profile
LAB_AUTH_ACCESS_TOKEN_TTL_SECS=3600
LAB_AUTH_REFRESH_TOKEN_TTL_SECS=2592000
LAB_AUTH_CODE_TTL_SECS=300
```

These non-secret overrides can also live in `config.toml` under `[auth]`.

Rules:

- `LAB_AUTH_MODE` defaults to `bearer`
- bearer mode keeps using `LAB_MCP_HTTP_TOKEN`
- oauth mode requires `LAB_PUBLIC_URL`, `LAB_GOOGLE_CLIENT_ID`, and `LAB_GOOGLE_CLIENT_SECRET`
- the old external issuer variables (`LAB_OAUTH_ISSUER`, `LAB_OAUTH_AUDIENCE`, `LAB_OAUTH_CLIENT_ID`) are no longer used
- `LAB_PUBLIC_URL` also feeds RFC 9728 metadata, JWT issuer/audience, and HTTP allowed-host derivation
