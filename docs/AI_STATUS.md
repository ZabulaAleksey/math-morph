# Статус проекта

## Снимок состояния

- **Статус:** этапы 001–051 реализованы и проверены.
- **Текущий этап:** 051 завершён на ветке `feature/stages-027-051`; ветка готова к пользовательской проверке и последующему merge по явному разрешению.
- **В работе:** нет.
- **Blockers:** нет.
- **Следующий ещё не начатый этап:** 052 (`booleans`).

## Реализовано

- Воспроизводимый Cargo/uv/pnpm monorepo и минимальные Rust/Python/Next.js каркасы.
- Project overlay, canonical docs, versioned fixture corpus и fail-closed validators.
- Утверждён канонический MathMorph Calm Blue UI design contract с `light`, `dark`, `system` и независимыми accessibility/density/workspace modes; пользовательский UI flow ещё не реализован.
- Content-based XMCD/MCDX detection, `FILE_EXTENSION_MISMATCH` и безопасный MCDX container manifest без extraction.
- UTF-8 XML root metadata inspection с запретом DTD/entities.
- `WorksheetParser` для подтверждённого legacy contract worksheet30 3.0.3 + math30 3.0.2.
- Metadata, recursive regions, finite layout, source/visual/z ordering, text/inline attributes, math, plot/picture/table references и source-backed opaque fragments.
- Синтаксический Math AST этапов 036–051: literals/arithmetic, definitions/evaluation/functions, unary/grouping/index, matrix/vector/range, calculus/aggregates/comparisons.
- Typed invalid/unsupported outcomes, source spans и configurable resource limits.

## Проверки этапов 027–051

- `python -B scripts/validate_project.py` — PASS.
- `python -B scripts/validate_fixtures.py` — PASS.
- `python -B -m unittest discover -s tests -p "test_*.py" -v` — PASS, 14/14.
- `cargo fmt --all -- --check` — PASS.
- `cargo test --workspace --locked` — PASS, 46 Rust integration tests.
- `cargo clippy --workspace --all-targets --locked -- -D warnings` — PASS.
- Security review — PASS после закрытия namespace-memory amplification и numeric Debug leakage.
- Independent code review — PASS после закрытия userData opaque preservation, one-element index sequence и signed-zero ordering.
- Production dependencies не менялись; предыдущий locked `cargo-audit` result остаётся применимым.

## Известные ограничения

- Parser реализует подтверждённое подмножество, а не полный runtime XSD validator.
- Corpus хранит synthetic fixtures; совместимость с реальными документами расширяется только легально доступными образцами.
- Prime MCDX безопасно инспектируется как контейнер, но его внутренний worksheet ещё не имеет подтверждённого content parser.
- Boolean AST/evaluation, units, generic `UnsupportedNode`, `DocumentIR`, evaluator, exporters, CLI, API endpoints и UI flow не реализованы.
- `math-engine`, `exporter-docx`, Python API и Next.js остаются каркасами.
- На Windows Rust MSVC требует Visual Studio Build Tools с workload `Desktop development with C++` и доступный `link.exe`.

## Следующие разумные действия

1. Пользовательская проверка ветки и merge только после явного разрешения.
2. Новый отдельный пакет начиная с этапа 052, не включая его автоматически в завершённый AST contract.
3. По мере появления legal real-world corpus добавлять compatibility fixtures и regressions.
