# AI Dev Team Codex Compatibility

## Purpose

This repository is a **project-specific overlay** on top of an already installed AI Dev Team Codex setup. The repository must not create a second generic development team.

## Priority model

1. System/user/global Codex configuration and the installed AI Dev Team remain the owner of generic engineering workflow.
2. Root `AGENTS.md` adds only Mathcad-platform invariants and routing.
3. Local `AGENTS.md` files add module-specific rules.
4. `.codex/agents/*.toml` contains only Mathcad-specific specialists.
5. The current task prompt narrows scope further.

When rules overlap, follow the higher-priority/effective Codex instruction and treat this repository as a domain overlay rather than a replacement.

## Do not duplicate global roles

Before delegating a generic task, inspect the agents/capabilities already available in the installed AI Dev Team.

Prefer existing global roles for capabilities such as:

- architect / architecture review;
- generic backend/frontend implementation;
- generic QA/test review;
- generic security review;
- code review;
- DevOps/CI/release;
- Git/GitHub workflow;
- documentation research.

Project agents should be used only when Mathcad-specific domain knowledge materially helps.

Active project-specific roles:

- `mathcad_format_forensics`
- `mathcad_parser_engineer`
- `mathcad_math_semantics`
- `mathcad_word_openxml`

Fallback generic project agents live in `.codex/agents-optional/` and are **not active**. Copy one into `.codex/agents/` only if you have confirmed that the AI Dev Team has no equivalent capability.

## Total subagent budget

The budget is shared across **global + project** agents; it is not additive.

- SIMPLE: 0 subagents normally.
- STANDARD: 1–2 total subagents.
- COMPLEX: up to 3–4 total subagents only for independent workstreams.

Never launch both a global reviewer and a project fallback reviewer for the same question.

Never let multiple agents edit the same files concurrently.

## Hooks

The installed AI Dev Team owns generic session-start, safety, formatting, quality-gate and release hooks unless a concrete gap is proven.

Therefore this pack does **not** register project hooks by default.

- Optional scripts: `.codex/hooks-optional/`
- Optional registration snippet: `.codex/hooks.optional.toml`

Before activating a project hook:

1. inspect existing global/user/project hook behavior;
2. identify the exact missing capability;
3. activate only the smallest missing hook;
4. avoid duplicate SessionStart/Stop/PostToolUse checks;
5. keep hook output tiny.

## MCP

The installed AI Dev Team is the source of truth for shared MCP servers.

This repository does not activate duplicate MCP servers by default.

- Optional MCP template: `.codex/mcp.optional.toml`
- Policy: `docs/MCP.md`

If GitHub, docs, browser/devtools, Context7 or another equivalent server already exists globally, reuse it instead of declaring another project server.

## Skills

Keep only domain-specific skills active:

- `mathcad-format-forensics`
- `mathcad-conversion-regression`
- `mathcad-security-overlay`

Generic release/quality workflows should come from the existing AI Dev Team. A fallback template exists in `.agents/skills-optional/` only for installations without such a workflow.

## Context minimization

Do not reload the entire AI Dev Team documentation from this repository.

The effective global rules are assumed to already be active. Project instructions should only add Mathcad-specific deltas.

Avoid:

- restating generic Git workflow;
- restating generic coding standards;
- restating generic QA/security roles;
- duplicating MCP descriptions;
- loading all Skills;
- loading all agent TOMLs;
- reading all roadmap/prompts for one local change.

Prefer capability lookup + the smallest relevant local document.
