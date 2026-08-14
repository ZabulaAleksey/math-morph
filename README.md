# MathMorph

MathMorph — monorepo расширяемой платформы parsing и конвертации Mathcad. Первый продуктовый путь: `.xmcd`/`.mcdx` → редактируемый DOCX/OMML.

## Текущее состояние

Реализованы безопасная входная граница, чтение подтверждённого legacy XMCD worksheet30 и синтаксический Math AST до structural comparisons (этапы 001–051). Parser сохраняет metadata, regions/layout/source spans и unsupported fragments, но не вычисляет формулы. Экспорт, API endpoints и пользовательский web-flow ещё не реализованы; Prime MCDX пока безопасно инспектируется как контейнер без содержательного разбора его внутреннего worksheet.

## Структура

- `crates/mathcad-parser/` — Rust parser недоверенных Mathcad inputs;
- `crates/math-engine/` — каркас будущей семантики и вычислений;
- `crates/exporter-docx/` — каркас будущего DOCX/OMML exporter;
- `services/api/` — Python package будущего FastAPI adapter;
- `apps/web/` — минимальный Next.js App Router shell;
- `specs/` — канонические проверяемые требования;
- `docs/` — архитектура, решения, планы, статус и учебные записи;
- `tests/fixtures/` — synthetic regression corpus и manifest.

## Подготовка Windows

Нужны Git, Python, Node.js + pnpm, uv и Rust. Для Rust MSVC установите Visual Studio 2022 Build Tools с workload `Desktop development with C++`; VS Code сам по себе не содержит `link.exe`.

Откройте PowerShell и перейдите именно в корень репозитория:

```powershell
cd ~/codex-workspace/projects/math-morph
Test-Path Cargo.toml
rustup show active-toolchain
cargo --version
```

`Test-Path` должен вернуть `True`. Файл `rust-toolchain.toml` автоматически выбирает Rust 1.88 для этого проекта.

## Основные проверки

```powershell
python -B scripts/validate_project.py
python -B scripts/validate_fixtures.py
python -B -m unittest discover -s tests -p "test_*.py" -v
cargo fmt --all -- --check
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
uv build --project services/api
pnpm.cmd install --frozen-lockfile
pnpm.cmd --filter @math-morph/web typecheck
pnpm.cmd --filter @math-morph/web build
```

`--locked` запрещает Cargo незаметно менять `Cargo.lock`. Если Cargo сообщает, что не найден `Cargo.toml`, команда запущена не из репозитория. Если не найден `link.exe`, установите C++ Build Tools и перезапустите терминал; подробнее — в `docs/LEARNING_LOG.md`.

Перед изменениями прочитайте корневой и ближайший модульный `AGENTS.md`, затем выберите требования через `specs/README.md`.
