# SPEC: Presentation transformation pipeline

**Статус:** accepted
**Версия:** 1.0.0
**Дата:** 2026-08-20
**Область:** этапы 095–099

## 1. Цель и границы

Создать backend-neutral границу `Original AST → Display AST`, которая подготавливает математическое выражение к отображению и экспорту, не вычисляя его и не изменяя исходную семантическую структуру.

В scope входят faithful-профиль, представление определений, typed registry отображений, лимиты обхода и regression semantic preservation. Вне scope: symbol table, dependency graph, substitution, evaluation, complex arithmetic, Word/HTTP/UI-specific logic и MathType wiring.

## 2. Требования

### FR-TRANSFORM-095 — отдельный Display AST

`TransformationPipeline` принимает `&MathExpression` и возвращает новый `TransformationResult`, содержащий `display` и детерминированный список применённых presentation-преобразований. Вход не изменяется; `ExpressionOrigin` и структура неподвергшихся правилу узлов сохраняются.

### FR-TRANSFORM-096 — presentation rule определений

`Definition` и `FunctionDefinition` остаются типизированными определениями. Faithful-профиль сохраняет исходные `kind` и `style`; профиль с выбранным `definition_style` меняет только presentation style в Display AST и фиксирует применение правила. Target, parameters, body/value и origin не меняются.

### FR-TRANSFORM-097 — SymbolMappingRegistry

`SymbolMappingRegistry` является typed allowlist presentation-отображений. Неизвестный ключ не заменяется эвристически и не меняет выражение. Registry не хранит raw XML, OMML или exporter-specific fragments.

### FR-TRANSFORM-098 — NotationProfile

Обязателен `NotationProfile::faithful()`. Профиль детерминирован и не включает вычисление, подстановку или неявный fallback. Пользовательские сериализуемые профили относятся к будущим этапам.

### FR-TRANSFORM-099 — semantic preservation

После любого разрешённого presentation-преобразования исходный AST остаётся равным собственной pre-transform копии, а Display AST отличается только полями, явно разрешёнными профилем. Повторный запуск с одинаковыми входом и профилем возвращает равный результат.

### NFR-TRANSFORM-001 — bounded traversal

Pipeline использует caller-configurable limits для maximum depth и node count, checked arithmetic и typed `TransformError`. Значения по умолчанию согласованы с Document IR: depth `256`, nodes `100000`.

### SEC-TRANSFORM-001 — redacted failures

Ошибки и `Debug` не содержат имена символов, literal values или полный AST. Неизвестная/неподдерживаемая семантика завершается typed error, а не похожим преобразованием.

## 3. Публичная граница

```text
TransformationPipeline + TransformationLimits
NotationProfile
SymbolMappingRegistry
TransformationResult { display, applied_transformations }
TransformError
```

`math-engine` зависит только от backend-neutral model crates и не зависит от parser, Document IR, DOCX, CLI или web.

## 4. Критерии приёмки

- `AC-TRANSFORM-095`: faithful transform создаёт отдельный равный Display AST и не изменяет Original AST.
- `AC-TRANSFORM-096`: definition style меняется только в Display AST и только при явном профиле.
- `AC-TRANSFORM-097`: неизвестное symbol mapping не создаёт эвристическую замену.
- `AC-TRANSFORM-098`: одинаковые input/profile дают одинаковые result и ordered applied transformations.
- `AC-TRANSFORM-099`: depth/node limits и redacted typed errors проверены отрицательными тестами.

## 5. Связь с тестами

| Требование | Проверка |
|---|---|
| FR-TRANSFORM-095, FR-TRANSFORM-099 | unit/regression immutable original и deterministic result |
| FR-TRANSFORM-096 | unit definitions и function definitions |
| FR-TRANSFORM-097, FR-TRANSFORM-098 | registry/profile unit tests |
| NFR-TRANSFORM-001, SEC-TRANSFORM-001 | limit и redaction tests |

## 6. История

- 1.0.0 — принят контракт этапов 095–099 для пути к minimal CLI conversion.
