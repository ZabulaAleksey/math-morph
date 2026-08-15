# Текущий план AI — этап 091

**Статус:** завершён 2026-08-15.
**Ветка:** `feature/stage-091`.

## Маршрутизация

- сложность: `STANDARD`;
- режим: production;
- SDLC: specification → testing → review;
- домен: mathematical document export и golden regression testing;
- стек: Rust 1.88, `exporter-mathml`, filesystem-backed test fixtures;
- SPEC: `specs/features/mathml-renderer.spec.md` версии 1.1.0.

## Ограниченный план

1. **Контракт — завершено:** зафиксировать 17-file snapshot inventory, canonical file format, exact comparison и update policy без automatic bless.
2. **Golden corpus — завершено:** добавлены synthetic standalone `.mathml` snapshots для всех stage-090 shape/data classes.
3. **Tests — завершено:** table-driven AST cases сравниваются byte-for-byte через production renderer; проверяются inventory/BOM/CR/newline/root, malformed envelope и origin invariance.
4. **Regression — завершено:** все stage-090 tests и fail-closed MathType behavior сохранены без изменений production-кода.
5. **Review — завершено:** targeted/workspace gates и два read-only review-cycle завершены; Windows EOL и incomplete root-envelope guard исправлены. Security review не требовался, поскольку trust boundary/production behavior не изменялись.
6. **Документация — завершено:** обновлены status/roadmap/traceability/testing/learning; architecture/decisions/security не менялись.
7. **Публикация — завершено:** итоговый commit подготовлен для push временной ветки `feature/stage-091`; PR/merge не выполняются.

## Интерфейсы и файлы

- `crates/exporter-mathml/tests/golden/*.mathml` — version-controlled expected output с закреплённым `.gitattributes` `eol=lf`;
- новый Rust integration test — единственный владелец inventory и AST-to-fixture mapping;
- `MathMlRenderer` и его public API не изменяются;
- новые dependencies, generators, snapshot libraries и update mode не добавляются.

## Non-goals

- новые MathML shapes или расширение AST coverage;
- experimental MathType adapter этапа 092;
- compatibility claims/docs этапа 093;
- feature-gated backend selection этапа 094;
- DOM/schema validation, browser visual testing либо Office automation.

## Откат

Golden directory и один integration test удаляются без изменения production crates/API. Stage-090 inline tests остаются исходной проверкой renderer behavior.
