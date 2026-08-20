# Текущий план AI

**Статус:** `in_progress`.
**Цель:** functional stage 148 — первая реальная локальная XMCD→DOCX конвертация через общий core и CLI.
**Ветка:** `feature/stage-148-cli-conversion`, stacked поверх этапов 093 и 154.
**Backend-этап 094:** `blocked by versioned live evidence`; MathType wiring запрещён.

## Dependency-aware scope

- Реализовать этапы 095–099: immutable Original AST → Display AST presentation pipeline.
- Этапы 100–142 сохранить `planned`: они не являются dependency первой faithful static conversion и не объявляются завершёнными.
- Реализовать этапы 143–147: `ConversionPipeline`, bounded diagnostics, severity, fidelity report и safe partial policy.
- Реализовать этап 148: `mathmorph convert <input.xmcd> --to docx` поверх production core.
- MCDX content parsing оставить explicit unsupported до отдельного подтверждённого Prime schema contract.

## Порядок

1. SPEC и ADR — in progress.
2. `math-engine` stages 095–099 — pending.
3. `conversion-core` stages 143–147 — pending.
4. `mathmorph-cli` stage 148 и live CLI→core→parser→IR→DOCX E2E — pending.
5. Full validation, security/reviewer cycles, traceability/status и atomic commits — pending.

## Канонические контракты

- `specs/features/transformation-pipeline.spec.md`
- `specs/features/conversion-pipeline-and-report.spec.md`
- `specs/features/minimal-cli-convert.spec.md`
- `docs/DECISIONS.md` — ADR-0015
