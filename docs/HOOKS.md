# Optional Codex Hooks

Активная проектная конфигурация находится в `.codex/config.toml`. Сейчас она не регистрирует hooks: глобальная AI Dev Team остаётся источником общих lifecycle-проверок.

Опциональные scripts находятся в `.codex/hooks-optional/`, а пример регистрации — в `.codex/hooks.optional.toml`. Пример не загружается автоматически. Включай только конкретный недостающий hook, вручную перенося проверенную секцию в `.codex/config.toml` после сравнения с global/user/plugin hooks.

Codex обнаруживает project hooks рядом с активным config layer и требует trust-review для unmanaged command hooks. После включения или изменения проверь точное содержимое через `/hooks` перед доверием.

## Доступные опциональные hooks

### SessionStart

`session_start.py`

Цель: вернуть только короткий routing/context reminder и небольшой фрагмент текущего `docs/PROGRESS.md`. Он не читает всю документацию и ограничивает output.

### PreToolUse

`pre_tool_use_policy.py`

Цель: guardrail для очевидно разрушительных shell-команд и прямых попыток прочитать/вставить secrets. Это не полноценная security boundary.

### Stop

`stop_quality_gate.py`

Цель: один раз напомнить завершить quality gate/`docs/PROGRESS.md`, если есть code changes. Hook защищён от бесконечного продолжения через `stop_hook_active`.

## Почему нет тяжёлого PostToolUse

Постоянный lint/test после каждого tool call создаёт лишнюю задержку и дополнительный контекст. Релевантные тесты запускает агент по DoD, а полный release gate выполняет Skill/CI.

## Правила

- Hook output — краткий.
- Hooks не должны читать user documents.
- Hooks не должны отправлять prompts/transcripts во внешнюю аналитику.
- Hooks не должны хранить secrets.
- Любое изменение hook script требует повторного trust-review.

Official reference: https://learn.chatgpt.com/docs/hooks
