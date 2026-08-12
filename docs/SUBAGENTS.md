# Subagents

Project agents: `.codex/agents/*.toml`.

## Набор

- `format_forensics` — read-only исследование XMCD/MCDX structures.
- `parser_engineer` — parser/AST/compatibility implementation specialist.
- `math_semantics` — dependency graph, substitution, complex math, transformation correctness.
- `word_openxml` — DOCX/OOXML/OMML specialist.
- `qa_adversary` — read-only edge cases/regression gaps.
- `security_reviewer` — read-only OWASP/threat-boundary review.
- `frontend_design_reviewer` — read-only UI compliance/accessibility review; DESIGN.md может быть пуст до заполнения.

## Использование без перегруза

- SIMPLE: не делегировать.
- STANDARD: 1–2 agents только для независимых вопросов.
- DEEP: максимум 3–4 одновременно.
- Не запускать несколько reviewers с одинаковой задачей.
- Сначала explorer/reviewer → затем один owner меняет код.
- Отчёт agent: findings/evidence/actions; без пересказа всех прочитанных файлов.
- Read-only agent не должен редактировать код.

Official reference: https://developers.openai.com/codex/agent-configuration/subagents
