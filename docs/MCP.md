# MCP Policy and Configuration

Активная `.codex/config.toml` сейчас не объявляет MCP servers. Выключенный пример находится в `.codex/mcp.optional.toml` и не загружается автоматически; нужную проверенную секцию следует вручную перенести в `.codex/config.toml` только при доказанном проектном пробеле.

## Принцип

MCP подключается не «на всякий случай», а когда внешний источник/инструмент даёт конкретную пользу. Чем меньше tool surface, тем меньше context/tool-choice noise и supply-chain risk.

MCP остаётся adapter к реальному источнику или сервису и не должен становиться вторым владельцем бизнес-логики MathMorph.

После стабилизации Application/Core допустим отдельный проектный MCP adapter с узкими semantic tools: `inspect_document`, `convert_document`, `extract_formulas`, `validate_conversion` и `get_conversion_report`. Он переиспользует те же authorization, scopes, jobs, diagnostics и privacy policy, что REST/CLI, и не получает unrestricted filesystem или shell.

## Настроенные/предусмотренные MCP

### OpenAI Developer Docs

Переиспользуется из глобальной AI Dev Team, если уже доступен. Проектный пример в `.codex/mcp.optional.toml` выключен и нужен только при отсутствии эквивалентного trusted documentation source.

### GitHub official MCP

В pack оставлен выключенным по умолчанию. Включайте для repository/PR/issues задач после OAuth. Предпочитайте узкий read-oriented endpoint/toolset и approval для write tools.

### Context7

Не включён автоматически. Если он нужен, добавляйте только после выбора/pinning проверенной версии локального package или доверенного endpoint. Не использовать `@latest` как долговременную production/dev-infra зависимость.

## Перед добавлением MCP

Зафиксировать:

1. конкретную проблему;
2. owner/source сервера;
3. transport (stdio/HTTP);
4. authentication/secrets;
5. permissions/toolsets;
6. read-only возможность;
7. data sent externally;
8. supply-chain/update policy;
9. approval mode;
10. как отключить сервер без поломки проекта.

После настройки проверь соединение, schemas, read/write границы и отказоустойчивость подходящим MCP inspector/client до использования write tools.

## Data rules

Запрещено передавать через MCP без отдельной необходимости:
- пользовательские Mathcad/DOCX файлы;
- формулы/текст документа;
- API/encryption/recovery secrets;
- plaintext zero-knowledge filenames.

## Context optimization

- Не включать MCP с сотнями нерелевантных tools.
- Использовать узкие toolsets/endpoints.
- Disabled MCP не должен считаться обязательной частью задач.
- Docs research лучше делегировать read-only subagent, чтобы основной thread получил только итог.

Official Codex reference: https://learn.chatgpt.com/docs/extend/mcp
Official GitHub MCP: https://github.com/github/github-mcp-server
