# Трассировка требований

## Назначение

Поддерживать компактный путь от утверждённых требований продукта к этапам дорожной карты, реализации и тестам без создания второй спецификации.

Канонические источники:

- требования продукта: `specs/system.spec.md`;
- порядок реализации и стабильные номера этапов: `docs/ROADMAP.md`;
- исполняемые части этапов: соответствующий раздел `docs/PROMPTS.md`;
- архитектурные ограничения и решения: `docs/ARCHITECTURE.md` и `docs/DECISIONS.md`;
- доказательства проверки: закоммиченный код, тесты, fixtures и результаты review.

Текущие ссылки уровня продукта используют стабильные номера разделов спецификации, например `SPEC-02`. Новые предметные или функциональные specs могут добавлять ID `REQ-*`, если трассировка на уровне разделов недостаточно точна; такие ID должны ссылаться на каноническое требование продукта.

## Исходное соответствие

Все этапы имеют статус `planned` до появления реализации и доказательств проверки.

| Область требований | Этапы дорожной карты | Ожидаемая реализация / доказательства проверки | Статус |
|---|---:|---|---|
| NFR-FOUNDATION-001/002 | 001 | workspace manifests/lockfiles, пустые каркасы, `scripts/validate_project.py`, `tests/test_validate_project.py`, сборки Python/Next.js и `cargo check --workspace --locked` | verified |
| NFR-CONTEXT-001/002 | 002–010 | канонические документы, project overlay и модульные `AGENTS.md`; `scripts/validate_project.py`, `tests/test_validate_project.py` | verified |
| SPEC-02 fixture corpus | 011–014 | `tests/fixtures/manifest.json`, taxonomy directories, `scripts/validate_fixtures.py`, `tests/test_validate_fixtures.py` | verified |
| SPEC-02 определение формата | 015–018 | `crates/mathcad-parser/src/format.rs`, `diagnostic.rs`, detector/mismatch tests в `tests/input_boundary.rs` | verified |
| SPEC-02 безопасный MCDX container | 019–025 | `crates/mathcad-parser/src/mcdx.rs`, path/collision/limits/manifest tests в `tests/input_boundary.rs` | verified |
| SPEC-02 XML root metadata | 026 | `crates/mathcad-parser/src/xml_metadata.rs`, UTF-8/DTD/namespace/schema/limits tests в `tests/input_boundary.rs` | verified |
| FR-WS-001..004, FR-REG-001..005 | 027–035 | `source.rs`, `worksheet.rs`, `region.rs`, `xml_worksheet.rs`; `tests/worksheet_structure.rs`; security/code review | verified |
| FR-AST-001..016, NFR-PARSE-001..003 | 036–051 | `ast.rs`, `math_xml.rs`; `tests/math_ast.rs`, `math_ast_forms.rs`, `math_ast_advanced.rs`; security/code review | verified |
| FR-AST completion / SPEC-04 | 052–054 | `crates/math-model/`, boolean/unit/unsupported AST, parser regressions в `math_ast_completion.rs`, Serde/privacy tests | verified |
| FR-IR-001..007 / SPEC-04 | 055–061 | `crates/document-ir/`, V1 golden/round-trip/validation и backend-neutral ports в `tests/document_ir.rs` | verified |
| FR-DOCX-001..006 | 062–069 | `crates/exporter-docx/src/{package,image,validator,xml}.rs`, 11 package/text/image/page/attack integration tests | verified |
| FR-OMML-001..003 | 070–076 | `document-ir::ports::EquationExporter`, `exporter-docx/src/omml.rs`, 7 OMML snapshots и 3 equation-in-DOCX tests; independent/security review | verified |
| SPEC-05 расширенный OMML и backend configuration | 077–089 | `specs/features/advanced-omml-and-backend-config.spec.md`; `crates/exporter-docx/src/{omml,package,validator}.rs`, `tests/{omml,docx_equations,advanced_omml}.rs`, `examples/advanced_omml_reference.rs`; canonical power/root/scripts/function/grouping/matrix/calculus shapes, shared bounded validator, generated reference DOCX, Word/Open XML SDK evidence, independent/security review | verified |
| FR-MATHML-001..004, NFR/SEC-MATHML-001..002 | 090 | `specs/features/mathml-renderer.spec.md`; `crates/exporter-mathml/`, focused structural/negative/limit/port tests и independent/security review | verified |
| FR-MATHML-005, NFR-MATHML-003 | 091 | 17 `tests/golden/*.mathml`, `mathml_snapshots.rs`, exact inventory/bytes/canonical LF-root guards, origin-invariance regression и independent review | verified |
| FR/NFR/SEC-MATHTYPE-001..005 | 092 | `specs/features/experimental-mathtype-adapter.spec.md`; `crates/exporter-mathtype/`, exact payload/port/unsupported/depth/node/output/redaction tests; `cargo test -p exporter-mathtype`, `cargo test -p exporter-docx`, workspace 106 tests, fmt, Clippy, Python 20/20, project validator, diff check и independent architecture/security review | verified |
| FR-MTCOMP-001..005, NFR-MTCOMP-001, SEC-MTCOMP-001 | 093 | `specs/features/mathtype-compatibility-evidence.spec.md`; `docs/MATHTYPE_COMPATIBILITY.md`; exact 17-case matrix, official-source scope, environment probe, manual smoke protocol, project validator и negative contract tests | verified |
| SPEC-05 feature-gated DOCX backend selection | 094 | Требуется versioned live MathType import/edit `PASS`; `EquationBackend::MathType` остаётся typed unavailable без fallback | blocked |
| FR-TRANSFORM-095..099, NFR/SEC-TRANSFORM-001 | 095–099 | `specs/features/transformation-pipeline.spec.md`; `crates/math-engine/`; immutable Original→Display AST, bounded transforms, profile/registry, deterministic semantic-preservation tests и два review-cycle | verified |
| FR/NFR/SEC-SEMANTIC-100 | 100 | `specs/features/semantic-dependency-analysis.spec.md`; `crates/math-engine/src/symbol_table.rs`; ordered scalar/function revisions, visible-before lookup, borrowed bounded preflight, shared canonical AST, 10 targeted regressions и independent/security review | verified |
| FR-SEMANTIC-101..105 | 101–105 | variable/callable references, dependency graph, faithful worksheet order, undefined/cycle diagnostics | planned |
| FR/NFR/SEC-SUBSTITUTE-106..111 | 106–111 | `specs/features/substitution-and-evaluation-display.spec.md`; substitution, trace, explicit display modes и precision policy | planned |
| FR-CONVERT-143, FR-DIAG-144, FR-SEVERITY-145, FR-REPORT-146, FR-PARTIAL-147 | 143–147 | `specs/features/conversion-pipeline-and-report.spec.md`; `crates/conversion-core/`; XMCD→parser→transform→Document IR→DOCX→validator, bounded/redacted diagnostics, report/fidelity/partial tests и два review-cycle | verified |
| SPEC-07 комплексные числа | 112–122 | round-trip проверки алгебраического и полярного представления и граничные тесты | planned |
| SPEC-08 графики | 123–132 | PlotIR/ChartIR, fallback предпросмотра и fixtures восстановления | planned |
| SPEC-09 схемы | 133–142 | DiagramIR, растровый fallback и доказательства редактируемого POC VSDX | planned |
| FR/NFR/SEC-CLI-148 | 148 | `specs/features/minimal-cli-convert.spec.md`; `crates/mathmorph-cli/`; настоящий process E2E, valid DOCX/editable OMML, bounded input, no-replace publication, redaction, same-path/symlink/reparse/security regressions и два reviewer/security cycle | verified |
| SPEC-13 расширение локального CLI | 149–153 | `inspect`, exporter registry/options и стабильный JSON/Document IR report | planned |
| FR-WEB-SHELL-001..005, NFR-WEB-SHELL-001..002, SEC-WEB-SHELL-001 | 154 | `specs/features/visible-nextjs-shell.spec.md`; `apps/web/app/`; unit/component/integration render tests, static production build и Playwright desktop/mobile/light/dark interaction smoke | verified |
| SPEC-10–11 converter UI и состояния конвертации | 155–161 | design compliance, dropzone, validation/settings/states, Error Boundary и localized error mapping | planned |
| SPEC-16 интернационализация | 162–165 | внешние каталоги и проверка отсутствующих ключей в CI | planned |
| SPEC-12 аутентификация и восстановление | 173–186 | граница аутентификации, тесты replay, brute force и восстановления | planned |
| SPEC-13 API и ключи API | 166–172, 191–217 | `services/api/`, тесты контракта, авторизации, задач и идемпотентности | planned |
| SPEC-14 сохранённые документы | 187–190, 200–217 | границы метаданных и объектного хранилища, тесты срока хранения и удаления | planned |
| SPEC-15 направление конфиденциальности | 218–232 | ADR конфиденциальности, прототипы WASM и шифрования и тесты границ | planned |
| SPEC-17 оплата и монетизация | 245–256 | абстракция provider, тесты прав и жизненного цикла | planned |
| SPEC-10 граница администрирования и конфиденциальности | 257–270 | RBAC, области метрик и тесты запрета открытого текста для администратора | planned |
| SPEC-18 нефункциональные требования | 233–244, 271–304 | доказательства безопасности, наблюдаемости, benchmarks, CI и масштабирования | planned |
| SPEC-19 критерии успеха MVP | совокупно | end-to-end набор приёмки, охватывающий полный путь MVP | planned |

## Правила обновления

- Обновляй матрицу в том же изменении, которое принимает изменение соответствия требования этапу или проверяет этап.
- Используй только `planned`, `in progress`, `blocked` или `verified`.
- `verified` требует закоммиченной реализации, успешных относящихся к ней тестов и завершённого review; письменного prompt, плана или каркаса недостаточно.
- После появления кода указывай точные пути к коду и тестам. Не придумывай будущие модули и не заявляй о невыполненных проверках.
- Если реализация намеренно меняет архитектурную границу, обнови или добавь ADR вместо незаметного изменения соответствия.
