# SPEC: Семантическая таблица символов и граф зависимостей

**Статус:** accepted
**Версия:** 1.0.0
**Дата:** 2026-08-20
**Область:** этапы 100–105

## 1. Цель и границы

Создать backend-neutral semantic boundary поверх `math-model`, которая индексирует определения листа, извлекает ссылки, строит детерминированный граф зависимостей и выдаёт порядок вычисления с явными диагностиками неизвестных символов и циклов.

Этапы 100–105 не вычисляют выражения, не подставляют значения и не изменяют Original AST. Вне scope также остаются parser-, Document IR-, DOCX-, CLI-, HTTP- и UI-specific типы.

## 2. Общие инварианты

- Идентичность символа включает точные `name` и `subscript`; variable и function symbols различаются, а функция дополнительно различается по arity.
- Порядок определений задаётся caller-ом и сохраняется без визуальной сортировки.
- Повторное определение допустимо и сохраняется как отдельная ordered revision; оно не перезаписывает историю.
- Неподдерживаемая или неоднозначная форма target/reference завершается typed redacted error, а не эвристикой.
- Все обходы и накопления ограничены caller-configurable budgets с checked arithmetic.

## 3. Требования

### FR-SEMANTIC-100 — `SymbolTable`

`SymbolTable` строится по последовательности top-level `MathExpression` со строго возрастающими source ordinals. Он индексирует только `Definition` с identifier target и `FunctionDefinition` с identifier name и identifier-only parameters. Для каждой записи сохраняются стабильный source ordinal, тип определения и одна каноническая неизменённая копия expression; индексы и ordered views могут разделять её через `Arc`.

Неопределяющие выражения не становятся символами. Правая часть определения и body функции на этом этапе не обходятся семантически, не вычисляются и не подставляются; bounded resource preflight обязан проверить их форму до клонирования. Повторные определения одного ключа доступны как детерминированная история в source order; lookup последней revision не удаляет предыдущие. Для последующих этапов доступен отдельный lookup последней revision строго перед ordinal точки использования.

### FR-SEMANTIC-101 — variable references

Reference collector извлекает свободные variable/function references из expressions с учётом bound function parameters и calculus-bound identifiers. Результат детерминирован, дедуплицирован по typed symbol identity и не включает definition target как reference.

### FR-SEMANTIC-102 — dependency graph

Dependency graph связывает каждую definition revision с видимыми referenced definitions. Scalar reference разрешается только в последнюю revision со строго меньшим source ordinal; поздние определения не влияют назад. Function body может ссылаться на собственную callable revision только для обнаружения recursion/cycle, но не для неявного выполнения. Граф сохраняет source ordinals и детерминированный порядок рёбер. Неизвестные и неоднозначные ссылки не создают guessed edge.

### FR-SEMANTIC-103 — worksheet evaluation order

Для ациклического графа возвращается стабильный topological order, совместимый с top-to-bottom worksheet visibility. При нескольких допустимых узлах tie-break выполняется по source ordinal. Forward reference не переупорядочивает worksheet, а диагностируется как undefined. Original worksheet/AST не переставляется и не изменяется.

### FR-SEMANTIC-104 — undefined-variable diagnostic

Каждая свободная ссылка без доступного определения создаёт typed diagnostic с category и source ordinal. Публичные `Debug`/`Display` не раскрывают identifier, subscript, literal или полный AST.

### FR-SEMANTIC-105 — circular dependency diagnostic

Цикл, включая self-cycle, создаёт typed redacted diagnostic. Порядок диагностик и идентификаторов узлов детерминирован; partial/guessed evaluation order не выдаётся как успешный результат.

### NFR-SEMANTIC-001 — resource limits

Public analysis entry points ограничивают число входных expressions, definitions, references, graph nodes/edges, глубину и суммарное число AST nodes, суммарные payload text bytes, размер одного identifier и число элементов вложенных коллекций. Проверка borrowed AST выполняется до clone/index allocation. Значения выше hard ceilings или нулевые обязательные budgets отклоняются как `InvalidLimits`. Переполнение счётчиков завершается typed error.

### SEC-SEMANTIC-001 — data minimization

Ошибки, diagnostics и custom `Debug` отражают только категории, ordinals, counts и наличие metadata. Они не содержат имена символов, значения, source bytes, filesystem paths или serialized AST.

## 4. Критерии приёмки этапа 100

- `AC-SEMANTIC-100-A`: variable и function definitions индексируются раздельно и детерминированно.
- `AC-SEMANTIC-100-B`: повторные определения сохраняются в source order без silent overwrite.
- `AC-SEMANTIC-100-C`: non-definition expressions не попадают в таблицу; RHS/body остаются неизменными и не вычисляются.
- `AC-SEMANTIC-100-D`: malformed targets/parameters и resource limits дают typed redacted errors.
- `AC-SEMANTIC-100-E`: одинаковый input и limits дают равный `SymbolTable`; Original AST не изменяется.
- `AC-SEMANTIC-100-F`: cumulative node/text/collection budgets применяются до clone, AST хранится одной canonical копией, а visible-before lookup не видит текущую или позднюю revision.

## 5. Критерии приёмки этапов 101–105

- `AC-SEMANTIC-101`: bound identifiers исключаются, свободные references детерминированно извлекаются и ограничиваются budgets.
- `AC-SEMANTIC-102`: graph edges соответствуют reference resolution без guess/fallback.
- `AC-SEMANTIC-103`: acyclic graph имеет стабильный topological order с source-order tie-break.
- `AC-SEMANTIC-104`: undefined reference выдаёт redacted typed diagnostic.
- `AC-SEMANTIC-105`: self/multi-node cycles выдаются как redacted typed failure без успешного partial order.

## 6. Связь с тестами

| Требование | Проверка |
|---|---|
| FR-SEMANTIC-100 | unit/integration tests variable/function indexing, revisions, immutability и no-evaluation |
| FR-SEMANTIC-101 | bound/free reference traversal и negative malformed forms |
| FR-SEMANTIC-102..103 | deterministic graph/topological-order fixtures |
| FR-SEMANTIC-104..105 | undefined/self-cycle/multi-cycle regressions и redaction assertions |
| NFR/SEC-SEMANTIC-001 | depth/count/overflow/invalid-limit tests без payload в errors |

## 7. История

- 1.0.0 — принят контракт этапов 100–105; реализация начинается с изолированного этапа 100.
