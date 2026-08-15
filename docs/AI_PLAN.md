# Текущий план AI — этап 090

**Статус:** завершён 2026-08-15.
**Ветка:** `feature/stage-090`.

## Маршрутизация

- сложность: `STANDARD`;
- режим: production;
- SDLC: specification → implementation → testing → review;
- домен: mathematical document export и безопасная XML serialization;
- стек: Rust 1.88, `math-model`, `document-ir`, `thiserror`;
- SPEC: `specs/features/mathml-renderer.spec.md`.

## Ограниченный план

1. **Контракт — завершено:** зафиксированы exact Presentation MathML Core shapes, supported scalar subset, typed errors, limits и non-goals 091+.
2. **Граница crate — завершено:** добавлен нейтральный `exporter-mathml`, зависящий только от `math-model`, `document-ir` и `thiserror`; workspace/project validator обновлён.
3. **Renderer — завершено:** реализованы deterministic standalone root, structural scalar elements, XML validation/escaping и backend-neutral `EquationExporter`.
4. **Budgets — завершено:** depth/node/output и cumulative input text bytes проверяются checked counters; traversal больших left-associated expressions полностью итеративен.
5. **Tests — завершено:** supported shapes, multiplication styles, escaping, invalid/unsupported paths, limits, redaction и port contract покрыты focused tests.
6. **Review — завершено:** independent review и повторный security review прошли после исправления input work budget, numeric validation и Cargo dependency scopes.
7. **Документация — завершено:** обновлены только затронутые architecture/status/roadmap/traceability/security/testing/learning records.
8. **Публикация — завершено:** итоговый commit подготовлен для push временной ветки `feature/stage-090`; PR/merge не выполняются.

## Интерфейсы

- вход: immutable `math_model::MathExpression`;
- порт: `document_ir::EquationExporter`;
- выход: opaque `MathMlFragment`;
- ошибки: typed redacted `MathMlError`;
- limits: caller-configurable `MathMlLimits` с безопасными defaults.

## Non-goals

- stage 091 snapshot corpus;
- stage 092 MathType adapter и изменение `EquationBackend::MathType`;
- MathML input/validator, Content MathML, DOCX/OLE/Office integration;
- расширение AST/parser/Document IR или UI/API/CLI.

## Откат

Новый crate и его workspace registration удаляются одним commit без изменения существующих public contracts. `exporter-docx` и `EquationBackend::MathType` остаются в прежнем fail-closed состоянии.
