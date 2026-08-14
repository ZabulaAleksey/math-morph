# Текущий план AI — этапы 077–089

**Статус:** завершён 2026-08-14.
**Ветка:** `feature/stages-077-089`.

## Маршрутизация

- сложность: `COMPLEX`;
- режим: production;
- SDLC: requirements/specification → implementation → testing → security/review;
- домен: scientific syntax, editable OMML, hostile XML/ZIP, DOCX/OPC и Office Open XML;
- стек: Rust 1.88, Serde/JSON, `quick-xml`, `zip`;
- SPEC: `specs/features/advanced-omml-and-backend-config.spec.md` как расширение `specs/features/document-ir-docx-omml.spec.md`.

## План реализации

1. **Контракт — завершено:** принята SPEC 077–089 с AC-077..AC-089, test mapping, fail-closed policy и границей 090+.
2. **077–080 — завершено:** реализованы канонические `m:sSup`, `m:rad`, `m:sSub` и `m:sSubSup` вместе со snapshot/negative tests.
3. **081–083 — завершено:** реализованы строго типизированные function calls, парные grouping и размерностно валидируемые vector/matrix OMML.
4. **084–086 — завершено:** реализованы non-presentational integral, derivative composition и sum/product через `m:nary`.
5. **087 — завершено:** добавлены deep-nesting regressions, equation byte/node/depth budgets и расширенный строгий `DocxValidator` allowlist/ordered shapes.
6. **088 — завершено:** добавлены воспроизводимый reference artifact, структурная проверка и ручное Word/Open XML SDK evidence.
7. **089 — завершено:** добавлены public `EquationBackend`/`DocxExportConfig`, `WordOmml` default и typed unavailable `MathType` без fallback.
8. **Проверка — завершено:** targeted/workspace tests, fmt/Clippy, project validator, DOCX validator, security review и независимый review прошли.
9. **Следующий этап — не начат:** 090 (`MathML renderer`) остаётся planned и требует отдельной SPEC/решения.

## Инварианты

- Никакого Mathcad XML → DOCX shortcut.
- `mathcad-parser` не знает Word; `exporter-docx` не зависит от parser.
- Unknown math сохраняется явно и не исполняется.
- Document IR V1 не содержит raw XML, paths, URLs или binary assets.
- DOCX содержит только internal allowlisted relationships и bounded PNG/JPEG assets.
- Уравнения остаются editable OMML; формы 077–086 поддерживаются только по точным правилам SPEC, остальные возвращают explicit unsupported.
- Никакого raw XML, неэкранированного текста или XML 1.0-invalid character в OMML; renderer и `DocxValidator` применяют byte/node/depth budgets.
- Линейный обход больших left-associated выражений итеративен и ограничен work budget.
- `EquationBackend::WordOmml` является default; `MathType` зарезервирован и всегда fail closed без fallback.
- Пользовательский `apps/web/next-env.d.ts` не входит в scope и не коммитится.

## Логические точки отката

1. SPEC и тестовые contracts 077–089;
2. scripts, function/grouping и matrices 077–083;
3. calculus/aggregate 084–086;
4. limits, validator и reference artifact 087–088;
5. public backend config 089;
6. review hardening и evidence.

Внешних миграций и записей нет. После публикации V1 artifact reader V1 сохраняется либо заменяется отдельной migration SPEC.
