# SPEC: Базовый renderer Presentation MathML

**Статус:** accepted
**Версия:** 1.1.0
**Дата:** 2026-08-15
**Область:** этапы 090–091

## 1. Цель и границы

Добавить независимый от Word renderer, который преобразует подтверждённое scalar-подмножество `math-model::MathExpression` в детерминированный standalone Presentation MathML. Renderer реализует backend-neutral `document_ir::EquationExporter` и не подключается к DOCX либо зарезервированному `EquationBackend::MathType` на этом этапе.

Канонический поток остаётся неизменным:

```text
MathExpression -> MathMlRenderer -> MathMlFragment
```

## 2. Вне области

- MathType adapter, OLE, COM, Office automation и выбор MathML как DOCX backend;
- MathML parser, универсальный validator произвольного MathML и Content MathML;
- функции, матрицы, calculus, definitions, comparisons, booleans, units и другие AST-формы, не перечисленные в `FR-MATHML-002`;
- изменение parser, `math-engine`, Document IR V1 или существующего Word OMML backend;
- новые внешние production-зависимости.

## 3. Функциональные требования

### FR-MATHML-001 — standalone Presentation MathML

Renderer возвращает ровно один compact UTF-8 XML fragment с корнем:

```xml
<math xmlns="http://www.w3.org/1998/Math/MathML" display="block">...</math>
```

Порядок элементов и атрибутов детерминирован. Между элементами не добавляется значимый formatting whitespace. `ExpressionOrigin` и source spans не попадают в результат.

### FR-MATHML-002 — поддерживаемое scalar-подмножество

Поддерживаются только следующие точные отображения:

- `Real` -> `mn` с исходным проверенным numeric lexeme;
- `Identifier` -> `mi`; literal subscript -> `msub(mi, mi)`;
- `Add`/`Subtract` -> `mrow(left, mo, right)` с `+` либо `&#x2212;`;
- `Multiply` -> `mrow(left, operator/style, right)`; `Default`, `AutoSelect`, `Dot`, `NarrowDot`, `LargeDot` используют `&#x00B7;`, `X` — `&#x00D7;`, `ThinSpace` — `&#x2009;`, `NoSpace` не создаёт видимый token;
- `Divide` -> `mfrac(left, right)`;
- `Power` -> `msup(left, right)`;
- `Unary(SquareRoot)` -> `msqrt(operand)`;
- парный `Grouping` -> Core-compatible `mrow` с двумя `mo fence="true"`; unpaired grouping отклоняется.

Каждый аргумент structural element сериализуется отдельным MathML element. Renderer не использует `mfenced`, raw XML и текстовые shortcuts наподобие `x/y`, `x^2` или `sqrt(x)`.

### FR-MATHML-003 — typed fail-closed errors

Неподдерживаемая либо неоднозначная AST-форма, invalid numeric lexeme, invalid XML 1.0 character и превышение лимита возвращают typed error. Частичный fragment не возвращается. Ошибка и `Debug` не содержат formula payload, identifier, numeric lexeme, source bytes или path.

### FR-MATHML-004 — backend-neutral port

`MathMlRenderer` реализует `document_ir::EquationExporter<Output = MathMlFragment>`. Публичный fragment предоставляет read-only `as_str()` и `byte_len()`, но не позволяет caller-у внедрить raw XML.

### FR-MATHML-005 — reviewable golden snapshots

Этап 091 добавляет version-controlled corpus `crates/exporter-mathml/tests/golden/*.mathml`. Каждый файл содержит один compact standalone fragment из `FR-MATHML-001` и ровно один финальный LF. Rust integration test строит synthetic AST, вызывает production `MathMlRenderer`, удаляет только финальный LF из expected fixture и сравнивает остальные bytes без normalization.

Фиксированный inventory:

- `numeric-binary.mathml`, `numeric-octal.mathml`, `numeric-decimal.mathml`, `numeric-hexadecimal.mathml`;
- `identifier.mathml`, `identifier-subscript.mathml`, `identifier-escaped.mathml`;
- `add.mathml`, `subtract.mathml`, `divide.mathml`, `power.mathml`;
- `multiply-dot.mathml`, `multiply-x.mathml`, `multiply-thin-space.mathml`, `multiply-no-space.mathml`;
- `square-root.mathml`, `grouping.mathml`.

Dot snapshot является представителем одной exact-output группы `Default`/`AutoSelect`/`Dot`/`NarrowDot`/`LargeDot`; stage-090 integration test продолжает проверять каждый enum variant отдельно. Snapshot corpus не содержит user files, raw Mathcad XML, absolute paths или внешние ресурсы.

## 4. Нефункциональные требования

### NFR-MATHML-001 — resource limits

`MathMlLimits` имеет caller-configurable maximum depth, node count и output bytes. Defaults: depth `256`, nodes `100000`, output `4 MiB`. Счётчики используют checked arithmetic и проверяются до неограниченного роста output.

### NFR-MATHML-002 — bounded traversal

Accounting не использует unbounded recursive descent. Большое left-associated линейное выражение либо обрабатывается итеративно в пределах budgets, либо завершается typed limit error без panic/stack overflow.

Renderer принимает borrowed `MathExpression`, уже полученный через bounded construction/deserialization boundary (например, безопасный Document IR entry point). Он ограничивает собственные traversal/rendering work, но не владеет caller-структурой и не может гарантировать stack-safe `Drop` произвольно глубокой вручную построенной AST после возврата; lifecycle такого значения остаётся ответственностью caller-а.

### NFR-MATHML-003 — snapshot stability

Snapshot test проверяет exact directory inventory, отсутствие BOM/CR, compact single-line payload, единственный финальный LF, обязательные root namespace/display и повторную детерминированную генерацию. Добавление, удаление или изменение golden-файла должно создавать обычный reviewable Git diff. Автоматический режим update/bless в этапе 091 не добавляется.

Repository `.gitattributes` закрепляет `text eol=lf` для `crates/exporter-mathml/tests/golden/*.mathml`, чтобы Windows `core.autocrlf` не изменял byte-level contract при checkout.

### SEC-MATHML-001 — XML output boundary

Весь dynamic text проходит XML 1.0 character validation и escaping минимум для `&`, `<`, `>`. Сериализатор создаёт только фиксированный allowlist элементов/атрибутов и не принимает raw markup, namespace, DTD, entity declaration, URL или external resource от caller-а.

До полного сканирования dynamic token renderer начисляет его byte length в общий work/input budget, ограниченный `max_output_bytes`. Поэтому один огромный identifier, subscript или numeric lexeme не обходит resource limits до начала output serialization.

## 5. Публичный интерфейс

Новый crate `exporter-mathml` публикует:

- `MathMlRenderer::new(limits)` и `Default`;
- `MathMlLimits` и documented defaults;
- opaque `MathMlFragment` с `as_str()`/`byte_len()`;
- `MathMlError` и `MathMlLimit`.

Crate зависит только от `math-model`, `document-ir` и уже закреплённого workspace `thiserror`.

## 6. Критерии приёмки

| ID | Критерий |
|---|---|
| AC-090-001 | Scalar input даёт один deterministic `math` root в MathML namespace с `display="block"`. |
| AC-090-002 | Real, identifier/subscript, five binary operators, square root и paired grouping имеют exact shapes из `FR-MATHML-002`. |
| AC-090-003 | Dynamic text escaped; invalid XML character и invalid numeric lexeme отклоняются typed redacted error. |
| AC-090-004 | Unsupported AST и unpaired grouping отклоняются без fragment и fallback. |
| AC-090-005 | Depth/node/output limits и deep left-associated input завершаются bounded результатом либо typed error без panic. |
| AC-090-006 | Renderer работает через `EquationExporter`; DOCX `MathType` backend остаётся unavailable и не получает MathML integration. |
| AC-091-001 | Все 17 файлов из `FR-MATHML-005` существуют, а directory inventory не содержит лишних или пропущенных файлов. |
| AC-091-002 | Production renderer byte-for-byte совпадает с каждым golden payload после удаления единственного завершающего LF. |
| AC-091-003 | Corpus покрывает numeric bases, identifiers/escaping/subscript, binary shapes/multiplication presentations, square root и grouping без расширения supported AST. |
| AC-091-004 | Snapshot guard отклоняет BOM, CR, multiline/лишний newline и неверный root envelope; изменение `ExpressionOrigin` не меняет output. |

## 7. Связь с тестами

| Требование | Автоматическое доказательство |
|---|---|
| FR-MATHML-001/002, AC-090-001/002 | focused integration tests structural/exact output для каждого поддержанного shape и multiplication policy |
| FR-MATHML-003, SEC-MATHML-001, AC-090-003/004 | escaping, invalid XML/number, unsupported/unpaired и redaction tests |
| FR-MATHML-004, AC-090-006 | generic `EquationExporter` contract test и regression существующего MathType fail-closed test |
| NFR-MATHML-001/002, AC-090-005 | depth/node/output boundaries и deep left-associated regression |
| FR-MATHML-005, NFR-MATHML-003, AC-091-001..004 | external golden corpus, exact inventory/bytes/canonical-file guard и origin-invariance regression |

Минимальные команды: `cargo test -p exporter-mathml --locked`, `cargo test --workspace --locked`, `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets --locked -- -D warnings`, `python -B scripts/validate_project.py`, `git diff --check`.

## 8. Совместимость и источники

Renderer следует [W3C MathML Core](https://www.w3.org/TR/mathml-core/), Presentation MathML из [MathML 4](https://www.w3.org/TR/mathml4/) и XML escaping из [XML 1.0](https://www.w3.org/TR/xml/). Эти стандарты не гарантируют поведение MathType importer; MathType-specific compatibility относится к этапам 092–093.

## 9. Открытые вопросы

Расширение поддерживаемого AST и выбор `inline`/`block` как runtime option рассматриваются отдельным изменением SPEC после этапа 091.

## 10. История изменений

- 2026-08-15 — версия 1.0.0: принят bounded scalar Presentation MathML contract этапа 090.
- 2026-08-15 — версия 1.1.0: принят reviewable exact golden snapshot contract этапа 091 без изменения renderer scope.
