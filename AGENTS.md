# MathMorph - local instructions

Before working here, read `~/codex-workspace/AGENTS.md`. MathMorph is a domain overlay, not a second generic AI team.

## Context routing

- Read `docs/AI_DEV_TEAM_COMPATIBILITY.md` before changing project agents, hooks, MCP, or context behavior.
- Parser work: `crates/mathcad-parser/AGENTS.md`.
- Math semantics: `crates/math-engine/AGENTS.md`.
- DOCX/OMML export: `crates/exporter-docx/AGENTS.md`.
- API: `services/api/AGENTS.md`.
- Web UI: `apps/web/AGENTS.md`.
- Tests and fixtures: `tests/AGENTS.md`.

## Project invariants

- Preserve the pipeline: input -> parser -> Mathcad AST -> semantics -> Document IR -> exporter.
- Parser and math-engine layers must not depend on Word, HTTP, or UI code.
- Supported equations remain editable structures; unsupported content produces explicit diagnostics rather than silent loss.
- For uploads, parsing, authentication, storage, or cryptography, read the relevant section of `docs/SECURITY.md`.
- Optional hooks and MCP snippets remain disabled until explicitly reviewed and enabled.

Validate context changes with `python scripts/validate_context_pack.py`. Load only the relevant document or SPEC section; never preload the whole prompt library, rules tree, or `LEARNING_LOG.md`.
