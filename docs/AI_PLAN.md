# Текущий план AI

**Статус:** `validated`; дальнейшее продвижение format-specific этапов заблокировано внешними evidence gates.
**Цель:** безопасно выполнимые этапы до 154 реализованы и проверены без ослабления принятых контрактов.
**Ветка:** `feature/stage-106-simple-substitution`, stacked поверх verified этапов 093, 105, 148 и 154.
**Backend-этап 094:** `blocked by versioned live evidence`; MathType wiring запрещён.

## Завершённый scope

- Этапы 095–124, 127 и 143–154 validated локально.
- Компонентные preview paths 126/134 validated; live Mathcad extraction отсутствует.
- Этапы 094, 125, 128–133 и 135–142 не объявлены выполненными без обязательного format/live evidence.
- Полный Rust/Python regression, reviewer, security review и синхронизация контекста завершены.

## Порядок

1. Baseline и gap audit 094/100–142 — completed.
2. Этап 100 `SymbolTable` — completed.
3. Этап 101 `ReferenceAnalyzer` — completed.
4. Этап 102 `DependencyGraph` — completed.
5. Этап 103 `EvaluationPlan` — completed.
6. Этапы 104–105 `SemanticDiagnostics` — completed.
7. Этапы 106–111 — completed.
8. Этапы 112–122 — completed.
9. Этапы 123–124 и 127 — completed; 126 component verified; 125 и 128–132 blocked by versioned plot evidence.
10. Этап 134 component verified; 133, 135–142 pending/evidence-gated, diagram detection/forensics из Mathcad требует format evidence, VSDX editability — live Visio evidence.
11. Этапы 143–148 — completed.
12. Этапы 149–153 — completed.
13. Этап 154 — completed.
14. Full regression, reviewer/security review, traceability/status — completed through stage 154 за исключением evidence-gated 094, 125, 128–133 и 135–142.
15. Этап 094 — blocked by external licensed live evidence.

## Следующая точка входа

1. Если появились legal versioned plot/diagram fixtures — продолжить с 125, затем повторно оценить 128–142.
2. Если evidence пока нет — следующий независимый этап после текущего диапазона: 155 (`design compliance checklist`).
3. Этап 094 возобновлять только после versioned MathType import/edit `PASS`.

## Канонические контракты

- `specs/features/transformation-pipeline.spec.md`
- `specs/features/semantic-dependency-analysis.spec.md`
- `specs/features/substitution-and-evaluation-display.spec.md`
- `specs/features/complex-numbers.spec.md`
- `specs/features/conversion-pipeline-and-report.spec.md`
- `specs/features/minimal-cli-convert.spec.md`
- `specs/features/cli-inspection-and-reports.spec.md`
- `specs/features/plot-diagram-evidence-gates.spec.md`
- `docs/DECISIONS.md` — ADR-0015, ADR-0016, ADR-0017, ADR-0018
