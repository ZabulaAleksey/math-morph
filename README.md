# MathMorph

MathMorph — monorepo расширяемой платформы parsing и конвертации Mathcad. Первый продуктовый путь: `.xmcd`/`.mcdx` → редактируемый DOCX/OMML.

## Состояние

Проект находится на этапе базового bootstrap. Предметная логика parser, math-engine, exporter, API и web ещё не реализована.

## Структура

- `crates/` — Rust core и exporters;
- `services/api/` — Python package будущего FastAPI adapter;
- `apps/web/` — минимальный Next.js App Router shell;
- `specs/` — канонические требования;
- `docs/` — архитектура, решения, план, статус и предметные контракты;
- `tests/` — project-level проверки и будущие fixtures.

## Быстрая проверка

```powershell
python scripts/validate_project.py
python -m unittest discover -s tests -p "test_*.py"
uv build --project services/api
pnpm.cmd install --frozen-lockfile
pnpm.cmd --filter @math-morph/web typecheck
pnpm.cmd --filter @math-morph/web build
cargo check --workspace
```

Команда `cargo check` требует установленного Rust toolchain. Перед изменениями прочитай корневой и ближайший модульный `AGENTS.md`, затем выбери требования через `specs/README.md`.
