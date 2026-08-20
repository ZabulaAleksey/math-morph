# Статус проекта

## Снимок состояния

- **Статус:** этапы 001–092 реализованы и проверены; fallback-policy integration присутствует в `main`.
- **Текущий продуктовый этап:** 092 завершён и интегрирован.
- **Технический hardening:** fallback-policy, TOML subagents и Node/pnpm toolchain contract синхронизированы и проверены в текущей fix-ветке.
- **Fallback catalog:** `docs/FALLBACKS.md` является канонической MathMorph-specific delta и обязателен для project validator.
- **Blockers:** нет.
- **Следующий ещё не начатый продуктовый этап:** 093 (`compatibility doc`).

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

## Известные ограничения

- Parser поддерживает подтверждённое legacy worksheet30/math30 подмножество, а не полный runtime XSD validator; Prime MCDX worksheet пока не имеет content parser.
- Corpus преимущественно synthetic; совместимость расширяется только легально доступными образцами.
- Document IR producer из worksheet AST ещё не реализован: `math-engine` остаётся каркасом.
- DOCX subset поддерживает одну страницу, text, PNG/JPEG и поддержанные OMML equations. Table, plot/diagram без preview, unsupported blocks и multiple pages отклоняются явно.
- MathML renderer и experimental adapter покрывают только принятый scalar subset. Adapter 092 лишь готовит payload; фактическая MathType compatibility не доказана, а DOCX `MathType` backend по-прежнему fail closed.
- CLI, API endpoints и пользовательский UI flow не реализованы; workspace не содержит CLI binary.
- На Windows Rust MSVC требует Visual Studio Build Tools с workload `Desktop development with C++` и доступный `link.exe`.

## Следующие разумные действия

1. Проверить и интегрировать текущую corrective fix-ветку только после явного решения владельца.
2. После интеграции при отдельном запросе начать этап 093 (`compatibility doc`) с реальными import/smoke evidence без изменения DOCX backend.
