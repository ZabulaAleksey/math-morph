# Текущий план AI

**Статус:** `in_progress`.
**Цель:** последовательно реализовать и проверить все незакрытые этапы до 148, не ослабляя принятые контракты и evidence gates.
**Ветка:** `feature/stage-104-undefined-diagnostics`, stacked поверх verified этапов 093, 103, 148 и 154.
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
6. Этап 104 `SemanticDiagnostics` — completed.
7. Этап 105 `circular dependency diagnostic` — next.
8. Этапы 106–111 — pending.
9. Этапы 112–122 — pending.
10. Этапы 123–132 — pending.
11. Этапы 133–142 — pending/evidence-gated.
12. Full regression, reviewer/security review, traceability/status — completed through stage 104.
13. Этап 094 — blocked by external licensed live evidence.

## Канонические контракты

- `specs/features/transformation-pipeline.spec.md`
- `specs/features/semantic-dependency-analysis.spec.md`
- `specs/features/substitution-and-evaluation-display.spec.md`
- `specs/features/complex-numbers.spec.md`
- `specs/features/conversion-pipeline-and-report.spec.md`
- `specs/features/minimal-cli-convert.spec.md`
- `docs/DECISIONS.md` — ADR-0015, ADR-0016, ADR-0017
