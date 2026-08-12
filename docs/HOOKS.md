# Codex Hooks

Актуальная проектная конфигурация находится в `.codex/config.toml`, scripts — `.codex/hooks/`.

Codex поддерживает project hooks рядом с config layer и требует trust-review для немanaged command hooks. После копирования/изменения используйте `/hooks` и проверьте точное содержимое перед доверием.

## Включённые hooks

### SessionStart

`session_start.py`

Цель: вернуть только короткий routing/context reminder и небольшой фрагмент текущего `PROGRESS.md`. Он не читает всю документацию и ограничивает output.

### PreToolUse

`pre_tool_use_policy.py`

Цель: guardrail для очевидно разрушительных shell-команд и прямых попыток прочитать/вставить secrets. Это не полноценная security boundary.

### Stop

`stop_quality_gate.py`

Цель: один раз напомнить завершить quality gate/`PROGRESS.md`, если есть code changes. Hook защищён от бесконечного продолжения через `stop_hook_active`.

## Почему нет тяжёлого PostToolUse

Постоянный lint/test после каждого tool call создаёт лишнюю задержку и дополнительный контекст. Релевантные тесты запускает агент по DoD, а полный release gate выполняет Skill/CI.

## Правила

- Hook output — краткий.
- Hooks не должны читать user documents.
- Hooks не должны отправлять prompts/transcripts во внешнюю аналитику.
- Hooks не должны хранить secrets.
- Любое изменение hook script требует повторного trust-review.

Official reference: https://developers.openai.com/codex/hooks
