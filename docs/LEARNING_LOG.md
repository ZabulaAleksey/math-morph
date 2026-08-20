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

## 2026-08-14 — Этапы 077–089: расширенный OMML, лимиты и проверка Word

### Что изменилось

В `exporter-docx` расширен только Word-specific слой: `WordEquationExporter` теперь строит канонические редактируемые OMML для powers, roots, literal subscripts, canonical sub+sup, typed function calls, парной grouping, vector/matrix и non-presentational integral/derivative/sum/product. Новые shapes проходят через строгий `DocxValidator`; неподдержанные и неоднозначные формы завершаются typed fail-closed error. `EquationBackend::WordOmml` выбран default через `DocxExportConfig`, а `MathType` зарезервирован без MathML, OLE, dependency или скрытого fallback. В `math-model` и `mathcad-parser` в итоговом scope этой работы изменений нет.

### Почему resource limits должны быть симметричными

Один writer не является достаточной защитой: полученный DOCX может быть создан другим producer или изменён между генерацией и проверкой. Поэтому renderer и validator считают одинаковые content-bearing equation nodes, equation depth и exact source bytes. Для длинных left-associated linear expressions используется итеративный обход с отдельным `linear_work_items` budget. Это отделяет две ошибки:

1. рекурсивная форма превышает допустимую OMML depth;
2. плоская, но очень длинная форма превышает node/work budget.

Практический вывод: «нет рекурсии в одном месте» не означает «нет DoS-риска». Нужно ограничивать и глубину, и количество работы, а writer и validator должны применять одинаковую модель счёта.

### Как читать decoder trust boundary

`VersionedDocumentIr::from_json_with_limit` ограничивает bytes, затем дважды проходит через `serde_json::from_slice` с default recursion limit и только после этого вызывает IR validation глубины/узлов. Это не следует смешивать в одну абстрактную «защиту JSON»: decoder recursion и semantic depth — разные рубежи. Прямой Rust/custom `Deserialize` не является bounded public input path: caller сам отвечает за ограниченный reader, конфигурацию decoder и последующую validation. XML path отдельно использует explicit parser stack depth.

### Word/Open XML evidence и troubleshooting

Команда `cargo run -p exporter-docx --example advanced_omml_reference` создаёт воспроизводимый reference artifact. Word 16.0 открыл его и exposed 1 `OMath`; изолированный `Linearize→BuildUp` edit check сохранил 1 `OMath`. Локальный Microsoft Open XML SDK 2.5.4728 validator сообщил 0 errors.

Первоначальная объединённая попытка «открыть → перечислить → сохранить» превысила timeout; созданный процесс Word был остановлен. Это было свойством сценария автоматизации, а не ограничением продукта. Последующие изолированные open/enumerate/edit checks прошли, поэтому в документации фиксируется именно evidence успешных проверок и кратко — способ диагностики timeout.

### Проверки

- `python -B scripts/validate_project.py` — PASS;
- `python -B scripts/validate_fixtures.py` — PASS;
- `python -B -m unittest discover -s tests -p "test_*.py" -v` — PASS, 15/15;
- `cargo fmt --all -- --check` — PASS;
- `cargo test --workspace --locked` — PASS, 92 Rust tests;
- `cargo clippy --workspace --all-targets --locked -- -D warnings` — PASS;
- automated review и security review — PASS после исправлений;
- `git diff --check` — PASS.

### Что можно повторить самостоятельно

1. Запустить `cargo test -p exporter-docx --test advanced_omml --locked` и сопоставить каждый новый OMML shape с проверкой allowlist.
2. Запустить `cargo test -p exporter-docx --test omml --locked` и найти regression для iterative linear traversal/work budget.
3. Запустить `cargo run -p exporter-docx --example advanced_omml_reference`, открыть полученный DOCX в Word и проверить, что в нём остаётся один редактируемый `OMath`.
4. Запустить `python -B scripts/validate_project.py` и `python -B -m unittest discover -s tests -p "test_*.py" -v`, чтобы отделить project gates от Rust gates.
5. Запустить `cargo test --workspace --locked`, затем `cargo clippy --workspace --all-targets --locked -- -D warnings` и проверить `git diff --check`.

## 2026-08-15 — Этап 090: первый самостоятельный Presentation MathML renderer

### Что и зачем изменено

Добавлен crate `exporter-mathml`. Он преобразует базовое scalar-подмножество общего `MathExpression` в standalone Presentation MathML Core: числа, identifiers и literal subscripts, сложение/вычитание/умножение, дробь, степень, квадратный корень и парную grouping. Это отдельный output backend, а не часть DOCX: Word продолжает получать OMML, а зарезервированный `MathType` всё ещё возвращает typed unavailable error.

### Ключевой поток данных / управления

```text
MathExpression
    -> Accountant: supported shape + depth/nodes/input bytes
    -> iterative Renderer: fixed MathML allowlist + escaping + output bytes
    -> opaque MathMlFragment
```

`MathMlRenderer` реализует тот же `document_ir::EquationExporter`, что и Word renderer. Поэтому consumer работает через общий порт, но конкретный output остаётся типизированным: `MathMlFragment` нельзя сконструировать с raw XML снаружи crate.

Корень всегда один и детерминирован:

```xml
<math xmlns="http://www.w3.org/1998/Math/MathML" display="block">...</math>
```

Presentation MathML описывает структуру отображения. Например, дробь — это `mfrac`, степень — `msup`, а скобки для MathML Core разворачиваются в `mrow` и два `mo fence="true"`; `mfenced` намеренно не используется. Это не Content MathML и пока не обещание совместимости с конкретной версией MathType.

### Команды и проверки

```text
cargo test -p exporter-mathml --locked
cargo test --workspace --locked
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
python -B scripts/validate_project.py
python -B -m unittest discover -s tests -p "test_*.py" -v
git diff --check
```

### Решения и trade-offs

- Новый crate не зависит от DOCX, parser или Office: только от общей модели, exporter port и `thiserror`.
- Этап 090 поддерживает небольшой явно записанный subset. Неподдержанная форма возвращает error, а не похожий текст и не screenshot.
- Renderer дважды защищает ресурсы: сначала считает AST nodes/depth и cumulative dynamic text bytes, затем ограничивает фактический output. Оба обхода итеративны.
- Renderer заимствует AST и поэтому не владеет её уничтожением. Безопасный caller должен получать выражение через bounded Document IR boundary; вручную построенное чрезмерно глубокое дерево caller обязан разбирать безопасно сам.

### Проблемы и способы исправления

Security review заметил, что первоначальная проверка identifier/numeric text могла полностью просканировать огромную строку до output-limit. Теперь byte length начисляется до content scan, а numeric validator использует boolean вместо потенциально переполняемого счётчика. Дополнительно project validator проверяет не только `[dependencies]`, но и build/dev/target-specific Cargo dependency tables.

### Как повторить самостоятельно

1. Открыть `crates/exporter-mathml/tests/mathml.rs` и сопоставить AST helpers с `mn`, `mi`, `mrow`, `mfrac`, `msup` и `msqrt`.
2. Запустить `cargo test -p exporter-mathml --locked` и убедиться, что positive, escaping, unsupported и limit cases проходят.
3. Изменить один expected operator code point в тесте и увидеть, что deterministic policy обнаруживает расхождение; затем вернуть изменение.
4. Запустить `python -B scripts/validate_project.py`, чтобы проверить workspace registration и dependency boundary.
5. Завершить workspace tests, Clippy и `git diff --check` командами выше.

## 2026-08-15 — Этап 091: reviewable MathML golden snapshots

### Что и зачем изменено

К существующему renderer добавлен не новый production behavior, а внешний регрессионный контракт: 17 небольших `.mathml` файлов. Они показывают точный output для четырёх numeric bases, identifiers/escaping/subscript, арифметики, четырёх визуально различающихся multiplication policies, дроби, степени, корня и grouping.

Inline assertion полезен разработчику, но отдельный golden-файл удобнее проверять в Git diff: изменение namespace, code point оператора, порядка tags или escaping видно как обычное изменение артефакта. Тест не умеет автоматически «благословлять» новый output, поэтому упавший snapshot нельзя случайно обновить вместе с ошибочной реализацией.

### Ключевой поток данных / управления

```text
synthetic MathExpression
    -> production MathMlRenderer
    -> MathMlFragment bytes
    -> exact comparison with tests/golden/<case>.mathml minus one final LF
```

Отдельный guard проверяет exact inventory, UTF-8 без BOM, отсутствие CR, compact single-line payload, ровно один финальный LF и один внешний `math` root. Второй regression меняет `ExpressionOrigin` на каждом вложенном узле и доказывает, что provenance не попадает в output.

### Команды и проверки

```text
cargo test -p exporter-mathml --test mathml_snapshots --locked
git check-attr text eol -- crates/exporter-mathml/tests/golden/add.mathml
git ls-files --eol -- crates/exporter-mathml/tests/golden/*.mathml
cargo test --workspace --locked
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
python -B scripts/validate_project.py
git diff --check
```

### Решения и trade-offs

- Golden corpus покрывает только уже поддержанные формы 090 и не расширяет renderer.
- Каждый файл является standalone MathML, а не общей текстовой сводкой: его можно открыть и проверять отдельно.
- Один финальный LF сохраняет обычный text-file workflow, а тест сравнивает точный payload после удаления только этого LF.
- Dot snapshot представляет одинаковый output `Default`/`AutoSelect`/трёх dot styles; существующий enum-level test продолжает проверять все варианты.

### Проблемы и способы исправления

На Windows `core.autocrlf=true` мог после checkout заменить LF на CRLF и сломать byte-level test. Scoped `.gitattributes` закрепляет `text eol=lf`; `git ls-files --eol` подтверждает `i/lf w/lf` для всех 17 fixtures.

Первый root guard считал только точную opening-строку и мог пропустить вложенный `<math>`, маскирующий незакрытый внешний root. Теперь guard выделяет body между exact prefix/suffix и запрещает любые дополнительные `<math`/`</math>`; malformed примеры находятся в отдельном negative test.

### Как повторить самостоятельно

1. Запустить targeted snapshot test и открыть любой файл из `crates/exporter-mathml/tests/golden/`.
2. Временно изменить один operator code point в golden-файле и увидеть точный mismatch; затем вернуть файл.
3. Добавить временный лишний `.mathml` и убедиться, что inventory guard падает; затем удалить файл.
4. Выполнить `git check-attr` и `git ls-files --eol`, чтобы проверить LF policy на Windows.
5. Завершить workspace test/Clippy/validator/diff checks командами выше.

## 2026-08-15 — Интеграция веток MathMorph и фактический запуск проекта

### Что объединено

Git graph показал, что `feature/stage-091` уже содержит `main` и все этапные ветки 001–090. Независимой оставалась только `docs/adopt-calm-blue-design`, поэтому временная ветка `integration/math-morph` создана от этапа 091 и получила один merge этой design-ветки. Такой порядок не дублирует коммиты и сохраняет историю обеих линий.

Единственный content conflict возник в `docs/AI_STATUS.md`: актуальный статус 001–091 объединён с Calm Blue design contract. После проверки временная ветка была fast-forward слита в `main`; все завершённые локальные и remote-ветки являются предками `main`.

### Что реально запускается

Сейчас рабочая часть проекта — Rust core и exporters. `services/api` является устанавливаемым Python package без HTTP endpoints, а `apps/web` — собираемым Next.js shell, где `/` пока возвращает `null`. Поэтому `pnpm.cmd run dev:web` запускает сервер, но браузер показывает пустую страницу; это ожидаемое состояние, а не runtime error.

Наглядный результат уже можно получить через DOCX example:

```powershell
cargo run -p exporter-docx --example advanced_omml_reference
Invoke-Item target/word-reference/advanced-omml-reference.docx
```

Файл содержит синтетическую редактируемую OMML-формулу. Reviewable MathML output хранится в `crates/exporter-mathml/tests/golden/`.

### Проверенный Windows workflow

```powershell
cd ~/codex-workspace/projects/math-morph

cargo test --workspace --locked
python -B scripts/validate_project.py
python -B scripts/validate_fixtures.py
python -B -m unittest discover -s tests -p "test_*.py" -v

uv sync --project services/api --locked
uv run --project services/api python -c "import math_morph_api; print(math_morph_api.__doc__)"

pnpm.cmd install --frozen-lockfile
pnpm.cmd run typecheck
pnpm.cmd run build:web
pnpm.cmd run dev:web
```

В PowerShell используется `pnpm.cmd`, потому что локальная execution policy может блокировать `pnpm.ps1`. Для Rust MSVC по-прежнему нужен Visual Studio Build Tools workload `Desktop development with C++`, предоставляющий `link.exe`.

### Проверки интеграции

- project и fixture validators — PASS;
- Python unittest — PASS, 18/18;
- Rust workspace tests — PASS, 102 tests;
- Rustfmt и Clippy с `-D warnings` — PASS;
- Next.js typecheck и production build — PASS;
- DOCX example generation — PASS;
- `git diff --check` — PASS.

### Как повторить самостоятельно

1. Выполнить `git log --graph --oneline --decorate --all` и найти design merge-коммит в истории `main`.
2. Запустить полный Rust test suite и Python validators.
3. Запустить `pnpm.cmd run dev:web`, открыть `http://localhost:3000` и убедиться, что пустой экран пока ожидаем.
4. Сгенерировать DOCX example и открыть его в Word.
5. Открыть несколько `.mathml` файлов из golden corpus и сопоставить их с тестами renderer.

## 2026-08-15 — Руководство для самостоятельного выполнения этапов

### Что и зачем изменено

Добавлен `docs/SELF_GUIDED_STAGE_WORKFLOW.md`: единый воспроизводимый маршрут для владельца проекта и ChatGPT от выбора одного этапа до статуса `verified`. Руководство отделяет roadmap/prompt от канонической SPEC, объясняет ветвление и checkpoint, содержит готовые prompts, матрицу команд для Rust/Python/frontend/API, Definition of Done, review-процесс и handoff при окончании лимита.

Project validator теперь требует этот документ и ключевые разделы DoD/handoff/final report. Негативный unit test доказывает, что пустая или урезанная заглушка не заменит living contract незаметно.

Одновременно устранён устаревший status context: `AI_PLAN` больше не утверждает активность завершённой ветки 091, а `ROADMAP` указывает интеграцию этапов 001–091 в `main`.

### Команды проверки

```text
python -B scripts/validate_project.py
python -B -m unittest discover -s tests -p "test_*.py" -v
git diff --check
```

### Как повторить самостоятельно

1. Открыть `docs/SELF_GUIDED_STAGE_WORKFLOW.md` и заполнить starter prompt номером следующего этапа.
2. Сопоставить test matrix с изменяемым модулем и выбрать targeted commands.
3. Проверить будущий этап по Definition of Done до присвоения статуса `verified`.
4. При остановке заполнить handoff template и отправить временную ветку.

## 2026-08-17 — Experimental MathType adapter, этап 092

### Зачем понадобился отдельный adapter

`MathMlRenderer` уже создаёт bounded allowlist Presentation MathML, но непосредственное подключение MathType SDK, OLE/COM, Word automation или сетевого WIRIS service одновременно добавило бы platform-, license- и security-boundary. Этап 092 поэтому реализует только чистую внутреннюю границу данных:

```text
MathExpression
  -> MathMlRenderer
  -> MathTypeAdapter
  -> opaque application/mathml+xml payload
```

Это проверяет направление зависимостей и будущую точку интеграции, не создавая ложного заявления о совместимости с конкретной версией MathType.

### Что добавлено

- новый crate `exporter-mathtype`;
- `MathTypeAdapter`, реализующий общий `EquationExporter`;
- read-only `MathTypePayload` без публичного конструктора из raw XML;
- typed/redacted `MathTypeError`;
- только internal path dependencies на `math-model`, `document-ir` и `exporter-mathml`;
- positive, port, unsupported, depth/node/output-limit и privacy regressions;
- отдельный SPEC и repository dependency guard.

### Что доказывают тесты

- supported scalar expression даёт payload byte-for-byte равный production `MathMlRenderer`;
- unsupported/invalid input и все три budget categories завершаются fail closed;
- payload/error Debug не раскрывает formula text;
- новый crate не активирует `EquationBackend::MathType` в DOCX;
- workspace dependency DAG, formatting, Clippy и project context остаются валидными.

Фактические результаты:

```text
cargo test -p exporter-mathtype --locked                         PASS, 4/4
cargo test -p exporter-docx --locked                             PASS, 30 tests
cargo test --workspace --locked                                  PASS, 106 tests
cargo fmt --all -- --check                                       PASS
cargo clippy --workspace --all-targets --locked -- -D warnings   PASS
python -B scripts/validate_project.py                            PASS
python unittest discovery                                       PASS, 20/20
git diff --check                                                 PASS; only LF→CRLF warnings
read-only architecture/security review                           PASS
```

### Что сознательно не доказано

- импорт generated payload в конкретную версию MathType или Word;
- полная совместимость поддерживаемых MathML shapes;
- MTEF/OLE object generation;
- licensed SDK/service integration;
- feature-gated DOCX backend.

Эти вопросы относятся к этапам 093–094. До них `EquationBackend::MathType` продолжает возвращать typed unavailable error.

## 2026-08-20 — Первая видимая public landing shell, этап 154

### Что и зачем изменено

Пустой маршрут `/` заменён статически рендеримой Next.js landing shell по Calm Blue UI. Страница показывает назначение MathMorph, synthetic workflow preview, возможности, будущий conversion flow, privacy/API/pricing/status и отдельно сообщает, что интерактивная загрузка ещё не подключена.

### Ключевой поток управления

```text
content catalog
  -> server-rendered HomePage
  -> scoped Calm Blue CSS tokens
  -> two small client boundaries: theme + compact navigation
  -> static Next.js output
```

### Команды и проверки

```text
pnpm.cmd --filter @math-morph/web test
pnpm.cmd --filter @math-morph/web typecheck
pnpm.cmd --filter @math-morph/web build
python -B scripts/validate_project.py
node %TEMP%\mathmorph-stage154-qa.mjs
```

### Решения и trade-offs

- Upload/file validation/backend intentionally remain stages 156+; CTA leads to an explicit unavailable state instead of imitating conversion.
- User copy is isolated in a Ukrainian catalog; full locale routing remains stages 162–165.
- The page is a server component; only theme and compact-menu interactions enter the client bundle.
- Browser plugin failed before tab acquisition, so exact Playwright 1.62.1 was used temporarily outside the project without manifest/lockfile changes.

### Как повторить самостоятельно

1. Выполнить `pnpm.cmd --filter @math-morph/web dev` и открыть `/`.
2. Проверить hero CTA и ссылку «Як це працює».
3. Переключить `system → light → dark` и перезагрузить страницу.
4. На ширине 390 px открыть menu, выбрать «Приватність» и убедиться, что menu закрыт, а focus вернулся к кнопке.
5. Запустить unit/component/integration tests, typecheck и production build.

## 2026-08-20 — Первый реальный локальный XMCD→DOCX путь, этапы 095–099 и 143–148

### Что изменено

- `math-engine` отделяет immutable Original AST от presentation Display AST и ограничивает depth/node work.
- `conversion-core` связывает detector, legacy XMCD parser, transformation, Document IR, DOCX exporter и validator; unsupported regions при разрешённой partial policy всегда попадают в report.
- `mathmorph-cli` добавляет настоящий binary `mathmorph` с bounded input, redacted diagnostics и safe no-replace output publication.

Текущий живой поток:

```text
file.xmcd
  -> mathmorph CLI
  -> conversion-core
  -> WorksheetParser + TransformationPipeline
  -> DocumentIrV1
  -> DocxExporter(WordOmml) + DocxValidator
  -> file.docx
```

### Как собрать и запустить

Из корня проекта:

```powershell
cd ~/codex-workspace/projects/math-morph
cargo build -p mathmorph-cli --release --locked
./target/release/mathmorph.exe convert ./path/to/input.xmcd --to docx
```

По умолчанию рядом появится `input.docx`. Для явного пути:

```powershell
./target/release/mathmorph.exe convert ./path/to/input.xmcd --to docx --output ./path/to/result.docx
```

Без release-сборки команда запускается так:

```powershell
cargo run -p mathmorph-cli --bin mathmorph --locked -- convert ./path/to/input.xmcd --to docx
```

### Ограничения

- Конвертируется подтверждённый legacy XMCD worksheet30; MCDX пока возвращает `MCDX_CONTENT_UNSUPPORTED` без output.
- Text и поддержанные formulas становятся DOCX/редактируемым Word OMML. Plot, picture без production asset path, table/diagram и неподдержанная math-семантика пропускаются только с явным warning в partial mode.
- Существующий DOCX не перезаписывается. Для другого результата нужно указать новый `--output` или убрать старый файл вручную после проверки.
- Web UI пока не вызывает этот CLI/core; это отдельные frontend/API adapter этапы.
- Полная украинская/русская/английская локализация относится к этапам 162–165.

### Как повторить самостоятельно

1. Собрать release binary командой `cargo build -p mathmorph-cli --release --locked`.
2. Запустить `mathmorph.exe convert` на копии legacy `.xmcd`.
3. Открыть созданный `.docx` в Word и проверить, что формула редактируется как equation.
4. Повторить команду без удаления output и убедиться, что файл не перезаписывается.
5. Запустить `cargo test -p mathmorph-cli --locked` и `cargo test -p conversion-core --locked`.

## 2026-08-20 — Этап 100: ordered и bounded `SymbolTable`

### Что изменено

- `math-engine` индексирует scalar и function definitions раздельно, хранит повторные revisions и выбирает только definition строго до точки использования.
- Borrowed AST проверяется до clone по cumulative budgets; одна canonical копия разделяется через `Arc`.
- Ошибки и `Debug` не содержат имена символов, literals или полный AST.

### Команды проверки

```powershell
cargo test -p math-engine --locked
cargo clippy -p math-engine --all-targets --locked -- -D warnings
cargo test --workspace --locked
python -B scripts/validate_project.py
```

### Как повторить самостоятельно

1. Создать два определения одной переменной с разными source ordinals.
2. Проверить, что history содержит обе revisions.
3. Вызвать visible-before lookup перед первой, между первой и второй и после второй revision.
4. Уменьшить node/text limit и убедиться, что возвращается typed error без partial table.
5. Запустить targeted tests и Clippy.
