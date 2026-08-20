# SPEC: Комплексные числа

**Статус:** accepted
**Версия:** 1.0.0
**Дата:** 2026-08-20
**Область:** этапы 112–122

## 1. Цель и границы

Добавить в `math-engine` самостоятельную scalar complex-number boundary, отделяющую семантическое значение от algebraic/polar presentation, trace и форматирования. Parser integration, complex matrices/units/functions и изменение source Math AST вне scope до появления подтверждённых format fixtures.

## 2. Каноническая модель

`ComplexValue` хранит Cartesian `real: f64` и `imaginary: f64`. Конструкторы и операции принимают только finite values; `NaN` и infinity отклоняются. Значение не реализует `Eq`; сравнение выполняется явным finite non-negative absolute/relative `Tolerance`.

Polar representation хранит non-negative magnitude и angle в radians, нормализованный в `[-π, π)`. Нулевой magnitude всегда имеет canonical angle `0`. Преобразования значения не округляют.

## 3. Требования

- `FR-COMPLEX-112`: validated immutable `ComplexValue`, отдельный от display mode.
- `FR-COMPLEX-113`: algebraic representation сохраняет real/imaginary без округления.
- `FR-COMPLEX-114`: validated canonical polar representation.
- `FR-COMPLEX-115`: algebraic→polar использует `hypot`/`atan2`, корректно обрабатывает оси, четыре квадранта и origin.
- `FR-COMPLEX-116`: polar→algebraic использует `magnitude*cos/sin`; round-trip проверяется через `Tolerance`.
- `FR-COMPLEX-117`: multiplication возвращает `(ac-bd)+(ad+bc)i` и bounded typed polar trace.
- `FR-COMPLEX-118`: division использует устойчивую checked policy, отклоняет нулевой denominator и non-finite result, возвращает bounded typed trace.
- `FR-COMPLEX-119`: addition выполняется в Cartesian representation.
- `FR-COMPLEX-120`: subtraction выполняется в Cartesian representation.
- `FR-COMPLEX-121`: output modes `Algebraic`, `Polar`, `Both` влияют только на presentation. `PrecisionPolicy` округляет только formatted output; signed zero нормализуется только при отображении.
- `FR-COMPLEX-122`: edge suite покрывает origin, pure real/imaginary, axes/quadrants, angle normalization, cancellation, division by zero, tolerance boundary, overflow/non-finite и rounding boundaries.

## 4. Trace, limits и ошибки

`ComplexTrace` хранит только operation kind и allowlisted steps без operands/formulas. `ComplexLimits` ограничивает trace steps и formatted output bytes; нулевые/over-hard values дают `InvalidLimits`. Arithmetic остаётся O(1), checked for finite result. Ошибки/Debug не раскрывают operands или пользовательский payload.

## 5. Критерии приёмки

- Каждый этап 112–121 имеет targeted unit/component test и negative/boundary test.
- Round-trip algebraic↔polar проходит в принятом `Tolerance`, без exact float equality.
- Inputs immutable; одинаковые inputs/policy дают deterministic representation, trace и formatted output.
- Workspace tests, fmt, Clippy и project validator проходят.
- Завершение 112–122 означает component-level engine support; XMCD complex support не заявляется без подтверждённого parser fixture/schema.

## 6. История

- 1.0.0 — принят standalone `f64` complex engine contract этапов 112–122.
