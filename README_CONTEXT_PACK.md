# Mathcad Converter & Parser Platform — repository context pack

This archive is laid out to be extracted **directly into the repository root**.
There is no wrapper directory inside the ZIP.

## Important compatibility rule

A global **AI Dev Team Codex** is already assumed to exist. This repository is a domain overlay, not a second generic agent team.

- Generic architect/QA/security/frontend/backend/DevOps/release/Git roles: reuse the global AI Dev Team.
- Active local agents: only Mathcad-specific format/parser/math/OpenXML specialists.
- Project hooks: disabled by default; templates are optional.
- Project MCP: disabled by default; templates are optional.
- Generic fallback agents/skills remain outside active discovery directories.

Read first:

1. `AGENTS.md`
2. `docs/AI_DEV_TEAM_COMPATIBILITY.md`
3. only the task-relevant local `AGENTS.md` and docs.

## Canonical documents

- `docs/SPECIFICATION.md` — product requirements.
- `docs/TECH_STACK.md` — approved baseline stack.
- `docs/ARCHITECTURE.md` — system boundaries and data flow.
- `docs/ROADMAP.md` — staged development sequence.
- `docs/PROMPTS.md` — Codex prompt library; read only the current stage.
- `docs/SECURITY.md` — OWASP Top 10:2025 mapping and project security rules.
- `docs/PRIVACY.md` — privacy/zero-knowledge rules.
- `docs/TESTING.md` — test strategy and DoD.
- `docs/FORMATS.md` — input/output format policy.
- `docs/API.md` — API contract direction.
- `docs/DESIGN.md` — intentionally empty until the owner supplies the design.

## Do not blindly overwrite an existing project AGENTS.md

If this repository already has a project-specific `AGENTS.md`, merge the Mathcad overlay rules rather than discarding existing repository rules. Global/user AI Dev Team rules are not copied here.
