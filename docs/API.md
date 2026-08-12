# API Specification Direction

Base path: `/api/v1`.

## Authentication

API-key/Bearer style credential bound to a user/account. Full secret shown once, verifier/hash stored server-side, with prefix, scopes, creation/last-use/expiry/revocation metadata.

## Conversion lifecycle

Conceptual endpoints:

- `POST /api/v1/conversions`
- `GET /api/v1/conversions/{id}`
- `GET /api/v1/conversions/{id}/result`
- `GET /api/v1/conversions/{id}/report`
- `GET /api/v1/documents`
- `DELETE /api/v1/documents/{id}`
- API-key create/list/revoke/rotate operations
- `GET /api/v1/usage`

Long operations use asynchronous job IDs. Never auto-submit a duplicate job merely because the browser lost connectivity; status must be recoverable by ID/idempotency key.

## Job states

`created`, `queued`, `processing`, `completed`, `completed_with_warnings`, `failed`, `cancelled`, `expired`.

## Error envelope

Stable machine-readable code, request ID and safe details. Raw exception/stack/document content is forbidden.

Important codes include file empty/too large/unsupported/mismatch/corrupted, unsupported Mathcad version/node, invalid options, conversion timeout/failure, auth/key/scope failures, quota/rate-limit failures, storage/service unavailable.

## Persistence

Explicit request option may override user profile save default according to documented policy. API conversions saved by policy appear in the same user history with source `API`.

## Future webhooks

Signed webhook events for completion/failure with retry/idempotency/delivery history. Not required for initial MVP.
