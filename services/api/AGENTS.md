# API Rules

- Read `docs/SECURITY.md` for every auth/authz/API-key/upload/storage change.
- Version endpoints under `/api/v1` unless an ADR changes policy.
- Authenticate AND authorize every protected object/action.
- Structured error code + request ID; no raw stack traces or document data.
- API key secret shown once; only hash/verifier stored.
- Web/API/CLI must reuse conversion core rather than duplicate semantics.
- Enforce server-side size/quota/rate/option validation.
- Job creation should be idempotent where duplicate requests are possible.
