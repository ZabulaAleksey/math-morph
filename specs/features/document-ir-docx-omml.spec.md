# SPEC: Document IR, DOCX foundation и базовый OMML

**Статус:** accepted  
**Версия:** 1.0.0  
**Дата:** 2026-08-14  
**Область:** этапы 055–076

## 1. Цель

Добавить версионируемую сериализуемую границу Document IR, детерминированный безопасный subset DOCX/WordprocessingML и заменяемый exporter редактируемых equations с OMML-покрытием до fraction включительно.

Запрещён shortcut Mathcad XML → DOCX: exporter принимает только Document IR и external asset resolver. Mathcad parser не знает Word namespaces, relationships или ZIP package output.

## 2. Модули и зависимости

```text
math-model <--- mathcad-parser
     ^
     +------ document-ir <--- exporter-docx
                  ^
                  +---------- math-engine (будущий producer IR)
```

- `math-model` владеет source-neutral `MathExpression`.
- `document-ir` владеет wire schema и backend-neutral `EquationExporter` port.
- `exporter-docx` зависит от `document-ir` и `math-model`, но не от `mathcad-parser`.
- `math-engine` на текущих этапах остаётся каркасом; evaluation и transformations не реализуются.

## 3. Versioned Document IR V1

### FR-IR-001 — version envelope

Wire format — UTF-8 JSON с обязательным integer `schema_version = 1`. Unknown version, unknown V1 field, malformed JSON, превышение serialized-size limit и нарушение model invariants дают typed redacted error. Production `to_json`/`from_json` выполняют validation; default maximum input — 64 MiB.

V1 использует стабильные `snake_case` names, explicit enum tags, ordered vectors и `BTreeMap`. `HashMap`, platform-dependent paths и неявные defaults в wire format запрещены. Compatibility test читает закоммиченный V1 golden JSON; round-trip сохраняет значение.

### FR-IR-002 — document/page/metadata

`DocumentIrV1` содержит `MetadataIr`, ordered `PageIr` и diagnostic/fidelity metadata без исходного raw payload. `PageIr` содержит physical size, margins и ordered blocks. Physical lengths сериализуются как integer micrometres; wire format не содержит `f32/f64`, NaN или Infinity.

`MetadataIr` имеет явный allowlist полей. Сериализация metadata является явным действием; DOCX core properties на этих этапах не генерируются автоматически.

### FR-IR-003 — blocks and provenance

`BlockIr` содержит stable ID, `ProvenanceIr`, `FidelityIr`, optional `BlockPlacementIr` и один `BlockContentIr`:

- `Text(TextBlockIr)`;
- `Equation(FormulaIr)`;
- `Table(TableIr)`;
- `Image(ImageIr)`;
- `Plot(PlotIr)`;
- `Diagram(DiagramIr)`;
- `Unsupported(UnsupportedBlockIr)`.

`FidelityIr`: `Exact | Approximate | Unsupported | FallbackRendered`. Provenance хранит source kind, optional region/source ordinal и source span, но не filesystem path, URL или полный source document. Document order и visual/z-order представлены отдельно.

### FR-IR-004 — text/table

`TextBlockIr` хранит ordered paragraphs и runs. Run style V1: bold, italic, underline, strike, subscript/superscript, optional font family, half-point font size и RGB color. Пустой paragraph разрешён; пустой text run не создаёт невалидную структуру.

`TableIr` хранит ordered rows/cells. Он не заполняется выдуманными данными из opaque Mathcad table component. Nested content ограничивается validation depth.

### FR-IR-005 — formula

`FormulaIr` хранит immutable optional `original`, обязательный `display` и inline/display mode. Exporters читают только `display`; преобразование никогда не перезаписывает `original`. На этих этапах producer может использовать один и тот же AST для обоих полей.

### FR-IR-006 — images, plots and diagrams

IR не сериализует binary bytes, filesystem paths или URLs. `ImageIr` содержит `AssetRefIr { id, media_type }`, alt text и optional explicit physical size. Asset bytes передаются exporter через `AssetResolver`.

`PlotIr` и `DiagramIr` могут ссылаться на preview asset и provenance, но не выдумывают series/axes/primitives из opaque source. Не распознанная semantics отражается fidelity. Raw SVG/XML, callback или executable payload запрещены.

### FR-IR-007 — layout validation

Page/image dimensions положительны; margins и checked sums не выходят за page size; placement width/height положительны; IDs уникальны в своей области. Validation не переупорядочивает blocks по coordinates или z-index.

## 4. DOCX/OPC subset

Target — Transitional Office Open XML. Namespaces:

- content types: `http://schemas.openxmlformats.org/package/2006/content-types`;
- package relationships: `http://schemas.openxmlformats.org/package/2006/relationships`;
- WordprocessingML: `http://schemas.openxmlformats.org/wordprocessingml/2006/main`;
- document relationships: `http://schemas.openxmlformats.org/officeDocument/2006/relationships`;
- DrawingML/WordprocessingDrawing/Picture и Office Math — соответствующие `.../2006/...` namespaces.

### FR-DOCX-001 — minimal package

Минимальный artifact содержит ровно необходимые base parts:

```text
[Content_Types].xml
_rels/.rels
word/document.xml
```

Root relationship имеет type `.../officeDocument`, target `word/document.xml`; main part content type — `application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml`; document root — `w:document/w:body`.

Entry order, relationship IDs, media names, XML ordering, ZIP timestamps и compression options фиксированы. Два экспорта одного IR и asset set дают одинаковые bytes.

### FR-DOCX-002 — text

Paragraph/run/text отображаются как `w:p/w:r/w:t`. Несколько paragraphs сохраняют порядок. Text и attributes проходят XML 1.0 validation и escaping; significant whitespace получает `xml:space="preserve"`. Basic run style отображается только в allowlisted `w:rPr` elements.

### FR-DOCX-003 — assets and images

`AssetResolver` возвращает bytes только по `AssetRefIr`; exporter сам генерирует part name, relationship ID и DrawingML IDs. Разрешены только internally embedded PNG/JPEG:

- media type совпадает с signature;
- encoded size, image count, total bytes, dimensions и pixel count ограничены;
- metadata-bearing/active/unsupported formats, SVG, OLE, WMF/EMF, HTML и external URL отклоняются;
- missing asset, duplicate asset ID и arithmetic overflow дают typed error.

Image relationship target существует внутри package. Explicit IR size конвертируется checked integer arithmetic: `1 µm = 36 EMU`. Отсутствующий размер не угадывается и является typed error на stage 067.

### FR-DOCX-004 — page

Single-page subset создаёт final `w:sectPr` последним child `w:body`; `w:pgSz` и margins используют twips с deterministic integer rounding. Multiple `PageIr` пока дают explicit unsupported error, чтобы не создавать неверные section breaks.

### FR-DOCX-005 — limits and errors

`DocxLimits` ограничивает output bytes, entries, XML bytes, blocks, paragraphs, runs, images, per-image/total asset bytes, pixels и equation output. Limits применяются до и во время allocation/write. Errors/Debug не включают document text, formulas, asset IDs, filenames или paths.

## 5. Structural validator

### FR-DOCX-006 — generated subset validator

`DocxValidator` проверяет генерируемый этим exporter subset, а не объявляется универсальным валидатором любого DOCX. Для недоверенного input он fail-closed проверяет:

- package size/count/compression limits;
- duplicate, case-colliding, absolute, backslash и traversal part names;
- encrypted/symlink entries;
- required parts, content types и ровно один root `officeDocument` relationship;
- unique relationship IDs, только internal targets и существование target parts;
- main document root/body, XML well-formedness, UTF-8 и запрет DTD;
- image relationship/content type consistency и unique `wp:docPr@id`;
- отсутствие macros, OLE, `altChunk` и `TargetMode="External"`.

Malformed XML/relationships, missing part и broken target получают typed validation error без payload.

## 6. Equation exporter и OMML

### FR-OMML-001 — port

`document-ir::ports::EquationExporter` является backend-neutral trait с associated `Output` и `Error`; он принимает `&MathExpression`. MathType implementation и dependency отсутствуют. Fake implementation компилируется без DOCX.

### FR-OMML-002 — Word implementation

`WordEquationExporter` возвращает `OmmlFragment` с private bounded bytes/string, содержащий ровно один well-formed `m:oMath`. DOCX layer добавляет `m:oMathPara` для display formula. Renderer ограничивает AST depth, node count и output bytes даже для deserialized IR.

Unsupported nodes возвращают typed error; exporter не выдаёт пустое equation и не меняет semantics. Constructs этапов 077+ не реализуются.

### FR-OMML-003 — supported expressions 072–076

- decimal и иные уже проверенные `RealLiteral` → editable `m:r/m:t`, исходный lexeme сохраняется;
- identifier без literal subscript → italic math run; subscript до stage 079 отклоняется явно;
- Add/Subtract → ordered child expressions и отдельный operator run (`+`, U+2212);
- Multiply → glyph по сохранённому `MultiplicationStyle`: cross `×`, dot variants `·`, thin/no-space по явной policy; неоднозначный `auto-select/default` использует документированный deterministic middle-dot default;
- Divide → `m:f` с обязательными `m:num` и `m:den`, bar fraction.

Вложение, требующее ещё не реализованных brackets/powers/roots, не рендерится с изменённой semantics: возвращается typed unsupported error. XML text всегда escaped.

## 7. Критерии приёмки

- **AC-055:** V1 JSON round-trip/golden, deterministic field order, redacted Debug, unknown version/field/oversize rejection.
- **AC-056:** paragraph/run order и scoped basic text model сохраняются.
- **AC-057:** original/display разделены; exporter использует display.
- **AC-058:** image содержит asset reference/metadata без bytes/path/URL.
- **AC-059:** plot preview/provenance/fidelity сохраняются без придуманной semantics.
- **AC-060:** diagram preview/typed optional primitives/fidelity сериализуются детерминированно.
- **AC-061:** integer layout/page units, document order и z-index не смешиваются; invalid geometry rejected.
- **AC-062:** minimal artifact содержит required package structure и проходит validator.
- **AC-063–065:** single/multiple paragraphs, escaping/whitespace и basic formatting дают ожидаемый WordprocessingML.
- **AC-066–067:** PNG/JPEG embedded internally с exact relationship/content type/EMU size; MIME mismatch, metadata, oversize, missing asset и overflow rejected.
- **AC-068:** page size/margins находятся в final `w:sectPr`; invalid/multiple page rejected.
- **AC-069:** validator ловит missing/duplicate/unsafe parts, broken/external relationships, malformed/DTD XML и запрещённые active parts.
- **AC-070:** backend-neutral trait работает с fake exporter без Word dependency.
- **AC-071:** Word exporter создаёт bounded validated `m:oMath` и typed unsupported error.
- **AC-072–076:** structural snapshots подтверждают number, variable, add/subtract, multiply и nested fraction; operand order сохраняется.

## 8. Проверка и rollback

Обязательны targeted integration tests для `math-model`, `document-ir`, `exporter-docx`; workspace fmt/test/clippy; project/fixture validators; dependency/security review. Generated DOCX дополнительно может проверяться Open XML SDK/Word smoke test, но core не зависит от установленного Office.

Rollback выполняется по слоям: AST extraction, IR V1, DOCX package/text, images/page/validator, OMML. После публикации V1 artifacts reader V1 нельзя удалить без отдельной migration SPEC.

## 9. Вне области

Powers и последующие OMML nodes (077+), MathType, full DOCX validator, core properties, headers/footers, multiple sections, tables/images from real MCDX resources, evaluator, API/UI/CLI и filesystem output.
