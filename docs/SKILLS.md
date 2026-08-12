# Skills

Repo skills находятся в `.agents/skills/` и используют progressive disclosure: полный `SKILL.md` загружается только когда workflow выбран.

## Активные Skills

- `mathcad-format-forensics` — анализ неизвестного fixture/формата без изменения production parser.
- `mathcad-conversion-regression` — targeted regression после parser/exporter changes.
- `mathcad-security-overlay` — Mathcad-specific дополнение к глобальному security review.

`mathcad-release-quality-gate-fallback` находится в `.agents/skills-optional/` и не является активным. Используй его только если в установленной AI Dev Team нет эквивалентного release/quality workflow.

Не переносить body Skills в root AGENTS.md — это намеренно разделено для экономии контекста.

Official reference: https://learn.chatgpt.com/docs/build-skills
