# Текущий план AI

**Статус:** `in_progress`.
**Цель:** последовательно реализовать и проверить все безопасно выполнимые этапы до 154, не ослабляя принятые контракты и evidence gates.
**Ветка:** `feature/stage-106-simple-substitution`, stacked поверх verified этапов 093, 105, 148 и 154.
**Backend-этап 094:** `blocked by versioned live evidence`; MathType wiring запрещён.

## Текущий scope

- Этапы 095–099 и 143–148 сохранить verified и защищать полной регрессией.
- Последовательно закрыть 100–105, 106–111, 112–122, 123–127, 128–132 и 133–142.
- Для каждого этапа соблюдать SPEC → implementation → targeted tests → integration/component regression → review → status.
- Этап 094 не объявлять выполненным без versioned live MathType import/edit `PASS`.

## Порядок

1. Baseline и gap audit 094/100–142 — completed.
2. Этап 100 `SymbolTable` — completed.
3. Этап 101 `ReferenceAnalyzer` — completed.
4. Этап 102 `DependencyGraph` — completed.
5. Этап 103 `EvaluationPlan` — completed.
6. Этапы 104–105 `SemanticDiagnostics` — completed.
7. Этапы 106–111 — completed.
8. Этапы 112–122 — completed.
9. Этапы 123–132 — pending/evidence-gated; разрешена только подтверждённая opaque metadata, preview/series reconstruction требует fixtures.
10. Этапы 133–142 — pending/evidence-gated; diagram detection/forensics из Mathcad требует format evidence.
11. Этапы 143–148 — completed.
12. Этапы 149–153 — completed.
13. Этап 154 — completed.
14. Full regression, reviewer/security review, traceability/status — completed through stage 154 за исключением evidence-gated 094 и 123–142.
15. Этап 094 — blocked by external licensed live evidence.

## Канонические контракты

- `specs/features/transformation-pipeline.spec.md`
- `specs/features/semantic-dependency-analysis.spec.md`
- `specs/features/substitution-and-evaluation-display.spec.md`
- `specs/features/complex-numbers.spec.md`
- `specs/features/conversion-pipeline-and-report.spec.md`
- `specs/features/minimal-cli-convert.spec.md`
- `specs/features/cli-inspection-and-reports.spec.md`
- `docs/DECISIONS.md` — ADR-0015, ADR-0016, ADR-0017
