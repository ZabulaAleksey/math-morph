# Product Specification — Mathcad Converter & Parser Platform

## 1. Purpose

Build an extensible SaaS/platform that parses Mathcad worksheets, preserves mathematical/document semantics, performs configurable transformations and exports editable documents. The first product path is Mathcad → Microsoft Word; the architecture must support future Markdown, PDF, LaTeX, HTML, JSON/web viewer, editable Excel charts and editable Visio diagrams.

The product is more than a file converter: it includes a parser, mathematical semantic analysis, substitution/evaluation traces, notation transformation, complex-number presentation, public API, account area, privacy controls, authentication, billing and administration.

## 2. Input formats

Initial mandatory inputs:

- `.xmcd` — legacy XML worksheet family;
- `.mcdx` — Mathcad Prime container family.

Format detection must inspect content, not trust extension/MIME alone. Extension/content mismatch is a structured warning/error depending on the actual condition. Corrupted or hostile input must fail safely.

## 3. Output formats

MVP:

- `.docx`.

Planned exporters:

- Markdown `.md`;
- PDF `.pdf`;
- LaTeX `.tex`;
- HTML `.html`;
- JSON `.json`;
- web viewer representation.

Future structured exports:

- editable Excel charts;
- editable Visio `.vsdx` diagrams.

## 4. Conversion architecture requirement

Do not implement direct XML → DOCX coupling. Required flow:

```text
Input → Format Detector / Safe Container Reader → Mathcad Parser
      → Mathcad AST → Semantic Analyzer → Transformation/Evaluation
      → Document IR → Exporter
```

Parser/AST/semantic layers must not depend on React, HTTP or Word-specific markup.

## 5. Formula conversion

Supported mathematical structures must be exported as editable equations, not raster images.

Equation backend abstraction:

- `WordEquationExporter` — native Microsoft Word equations / OMML, primary backend;
- `MathTypeExporter` — later/optional backend, isolated from parser core, potentially via MathML.

A controlled image/text fallback is permitted only for unsupported constructs and must emit a visible conversion warning.

## 6. Mathematical transformations

The system must preserve original mathematical semantics and apply presentation transformations on a separate display/transformation layer.

Examples:

- Mathcad definition `:=` may be rendered as `=`, `≔` or original notation according to a notation profile while remaining a `Definition` internally;
- configurable substitution of previously defined values;
- result-only, substitution and detailed trace modes;
- independent computation precision and presentation rounding.

## 7. Complex numbers

Support algebraic and polar representations.

Presentation rules:

- multiplication/division: allow trace through polar form, then final result in polar and algebraic form;
- addition/subtraction: primarily algebraic calculation;
- output options: algebraic, polar or both;
- edge cases: zero, pure real/imaginary, quadrants, angle normalization, division by zero, rounding boundaries.

## 8. Graphs

Initial behavior: preserve/copy graph preview into DOCX with size/aspect ratio and warning fallback if unavailable.

Architecture must preserve `PlotIR`/future `ChartIR`; raster output must not destroy semantics required for later editable Excel chart reconstruction.

## 9. Diagrams/schemes

Initial behavior: copy/render as image into DOCX.

Future `DiagramIR` must support shapes, connectors, groups, text, styles, coordinates and bounds. Future VSDX output must be a real editable Visio project; inserting one large image into VSDX does not satisfy the requirement.

## 10. Main website pages

### Home

Blocks: header/navigation, hero upload CTA, capabilities, privacy/security explanation, API teaser, pricing teaser, footer/legal/status.

### Converter

Blocks: dropzone, file information/validation, output format, equation backend, notation/substitution/precision/complex options, save preferences, convert action, progress state, result/report.

### Result

Show status, file/result metadata, warnings, download, retry/reconfigure, report.

### Account

Sections: overview, documents, conversion history, API keys, API usage, security, connected accounts, billing, settings/privacy.

### API documentation

Human-readable quick start, authentication, conversion creation, async job states, download/report, errors, limits, persistence policy, SDK/examples, future webhooks.

### Admin

Users/account metadata, plans/billing metadata, conversion/worker/queue metrics, parser error and unsupported-node statistics, feature flags, security events. Admin must not bypass privacy-protected document boundaries.

## 11. File and conversion UX states

Explicitly handle:

- empty file;
- unsupported extension;
- extension/content mismatch;
- corrupted MCDX/XMCD;
- unsupported/partially supported Mathcad version;
- file too large (including exact limit boundary);
- conversion warning/partial conversion;
- fatal conversion failure;
- timeout;
- network loss before upload/during upload/after job creation/during result download;
- service unavailable;
- quota/rate limit exceeded.

Every user-facing error must answer: what happened, whether data is safe, and what the user can do next. Never expose raw stack traces.

## 12. Authentication and account recovery

Support standard login/password and extensible OIDC/OAuth providers (e.g. Google, Microsoft, GitHub).

Security features:

- email verification;
- TOTP 2FA;
- recovery codes;
- WebAuthn/passkeys;
- confirmed email recovery;
- confirmed phone/SMS recovery;
- Telegram linking and later recovery through a previously linked Telegram account.

Telegram recovery must use an explicit account linking flow with one-time, expiring, replay-protected tokens; never trust a typed `@username` as identity proof.

Account recovery and encrypted-document recovery are separate processes.

## 13. API

Versioned REST API starting at `/api/v1`.

Core flow:

```text
Client → auth/API key → validate/quota → create conversion job
       → queue/worker → result/report → policy-controlled storage
```

Core endpoints conceptually include conversions, status, result/report, documents, API keys and usage. Long-running conversions use asynchronous jobs.

Job states: created, queued, processing, completed, completed_with_warnings, failed, cancelled, expired.

API keys:

- shown in full only once;
- stored only as secure hash/derived verifier plus prefix/metadata;
- revocable, expirable and scoped;
- usage/rate-limit accounting.

Explicit request settings override profile defaults when policy permits; otherwise profile defaults apply.

## 14. Saved documents

User preferences independently control saving web conversions, API conversions, originals and outputs.

Stored file objects belong in S3-compatible object storage; PostgreSQL stores metadata only. Retention must be configurable by plan/policy. Delete requests must clean metadata and schedule object cleanup.

## 15. Privacy / zero-knowledge direction

Preferred high-privacy path uses the Rust core compiled to WebAssembly for local processing where supported, plus client-side authenticated encryption for saved objects.

Do not promise absolute zero-knowledge for operations that require plaintext server-side conversion. Secure marketing claims must map to actual architecture.

If server-side storage cannot read encrypted documents, administrators and support cannot read them either without explicit user-controlled sharing/recovery capability.

## 16. Internationalization

All user-facing strings are externalized through i18n catalogs. Adding a locale must not require business-logic changes. Backend returns stable error codes; frontend localizes them.

## 17. Billing/monetization

Plan model should support Free, Pro, API, Team and future Enterprise. Quotas can vary by conversion count, file size, formats, storage/retention, API allowance, batch processing and queue priority.

Billing must use a provider abstraction to allow regional/international providers without coupling core logic.

## 18. Non-functional requirements

- modular and extensible architecture;
- deterministic parser/AST where applicable;
- streaming/bounded parsing for large XML/container input;
- stateless API where practical;
- horizontally scalable workers;
- structured diagnostics and observability without content leakage;
- accessibility and responsive UI;
- comprehensive automated tests;
- secure defaults and fail-closed behavior for auth/access/crypto/integrity boundaries.

## 19. MVP success criteria

A user can register/login, upload supported Mathcad input, receive safe validation, configure DOCX/Word-equation transformation options, convert supported text/formulas, preserve plots as images, receive clear warnings for unsupported elements, download the result, choose whether to save it, view saved/history metadata, create/use an API key, perform equivalent API conversion, switch locale and use 2FA/recovery without document contents leaking into logs.
