# Subagents

Project agents: `.codex/agents/*.toml`.

## Активные Mathcad-специалисты

- `mathcad_format_forensics` — read-only исследование XMCD/MCDX structures.
- `mathcad_parser_engineer` — parser/AST/compatibility implementation specialist.
- `mathcad_math_semantics` — dependency graph, substitution, complex math, transformation correctness.
- `mathcad_word_openxml` — DOCX/OOXML/OMML specialist.

Generic QA, security, frontend, architecture and review roles предоставляет глобальная AI Dev Team. Локальные fallback-роли `mathcad_qa_fallback`, `mathcad_security_fallback` и `mathcad_frontend_review_fallback` лежат в `.codex/agents-optional/` и не активны. Переносить их в `.codex/agents/` можно только после подтверждения реального пробела в глобальных возможностях.

## Использование без перегруза

- SIMPLE: не делегировать.
- STANDARD: 1–2 agents только для независимых вопросов.
- COMPLEX: максимум 3–4 одновременно для независимых направлений.
- Не запускать несколько reviewers с одинаковой задачей.
- Сначала explorer/reviewer → затем один owner меняет код.
- Отчёт agent: findings/evidence/actions; без пересказа всех прочитанных файлов.
- Read-only agent не должен редактировать код.

## Handoff contract

Для передачи работы укажи только:

- цель и относящиеся требования/этапы;
- принадлежащие агенту файлы или read-only scope;
- принятые допущения и решения;
- уже выполненные проверки;
- конкретные незакрытые вопросы или блокеры.

Два write-capable агента не должны владеть одним manifest или общим документом одновременно.

Official reference: https://learn.chatgpt.com/docs/agent-configuration/subagents
