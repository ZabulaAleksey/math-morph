# Skills

Repo skills находятся в `.agents/skills/` и используют progressive disclosure: полный `SKILL.md` загружается только когда workflow выбран.

## Skills в pack

- `mathcad-format-forensics` — анализ неизвестного fixture/формата без изменения production parser.
- `conversion-regression` — targeted regression после parser/exporter changes.
- `owasp-security-review` — security review по OWASP Top 10:2025 и проектным trust boundaries.
- `release-quality-gate` — полный pre-release quality check.

Не переносить body Skills в root AGENTS.md — это намеренно разделено для экономии контекста.

Official reference: https://developers.openai.com/codex/build-skills
