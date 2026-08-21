# MathMorph

MathMorph — monorepo расширяемой платформы parsing и конвертации Mathcad. Первый продуктовый путь: `.xmcd`/`.mcdx` → редактируемый DOCX/OMML.

## Текущее состояние

Реализованы и проверены этапы 001–093, 095–122, 143–154 и независимый frontend-этап 154: безопасная входная граница, legacy XMCD worksheet30 parser, структурный Math AST, presentation/semantic math engine, versioned Document IR, общий conversion core, детерминированный DOCX/OMML exporter и расширенный локальный CLI. Этапы 123–142 зависят от подтверждённых plot/diagram payload fixtures и остаются evidence-gated; этап 094 зависит от live MathType evidence. Parser сохраняет metadata, regions/layout/source spans и unsupported fragments, но полный worksheet evaluator пока не подключён. API endpoints и интерактивный web converter flow ещё не реализованы; Prime MCDX безопасно определяется, но не имеет content parser.

## Структура

- `crates/mathcad-parser/` — Rust parser недоверенных Mathcad inputs;
- `crates/math-model/` — общая source-neutral Math AST;
- `crates/document-ir/` — backend-neutral модель документа и exporter ports;
- `crates/math-engine/` — bounded Original AST→Display AST presentation transforms; evaluation остаётся будущим этапом;
- `crates/exporter-docx/` — DOCX/WordprocessingML и редактируемый OMML subset;
- `crates/exporter-mathml/` — Presentation MathML Core renderer;
- `crates/exporter-mathtype/` — experimental opaque MathML payload adapter без SDK/OLE/DOCX wiring;
- `crates/conversion-core/` — общий XMCD→Document IR→DOCX orchestration, diagnostics/report и partial policy;
- `crates/mathmorph-cli/` — локальный binary `mathmorph`;
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

Подробный самостоятельный workflow с шаблонами prompts, матрицей тестов, Definition of Done и handoff при окончании лимита находится в [`docs/SELF_GUIDED_STAGE_WORKFLOW.md`](docs/SELF_GUIDED_STAGE_WORKFLOW.md).

## Что можно запустить сейчас

Собрать и запустить реальную конвертацию legacy XMCD→DOCX:

```powershell
cargo build -p mathmorph-cli --release --locked
./target/release/mathmorph.exe convert ./path/to/input.xmcd --to docx
```

По умолчанию рядом создаётся `input.docx`. Явный output задаётся через `--output ./path/to/result.docx`; существующий файл не перезаписывается. Prime `.mcdx` пока возвращает `MCDX_CONTENT_UNSUPPORTED` без output.

Проверить структуру входа, выполнить полный validation path без публикации DOCX или экспортировать versioned Document IR:

```powershell
./target/release/mathmorph.exe inspect ./path/to/input.xmcd
./target/release/mathmorph.exe validate ./path/to/input.xmcd
./target/release/mathmorph.exe export-ir ./path/to/input.xmcd --output ./path/to/input.ir.json
```

Для `convert` также доступны alias `--format`, режим представления комплексных чисел и precision policy:

```powershell
./target/release/mathmorph.exe convert ./path/to/input.xmcd --format docx --complex-mode both --precision 15
```

Registry распознаёт `docx`, `markdown`, `latex`, `html`, `pdf`, `json` и `typst`, но production exporter пока существует только для `docx`; остальные известные форматы возвращают `EXPORTER_UNAVAILABLE` до чтения input.

Сгенерировать проверенный DOCX с редактируемой формулой и открыть его в Word:

```powershell
cargo run -p exporter-docx --example advanced_omml_reference
Invoke-Item target/word-reference/advanced-omml-reference.docx
```

Проверить standalone MathML snapshots и experimental MathType payload adapter:

```powershell
cargo test -p exporter-mathml --locked
cargo test -p exporter-mathtype --locked
Get-Content crates/exporter-mathml/tests/golden/add.mathml
```

Запустить Next.js shell:

```powershell
pnpm.cmd run dev:web
```

После этого откройте `http://localhost:3000`. Маршрут `/` показывает Calm Blue landing shell; выбор файла и подключение web UI к conversion core относятся к следующим frontend/API adapter этапам.

Python package можно установить и проверить, но HTTP-сервер запускать пока нечего:

```powershell
uv sync --project services/api --locked
uv run --project services/api python -c "import math_morph_api; print(math_morph_api.__doc__)"
```
