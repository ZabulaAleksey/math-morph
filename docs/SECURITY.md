# Security Baseline — OWASP Top 10:2025

Источник: https://owasp.org/Top10/2025/

Этот документ задаёт минимальный baseline, а не заменяет threat modeling, ASVS, secure code review или penetration testing.

## Security invariants проекта

- Пользовательский файл — недоверенный input независимо от extension/MIME.
- Документ/формула/filename могут содержать секретные или вредоносные данные.
- Admin role не получает автоматического права расшифровать пользовательский документ.
- Authenticated != authorized: каждая object/action проверяет ownership/role/scope.
- Ключи/пароли/tokens не логируются и не хранятся plaintext.
- Security failure должен fail closed там, где это не разрушает recoverability.
- Parser/worker должен переживать malformed input без process crash/panic.

---

## A01:2025 — Broken Access Control

Применение:
- object-level authorization для documents/conversions/API keys/billing;
- tenant/user isolation;
- deny-by-default routes;
- server-side authorization для каждого protected action;
- API scopes + ownership checks одновременно;
- admin RBAC с минимальными правами;
- signed URLs короткоживущие и привязанные к разрешённому объекту;
- CSRF защита там, где используется cookie-based state change;
- SSRF controls для webhook/remote-fetch функций.

Tests:
- IDOR: user A не читает/удаляет object user B;
- revoked API key;
- scope missing;
- admin privacy boundary;
- direct endpoint access без UI.

## A02:2025 — Security Misconfiguration

Требования:
- secure defaults;
- production debug off;
- минимальные CORS origins;
- CSP/security headers;
- non-public S3 buckets;
- default Keycloak/admin credentials запрещены;
- unnecessary ports/services disabled;
- separate dev/stage/prod config;
- secrets только через environment/secret manager;
- infrastructure configuration review.

Tests/checks:
- configuration lint;
- public-bucket check;
- security header test;
- accidental debug endpoint scan.

## A03:2025 — Software Supply Chain Failures

Требования:
- lockfiles и reproducible install;
- минимизация dependencies;
- provenance/source review перед новой production dependency;
- pin/lock versions; не использовать `latest` в production automation;
- dependency audit для npm/pnpm, Python/uv, Rust/Cargo и container images;
- SBOM для release по мере зрелости;
- проверять install/postinstall scripts;
- CI actions pin на immutable refs/digests по возможности;
- container images pin на version/digest;
- review MCP servers/plugins/hooks как executable supply-chain components.

Нельзя автоматически подключать случайный MCP/npm package только ради удобства.

## A04:2025 — Cryptographic Failures

Требования:
- TLS для data in transit;
- authenticated encryption для client-side document encryption (например AES-GCM);
- уникальные nonces/IV согласно библиотеке/алгоритму;
- keys отделены от ciphertext;
- не писать собственную криптографию;
- API keys хранить hash/secure verifier, секрет показывать один раз;
- document recovery key не смешивать с account password recovery;
- backup/key lifecycle документировать;
- crypto errors не должны silently fallback на plaintext.

Tests:
- ciphertext tampering fails;
- plaintext marker отсутствует в storage/logs;
- lost/invalid key не вызывает plaintext fallback.

## A05:2025 — Injection

Threats:
- SQL/NoSQL injection;
- shell/command injection;
- XML attacks;
- template injection;
- XSS через filenames/metadata/errors;
- future spreadsheet formula injection;
- unsafe OOXML/SVG/content embedding.

Controls:
- parameterized ORM/queries;
- не строить shell command из пользовательского input;
- безопасный XML parser: DTD/external entities запрещены, limits enforced;
- sanitize/escape UI output по контексту;
- allow-list output formats/options;
- Excel export в будущем экранирует/контролирует formula-like cell content;
- SVG/HTML imports рассматриваются как active content.

## A06:2025 — Insecure Design

Требования:
- threat model перед auth/crypto/storage/API support-sharing;
- abuse cases: oversized files, conversion bombs, quota bypass, recovery abuse, webhook SSRF;
- explicit trust modes: local/WASM vs server conversion;
- privacy claims должны соответствовать технической реальности;
- rate/size/time/memory limits — часть дизайна, не поздний patch;
- dangerous features за feature flags + staged rollout.

## A07:2025 — Authentication Failures

Требования:
- использовать зрелый IdP (например Keycloak), не самописную password auth;
- MFA TOTP/WebAuthn/passkeys;
- recovery tokens one-time + TTL + rate limit;
- Telegram recovery только после подтверждённого account linking;
- session rotation после login/recovery/privilege change;
- revoke sessions;
- brute-force/rate-limit controls;
- generic recovery responses, не раскрывающие наличие аккаунта без необходимости;
- secure cookies (`HttpOnly`, `Secure`, appropriate `SameSite`) при cookie sessions.

## A08:2025 — Software or Data Integrity Failures

Требования:
- проверять integrity загруженных контейнеров;
- conversion reports/version metadata позволяют воспроизводимость;
- signed/verified payment webhooks;
- signed/verified internal callbacks where relevant;
- CI/release artifacts должны происходить из доверенного pipeline;
- нельзя автоматически принимать изменённый golden output;
- migrations/fixtures/versioned transformations reviewable;
- untrusted deserialization запрещена.

## A09:2025 — Security Logging and Alerting Failures

Логировать:
- auth failures с безопасной агрегацией;
- recovery attempts;
- API key lifecycle;
- privilege/admin actions;
- access-control denials;
- rate-limit/security events;
- worker/security failures;
- webhook signature failures.

Не логировать:
- document contents;
- formulas;
- passwords;
- recovery tokens;
- API secrets;
- encryption keys;
- decrypted filenames в zero-knowledge режиме.

Требования:
- request/correlation ID;
- log redaction;
- alerting для abnormal security events;
- audit trail должен быть защищён от обычного пользователя.

## A10:2025 — Mishandling of Exceptional Conditions

Особенно важно для конвертера.

Требования:
- invalid/corrupted/huge files имеют controlled errors;
- fail closed для auth/authz/crypto/integrity;
- timeouts, memory/size/depth limits;
- retry только transient errors; malformed input не retry бесконечно;
- dead-letter/error flow;
- idempotency для job creation/payment/webhook flows;
- partial conversion только с явным warning/report;
- network disconnect не создаёт дубликат job автоматически;
- frontend Error Boundaries и recoverable UI state;
- error message не раскрывает stack trace/secrets/internal paths в production.

## File/parser specific hardening

- Verify content signature/structure, not extension only.
- ZIP: entry count, uncompressed size, ratio, path traversal, duplicate/conflicting names.
- XML: external entities/DTD off; depth/size limits; strict encoding handling.
- Embedded images/SVG/relationships: validate type/size/URI; remote relationships запрещены/контролируются.
- Temporary files: random names, restrictive permissions, cleanup, quotas.
- Conversion worker: sandbox/isolation as feasible; least privilege; no access to unrelated user objects.

## Required security gates by change type

| Change | Minimum checks |
|---|---|
| Parser/upload | A05, A06, A10 + malformed/fuzz tests |
| Auth/recovery | A01, A07, A09, A10 |
| Crypto/storage | A01, A04, A06, A08, A09 |
| Dependencies/CI/MCP/hooks | A02, A03, A08 |
| API/admin | A01, A02, A05, A07, A09 |
| Billing/webhooks | A01, A05, A08, A09, A10 |

## Security DoD

- [ ] Trust boundary identified.
- [ ] Relevant OWASP 2025 categories reviewed.
- [ ] Negative/abuse test added where behavior changed.
- [ ] No secrets/document contents introduced into logs.
- [ ] Access control checked server-side.
- [ ] Limits/timeouts considered.
- [ ] Dependency/supply-chain impact checked.
- [ ] Failure mode documented and fail-open avoided for security controls.
