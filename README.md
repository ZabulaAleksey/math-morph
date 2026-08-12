# Mathcad Converter & Parser Platform — Codex Overlay

See `README_CONTEXT_PACK.md` for installation/compatibility.

# Mathcad Converter & Parser Platform — AI Dev Team Overlay

This pack is designed for a machine where **AI Dev Team Codex is already installed**.

It is intentionally **not** a second full agent team. It adds only Mathcad-specific rules, specialists and workflows.

## What is active by default

- root + local `AGENTS.md` project rules;
- Mathcad-specific subagents in `.codex/agents/`;
- Mathcad-specific Skills in `.agents/skills/`;
- architecture/security/context documentation.

## What is intentionally inactive by default

- generic QA/security/frontend fallback agents → `.codex/agents-optional/`;
- project hooks → `.codex/hooks-optional/` + `.codex/hooks.optional.toml`;
- project MCP servers → `.codex/mcp.optional.toml`;
- fallback full release Skill → `.agents/skills-optional/`.

The global AI Dev Team should continue to own generic architecture, QA, security, frontend/backend, Git, DevOps, CI and release workflows whenever those capabilities already exist.

## Install

1. Copy the pack into the project repository.
2. Keep your existing global/user Codex configuration untouched.
3. Read `docs/AI_DEV_TEAM_COMPATIBILITY.md` before enabling any optional agent/hook/MCP.
4. Add your canonical `SPECIFICATION.md`, `TECH_STACK.md`, `PROMPTS.md`, `ROADMAP.md` if not already present.
5. Fill `docs/DESIGN.md` separately; it is intentionally empty.
6. Run `python scripts/validate_context_pack.py`.

## Context strategy

The parent thread should receive only project deltas. Avoid reloading the whole global AI Dev Team setup, all agents, all Skills or the whole roadmap. Delegate only when a domain specialist materially improves the task.
