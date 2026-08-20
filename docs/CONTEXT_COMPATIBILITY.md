# Совместимость проектного контекста

## Назначение

MathMorph является предметным overlay над `~/codex-workspace`. Глобальные правила, Git workflow, универсальные agents, hooks, MCP и release-процессы здесь не копируются. Проект хранит только требования продукта, архитектурные границы и Mathcad-специфичные расширения.

Статусы соответствуют `~/codex-workspace/docs/CONTEXT_COMPATIBILITY.md`: `INHERITED`, `EXTEND`, `PROJECT_ONLY`, `CONFLICT`, `OBSOLETE`.

## Аудит 2026-08-14

| Возможность | Глобальный источник | Потребность MathMorph | Статус | Каноническое решение |
|---|---|---|---|---|
| Git workflow и commits | `~/codex-workspace/AGENTS.md` | локальных отличий нет | `INHERITED` | проект не хранит второй Git workflow |
| Универсальные implementation/review/QA/security/release роли | активная AI Dev Team | предметных отличий нет | `INHERITED` | использовать глобальные роли |
| Корневые и модульные инструкции | каскад `AGENTS.md` workspace | инварианты parser, math-engine, exporter, API, web и tests | `EXTEND` | корневой и ближайшие модульные `AGENTS.md` |
| Требования | SDD и шаблоны workspace | требования продукта MathMorph | `PROJECT_ONLY` | `specs/system.spec.md`; индекс — `specs/README.md` |
| План и состояние | project framework workspace | один текущий срез и один снимок | `PROJECT_ONLY` | `docs/AI_PLAN.md` и `docs/AI_STATUS.md` |
| Архитектура, дизайн и решения | project framework workspace | границы Mathcad-конвейера и утверждённый MathMorph Calm Blue UI contract | `PROJECT_ONLY` | `docs/ARCHITECTURE.md`, `docs/DESIGN.md`, `docs/DECISIONS.md` |
| Mathcad agents | универсальные agents не содержат экспертизу форматов Mathcad | forensics, parser, math semantics, OpenXML | `PROJECT_ONLY` | `.codex/agents/*.toml`; запускать только при предметной пользе |
| Mathcad Skills | общие workflows не содержат предметных регрессий Mathcad | forensics, conversion regression, security overlay | `PROJECT_ONLY` | `.agents/skills/*/SKILL.md` |
| Универсальные fallback agents/Skill | уже доступны глобально | локальной потребности нет | `OBSOLETE` | удалены из optional-каталогов проекта |
| Hooks | глобальные lifecycle hooks | подтверждённого пробела нет | `INHERITED` | проектные hook-шаблоны удалены |
| MCP | глобальная конфигурация и plugins | подтверждённого проектного сервера нет | `INHERITED` | проектный MCP-шаблон удалён |
| Codex config | глобальная конфигурация | ограничить размер project docs | `EXTEND` | `.codex/config.toml` содержит только `project_doc_max_bytes` |
| Документация context pack | project framework workspace | после создания репозитория не нужна | `OBSOLETE` | distribution README, manifest и дублирующие policy/inventory docs удалены |
| Глобальная Fallback Policy | `~/codex-workspace/rules/fallback-policy.md` | общий retry/fallback/degraded/fail-closed contract | `INHERITED` | не копировать глобальную policy |
| MathMorph fallback catalog | глобальная policy не содержит семантики Mathcad | parser/export/backend/privacy/recovery delta | `EXTEND` | `docs/FALLBACKS.md` |

## Правила расширения

- Сначала проверить глобально доступную возможность.
- Локальное расширение добавлять только для подтверждённого предметного пробела.
- Для нового agent, Skill, hook, MCP или config фиксировать владельца, узкую область, способ проверки и безопасный fallback в этой таблице.
- Не считать optional или неактивный файл установленной возможностью.
