# Текущий план AI — этапы 027–051

**Статус:** завершён и проверен 2026-08-14.
**Ветка:** `feature/stages-027-051`.

## Достигнутый результат

Реализовано XSD-backed чтение структуры legacy XMCD worksheet30 и синтаксический Math AST до structural comparisons. Формулы не вычисляются, binary payload не декодируется, этап 052 не начат.

## Маршрутизация

- сложность: `COMPLEX`;
- режим: production;
- SDLC: requirements/specification → architecture → implementation → testing → security/review;
- домен: hostile document/XML parsing и scientific math syntax;
- стек: Rust 1.88, `quick-xml`;
- SPEC: `specs/system.spec.md`, `specs/features/worksheet-structure-and-ast.spec.md`.

## Выполненные блоки

1. **Контракт:** подтверждены `worksheet30.xsd` 3.0.3 и `math30.xsd` 3.0.2, создана feature-SPEC и ADR-0007.
2. **027–035:** metadata, recursive regions, layout/source/visual/z order, text, math, plot, picture и opaque fallbacks.
3. **036–037:** real/id/arithmetic AST, source spans и canonical test-only snapshots.
4. **038–044:** definitions, evaluation, functions, unary, grouping и index/subscript.
5. **045–051:** matrix/vector, range, calculus, sum/product и six comparisons.
6. **Hardening:** namespace interning, full QName bounds, payload-redacted Debug, preservation/arity/signed-zero regressions.
7. **Проверка:** 46 Rust integration tests, fmt, Clippy `-D warnings`, validators, 14 Python tests, независимые security/code reviews.

## Инварианты

- Expanded QName, не XML prefix.
- Source, visual и z-order не смешиваются.
- Opaque content source-backed; unsupported не исполняется.
- DTD/entities, сеть, filesystem extraction и evaluator отсутствуют.
- `ml:program` — unsupported math; table — result reference; vector — matrix specialization.
- Prime MCDX worksheet parsing не заявляется без отдельного schema contract.

## Коммиты логических блоков

- `80375bf` — SPEC/docs;
- `124f43b` — worksheet 027–035;
- `dbfc2c3` — AST 036–037;
- `39371dd` — AST 038–044;
- `183aa07` — AST 045–051;
- `2be0fd9`, `1c205aa` — security/review hardening.

## Следующий отдельный план

Этап 052 (`booleans`) требует нового ограниченного плана и продолжает AST, не меняя завершённый контракт этапов 027–051.

## Откат

Миграций данных и внешнего состояния нет. Логические блоки отменяются соответствующими коммитами; public API после публикации меняется только отдельным решением.
