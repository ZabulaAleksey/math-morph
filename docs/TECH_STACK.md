# Технологический стек

## Frontend

- Next.js + React + TypeScript
- Tailwind CSS + shadcn/ui
- TanStack Query для серверного состояния
- Zustand для небольшого локального состояния UI
- React Hook Form + Zod
- next-intl и каталоги, совместимые с ICU
- Web Workers для неблокирующей локальной конвертации
- WebAssembly для core Rust в браузере
- Web Crypto API для клиентской криптографии

## Mathcad core

Язык: **Rust**.

Модули и crates должны развиваться вокруг:

- определения формата и безопасного чтения контейнера;
- parsing XMCD/MCDX (`quick-xml`, Serde, где уместно);
- Mathcad AST;
- семантического анализатора, таблицы символов и графа зависимостей;
- evaluator и `EvaluationTrace`;
- AST преобразований и отображения;
- поддержки комплексных чисел (`num-complex`);
- линейной алгебры при необходимости (`nalgebra`);
- `DocumentIR`.

## Уравнения и документы

- DOCX / Office Open XML / WordprocessingML
- OMML / Office Math для нативных редактируемых уравнений Word
- MathML как слой совместимости для необязательной поддержки MathType
- будущие контракты exporters для Markdown, LaTeX, HTML, JSON и PDF
- `PlotIR`/`ChartIR` для будущих диаграмм Excel
- `DiagramIR` для будущего редактируемого вывода VSDX/Visio

## Backend/API

- Python + FastAPI
- Pydantic
- SQLAlchemy + Alembic
- HTTPX при необходимости
- управление зависимостями Python: **uv**, проектный `.venv`, общий кэш
- публичный API, ориентированный на OpenAPI 3.1

## Данные и задачи

- PostgreSQL — авторитетные метаданные
- RabbitMQ — broker задач
- Celery — workers серверной конвертации
- Redis — только кэш, rate limit и временная координация
- S3-совместимое объектное хранилище; MinIO для локальной разработки

## Аутентификация

- Keycloak или эквивалентная основанная на стандартах граница идентификации
- OAuth 2.0 / OpenID Connect
- TOTP
- WebAuthn/passkeys
- коды восстановления
- подтверждённое восстановление через email или телефон
- Telegram Bot API для явной привязки учётной записи и необязательного восстановления

## Оплата

Абстракция provider; региональные и международные адаптеры могут включать LiqPay, WayForPay и Merchant of Record, например Paddle, где это уместно юридически и операционно. Не фиксируй одного provider в основной бизнес-логике.

## Testing

Rust:
- `cargo test`
- property-based тестирование (`proptest`)
- fuzzing (`cargo-fuzz`)
- snapshots и эталонные тесты AST/IR, где уместно

Python:
- pytest
- pytest-asyncio
- Hypothesis
- интеграционные тесты HTTPX

Frontend:
- Vitest
- React Testing Library
- Playwright

## Наблюдаемость

- OpenTelemetry
- Prometheus
- Grafana
- Sentry и отслеживание ошибок со строгим скрытием данных

## Инфраструктура

Изначально:
- Docker + Docker Compose
- GitHub Actions

Позднее, только при наличии оснований:
- Terraform/OpenTofu
- Kubernetes
- Helm

## Безопасность и цепочка поставки

- lockfiles и воспроизводимые сборки
- аудит зависимостей и поиск секретов в CI
- MCP, plugins, Skills и hooks считаются зависимостями цепочки поставки
- отсутствие непроверенных зависимостей `latest` для долговечной инфраструктуры
- соответствие OWASP Top 10:2025 в `SECURITY.md`

## Интеграция AI/Codex

Глобальная AI Dev Team владеет универсальными инженерными ролями и workflows. Проектный overlay содержит только специфичных для Mathcad агентов и skills. Проектные hooks и общие MCP-серверы по умолчанию отключены, чтобы избежать дублирования выполнения и роста поверхности контекста и инструментов.
