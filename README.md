# MathMorph

MathMorph — monorepo расширяемой платформы parsing и конвертации Mathcad. Первый продуктовый путь: `.xmcd`/`.mcdx` → редактируемый DOCX/OMML.

## Текущее состояние

Реализованы и проверены этапы 001–092: безопасная входная граница, чтение подтверждённого legacy XMCD worksheet30, структурный Math AST, versioned Document IR, детерминированный DOCX/OMML exporter, bounded Presentation MathML renderer с golden snapshots и pure experimental MathType payload adapter без SDK/OLE/DOCX wiring. Parser сохраняет metadata, regions/layout/source spans и unsupported fragments, но формулы пока не вычисляет. API endpoints и пользовательский web-flow ещё не реализованы; Prime MCDX безопасно инспектируется как контейнер без содержательного разбора внутреннего worksheet.

## Структура

- `crates/mathcad-parser/` — Rust parser недоверенных Mathcad inputs;
- `crates/math-model/` — общая source-neutral Math AST;
- `crates/document-ir/` — backend-neutral модель документа и exporter ports;
- `crates/math-engine/` — каркас будущей семантики и вычислений;
- `crates/exporter-docx/` — DOCX/WordprocessingML и редактируемый OMML subset;
- `crates/exporter-mathml/` — Presentation MathML Core renderer;
- `crates/exporter-mathtype/` — experimental opaque MathML payload adapter без SDK/OLE/DOCX wiring;
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

После этого откройте `http://localhost:3000`. Сейчас маршрут `/` намеренно возвращает пустую страницу: ветка содержит утверждённый Calm Blue design contract, но React-компоненты пользовательского flow ещё не реализованы.

Python package можно установить и проверить, но HTTP-сервер запускать пока нечего:

```powershell
uv sync --project services/api --locked
uv run --project services/api python -c "import math_morph_api; print(math_morph_api.__doc__)"
```
