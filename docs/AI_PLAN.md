# Текущий план AI — этапы 052–076

**Статус:** выполняется с 2026-08-14.
**Ветка:** `feature/stages-052-076`.

## Маршрутизация

- сложность: `COMPLEX`;
- режим: production;
- SDLC: requirements/specification → architecture → implementation → testing → security/review;
- домен: hostile XML/ZIP, scientific syntax, versioned IR и Office Open XML;
- стек: Rust 1.88, Serde/JSON, `quick-xml`, `zip`;
- SPEC: `specs/features/math-ast-completion.spec.md`, `specs/features/document-ir-docx-omml.spec.md` и системные разделы 4–5.

## План реализации

1. **Контракт и архитектура:** принять feature-SPEC, dependency DAG, wire/version/security policies и acceptance criteria.
2. **052–054:** выделить `math-model`, добавить boolean/unit/unsupported AST, typed errors, diagnostics и regressions.
3. **055–061:** добавить `document-ir` V1, bounded JSON round-trip, blocks, provenance, fidelity, assets и integer layout.
4. **062–069:** реализовать deterministic DOCX package, text/formatting/images/page и structural validator генерируемого subset.
5. **070–076:** добавить `EquationExporter`, `WordEquationExporter` и bounded editable OMML для number/variable/add/subtract/multiply/fraction.
6. **Проверка:** targeted/full tests, fmt, Clippy, validators, dependency/security review, documentation и Learning.

## Инварианты

- Никакого Mathcad XML → DOCX shortcut.
- `mathcad-parser` не знает Word; `exporter-docx` не зависит от parser.
- Unknown math сохраняется явно и не исполняется.
- Document IR V1 не содержит raw XML, paths, URLs или binary assets.
- DOCX содержит только internal allowlisted relationships и bounded PNG/JPEG assets.
- Уравнения остаются editable OMML; функции 077+ возвращают explicit unsupported.
- Пользовательский `apps/web/next-env.d.ts` не входит в scope и не коммитится.

## Логические точки отката

1. SPEC/ADR и workspace crates;
2. parser/model 052–054;
3. Document IR 055–061;
4. DOCX 062–069;
5. OMML 070–076;
6. review hardening и docs.

Внешних миграций и записей нет. После публикации V1 artifact reader V1 сохраняется либо заменяется отдельной migration SPEC.
