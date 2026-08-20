# Текущий план AI

**Статус:** `completed`.
**Завершённый продуктовый этап:** 093 — `compatibility doc`.
**Последний интегрированный в `main` этап:** 092 — `experimental MathType adapter`.
**Следующий продуктовый этап:** 094 — `feature-gated backend selection`, `blocked by versioned live evidence`.

## Результат

- Создана и проиндексирована SPEC этапа 093.
- `docs/MATHTYPE_COMPATIBILITY.md` сопоставляет exact 17-case golden inventory с official static coverage и отдельными live/edit statuses.
- Локальный environment probe не обнаружил MathType/SDK; MathType Web/Desktop live smoke честно отмечен `NOT_RUN / UNVERIFIED`.
- Project validator и девять новых negative/positive tests защищают inventory, evidence vocabulary и versioned provenance от завышенных compatibility claims.
- Runtime crates, DOCX backend selection и fallback contract не изменены.
- Unit, integration, component/build, workspace и lint checks прошли.

## Handoff

Этап 094 не начинается автоматически: сначала требуется versioned live MathType import/edit `PASS` для явно выбранной поверхности.
До появления такого evidence `EquationBackend::MathType` остаётся typed unavailable без silent fallback.
