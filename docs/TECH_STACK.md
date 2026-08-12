# Technology Stack

## Frontend

- Next.js + React + TypeScript
- Tailwind CSS + shadcn/ui
- TanStack Query for server state
- Zustand for small local UI state
- React Hook Form + Zod
- next-intl / ICU-compatible catalogs
- Web Workers for non-blocking local conversion
- WebAssembly for Rust core in browser
- Web Crypto API for client-side cryptography

## Mathcad core

Language: **Rust**.

Modules/crates should evolve around:

- format detection / safe container reading;
- XMCD/MCDX parsing (`quick-xml`, Serde where appropriate);
- Mathcad AST;
- semantic analyzer / symbol table / dependency graph;
- evaluator and `EvaluationTrace`;
- transformation/display AST;
- complex-number support (`num-complex`);
- linear algebra when required (`nalgebra`);
- `DocumentIR`.

## Equations and documents

- DOCX / Office Open XML / WordprocessingML
- OMML / Office Math for native editable Word equations
- MathML as an interoperability layer for optional MathType support
- future exporter contracts for Markdown, LaTeX, HTML, JSON, PDF
- `PlotIR`/`ChartIR` for future Excel charts
- `DiagramIR` for future editable VSDX/Visio output

## Backend/API

- Python + FastAPI
- Pydantic
- SQLAlchemy + Alembic
- HTTPX where needed
- Python dependency management: **uv**, project `.venv`, shared cache
- OpenAPI 3.1-oriented public API

## Data and jobs

- PostgreSQL — system-of-record metadata
- RabbitMQ — job broker
- Celery — server-side conversion workers
- Redis — cache/rate-limit/ephemeral coordination only
- S3-compatible object storage; MinIO for local/dev

## Authentication

- Keycloak or equivalent standards-based identity boundary
- OAuth 2.0 / OpenID Connect
- TOTP
- WebAuthn/passkeys
- recovery codes
- confirmed email/phone recovery
- Telegram Bot API for explicit account linking and optional recovery

## Billing

Provider abstraction; regional/international adapters may include LiqPay, WayForPay and a Merchant-of-Record provider such as Paddle where legally/operationally appropriate. Do not hard-code one provider into core business logic.

## Testing

Rust:
- `cargo test`
- property testing (`proptest`)
- fuzzing (`cargo-fuzz`)
- AST/IR snapshots/golden tests where appropriate

Python:
- pytest
- pytest-asyncio
- Hypothesis
- HTTPX integration tests

Frontend:
- Vitest
- React Testing Library
- Playwright

## Observability

- OpenTelemetry
- Prometheus
- Grafana
- Sentry/error tracking with strict redaction

## Infrastructure

Initial:
- Docker + Docker Compose
- GitHub Actions

Later only when justified:
- Terraform/OpenTofu
- Kubernetes
- Helm

## Security/supply chain

- lockfiles and reproducible builds
- dependency audit and secret scanning in CI
- MCP/plugins/Skills/hooks treated as supply-chain dependencies
- no unreviewed `latest` dependencies for durable infrastructure
- OWASP Top 10:2025 mapping in `SECURITY.md`

## AI/Codex integration

Global AI Dev Team owns generic engineering roles/workflows. Project overlay contains only Mathcad-specific agents and skills. Project hooks and shared MCP servers are disabled by default to avoid duplicate execution and context/tool-surface growth.
