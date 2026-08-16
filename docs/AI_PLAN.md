# Текущий план AI

**Статус:** `completed`.
**Каноническая ветка:** `feature/stage-092-mathtype-adapter`.
**Завершённый этап:** 092 — `experimental MathType adapter`.
**SPEC:** `specs/features/experimental-mathtype-adapter.spec.md`.
**Следующий ещё не начатый этап:** 093 — `compatibility doc`.

## Результат

Добавлен отдельный pure/offline crate `exporter-mathtype`. Он принимает поддерживаемый `MathExpression`, переиспользует bounded production `MathMlRenderer` и возвращает opaque read-only `application/mathml+xml` payload для будущего отдельно проверяемого MathType bridge.

`EquationBackend::MathType` в `exporter-docx` намеренно остаётся `EquationBackendUnavailable`; совместимость с установленным MathType/Word и DOCX wiring не заявляются.

## Выполненный план

1. Принят отдельный SPEC этапа 092 с acceptance criteria и явными non-goals.
2. Добавлен workspace crate только с internal path dependencies.
3. Реализованы opaque payload, typed redacted error и общий `EquationExporter` port.
4. Добавлены positive, unsupported, depth/node/output-limit и redaction tests.
5. Обновлены workspace, lockfile, project validator, architecture, ADR, testing и traceability.
6. Выполнены targeted/workspace gates и независимый architecture/security review.

## Фактические проверки

- `cargo fmt --all -- --check` — PASS.
- `cargo test -p exporter-mathtype --locked` — PASS, 4/4 integration tests.
- `cargo test -p exporter-docx --locked` — PASS, 30 Rust tests; зарезервированный MathType backend остаётся fail closed.
- `cargo test --workspace --locked` — PASS, 106 Rust tests.
- `cargo clippy --workspace --all-targets --locked -- -D warnings` — PASS.
- `python -B scripts/validate_project.py` — PASS (`math-morph project: OK`).
- `python -B -m unittest discover -s tests -p "test_*.py" -v` — PASS, 20/20.
- `git diff --check` — PASS без whitespace errors; Git вывел только предупреждения Windows LF→CRLF.
- Отдельный read-only architecture/security review — PASS, существенных findings нет.

## Сохранённые non-goals

- compatibility matrix и реальный import smoke этапа 093;
- feature-gated DOCX selection этапа 094;
- MathType/WIRIS SDK или service;
- OLE/COM/VBA/Word automation и MTEF;
- raw XML input, filesystem/network/process/registry access;
- расширение MathML AST subset;
- UI/API/CLI configuration.

## Откат

Удалить `exporter-mathtype`, его workspace/lockfile registration, SPEC и относящиеся documentation rows. `exporter-mathml`, `exporter-docx`, parser, Document IR и существующие backend defaults останутся неизменными.

## Handoff

Этап 092 готов к одному checkpoint commit и push временной ветки. После интеграции можно начинать только этап 093 — документирование реальной совместимости без преждевременного включения DOCX backend.
