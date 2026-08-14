# SPEC: структура worksheet и синтаксический Math AST

**Статус:** accepted
**Версия:** 1.0.0
**Дата:** 2026-08-14
**Область:** этапы 027–051

## 1. Цель

Добавить в `mathcad-parser` детерминированное и ограниченное чтение структуры legacy XMCD worksheet и синтаксического математического дерева без вычислений, экспорта и сетевых обращений.

Контракт основан на официальных схемах Mathcad 15 `worksheet30.xsd` версии 3.0.3 и `math30.xsd` версии 3.0.2. Файлы схем и содержимое поставляемых worksheet не копируются в репозиторий; synthetic fixtures используют только подтверждённые QName, формы и атрибуты.

## 2. Граница совместимости

- Поддерживаемый worksheet contract: expanded QName `{http://schemas.mathsoft.com/worksheet30}worksheet` и `version="3.0.3"`.
- Поддерживаемый math contract: namespace `http://schemas.mathsoft.com/math30`.
- XML prefix не имеет семантики: сравнивается пара `(namespace URI, local name)`.
- `worksheet10/20`, `math10/20`, иные версии и смешанные namespace не разбираются как worksheet30 и возвращают типизированную ошибку совместимости.
- Парсер реализует подтверждённое подмножество схем, но не является runtime XSD validator. Не относящиеся к подмножеству настройки worksheet могут быть пропущены с сохранением границ источника.
- MCDX на этом этапе остаётся безопасно инспектируемым контейнером из предыдущей SPEC. Содержательное чтение внутреннего Prime worksheet не заявляется: для него нет подтверждённого schema contract.

## 3. Публичная модель worksheet

### FR-WS-001 — корень и metadata

Парсер обязан:

- проверить QName и `version` корня;
- прочитать необязательный `ws:metadata` независимо от порядка его детей;
- поддержать `generator`, `identityInfo` (`documentID`, `branchID`, `versionID`, `parentVersionID`, `revision`, `savedOn`) и `userData` (`author`, `company`, `description`, `keywords`, `revisedBy`, `title`, `customValues`);
- сохранить неизвестное содержимое `comment` и неизвестные metadata-расширения как source-backed opaque fragment;
- не логировать и не включать исходный payload в `Debug` ошибок.

### FR-WS-002 — discovery регионов

Парсер обязан обнаруживать `ws:region` внутри `ws:regions` и рекурсивно внутри `ws:area`. Каждый регион получает `source_ordinal`, source span и идентификатор. Повторяющийся `region-id` является типизированной ошибкой.

### FR-WS-003 — layout

У региона обязательны `region-id`, `top`, `left`, `height`, `width`; необязательный `z-order` имеет default `0`. Числа сохраняются вместе с исходной лексемой и обязаны быть конечными. Отсутствующие, некорректные, `NaN` и бесконечные координаты отклоняются.

### FR-WS-004 — порядок

- Канонический порядок `regions` совпадает с document/source order и не изменяется сортировкой.
- Отдельное представление visual order сортируется стабильно по `(top, left, source_ordinal)`.
- `z-order` хранится отдельно и означает только порядок рисования; он не подменяет source или visual order.

## 4. Содержимое регионов

### FR-REG-001 — text

`ws:text/ws:p` преобразуется в ordered paragraphs/runs. Сохраняются текстовые узлы и inline `b`, `i`, `u`, `so`, `sub`, `sup`, `c`, `f`, `link`, `br`, `tab`, `sp` в исходном порядке. Неизвестный inline-узел сохраняется opaque и создаёт диагностику.

### FR-REG-002 — math

`ws:math` содержит ровно одно выражение namespace math30 и необязательный `ws:resultFormat`. Сохраняются `disable-calc`, `optimize`, исходный span и результат синтаксического разбора. Ошибка или неподдержанный math-узел не приводит к panic: сохраняется raw math span, а AST отсутствует и возвращается структурированная диагностика.

### FR-REG-003 — plot

`ws:plot` хранит `item-idref` и `disable-calc`. Plot payload остаётся opaque binary reference; traces, axes и вычисления не выдумываются.

### FR-REG-004 — picture

`ws:picture` различает `png`, `jpg`, `metafile` и хранит подтверждённые размеры, quality/mapping metadata и `item-idref`. Binary payload не декодируется в этой SPEC.

### FR-REG-005 — фактическая модель table/program/unknown

- Самостоятельных `ws:table` и `ws:program` region в worksheet30 нет.
- `ws:resultFormat/ws:table` хранится как opaque table result reference.
- `ml:program` распознаётся как неподдержанное math-выражение, сохраняет source span и диагностику; это не region.
- Неизвестный region content или forward-compatible QName сохраняется как opaque region content с диагностикой, даже если такой узел невалиден по worksheet30 XSD.

## 5. Синтаксический Math AST

Все AST nodes сохраняют source span. AST описывает синтаксис и не вычисляет значения.

### FR-AST-001 — базовые узлы и арифметика

Поддерживаются:

- `ml:real`: исходный lexeme и `base ∈ {2,8,10,16}`; лексема валидируется для radix, но не преобразуется с потерей точности;
- `ml:id`: имя и literal `subscript` как часть идентификатора;
- binary `ml:apply`: `plus`, `minus`, `mult`, `div`, `pow` ровно с двумя operands.

### FR-AST-002 — snapshot contract

Тесты используют собственное каноническое S-expression представление без production serialization dependency. Snapshot включает типы узлов, значимые атрибуты и порядок детей, но не машинозависимые пути и не полный исходный payload.

### FR-AST-003 — Definition

`ml:define`, `ml:globalDefine`, `ml:localDefine` преобразуются в `Definition` с kind/style, target и value. Target обязан быть identifier или function target; иной LHS возвращает типизированную ошибку AST.

### FR-AST-004 — Evaluation

`ml:eval` хранит expression, необязательный `unitOverride` и необязательный saved `result` раздельно. Отсутствие сохранённого result допустимо.

### FR-AST-005 — FunctionCall

Функциональная форма `ml:apply` хранит callee и один или более аргументов. Нулевой вызов отклоняется.

### FR-AST-006 — FunctionDefinition

Форма `ml:define/ml:function` преобразуется в function name, непустой ordered список identifier parameters и body. Не-identifier имя или parameter отклоняются.

### FR-AST-007 — unary operations

Поддерживаются `absval`, `conjugate`, `factorial`, `neg`, `sqrt`, `transpose`, `vectorize`, `vectorSum`, `determinant` ровно с одним operand. Boolean `not` относится к этапу 052 и остаётся неподдержанным.

### FR-AST-008 — grouping

`ml:parens` содержит ровно одно выражение и сохраняется отдельным узлом с `unpaired`.

### FR-AST-009 — index и literal subscript

`ml:id @subscript` остаётся частью identifier. Array index — отдельный binary `ml:apply/ml:indexer`; второй operand может быть `ml:sequence` для нескольких индексов. Эти формы нельзя сливать.

### FR-AST-010 — matrix

`ml:matrix` хранит positive `rows`, positive `cols` и flat row-major elements. Проверяется `rows × cols == elements.len()` с checked arithmetic и общим лимитом элементов.

### FR-AST-011 — vector

Отдельного `ml:vector` QName нет. Матрица `N×1` или `1×N`, где `N > 1`, получает семантическую форму `Vector` с orientation. Матрица `1×1` остаётся Matrix.

### FR-AST-012 — range

`ml:range` имеет `end` и либо простой `start`, либо `ml:sequence(start, next)` как явный шаг. Другие arity/form отклоняются; вычисление диапазона не выполняется.

### FR-AST-013 — integral

`ml:apply/ml:integral` хранит ровно одну bound variable, integrand и необязательные lower/upper bounds из `ml:bounds`; algorithm metadata сохраняется без выполнения.

### FR-AST-014 — derivative

`ml:apply/ml:derivative` хранит одну bound variable, expression и необязательную degree expression; style сохраняется без вычисления.

### FR-AST-015 — summation/product

`ml:summation` и `ml:product` используют одну bound variable, body и необязательную пару bounds. Arity проверяется строго.

### FR-AST-016 — comparisons

Поддерживаются строго binary `equal`, `notEqual`, `greaterOrEqual`, `greaterThan`, `lessOrEqual`, `lessThan`. Цепочка сохраняется как исходно вложенный AST. Boolean evaluation относится к этапу 052.

## 6. Диагностика и ошибки

### FR-DIAG-001

Публичные диагностики имеют стабильные machine-readable codes как минимум для unsupported worksheet version, unknown region, unknown inline node и unsupported math node. Ошибки schema/arity/layout/limits представлены типизированными variants, не определяются сравнением текстов dependency errors и не содержат payload.

## 7. Нефункциональные требования

### NFR-PARSE-001 — безопасность

- Только UTF-8; DTD, entity declarations и внешние сущности запрещены.
- Parser не выполняет filesystem extraction, URI resolution, сеть, macro/program execution или formula evaluation.
- Input size, XML depth, regions, AST nodes, token/attribute text, matrix elements и суммарный сохраняемый текст ограничены настраиваемыми лимитами.
- Счётчики и `rows × cols` используют checked arithmetic; превышение лимита завершается fail-closed.

### NFR-PARSE-002 — детерминизм и provenance

Одинаковые bytes и limits дают одинаковые AST, diagnostics и ordering. Source spans используют byte offsets исходного immutable buffer; opaque fragments ссылаются на него без безусловного копирования payload.

### NFR-PARSE-003 — модульные границы

XML parsing, worksheet model и math AST находятся в `mathcad-parser`. Они не зависят от evaluator, `DocumentIR`, exporters, HTTP/Python/React и не реализуют прямой XML→DOCX поток.

## 8. Критерии приёмки по этапам

| Этап | Проверяемый результат |
|---|---|
| 027 | `AC-027`: root/version и reordered metadata разбираются, unsupported version типизирован |
| 028 | `AC-028`: top-level и nested-area regions обнаружены в source order |
| 029 | `AC-029`: layout/default z-order прочитаны; missing/non-finite coordinate отклонён |
| 030 | `AC-030`: source, visual и z-order не смешиваются; tie стабилен |
| 031 | `AC-031`: mixed text/inline порядок сохранён |
| 032 | `AC-032`: math region хранит raw span и AST/diagnostic outcome |
| 033 | `AC-033`: plot reference прочитан без payload interpretation |
| 034 | `AC-034`: picture kind/reference/metadata прочитаны |
| 035 | `AC-035`: table result, program expression и unknown region классифицированы на правильном уровне |
| 036 | `AC-036`: real/id/arithmetic AST работает без evaluator |
| 037 | `AC-037`: canonical snapshots детерминированы |
| 038 | `AC-038`: три вида Definition и invalid target покрыты |
| 039 | `AC-039`: Evaluation разделяет expression/unit/result |
| 040 | `AC-040`: FunctionCall хранит callee/args и проверяет arity |
| 041 | `AC-041`: FunctionDefinition проверяет name/parameters/body |
| 042 | `AC-042`: все scoped unary operations и wrong arity покрыты |
| 043 | `AC-043`: grouping сохраняется отдельным узлом |
| 044 | `AC-044`: literal subscript и array index различаются |
| 045 | `AC-045`: matrix dimensions/count/limit проверены |
| 046 | `AC-046`: row/column vector specialization детерминирована |
| 047 | `AC-047`: simple и explicit-next range различаются |
| 048 | `AC-048`: integral lambda/bounds/algorithm разобраны |
| 049 | `AC-049`: derivative lambda/degree/style разобраны |
| 050 | `AC-050`: summation/product lambda/bounds разобраны |
| 051 | `AC-051`: шесть сравнений parsed structurally без boolean evaluation |

## 9. Обязательные проверки

- Positive integration tests по каждой строке `AC-027..051`.
- Negative tests: namespace/version mismatch, duplicate region id, layout error, depth/node/text/matrix limits, malformed radix real, wrong arity, invalid definition/function target, unknown QName.
- Regression: DTD/entities по-прежнему запрещены; ошибки и `Debug` не раскрывают payload.
- `cargo fmt --all -- --check`.
- `cargo test --workspace --locked`.
- `cargo clippy --workspace --all-targets --locked -- -D warnings`.
- Python project/fixture validators и unit tests.
- Security review и независимый code review перед статусом `verified`.

## 10. Вне области 027–051

- boolean AST/evaluation (`and`, `or`, `xor`, `not`) — этап 052;
- units — этап 053;
- универсальный публичный `UnsupportedNode` — этап 054;
- `DocumentIR`, evaluator, exporters, API и UI;
- Prime MCDX worksheet schema parsing;
- plot/table/program binary decoding;
- runtime XSD validation и копирование vendor XSD в репозиторий.
