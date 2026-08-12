# Architecture Decisions

## ADR-0001 — Progressive context loading

**Decision:** keep root AGENTS compact; domain rules live in nested AGENTS, Skills and focused docs.

**Reason:** reduce context duplication while preserving precise instructions near the affected code.

## ADR-0002 — Hooks are guardrails, not application security

**Decision:** hooks enforce only deterministic developer-workflow checks. Application/CI security remains authoritative.

## ADR-0003 — Subagents inherit parent model unless explicitly needed

**Decision:** project agent TOML files do not hardcode model names. This avoids stale model configuration and lets the parent/session policy choose the model.
