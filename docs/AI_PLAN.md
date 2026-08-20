# Текущий план AI

**Статус:** `in_progress`.
**Цель:** последовательно реализовать и проверить все незакрытые этапы до 148, не ослабляя принятые контракты и evidence gates.
**Ветка:** `feature/stage-148-cli-conversion`, stacked поверх этапов 093 и 154.
**Backend-этап 094:** `blocked by versioned live evidence`; MathType wiring запрещён.

## Текущий scope

- Этапы 095–099 и 143–148 сохранить verified и защищать полной регрессией.
- Последовательно закрыть 100–105, 106–111, 112–122, 123–127, 128–132 и 133–142.
- Для каждого этапа соблюдать SPEC → implementation → targeted tests → integration/component regression → review → status.
- Этап 094 не объявлять выполненным без versioned live MathType import/edit `PASS`.

## Порядок

1. Baseline и gap audit 094/100–142 — completed.
2. Этап 100 `SymbolTable` — completed.
3. Этапы 101–105 — next.
4. Этапы 106–111 — pending.
5. Этапы 112–122 — pending.
6. Этапы 123–132 — pending.
7. Этапы 133–142 — pending.
8. Full regression, reviewer/security review, traceability/status — pending.
9. Этап 094 — blocked by external licensed live evidence.

## Канонические контракты

- `specs/features/transformation-pipeline.spec.md`
- `specs/features/semantic-dependency-analysis.spec.md`
- `specs/features/substitution-and-evaluation-display.spec.md`
- `specs/features/complex-numbers.spec.md`
- `specs/features/conversion-pipeline-and-report.spec.md`
- `specs/features/minimal-cli-convert.spec.md`
- `docs/DECISIONS.md` — ADR-0015, ADR-0016, ADR-0017
