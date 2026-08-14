# SPEC: завершение базового синтаксического Math AST

**Статус:** accepted  
**Версия:** 1.0.0  
**Дата:** 2026-08-14  
**Область:** этапы 052–054

## 1. Цель

Завершить подтверждённое XSD-подмножество синтаксического Math AST: boolean expressions, значения с единицами и явное сохранение неподдерживаемых узлов. Парсер строит структуру и диагностику, но не вычисляет выражения, не преобразует единицы и не выполняет `ml:program`.

Контракт основан на официальных `math30.xsd` 3.0.2 и `units10.xsd` 12.0.1 из локальной установки Mathcad 15. Vendor XSD и официальные worksheet не копируются в репозиторий; тесты используют synthetic XML с подтверждёнными QName и cardinality.

## 2. Границы и владение моделью

- Source-neutral AST и `SourceSpan` принадлежат crate `math-model`.
- `mathcad-parser` зависит от `math-model` и сохраняет прежние публичные импорты через `pub use`.
- `math-model` не зависит от XML, ZIP, Word, HTTP, frontend или evaluator.
- Каждый исходный AST-узел имеет `ExpressionOrigin::Source(SourceSpan)`; будущие преобразованные узлы могут иметь `ExpressionOrigin::Derived` без фиктивного span.
- Raw XML не копируется в AST. Он остаётся только в `SourceDocument` и доступен по проверенному source span.

## 3. Boolean expressions

### FR-AST2-001 — binary boolean

В namespace `http://schemas.mathsoft.com/math30` поддерживаются:

```xml
<ml:apply><ml:and/><expr/><expr/></ml:apply>
<ml:apply><ml:or/><expr/><expr/></ml:apply>
<ml:apply><ml:xor/><expr/><expr/></ml:apply>
```

`and`, `or` и `xor` имеют ровно два operands. Operator marker обязан быть пустым. AST хранит operator и оба operand в исходном порядке.

### FR-AST2-002 — logical not

Форма `<ml:apply><ml:not/><expr/></ml:apply>` имеет ровно один operand и пустой marker. Отдельный boolean literal не добавляется: подтверждённый `baseValueClass` его не содержит.

Неверная arity, непустой marker и неверный QName возвращают typed `MathAstError`; вычисление truth value отсутствует.

## 4. Units

### FR-AST2-003 — united value

Поддерживается только форма:

```xml
<ml:unitedValue>
  <ml:baseValue/>
  <u:unitMonomial system="optional">
    <u:unitReference unit="required" power-numerator="1" power-denominator="1"/>
  </u:unitMonomial>
</ml:unitedValue>
```

Где `ml=http://schemas.mathsoft.com/math30`, `u=http://schemas.mathsoft.com/units10`, а `baseValue` принадлежит подтверждённому XSD group `real | imag | complex | str | matrix | placeholder`. Поддерживаемый базовый узел разбирается обычным parser; пока неподдерживаемый базовый узел сохраняется как `UnsupportedNode`.

### FR-AST2-004 — unit monomial

- `unitMonomial` содержит один или более `unitReference`;
- `system` сохраняется как optional строка без semantic normalization;
- `unit` обязателен и не пуст;
- `power-numerator` и `power-denominator` читаются как bounded signed 64-bit integers с default `1`;
- denominator `0`, integer overflow, неверный namespace/cardinality и превышение `WorksheetLimits::max_unit_factors` являются typed errors;
- unit definitions не загружаются из сети или filesystem, conversion и dimension algebra отсутствуют.

## 5. UnsupportedNode

### FR-AST2-005 — явное сохранение

Публичный `UnsupportedNode` хранит:

- expanded QName неизвестного expression;
- optional expanded QName неизвестного operator/feature внутри `ml:apply`;
- `SourceSpan` полного неподдерживаемого узла;
- стабильную категорию причины без исходного payload.

Неизвестный вложенный expression становится `MathExpressionKind::Unsupported`, поэтому известные parent/sibling nodes не теряются. Неизвестный operator делает unsupported соответствующий `ml:apply` целиком. Для каждого такого узла создаётся `UNSUPPORTED_MATH_NODE` warning.

### FR-AST2-006 — результат parse

`MathParseOutcome` различает:

- `Parsed { expression, diagnostics }` — в том числе AST с `UnsupportedNode`;
- `Invalid(MathAstError)` — известная форма нарушает контракт.

Старый whole-expression `Unsupported(Diagnostic)` удаляется как pre-1.0 migration. `Debug`, `Display` ошибок и diagnostics не включают QName, unit name, identifier, literal или raw XML.

## 6. Multiplication provenance

### FR-AST2-007 — multiplication style

`BinaryExpression` сохраняет XSD attribute `ml:mult@style`: `default`, `auto-select`, `dot`, `narrow-dot`, `large-dot`, `x`, `thin-space`, `no-space`. Для остальных binary operators style отсутствует. Неверное значение является typed error.

Это поле сохраняет представление для stage 075, но parser не выбирает glyph и не меняет семантику multiplication.

## 7. Ограничения и безопасность

- Все новые ветви учитываются общим `max_ast_nodes`; unit factors имеют отдельный limit.
- Checked arithmetic обязателен для counters и integer powers.
- DTD/entities, runtime XSD resolver, сеть, filesystem extraction и evaluator отсутствуют.
- Unknown subtree не копируется и не форматируется в сообщение об ошибке.
- AST сериализуем через Serde только как явная часть будущего Document IR; `Debug` остаётся redacted.

## 8. Критерии приёмки

- **AC-052:** `and/or/xor/not` строят structural AST; nested comparisons сохраняются; wrong arity, marker content и foreign QName отклоняются; evaluator отсутствует.
- **AC-053:** simple, compound и rational-power units разбираются; defaults сохраняются; zero denominator, missing unit, namespace mismatch и factor limit дают typed errors.
- **AC-054:** unknown nested QName сохраняется с span в `UnsupportedNode`, parent/siblings остаются в AST, warning стабилен, payload отсутствует в `Debug`/errors.
- Existing AC-027–051 и parser resource/security tests продолжают проходить.

## 9. Вне области

Evaluation boolean/unit semantics, unit catalogue, dimensional analysis, Prime MCDX worksheet parsing, complex/string literals, program execution, общий diagnostics collector и exporters последующих этапов.
