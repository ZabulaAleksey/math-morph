# MCP Policy and Configuration

Конфигурация: `.codex/config.toml`.

## Принцип

MCP подключается не «на всякий случай», а когда внешний источник/инструмент даёт конкретную пользу. Чем меньше tool surface, тем меньше context/tool-choice noise и supply-chain risk.

## Настроенные/предусмотренные MCP

### OpenAI Developer Docs

Используется для актуальной документации Codex/OpenAI. Может быть включён по умолчанию, поскольку это узкий documentation source.

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

Official Codex reference: https://developers.openai.com/codex/mcp
Official GitHub MCP: https://github.com/github/github-mcp-server
