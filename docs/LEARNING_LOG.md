# Учебный журнал

Этот файл объясняет воспроизводимые действия и устройство проекта. Это не скрытые рассуждения AI и не замена `AI_STATUS.md`.

## 2026-08-14 — Как запускать проект и что делает Cargo

### Из чего состоит MathMorph

MathMorph — monorepo: один Git-репозиторий содержит несколько языков и модулей. Сейчас Rust — ядро обработки документов, Python — будущая API-обвязка, Next.js — будущий web-интерфейс.

```text
input .xmcd/.mcdx
        |
        v
crates/mathcad-parser      проверка и синтаксический разбор
        |
        v
crates/math-engine        будущая семантика/вычисления
        |
        v
crates/exporter-docx      будущий DOCX/OMML

services/api и apps/web   будущая пользовательская граница
```

Наличие каталогов не означает, что весь поток уже работает: текущую правду всегда показывает `docs/AI_STATUS.md`.

### Что такое Rust, rustup, rustc и Cargo

- `rustup` устанавливает toolchains и выбирает нужную версию Rust.
- `rustc` — компилятор одного Rust crate.
- `cargo` — package manager и build tool: читает manifests, разрешает зависимости, вызывает `rustc`, запускает tests/Clippy и управляет workspace.
- `Cargo.toml` описывает workspace/packages/dependencies.
- `Cargo.lock` фиксирует точные версии зависимостей для воспроизводимой сборки.
- `rust-toolchain.toml` закрепляет Rust 1.88 для этого репозитория. Поэтому глобальная версия может быть 1.97, а внутри проекта автоматически активируется 1.88 — это нормально.

### Почему команды обязательно запускать из корня

Cargo ищет `Cargo.toml` в текущем каталоге и выше. Если PowerShell стоит в `C:\Users\aleks`, он не знает о репозитории и пишет `could not find Cargo.toml`.

```powershell
cd ~/codex-workspace/projects/math-morph
Test-Path Cargo.toml
git status --short --branch
rustup show active-toolchain
```

Первый результат должен быть `True`, а `git status` — показывать ветку MathMorph.

### Зачем Windows нужен `link.exe`

`cargo check` всё равно может собирать build scripts/proc macros. Target `x86_64-pc-windows-msvc` завершает эту работу Microsoft linker `link.exe`. Он поставляется не с VS Code и не с rustup, а с Visual Studio Build Tools.

Нужна установка Visual Studio 2022 Build Tools:

1. выбрать workload `Desktop development with C++`;
2. оставить MSVC toolset и Windows SDK;
3. завершить установку и перезапустить PowerShell;
4. при необходимости открыть `Developer PowerShell for VS 2022`;
5. проверить `Get-Command link.exe` и повторить Cargo command.

Ошибка `link.exe not found` означает проблему окружения сборки, а не дефект кода MathMorph.

### Что делают основные Cargo-команды

```powershell
cargo check --workspace --locked
cargo test --workspace --locked
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
```

- `check` быстро проверяет компиляцию без итоговых executable.
- `test` компилирует и запускает unit/integration tests.
- `fmt --check` проверяет форматирование, ничего не переписывая.
- `clippy ... -D warnings` выполняет строгий статический анализ и делает warning ошибкой.
- `--workspace` охватывает все Rust crates.
- `--locked` требует использовать существующий `Cargo.lock` без обновления.

### Полная локальная проверка

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

В PowerShell используется `pnpm.cmd`, чтобы не упереться в execution policy для `pnpm.ps1`.

### Типовые ошибки

| Сообщение | Причина | Что сделать |
|---|---|---|
| `cargo is not recognized` | Rust не установлен или PATH терминала устарел | установить rustup, закрыть и открыть терминал, проверить `cargo --version` |
| `could not find Cargo.toml` | неверный текущий каталог | `cd ~/codex-workspace/projects/math-morph` |
| `link.exe not found` | нет MSVC C++ toolchain/Developer environment | установить Build Tools + C++ workload, перезапустить терминал |
| активен Rust 1.88 вместо глобального 1.97 | сработал project pin | это ожидаемо; проверить `rust-toolchain.toml` |
| `Cargo.lock needs to be updated` с `--locked` | manifest и lockfile расходятся | не удалять флаг; обновить lockfile осознанно в отдельном change |

## 2026-08-14 — Безопасная граница Mathcad input, этапы 011–026

### Что изменено

- `FormatDetector` определяет XMCD/MCDX по байтам; расширение используется только для диагностики.
- `SafeMcdxReader` проверяет ZIP metadata, имена, collisions, compression/size limits и фактически читает entries в ограниченном режиме без записи на диск.
- XML inspector принимает только UTF-8, запрещает DTD/entities и читает только root namespace/schema envelope.
- Fixture corpus имеет versioned manifest и fail-closed validator.

### Почему это отдельный слой

Документ — недоверенный ввод. До содержательного parsing нужно доказать, что archive paths, размеры, XML encoding и namespaces безопасны. CRC32 проверяет случайное повреждение, но не является security integrity mechanism.

```text
bytes -> format detection -> ZIP/XML boundary checks -> worksheet parser
```

URI из XML сохраняются как строки metadata; сеть не вызывается. Entries не извлекаются на filesystem.

### Что нашёл review

Review выявил drive-relative ZIP paths, unchecked offset arithmetic, неполную проверку XML attributes и неточное сопоставление namespace-limit error. Исправления получили отдельные regression tests — именно так review превращается в долговременную защиту.

## 2026-08-14 — План worksheet parser и Math AST, этапы 027–051

### Почему сначала понадобилась SPEC

Короткие названия дорожной карты недостаточны для XML parser. Проверка официальных `worksheet30.xsd`/`math30.xsd` показала три важных расхождения с бытовыми терминами:

- table не является самостоятельным worksheet region: это opaque reference внутри `resultFormat`;
- program — `ml:program` внутри математического выражения, а не region;
- отдельного `ml:vector` нет: row/column vector кодируется `ml:matrix`.

Реализация обязана следовать expanded QName `(namespace URI, local name)`, а не prefix `ws`/`ml`: prefix пользователь может переименовать без изменения XML смысла.

### Разные виды порядка

- source order нужен для воспроизводимости и семантической последовательности;
- visual order можно вычислить стабильно по координатам;
- z-order означает порядок рисования перекрывающихся объектов.

Смешивание этих порядков — тихая ошибка: документ выглядит почти правильно, но определения или layout могут поменять смысл.

### Где заканчивается этот блок

Этапы 027–051 строят только syntax tree. Они не вычисляют формулы. Boolean operations начинаются с 052; units, generic unsupported nodes, `DocumentIR`, export, API и UI идут позже. Такое ограничение сохраняет архитектурную цепочку parser → semantics → IR → exporters.

### Как повторить исследование безопасно

1. Открыть `specs/features/worksheet-structure-and-ast.spec.md` и найти `AC-027..051`.
2. Сравнить термины `table`, `program`, `vector` с разделом «Вне области» и ADR-0007.
3. После реализации запустить validators и Rust format/test/clippy с `--locked`.
4. Проверить `docs/TRACEABILITY.md`: `verified` допустим только рядом с конкретными tests/review evidence.
