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

## 2026-08-14 — Worksheet parser и Math AST, этапы 027–051

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

### Что получилось в коде

```text
WorksheetParser::parse(bytes)
  -> bounded XML tree builder
  -> exact worksheet30/version gate
  -> WorksheetMetadata
  -> recursive Region discovery
       -> Text / Math / Plot / Picture / Area / Opaque
  -> math30 parser
       -> MathExpression + SourceSpan
       -> Invalid(MathAstError) или Unsupported(diagnostic)
```

`SourceDocument` владеет immutable копией входных bytes. `SourceSpan { start, end }` — полуоткрытый диапазон: первый byte входит, byte с индексом `end` уже не входит. Opaque fragment хранит QName + span, поэтому неизвестный XML не теряется и не копируется в каждую структуру. Метод `source.bytes(span)` возвращает fragment только при корректных границах.

### Metadata, regions и порядок

`WorksheetMetadata` читает generator, identity/user fields и typed custom values. Содержимое comment/extensions остаётся opaque. Каждый `Region` содержит:

- `id` и `source_ordinal`;
- `RegionLayout` с исходной числовой лексемой и конечным `f64`;
- `z_order` отдельно;
- source span;
- typed или opaque content.

`worksheet.regions` всегда остаётся в document order. `visual_order()` возвращает отдельный стабильный view по `(top, left, source_ordinal)`, а `z_order()` — по `(z_order, source_ordinal)`. Это views из ссылок, а не перестановка исходного документа.

### Как устроен Math AST

Этапы 036–051 добавили только синтаксис:

- literals/identifiers и arithmetic;
- definitions, saved evaluation results, function calls/definitions;
- unary operations, grouping, literal subscript и array index;
- matrix/vector shape, range;
- integral, derivative, summation/product;
- six comparisons.

`RealLiteral` сохраняет lexeme и radix, потому что ранний перевод в floating point потерял бы точность. `Matrix` проверяет positive dimensions и `rows × cols == elements.len()` через checked arithmetic. `Vector` не ищется как XML element: это специализация `1×N` или `N×1` matrix. Calculus nodes сохраняют bound variable, body, bounds/degree и typed algorithm/style, но ничего не вычисляют.

### Parsed, Invalid и Unsupported — разные состояния

- `Parsed(ast)` — форма входит в подтверждённое подмножество и структурно корректна.
- `Invalid(error)` — QName известен, но arity, radix, shape или wrapper нарушены.
- `Unsupported(diagnostic)` — форма существует вне текущей области, например `ml:program` или boolean `not` до этапа 052.

Так damaged supported input не смешивается с корректной, но ещё не реализованной возможностью.

### Ограничения ресурсов

`WorksheetLimits` ограничивает input, XML depth/nodes, regions, namespaces, attributes, token/attribute/text bytes, AST nodes и matrix elements. Проверяются не только большие файлы, но и маленькие документы с патологически глубокой/широкой структурой. DTD запрещён до любого entity resolution; namespace attributes также декодируются и валидируются.

### Как читать тесты по этапам

1. `input_boundary.rs` — входной XMCD/MCDX perimeter.
2. `worksheet_structure.rs` — AC-027–035.
3. `math_ast.rs` — AC-036–037.
4. `math_ast_forms.rs` — AC-038–044.
5. `math_ast_advanced.rs` — AC-045–051.

Snapshots — обычные ожидаемые S-expression strings внутри tests. Если snapshot изменился, сначала нужно доказать изменение SPEC; автоматическое «принятие нового golden» запрещено.

### Что сознательно ещё не работает

- `cargo run` по-прежнему нечего запускать: в workspace нет CLI binary;
- parser не является evaluator;
- boolean AST начинается на этапе 052;
- units и generic `UnsupportedNode` — этапы 053–054;
- `DocumentIR`, DOCX, API и UI идут позже;
- внутренний Prime MCDX worksheet не передаётся legacy worksheet30 parser без отдельной схемы.

### Что нашли независимые reviews

Security review воспроизвёл memory amplification: один длинный namespace URI копировался в каждый XML node. Исправление интернирует URI как `Arc<str>`, поэтому все повторения делят одну строку; regression проверяет общую allocation через `Arc::ptr_eq`. Полный QName, включая prefix, теперь также ограничен. Отдельно custom `Debug` для чисел перестал выводить исходную лексему.

Code review нашёл три семантических края: direct extension внутри `userData` терялся, одноэлементный `sequence` принимался за multi-index, а `-0.0` сортировался раньше `+0.0`. После исправлений каждый случай имеет regression test. Это полезный пример: зелёный happy-path test не доказывает preservation, точную arity и стабильность математически равных layout values.

### Практика для самостоятельного повторения

1. Открыть один test `ac_045_and_046...` и вручную сопоставить XML children с AST fields.
2. Изменить только локально одну matrix dimension и увидеть typed `MatrixElementCountMismatch`.
3. Сравнить `id@subscript` с `apply/indexer/sequence` в тестах этапа 044.
4. Запустить `cargo test -p mathcad-parser --test math_ast_advanced --locked`.
5. Затем запустить полный `cargo test --workspace --locked` и Clippy.

## 2026-08-14 — Shared AST, Document IR и первый DOCX/OMML, этапы 052–076

### Зачем AST вынесен в `math-model`

Parser раньше владел всеми math types. Если бы `exporter-docx` зависел прямо от parser, Word-specific слой получил бы доступ к Mathcad XML и мог бы обойти pipeline. Поэтому source-neutral AST вынесен в `math-model`, а parser сохранил compatibility re-exports.

```text
Mathcad bytes -> mathcad-parser -> math-model AST
                                      |
                                      v
                                document-ir
                                      |
                                      v
                                exporter-docx
```

Boolean AST хранит binary `and/or/xor` отдельно от unary `not`. Unit powers используют `NonZeroI64` для denominator: невозможное состояние `0` отклоняется уже при XML/JSON parsing, а не проверяется случайным `if` в каждом consumer. Unknown nested math становится `UnsupportedNode` + diagnostic, а не исчезает.

### Почему Document IR — отдельный versioned contract

Document IR отделяет смысл документа от конкретного входного и выходного формата. V1 JSON имеет обязательный `schema_version = 1`, strict unknown-field rejection и bounded serialization. Physical lengths — integer micrometres, поэтому wire format не зависит от floating-point, locale или платформы.

`FormulaIr` различает:

- `original` — исходное выражение для provenance/audit;
- `display` — выражение, которое разрешено показывать после будущих transformations.

Exporter читает только `display`. Images содержат только `AssetRefIr`; bytes, paths и URLs не попадают в JSON. Это делает resolver явной trust boundary.

### Из чего состоит минимальный DOCX

DOCX — ZIP/OPC package, а не один XML-файл. Минимальный artifact содержит:

```text
[Content_Types].xml
_rels/.rels
word/document.xml
```

При images добавляются `word/_rels/document.xml.rels` и `word/media/imageN.png|jpg`. Relationship всегда internal. Порядок parts, `rId`, drawing IDs, compression и timestamps фиксированы, поэтому одинаковый IR и assets дают одинаковые bytes.

Text превращается в `w:p/w:r/w:t`; leading/trailing или repeated whitespace получает `xml:space="preserve"`. Page size/margins переводятся из micrometres в twips checked integer arithmetic, image size — по точному правилу `1 µm = 36 EMU`.

### Почему validator нужен даже собственному exporter

Writer и validator ловят разные классы ошибок. Writer отвечает за безопасное построение. Validator заново открывает ZIP как недоверенный и проверяет parts, content types, relationships, XML namespaces и структуру. Это позволяет тестировать package contract независимо и позже применять тот же строгий subset к передаче artifact между слоями.

Security review нашёл показательный asymmetry bug: generation соблюдал equation limits, validator — только более широкие XML limits. Исправление хранит exact source span XML-node и отдельно считает OMML semantic nodes/fraction depth. Три regression tests проверяют byte, node и depth budgets на недоверенном DOCX.

### Как OMML остаётся редактируемым

`WordEquationExporter` создаёт структурный `m:oMath`, а не screenshot:

- number и identifier → `m:r/m:t`;
- identifier получает italic math style;
- add/subtract/multiply → ordered runs и operator glyph;
- divide → `m:f` с `m:num` и `m:den`.

`OmmlFragment` имеет private field, поэтому внешний caller не может внедрить raw XML. Unsupported power/subscript/function и выражение, которому нужны ещё не реализованные brackets, дают typed error вместо тихого изменения смысла.

### Что можно повторить самостоятельно

1. Запустить `cargo test -p document-ir --locked` и открыть golden `crates/document-ir/tests/golden/document-ir-v1.json`.
2. Запустить `cargo test -p exporter-docx --test docx_foundation --locked` и сопоставить ZIP parts с тестом `minimal_docx_is_deterministic_and_valid`.
3. Запустить `cargo test -p exporter-docx --test omml --locked` и прочитать snapshots от number до nested fraction.
4. Запустить `cargo test -p exporter-docx --test docx_equations --locked` и увидеть, что exporter использует `display`, а не `original`.
5. Завершить полным `cargo fmt --all -- --check`, `cargo test --workspace --locked` и `cargo clippy --workspace --all-targets --locked -- -D warnings`.
