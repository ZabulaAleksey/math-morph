# Статус проекта

## Снимок состояния

- **Статус:** этапы 001–076 реализованы и проверены.
- **Текущий этап:** 076 завершён на ветке `feature/stages-052-076`.
- **В работе:** нет.
- **Blockers:** нет.
- **Следующий ещё не начатый этап:** 077 (`powers` в OMML).

## Реализовано

- Воспроизводимый Cargo/uv/pnpm monorepo, project overlay, canonical docs и versioned fixture corpus.
- Безопасная входная граница XMCD/MCDX, worksheet30 parser и синтаксический Math AST этапов 015–051.
- `math-model`: source-neutral AST, boolean expressions, units, `UnsupportedNode`, строгий Serde contract и redacted `Debug`.
- `document-ir`: versioned V1 JSON envelope, metadata/pages/layout, text/equation/table/image/plot/diagram blocks, provenance/fidelity и external asset ports.
- `exporter-docx`: детерминированный OPC/DOCX subset, WordprocessingML text/styles, bounded PNG/JPEG embedding, page settings и fail-closed structural validator.
- `WordEquationExporter`: editable OMML для numbers, identifiers, add/subtract, multiplication styles и nested fractions; exporter использует только `FormulaIr.display`.
- Typed errors и configurable limits на JSON, XML, ZIP, images, AST/OMML depth, nodes и output bytes.

## Проверки этапов 052–076

- `python -B scripts/validate_project.py` — PASS.
- `python -B scripts/validate_fixtures.py` — PASS.
- `python -B -m unittest discover -s tests -p "test_*.py" -v` — PASS, 15/15.
- `cargo fmt --all -- --check` — PASS.
- `cargo test --workspace --locked` — PASS, 85 Rust tests.
- `cargo clippy --workspace --all-targets --locked -- -D warnings` — PASS.
- Independent review 052–054 — PASS.
- Independent review 070–076 — PASS.
- Security review DOCX/OMML — PASS после закрытия обхода equation byte/node/depth limits в validator.
- `cargo audit` — не запускался: subcommand не установлен; direct dependency review выполнен, автоматический `Cargo.lock` advisory scan остаётся отдельным этапом 242.
- `git diff --check` — PASS.

## Известные ограничения

- Parser поддерживает подтверждённое legacy worksheet30/math30 подмножество, а не полный runtime XSD validator; Prime MCDX worksheet пока не имеет content parser.
- Corpus преимущественно synthetic; совместимость расширяется только легально доступными образцами.
- Document IR producer из worksheet AST ещё не реализован: `math-engine` остаётся каркасом.
- DOCX subset поддерживает одну страницу, text, PNG/JPEG и базовые equations. Table, plot/diagram без preview, unsupported blocks и multiple pages отклоняются явно.
- OMML stages 077+ (powers, roots, subscripts, functions, brackets, matrices, calculus) не начаты.
- CLI, API endpoints и пользовательский UI flow не реализованы; `cargo run` пока нечего запускать.
- На Windows Rust MSVC требует Visual Studio Build Tools с workload `Desktop development with C++` и доступный `link.exe`.

## Следующие разумные действия

1. Пользовательская проверка ветки и merge только после явного разрешения.
2. Следующий отдельный пакет начинать с этапа 077, не расширяя автоматически verified OMML subset.
3. Добавить ручной Word/Open XML SDK smoke test, когда будет доступно окружение с Microsoft Word или SDK.
4. По мере появления legal real-world corpus добавлять compatibility fixtures и regressions.
