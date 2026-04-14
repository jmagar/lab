# Gateway Admin UI

Standalone Next.js admin UI for the `gateway` surface, vendored into the `lab` repo at `apps/gateway-admin`.

This lane only places the UI app in-repo and keeps it runnable as an isolated frontend. Full backend and repo-wide integration is intentionally deferred.

## Current State

- App framework: Next.js 16 + React 19
- Package manager: `pnpm` (lockfile included)
- Data mode: frontend mock data plus a small gateway API client under `lib/api/`

## Local Usage

From this directory:

```bash
pnpm install
pnpm dev
```

The app defaults `NEXT_PUBLIC_API_URL` to `/api`. Override it when pointing the UI at a different backend origin.

```bash
NEXT_PUBLIC_API_URL=http://localhost:3000/api pnpm dev
```

## Notes

- The imported UI code was originally developed as its own repository and is now tracked as normal source under this repo.
- Nested git metadata was removed on import so `apps/gateway-admin` behaves like a standard in-repo app directory.
