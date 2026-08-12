# MathMorph — локальные инструкции

Перед началом работы прочитай `~/codex-workspace/AGENTS.md`. MathMorph является предметным overlay, а не второй универсальной AI-командой.

## Маршрутизация контекста

- Перед изменением проектных агентов, hooks, MCP или поведения контекста прочитай `docs/AI_DEV_TEAM_COMPATIBILITY.md`.
- Работа с parser: `crates/mathcad-parser/AGENTS.md`.
- Математическая семантика: `crates/math-engine/AGENTS.md`.
- Экспорт DOCX/OMML: `crates/exporter-docx/AGENTS.md`.
- API: `services/api/AGENTS.md`.
- Веб-интерфейс: `apps/web/AGENTS.md`.
- Тесты и fixtures: `tests/AGENTS.md`.

## Инварианты проекта

- Сохраняй конвейер: ввод -> parser -> Mathcad AST -> семантика -> Document IR -> exporter.
- Слои parser и math-engine не должны зависеть от Word, HTTP или кода UI.
- Поддерживаемые уравнения остаются редактируемыми структурами; неподдерживаемое содержимое создаёт явную диагностику вместо незаметной потери.
- При работе с загрузкой файлов, parsing, аутентификацией, хранением или криптографией прочитай соответствующий раздел `docs/SECURITY.md`.
- Необязательные hooks и фрагменты MCP остаются отключёнными до явной проверки и включения.

Проверяй изменения контекста командой `python scripts/validate_context_pack.py`. Загружай только относящийся к задаче документ или раздел SPEC; никогда не загружай заранее всю библиотеку prompts, дерево правил или `LEARNING_LOG.md`.
