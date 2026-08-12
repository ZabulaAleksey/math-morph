# Dependency Policy

## Основной стек

- Frontend: Next.js, React, TypeScript, Tailwind, shadcn/ui, TanStack Query, Zustand, React Hook Form, Zod, next-intl.
- Core: Rust, quick-xml, Serde, num/num-complex, nalgebra, WASM.
- Backend: Python, FastAPI, Pydantic, SQLAlchemy, Alembic, HTTPX, `uv`.
- Data/async: PostgreSQL, RabbitMQ, Celery, Redis, S3-compatible storage/MinIO.
- Auth: Keycloak/OIDC/OAuth2/WebAuthn/TOTP; Telegram Bot API only for explicitly linked/recovery flows.
- Observability: OpenTelemetry, Prometheus, Grafana, Sentry with redaction.
- Infra: Docker/Compose; Kubernetes/Helm only later when justified.

## Добавление новой dependency

До изменения:

1. Есть ли функция в standard library/уже установленной dependency?
2. Нужна ли dependency в production или только dev/test?
3. Кто publisher/maintainer, есть ли официальный репозиторий?
4. Лицензия совместима?
5. Есть ли известные advisories/abandoned status?
6. Есть ли install/postinstall scripts?
7. Можно ли pin/lock version?
8. Какой размер/транзитивный граф?
9. Какие permissions/network/filesystem она получает?
10. Как удалить/заменить её в будущем?

## Supply-chain rules

- Не использовать `latest` в production manifests/scripts.
- Lockfiles коммитятся.
- CI dependency update должен проходить tests/security audit.
- GitHub Actions и container images pin максимально стабильно.
- MCP server, Codex plugin, Skill scripts и hooks считать executable dependency.
- Remote MCP получает минимальные toolsets/approval mode.
- Никакие secrets не записываются в tracked TOML/MD.

## Python

Использовать `uv`, project `.venv`, общий cache. Не смешивать системный Python с project dependencies.

## Experimental dependencies

Экспериментальная dependency должна быть feature-flagged/isolated и не становиться обязательной для core без отдельного решения.
