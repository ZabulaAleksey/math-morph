# Политика зависимостей

## Основной стек

- Frontend: Next.js, React, TypeScript, Tailwind, shadcn/ui, TanStack Query, Zustand, React Hook Form, Zod, next-intl.
- Core: Rust 1.88+, `quick-xml = 0.41.0`, `zip = 8.6.0`, `serde = 1.0.229`, `serde_json = 1.0.151`, `thiserror = 2.0.17`.
- Backend: Python, FastAPI, Pydantic, SQLAlchemy, Alembic, HTTPX, `uv`.
- Данные и асинхронность: PostgreSQL, RabbitMQ, Celery, Redis, S3-совместимое хранилище/MinIO.
- Аутентификация: Keycloak/OIDC/OAuth2/WebAuthn/TOTP; Telegram Bot API только для явно связанных flows и восстановления.
- Наблюдаемость: OpenTelemetry, Prometheus, Grafana, Sentry со скрытием данных.
- Инфраструктура: Docker/Compose; Kubernetes/Helm только позднее при наличии оснований.

## Добавление новой зависимости

До изменения:

1. Есть ли функция в стандартной библиотеке или уже установленной зависимости?
2. Нужна ли зависимость в production или только для разработки и тестов?
3. Кто её издатель и сопровождающий, есть ли официальный репозиторий?
4. Лицензия совместима?
5. Есть ли известные advisories или статус заброшенного проекта?
6. Есть ли scripts установки или postinstall?
7. Можно ли зафиксировать версию?
8. Какой размер/транзитивный граф?
9. Какие разрешения, сетевой и файловый доступ она получает?
10. Как удалить/заменить её в будущем?

## Правила цепочки поставки

- Не использовать `latest` в production-манифестах и scripts.
- Lockfiles коммитятся.
- Обновление зависимостей в CI должно проходить тесты и аудит безопасности.
- Версии GitHub Actions и образов контейнеров фиксируются максимально стабильно.
- MCP-сервер, plugin Codex, scripts Skills и hooks считаются исполняемыми зависимостями.
- Удалённый MCP получает минимальные наборы инструментов и режим разрешений.
- Никакие секреты не записываются в отслеживаемые TOML/MD.

## Python

Использовать `uv`, проектный `.venv` и общий кэш. Не смешивать системный Python с зависимостями проекта.

## Экспериментальные зависимости

Экспериментальная зависимость должна быть изолирована за feature flag и не становиться обязательной для core без отдельного решения.

## Принятые зависимости входной границы

- `quick-xml = 0.41.0`: потоковый namespace-aware XML reader без optional features; выбранная версия закрывает известные DoS-проблемы более ранних releases.
- `zip = 8.6.0`: поддерживаемая upstream ветка, default features отключены, включён только Rust Deflate backend; решение и повышение MSRV описаны в ADR-0006.
- `thiserror = 2.0.17`: typed domain errors без включения payload в публичный `Display`.
- `serde = 1.0.229` и `serde_json = 1.0.151`: строгий versioned JSON contract для `math-model`/`document-ir`; unknown fields отклоняются, а input/output ограничиваются до и во время сериализации.

Все версии и транзитивный граф закреплены в `Cargo.lock`. ZIP/XML/Serde libraries не получают сетевой или файловый доступ и не выполняют postinstall scripts.

На 2026-08-14 локальный `cargo-audit` не установлен, поэтому полный автоматический advisory scan `Cargo.lock` не заявляется выполненным. Direct dependency review выполнен вручную; `quick-xml = 0.41.0` удовлетворяет patched boundary `>= 0.41.0` для [RUSTSEC-2026-0195](https://rustsec.org/advisories/RUSTSEC-2026-0195.html). Автоматизация audit остаётся этапом 242.
