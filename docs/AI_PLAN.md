# Текущий план AI — этапы 027–051

**Статус:** выполняется с 2026-08-14.
**Ветка:** `feature/stages-027-051`.

## Цель

Реализовать XSD-backed чтение структуры legacy XMCD worksheet и синтаксический Math AST от базовых выражений до сравнений. Не выполнять формулы, не декодировать binary payload и не начинать этап 052.

## Маршрутизация задачи

- сложность: `COMPLEX`;
- режим: production;
- SDLC: requirements/specification → architecture → implementation → testing → security/review;
- домен: hostile document/XML parsing и scientific math syntax;
- стек: Rust 1.88, `quick-xml`;
- SPEC: `specs/system.spec.md`, `specs/features/worksheet-structure-and-ast.spec.md`.

## Подтверждённый источник формата

Контракт сверен с официальными `worksheet30.xsd` 3.0.3 и `math30.xsd` 3.0.2 из локальной установки Mathcad 15. Vendor-файлы и содержимое официальных worksheets в репозиторий не копируются; тестовые данные синтетические.

## Логические блоки

1. **Контракт и архитектура:** зафиксировать namespaces, реальные уровни table/program/vector, limits, source provenance и диагностику; обновить учебный контекст.
2. **027–035 — worksheet:** metadata, recursive regions, layout/order, text, math, plot, picture и opaque fallbacks.
3. **036–037 — ядро AST:** real/id/arithmetic и детерминированные snapshot tests.
4. **038–044 — определения и формы:** Definition, Evaluation, FunctionCall, FunctionDefinition, unary, grouping, index/subscript.
5. **045–051 — составные выражения:** matrix/vector, range, integral, derivative, sum/product, comparisons.
6. **Проверка:** negative/limit/security regressions, fmt/test/clippy, validators, независимые security/code reviews.
7. **Завершение:** обновить архитектуру, форматы, status, traceability и learning log; сделать отдельные коммиты для проверенных блоков.

## Инварианты

- Сравниваются expanded QName, не XML prefixes.
- Source order, visual order и z-order остаются разными понятиями.
- Неизвестное содержимое сохраняется source-backed и диагностируется, а не исполняется.
- DTD/entities, сеть, filesystem extraction и evaluator отсутствуют.
- `ml:program` — math expression; table — result-format reference; vector — специализация matrix.
- MCDX Prime worksheet parsing не заявляется без отдельного подтверждённого schema contract.

## Контрольные точки и коммиты

- contract/docs;
- worksheet 027–035;
- AST 036–037;
- AST 038–044;
- AST 045–051;
- review hardening и verified docs.

## Откат

Каждый логический блок оформляется отдельным коммитом. Миграций данных и внешнего состояния нет; откат выполняется отменой соответствующего коммита. Изменение public API после публикации требует отдельного решения.
