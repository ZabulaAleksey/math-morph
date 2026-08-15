# Статус проекта

## Снимок состояния

- **Статус:** этапы 001–090 реализованы и проверены.
- **Текущий этап:** 090 завершён на ветке `feature/stage-090`.
- **В работе:** нет.
- **Blockers:** нет.
- **Следующий ещё не начатый этап:** 091 (`MathML snapshots`).

## Реализовано

- Воспроизводимый Cargo/uv/pnpm monorepo, project overlay, canonical docs и versioned fixture corpus.
- Безопасная входная граница XMCD/MCDX, worksheet30 parser и синтаксический Math AST этапов 015–051.
- `math-model`: source-neutral AST, boolean expressions, units, `UnsupportedNode`, строгий Serde contract и redacted `Debug`.
- `document-ir`: versioned V1 JSON envelope, metadata/pages/layout, text/equation/table/image/plot/diagram blocks, provenance/fidelity и external asset ports.
- `exporter-docx`: детерминированный OPC/DOCX subset, WordprocessingML text/styles, bounded PNG/JPEG embedding, page settings и fail-closed structural validator.
- `WordEquationExporter`: editable OMML для базовых и расширенных форм 077–086 — powers, roots, scripts, functions, grouping, vector/matrix и calculus/aggregate — с каноническими shapes и typed fail-closed errors.
- Этап 087: общие equation byte/node/depth quotas renderer/validator, iterative linear traversal с work budget, расширенный OMML allowlist и negative regressions.
- Этап 088: воспроизводимый `advanced_omml_reference.docx`, example generator и структурные проверки; Word/Open XML SDK evidence зафиксировано в `docs/LEARNING_LOG.md`.
- Этап 089: public `EquationBackend` и `DocxExportConfig`; `WordOmml` — default, `MathType` зарезервирован и завершается typed unavailable error без fallback.
- `exporter-mathml`: отдельный bounded Presentation MathML Core renderer этапа 090 для real/identifier/subscript, basic binary operators, square root и paired grouping; реализует backend-neutral `EquationExporter` без подключения DOCX/MathType.
- Typed errors и configurable limits на JSON, XML, ZIP, images, AST/OMML depth, nodes и output bytes.

## Проверки этапов 001–090

- `python -B scripts/validate_project.py` — PASS.
- `python -B scripts/validate_fixtures.py` — PASS.
- `python -B -m unittest discover -s tests -p "test_*.py" -v` — PASS, 17/17.
- `cargo fmt --all -- --check` — PASS.
- `cargo test --workspace --locked` — PASS, 98 Rust tests.
- `cargo clippy --workspace --all-targets --locked -- -D warnings` — PASS.
- `cargo run -p exporter-docx --example advanced_omml_reference` — PASS, reference artifact generated.
- Structural DOCX/OMML validator — PASS.
- Independent automated review — PASS после исправлений.
- Security review DOCX/OMML — PASS после закрытия обхода equation byte/node/depth limits в validator.
- Independent/security review MathML — PASS после hardening cumulative input-text budget, numeric validation и Cargo dependency scopes.
- Word 16.0 open/enumerate/edit check — PASS: artifact opened, one `OMath` exposed, `Linearize→BuildUp` preserved one `OMath`.
- Microsoft Open XML SDK 2.5.4728 validator — PASS, 0 errors.
- `cargo audit` — не запускался: subcommand не установлен; direct dependency review выполнен, автоматический `Cargo.lock` advisory scan остаётся отдельным этапом 242.
- `git diff --check` — PASS.

## Известные ограничения

- Parser поддерживает подтверждённое legacy worksheet30/math30 подмножество, а не полный runtime XSD validator; Prime MCDX worksheet пока не имеет content parser.
- Corpus преимущественно synthetic; совместимость расширяется только легально доступными образцами.
- Document IR producer из worksheet AST ещё не реализован: `math-engine` остаётся каркасом.
- DOCX subset поддерживает одну страницу, text, PNG/JPEG и поддержанные OMML equations. Table, plot/diagram без preview, unsupported blocks и multiple pages отклоняются явно.
- MathML этапа 090 поддерживает только принятый scalar subset; comprehensive snapshots 091 и MathType adapter 092–094 не начаты, текущий `MathType` backend по-прежнему fail closed.
- CLI, API endpoints и пользовательский UI flow не реализованы; workspace не содержит CLI binary.
- На Windows Rust MSVC требует Visual Studio Build Tools с workload `Desktop development with C++` и доступный `link.exe`.

## Следующие разумные действия

1. Пользовательская проверка ветки и merge только после явного разрешения.
2. Выполнить отдельно этап 091 (`MathML snapshots`) по существующей SPEC, не подключая преждевременно MathType adapter.
3. По мере появления legal real-world corpus добавлять compatibility fixtures и regressions.
