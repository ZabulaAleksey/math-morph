# Индекс спецификаций

- [`system.spec.md`](system.spec.md) — канонические системные и продуктовые требования MathMorph.
- [`features/input-formats-and-containers.spec.md`](features/input-formats-and-containers.spec.md) — проверяемый контракт этапов 011–026 для fixtures, определения формата, безопасного MCDX и XML metadata.
- [`features/worksheet-structure-and-ast.spec.md`](features/worksheet-structure-and-ast.spec.md) — XSD-backed контракт этапов 027–051 для worksheet regions и синтаксического Math AST без вычислений.
- [`features/math-ast-completion.spec.md`](features/math-ast-completion.spec.md) — XSD-backed контракт этапов 052–054 для boolean/unit AST и явного `UnsupportedNode`.
- [`features/document-ir-docx-omml.spec.md`](features/document-ir-docx-omml.spec.md) — контракт этапов 055–076 для Document IR V1, безопасного DOCX subset и базового OMML.
- [`features/advanced-omml-and-backend-config.spec.md`](features/advanced-omml-and-backend-config.spec.md) — принятый контракт этапов 077–089 для расширяемого редактируемого OMML, resource limits, reference Word smoke и явного DOCX equation backend.
- [`features/mathml-renderer.spec.md`](features/mathml-renderer.spec.md) — контракт этапов 090–091 для bounded standalone Presentation MathML renderer и reviewable exact golden snapshots без подключения MathType/DOCX adapter.
- [`features/experimental-mathtype-adapter.spec.md`](features/experimental-mathtype-adapter.spec.md) — контракт этапа 092 для pure offline adapter `MathExpression` → opaque bounded Presentation MathML payload без SDK/OLE/DOCX integration.
- [`features/mathtype-compatibility-evidence.spec.md`](features/mathtype-compatibility-evidence.spec.md) — контракт этапа 093 для versioned compatibility matrix, evidence levels и воспроизводимого MathType import/edit smoke без преждевременного включения DOCX backend.
- [`features/visible-nextjs-shell.spec.md`](features/visible-nextjs-shell.spec.md) — контракт этапа 154 для первой видимой публичной Calm Blue UI оболочки без преждевременного upload/backend flow.
- [`features/transformation-pipeline.spec.md`](features/transformation-pipeline.spec.md) — контракт этапов 095–099 для immutable Original AST → Display AST presentation pipeline.
- [`features/semantic-dependency-analysis.spec.md`](features/semantic-dependency-analysis.spec.md) — контракт этапов 100–105 для `SymbolTable`, references, dependency graph, evaluation order и typed diagnostics.
- [`features/substitution-and-evaluation-display.spec.md`](features/substitution-and-evaluation-display.spec.md) — контракт этапов 106–111 для bounded substitution, trace, display modes и `PrecisionPolicy` без преждевременного evaluator.
- [`features/complex-numbers.spec.md`](features/complex-numbers.spec.md) — контракт этапов 112–122 для standalone scalar complex-number engine, polar/algebraic conversion, trace и output policy.
- [`features/conversion-pipeline-and-report.spec.md`](features/conversion-pipeline-and-report.spec.md) — контракт этапов 143–147 для общего XMCD→DOCX application core, diagnostics, fidelity report и safe partial conversion.
- [`features/minimal-cli-convert.spec.md`](features/minimal-cli-convert.spec.md) — контракт этапа 148 для настоящей локальной команды `mathmorph convert`.

Новые feature-SPEC создаются в `specs/features/` только когда системной спецификации недостаточно. `docs/ROADMAP.md`, `docs/PROMPTS.md` и `docs/TRACEABILITY.md` не являются источниками требований.
