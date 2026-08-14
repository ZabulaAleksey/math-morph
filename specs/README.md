# Индекс спецификаций

- [`system.spec.md`](system.spec.md) — канонические системные и продуктовые требования MathMorph.
- [`features/input-formats-and-containers.spec.md`](features/input-formats-and-containers.spec.md) — проверяемый контракт этапов 011–026 для fixtures, определения формата, безопасного MCDX и XML metadata.
- [`features/worksheet-structure-and-ast.spec.md`](features/worksheet-structure-and-ast.spec.md) — XSD-backed контракт этапов 027–051 для worksheet regions и синтаксического Math AST без вычислений.
- [`features/math-ast-completion.spec.md`](features/math-ast-completion.spec.md) — XSD-backed контракт этапов 052–054 для boolean/unit AST и явного `UnsupportedNode`.
- [`features/document-ir-docx-omml.spec.md`](features/document-ir-docx-omml.spec.md) — контракт этапов 055–076 для Document IR V1, безопасного DOCX subset и базового OMML.
- [`features/advanced-omml-and-backend-config.spec.md`](features/advanced-omml-and-backend-config.spec.md) — принятый контракт этапов 077–089 для расширяемого редактируемого OMML, resource limits, reference Word smoke и явного DOCX equation backend.

Новые feature-SPEC создаются в `specs/features/` только когда системной спецификации недостаточно. `docs/ROADMAP.md`, `docs/PROMPTS.md` и `docs/TRACEABILITY.md` не являются источниками требований.
