# Architecture Decisions

## ADR-0001 — Progressive context loading

**Status:** accepted.

**Decision:** keep root AGENTS compact; domain rules live in nested AGENTS, Skills and focused docs.

**Reason:** reduce context duplication while preserving precise instructions near the affected code.

## ADR-0002 — Hooks are guardrails, not application security

**Status:** accepted.

**Decision:** hooks enforce only deterministic developer-workflow checks. Application/CI security remains authoritative.

## ADR-0003 — Subagents inherit parent model unless explicitly needed

**Status:** accepted.

**Decision:** project agent TOML files do not hardcode model names. This avoids stale model configuration and lets the parent/session policy choose the model.

## ADR-0004 — Trace requirements without duplicating them

**Status:** accepted.

**Decision:** `docs/TRACEABILITY.md` maps canonical specification sections to roadmap stages and verification evidence. It does not restate product requirements.

**Reason:** future implementation needs a stable path from requirement to code and tests, while duplicate SPEC files would create context conflicts.

**Consequences:** update the mapping when an accepted requirement changes or a stage becomes verified. A roadmap item or prompt alone never proves implementation.

## ADR template

Use this only for consequential architectural or technical decisions.

### ADR-XXXX — Title

**Status:** proposed / accepted / superseded.

**Context:** problem, constraints and affected trust/module boundaries.

**Options:** viable alternatives considered.

**Decision:** selected approach.

**Reason:** why it best satisfies the constraints.

**Consequences:** trade-offs, migration and operational impact.

**Fallback / rollback:** safe reversal path.

**Verification:** tests or benchmarks that support the decision.

**Related requirements:** links to canonical SPEC sections or stable requirement IDs.
