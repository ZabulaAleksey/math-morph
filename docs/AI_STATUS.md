# Статус проекта

## Снимок состояния

- **Статус:** этапы 001–092 интегрированы в `main`; этап 093 реализован и проверен в `feature/stage-093-mathtype-compatibility`.
- **Текущий продуктовый этап:** 093 (`compatibility doc`) завершён в feature-ветке; live MathType compatibility остаётся `UNVERIFIED`.
- **Технический hardening:** fallback-policy, TOML subagents и Node/pnpm toolchain contract синхронизированы и проверены в текущей fix-ветке.
- **Fallback catalog:** `docs/FALLBACKS.md` является канонической MathMorph-specific delta и обязателен для project validator.
- **Blockers:** этап 094 нельзя начинать без versioned live MathType import/edit `PASS`; локально MathType/SDK не установлен, SDK license отсутствует, интерактивный web smoke runner недоступен.
- **Следующий продуктовый этап:** 094 (`feature-gated backend selection`) — `blocked by versioned live evidence`.

## Реализовано

- Воспроизводимый Cargo/uv/pnpm monorepo, project overlay, canonical docs и versioned fixture corpus.
- Безопасная входная граница XMCD/MCDX, worksheet30 parser и синтаксический Math AST этапов 015–051.
- Утверждён канонический MathMorph Calm Blue UI design contract с `light`, `dark`, `system` и независимыми accessibility/density/workspace modes; пользовательский UI flow ещё не реализован.
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
- Typed errors и configurable limits на JSON, XML, ZIP, images, AST/OMML depth, nodes и output bytes.

## Проверки этапов 001–092

- `python -B scripts/validate_project.py` — PASS.
- `python -B scripts/validate_fixtures.py` — PASS.
- `python -B -m unittest discover -s tests -p "test_*.py" -v` — PASS, 20/20.
- `cargo fmt --all -- --check` — PASS.
- `cargo test --workspace --locked` — PASS, 106 Rust tests.
- `cargo clippy --workspace --all-targets --locked -- -D warnings` — PASS.
- `pnpm.cmd run typecheck` и `pnpm.cmd run build:web` — PASS; `/` статически собирается, но пока возвращает пустую страницу.
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

## Известные ограничения

- Parser поддерживает подтверждённое legacy worksheet30/math30 подмножество, а не полный runtime XSD validator; Prime MCDX worksheet пока не имеет content parser.
- Corpus преимущественно synthetic; совместимость расширяется только легально доступными образцами.
- Document IR producer из worksheet AST ещё не реализован: `math-engine` остаётся каркасом.
- DOCX subset поддерживает одну страницу, text, PNG/JPEG и поддержанные OMML equations. Table, plot/diagram без preview, unsupported blocks и multiple pages отклоняются явно.
- MathML renderer и experimental adapter покрывают только принятый scalar subset. Матрица этапа 093 документирует static coverage, но live MathType Web/Desktop compatibility не доказана; DOCX `MathType` backend по-прежнему fail closed.
- CLI, API endpoints и пользовательский UI flow не реализованы; workspace не содержит CLI binary.
- На Windows Rust MSVC требует Visual Studio Build Tools с workload `Desktop development with C++` и доступный `link.exe`.

## Следующие разумные действия

1. Проверить feature-ветку этапа 093 и интегрировать её только после явного решения владельца.
2. Предоставить явно разрешённую среду MathType Web либо установленный MathType 7 + SDK license и выполнить 17-case live import/edit smoke по `docs/MATHTYPE_COMPATIBILITY.md`.
3. Планировать этап 094 только после versioned `PASS` evidence для выбранной поверхности; до этого `EquationBackend::MathType` сохраняет typed fail-closed поведение.
