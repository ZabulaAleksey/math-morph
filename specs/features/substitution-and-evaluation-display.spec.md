# SPEC: Подстановка, trace и политика отображения

**Статус:** accepted
**Версия:** 1.0.0
**Дата:** 2026-08-20
**Область:** этапы 106–111

## 1. Цель и границы

Добавить отдельную semantic transformation boundary для подстановки ранее определённых scalar values, bounded trace и режимов отображения. Original AST остаётся неизменяемым. Числовое вычисление, function evaluation и parser/exporter-specific поведение не входят в этапы 106–111.

## 2. Требования

### FR-SUBSTITUTE-106 — simple substitution

`SubstitutionEngine::once` заменяет свободную variable reference на cloned RHS последней scalar revision со строго меньшим source ordinal. Definition targets, function parameters и calculus/aggregate bound variables не заменяются. Function call evaluation завершается typed unsupported error.

### FR-SUBSTITUTE-107 — recursive substitution

Recursive mode повторяет правило 106 внутри подставленного RHS до отсутствия замен либо typed failure. На каждом шаге используется visibility исходной точки определения/использования; глобальный latest lookup запрещён. Original AST и SymbolTable не изменяются.

### NFR-SUBSTITUTE-108 — limits

Caller задаёт положительные hard-bounded limits для input depth/nodes/text, output nodes/text, substitution depth и expansion steps. Все счётчики checked и cumulative. Cycle, depth, node, text и expansion failures типизированы и не возвращают partial result как полный.

### FR-TRACE-109 — `EvaluationTrace`

Trace является ordered deterministic списком typed steps: reference observed, binding selected, substitution applied, branch skipped, completed/failed. Step хранит только source ordinals, operation kind, depth/count/status; identifier, literal, formula и AST payload в trace/Debug отсутствуют. Размер trace ограничен отдельным budget.

### FR-DISPLAY-110 — display modes

Поддерживаются `Substitution`, `DetailedTrace` и `ResultOnly`. Первые два возвращают substituted AST (и trace для detailed режима). Пока numeric evaluator отсутствует, `ResultOnly` завершается `ResultUnavailable`, а не возвращает исходный AST или guessed value.

### FR-PRECISION-111 — `PrecisionPolicy`

Policy отдельно хранит working/computation precision и display rounding precision. На этапах 106–111 она валидируется и передаётся как backend-neutral configuration, но не округляет AST и не меняет substitution. Реальное применение числового/display precision начинается в компонентном complex-number engine этапов 112–122.

### SEC-SUBSTITUTE-001

Все ошибки и `Debug` redacted. Неизвестный symbol, unsupported callable, ambiguity, cycle и limit failure завершаются fail closed без эвристической подстановки.

## 3. Критерии приёмки

- `AC-SUBSTITUTE-106`: одноуровневая scalar substitution сохраняет Original AST и bound identifiers.
- `AC-SUBSTITUTE-107`: chain `a → b → c` раскрывается детерминированно; поздняя revision не влияет назад.
- `AC-SUBSTITUTE-108`: cycle/depth/node/text/expansion limits проверены negative tests.
- `AC-TRACE-109`: одинаковый input даёт равный redacted ordered trace в пределах budget.
- `AC-DISPLAY-110`: три режима различаются явно; `ResultOnly` без evaluator даёт typed unavailable.
- `AC-PRECISION-111`: computation/display precision независимы, invalid policy отклоняется, substitution result не зависит от display rounding.

## 4. История

- 1.0.0 — принят контракт этапов 106–111 без преждевременного numeric evaluator.
