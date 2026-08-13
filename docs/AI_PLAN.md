# Текущий план AI — этап 001

**Статус:** завершён и проверен 2026-08-14.

## Цель

Создать минимальный собираемый monorepo-каркас без бизнес-логики и одновременно привести project overlay к каноническому глобальному контракту.

## Требования

- `NFR-FOUNDATION-001` — единый monorepo с изолированными Rust, Python и Next.js областями.
- `NFR-CONTEXT-001` — один системный SPEC, один текущий план и один снимок состояния без дублирования глобального контекста.

## Области файлов

- workspace manifests в корне;
- `crates/mathcad-parser`, `crates/math-engine`, `crates/exporter-docx`;
- `services/api`;
- `apps/web`;
- `specs/`, канонические документы `docs/` и validator проекта.

## Порядок

1. Мигрировать устаревшие имена и убрать obsolete context-pack документы и fallback-автоматизацию.
2. Добавить workspace manifests и минимальные собираемые каркасы без предметной логики.
3. Добавить структурную проверку и негативный тест.
4. Запустить доступные проверки, выполнить review и обновить `AI_STATUS`/traceability.

## Критерии приёмки

- `AC-FOUNDATION-001`: validator подтверждает канонические документы и отсутствие устаревших источников состояния.
- `AC-FOUNDATION-002`: Rust manifests образуют workspace из трёх существующих crates.
- `AC-FOUNDATION-003`: Python package собирается через `uv build`.
- `AC-FOUNDATION-004`: Next.js app проходит typecheck и production build.
- `AC-FOUNDATION-005`: в каркасах нет parser, conversion, API endpoint или UI-функций будущих этапов.

Все критерии этапа выполнены. `cargo check --workspace --locked` подтверждён в официальном контейнере Rust 1.85; остальные проверки перечислены в `docs/AI_STATUS.md` и итоговом отчёте этапа.

## Откат

Изменение ограничено одним Git-коммитом; откат выполняется отменой этого коммита. Миграций данных и внешнего состояния нет.
