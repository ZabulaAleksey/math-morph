# Статус проекта

## Снимок состояния

- **Статус:** этапы 001–093, 095–105, 143–148 и независимый frontend-этап 154 реализованы и проверены; этапы 106–142 остаются planned.
- **Текущий backend-этап:** 148 завершён — доступен реальный локальный legacy XMCD→DOCX путь через binary `mathmorph`.
- **Текущий frontend-этап:** 154 (`Next.js shell`) завершён; публичный UX/UI уже виден, но upload/converter flow намеренно не подключён.
- **Технический hardening:** fallback-policy, TOML subagents и Node/pnpm toolchain contract синхронизированы и проверены в текущей fix-ветке.
- **Fallback catalog:** `docs/FALLBACKS.md` является канонической MathMorph-specific delta и обязателен для project validator.
- **Blockers:** этап 094 нельзя начинать без versioned live MathType import/edit `PASS`; локально MathType/SDK не установлен, SDK license отсутствует, интерактивный web smoke runner недоступен.
- **Следующие этапы:** substitution/display 106–111, затем complex engine 112–122. Этап 094 остаётся `blocked by versioned live evidence`; diagram track 133–140 требует подтверждённых format fixtures/schema.

## Реализовано

- Воспроизводимый Cargo/uv/pnpm monorepo, project overlay, canonical docs и versioned fixture corpus.
- Безопасная входная граница XMCD/MCDX, worksheet30 parser и синтаксический Math AST этапов 015–051.
- Утверждён канонический MathMorph Calm Blue UI design contract с `light`, `dark`, `system` и независимыми accessibility/density/workspace modes.
- Этап 154: `/` статически рендерит видимую украинскую public landing shell с responsive navigation, hero/workflow preview, feature/process/privacy/API/pricing/status sections, честным staged converter state и переключателем `system → light → dark` без flash.
- `math-model`: source-neutral AST, boolean expressions, units, `UnsupportedNode`, строгий Serde contract и redacted `Debug`.
- `document-ir`: versioned V1 JSON envelope, metadata/pages/layout, text/equation/table/image/plot/diagram blocks, provenance/fidelity и external asset ports.
- `exporter-docx`: детерминированный OPC/DOCX subset, WordprocessingML text/styles, bounded PNG/JPEG embedding, page settings и fail-closed structural validator.
- `WordEquationExporter`: editable OMML для базовых и расширенных форм 077–086 — powers, roots, scripts, functions, grouping, vector/matrix и calculus/aggregate — с каноническими shapes и typed fail-closed errors.
- Этап 087: общие equation byte/node/depth quotas renderer/validator, iterative linear traversal с work budget, расширенный OMML allowlist и negative regressions.
- Этап 088: воспроизводимый `advanced_omml_reference.docx`, example generator и структурные проверки; Word/Open XML SDK evidence зафиксировано в `docs/LEARNING_LOG.md`.
- Этап 089: public `EquationBackend` и `DocxExportConfig`; `WordOmml` — default, `MathType` зарезервирован и завершается typed unavailable error без fallback.
- `exporter-mathml`: отдельный bounded Presentation MathML Core renderer этапа 090 для real/identifier/subscript, basic binary operators, square root и paired grouping; реализует backend-neutral `EquationExporter` без подключения DOCX/MathType.
- Этап 091: 17 reviewable standalone `.mathml` golden snapshots, exact inventory/byte comparison, canonical UTF-8/LF/root guard и recursive origin-invariance regression; production renderer/API не изменялись.
- Этап 092: отдельный `exporter-mathtype` формирует opaque bounded Presentation MathML payload через production `MathMlRenderer`, не принимает raw XML и не подключает SDK/OLE/DOCX backend.
- Этап 093: `docs/MATHTYPE_COMPATIBILITY.md` фиксирует exact 17-case matrix, official-source scope, независимые static/live/edit evidence statuses, read-only environment probe и воспроизводимый smoke protocol; validator запрещает missing/duplicate cases, неизвестные статусы и ложный `VERIFIED` при `NOT_RUN`.
- Этапы 095–099: `math-engine` реализует bounded immutable Original AST→Display AST pipeline, explicit definition/symbol presentation rules, `NotationProfile` и deterministic semantic-preservation regressions.
- Этап 100: `math-engine::SymbolTable` хранит ordered scalar/function revisions с arity, top-to-bottom visible-before lookup, borrowed cumulative AST/text/collection preflight и одной canonical AST-копией через `Arc`.
- Этап 101: `math-engine::ReferenceAnalyzer` детерминированно извлекает свободные variable/function references, дедуплицирует их внутри source site, учитывает lexical binders и отклоняет malformed/unsupported формы через bounded redacted errors.
- Этап 102: `math-engine::DependencyGraph` связывает definition revisions только с видимыми prior definitions, сохраняет forward/missing references отдельно и создаёт exact callable self-edge только для последующей диагностики recursion/cycle.
- Этап 103: `math-engine::EvaluationPlan` итеративно строит полный dependency-first порядок со stable source-ordinal tie-break; unresolved/cyclic graph возвращает typed error без partial plan.
- Этап 104: `math-engine::SemanticDiagnostics` детерминированно превращает unresolved variable/function references в bounded typed diagnostics без сохранения symbol identity; public Debug/Display остаются redacted.
- Этапы 143–147: `conversion-core` реализует независимый от адаптеров путь `detect → parse → transform → Document IR → DOCX → validate`, bounded/redacted diagnostics, severity/fidelity report и explicit safe partial policy.
- Этап 148: binary `mathmorph convert <input.xmcd> --to docx [--output <path>]` читает input с hard limit, использует `AllowSafePartial`, не перезаписывает output и публикует DOCX через same-directory atomic no-replace hard link.
- Typed errors и configurable limits на JSON, XML, ZIP, images, AST/OMML depth, nodes и output bytes.

## Проверка этапов 100–103

- `cargo test -p math-engine --locked` — PASS, включая 11 `SymbolTable`, 11 `ReferenceAnalyzer`, 12 `DependencyGraph` и 9 `EvaluationPlan` regressions.
- `cargo test --workspace --locked` — PASS, 189 Rust tests.
- `cargo fmt --all -- --check` — PASS.
- `cargo clippy --workspace --all-targets --locked -- -D warnings` — PASS.
- `python -B scripts/validate_project.py` и `python -B scripts/validate_fixtures.py` — PASS.
- Python repository tests — PASS, 29/29.
- Independent code review — PASS после исправления cumulative budgets, zero-depth contract и clone amplification.
- Security review — PASS после borrowed preflight, payload/collection accounting, bounded lookup и extreme-depth subprocess regression.
- Stage 101 independent/security review — PASS после исправления nested `UnitedValue` preflight, malformed identifiers, per-site dedup и O(1)-average lexical scope lookup.
- Stage 102 independent/security review — PASS после переноса graph allocation за preflight, разделения raw/materialized reference budgets и бинарного revision lookup.
- Stage 103 independent/security review — PASS; bounded iterative Kahn traversal, stable ordering, checked limits и fail-closed unresolved/cycle semantics подтверждены.

## Проверки этапов 001–092

- `python -B scripts/validate_project.py` — PASS.
- `python -B scripts/validate_fixtures.py` — PASS.
- `python -B -m unittest discover -s tests -p "test_*.py" -v` — PASS, 20/20.
- `cargo fmt --all -- --check` — PASS.
- `cargo test --workspace --locked` — PASS, 106 Rust tests.
- `cargo clippy --workspace --all-targets --locked -- -D warnings` — PASS.
- `pnpm.cmd run typecheck` и `pnpm.cmd run build:web` — PASS; на момент проверки этапов 001–092 маршрут `/` ещё возвращал пустую страницу (заменено stage-154 shell).
- `cargo run -p exporter-docx --example advanced_omml_reference` — PASS, reference artifact generated.
- Structural DOCX/OMML validator — PASS.
- Independent automated review — PASS после исправлений.
- Security review DOCX/OMML — PASS после закрытия обхода equation byte/node/depth limits в validator.
- Independent/security review MathML — PASS после hardening cumulative input-text budget, numeric validation и Cargo dependency scopes.
- Independent review MathML snapshots — завершён после исправления Windows `core.autocrlf` portability и malformed nested-root guard; targeted regression PASS.
- Word 16.0 open/enumerate/edit check — PASS: artifact opened, one `OMath` exposed, `Linearize→BuildUp` preserved one `OMath`.
- Microsoft Open XML SDK 2.5.4728 validator — PASS, 0 errors.
- `cargo audit` — не запускался: subcommand не установлен; direct dependency review выполнен, автоматический `Cargo.lock` advisory scan остаётся отдельным этапом 242.
- `git diff --check` — PASS.

## Проверка этапа 092

- `cargo fmt --all -- --check` — PASS.
- `cargo test -p exporter-mathtype --locked` — PASS, 4/4 integration tests.
- `cargo test -p exporter-docx --locked` — PASS, 30 Rust tests.
- `cargo test --workspace --locked` — PASS, 106 Rust tests.
- `cargo clippy --workspace --all-targets --locked -- -D warnings` — PASS.
- `python -B scripts/validate_project.py` — PASS.
- Python repository tests — PASS, 20/20.
- `git diff --check` — без whitespace errors; присутствуют только информационные LF→CRLF warnings Windows Git.
- Read-only architecture/security review — PASS, существенных findings нет.
- `EquationBackend::MathType` намеренно остаётся `EquationBackendUnavailable`.

## Проверка corrective stage

- `python -B scripts/validate_project.py` — PASS.
- `python -B scripts/validate_fixtures.py` — PASS.
- Python repository tests — PASS, 20/20.
- TOML subagent configs — PASS через Python `tomllib`.
- `cargo fmt --all -- --check` — PASS.
- `cargo test --workspace --locked` — PASS.
- `cargo clippy --workspace --all-targets --locked -- -D warnings` — PASS.
- Node `v22.23.1` и pnpm `11.20.0` соответствуют project contract.
- `pnpm.cmd install --frozen-lockfile` — PASS.
- `pnpm.cmd run typecheck` и `pnpm.cmd run build:web` — PASS.

## Проверка этапа 093

- Focused `test_mathtype_compatibility.py` — PASS, 9/9.
- Python repository tests — PASS, 29/29.
- `python -B scripts/validate_project.py` и `python -B scripts/validate_fixtures.py` — PASS.
- `cargo fmt --all -- --check` — PASS.
- `cargo test -p exporter-mathml --locked` — PASS, 10 integration tests.
- `cargo test -p exporter-docx --locked` — PASS, 30 Rust tests.
- `cargo test --workspace --locked` — PASS, 106 Rust tests.
- `cargo clippy --workspace --all-targets --locked -- -D warnings` — PASS.
- `pnpm.cmd run typecheck` и `pnpm.cmd run build:web` — PASS.
- `git diff --check` — PASS; только информационные Windows LF→CRLF warnings.
- Два read-only review-cycle завершены; все findings исправлены, включая fail-closed provenance records, календарные даты, artifact SHA-256 и уникальный overall status; финальная self-validation — PASS.
- Live MathType Web/Desktop import/edit — `NOT_RUN / UNVERIFIED` по зафиксированной причине; static documentation не выдана за runtime evidence.

## Проверка этапа 154

- `pnpm.cmd --filter @math-morph/web test` — PASS, unit/component/integration 3/3.
- `pnpm.cmd --filter @math-morph/web typecheck` — PASS.
- `pnpm.cmd --filter @math-morph/web build` — PASS; `/` статически prerendered.
- `python -B scripts/validate_project.py` — PASS.
- Playwright 1.62.1 production smoke на `1440 × 1000`, `390 × 844` и граничных `320 × 800` — PASS: правильные URL/title/landmarks, нет blank/overlay/console errors, нет horizontal overflow; keyboard menu также проверен с `forced-colors` и `reduced-motion`.
- Interaction smoke — PASS: hero anchor, theme `system → light → dark`, compact menu open → privacy anchor → close → focus return.
- Visual QA accepted concepts ↔ latest desktop/mobile/dark screenshots — PASS; сохранены плоский primary token, thin borders, whitespace, section hierarchy и staged-state copy.
- Финальный независимый Lighthouse review на `320 × 900` — PASS: Accessibility, Best Practices и SEO получили `100` в light и dark themes; contrast regression закрыт через semantic `--cbui-color-primary-text`.
- Встроенный Browser runtime не загрузился из-за инфраструктурного запрета импорта `node:process`; использован разрешённый Playwright fallback без изменения project dependencies/lockfile.

## Проверка функционального этапа 148

- `cargo fmt --all -- --check` — PASS.
- `cargo test --workspace --locked` — PASS, 146 Rust tests.
- `cargo clippy --workspace --all-targets --locked -- -D warnings` — PASS.
- `python -B scripts/validate_project.py` и `python -B scripts/validate_fixtures.py` — PASS.
- Python repository tests — PASS, 29/29.
- Process E2E настоящего `mathmorph` binary — PASS: supported XMCD создаёт structurally valid DOCX с editable `m:oMath`; mixed content возвращает warning и artifact; invalid/MCDX/oversized input не создают output.
- Existing output, same input/output, symlink/reparse components, Windows UNC/device namespace, no-replace race, temp ownership, Unix `0600` и payload/path redaction regressions — PASS.
- Два независимых review/security cycle завершены; overwrite через Unix `rename`, temp ownership, redacted `Debug` и post-commit cleanup semantics исправлены. Остаточные std-only TOCTOU/ACL/directory-fsync ограничения задокументированы и не имеют silent fallback.

## Известные ограничения

- Parser поддерживает подтверждённое legacy worksheet30/math30 подмножество, а не полный runtime XSD validator; Prime MCDX worksheet пока не имеет content parser.
- Corpus преимущественно synthetic; совместимость расширяется только легально доступными образцами.
- `conversion-core` producer реализован только для legacy XMCD text и поддержанной math-семантики; Prime MCDX content, assets, plots/tables/diagrams и evaluation остаются explicit unsupported/planned.
- DOCX subset поддерживает одну страницу, text, PNG/JPEG и поддержанные OMML equations. Table, plot/diagram без preview, unsupported blocks и multiple pages отклоняются явно.
- MathML renderer и experimental adapter покрывают только принятый scalar subset. Матрица этапа 093 документирует static coverage, но live MathType Web/Desktop compatibility не доказана; DOCX `MathType` backend по-прежнему fail closed.
- Минимальный CLI реализован, но `inspect`, расширенные options и стабильный JSON report относятся к этапам 149–153. API endpoints и интерактивный web converter flow ещё не подключены; landing shell не содержит file input или сетевых запросов.
- Пользовательские строки stage 154 изолированы в украинском typed catalog, но полноценные locale routing/catalog loading и проверка отсутствующих ключей остаются этапами 162–165; текущая shell не является полной i18n-реализацией.
- На Windows Rust MSVC требует Visual Studio Build Tools с workload `Desktop development with C++` и доступный `link.exe`.

## Следующие разумные действия

1. Проверить локальную команду этапа 148 на реальном legacy `.xmcd` и решить вопрос merge stacked ветки.
2. Реализовать CLI этапы 149–153: `inspect`, exporter/options registry и стабильный machine-readable report.
3. Выполнить frontend-этап 155 перед dropzone и подключать UI к живому пути только после появления выбранной browser/API adapter boundary.
4. Реализовать i18n 162–165 для полноценного Ukrainian/Russian/English locale coverage; текущий украинский catalog stage 154 остаётся промежуточным.
5. Планировать этап 094 только после versioned MathType `PASS`; до этого `EquationBackend::MathType` сохраняет typed fail-closed поведение.
