# MathMorph — локальные инструкции

Перед началом работы прочитай `~/codex-workspace/AGENTS.md`. MathMorph является предметным overlay, а не второй универсальной AI-командой.

## Маршрутизация контекста

- Перед изменением проектных agents, Skills, hooks, MCP или поведения контекста прочитай `docs/CONTEXT_COMPATIBILITY.md`.
- Для существенной реализации выбери SPEC через `specs/README.md`, затем прочитай текущие `docs/AI_PLAN.md` и `docs/AI_STATUS.md`.
- Работа с parser: `crates/mathcad-parser/AGENTS.md`.
- Математическая семантика: `crates/math-engine/AGENTS.md`.
- Экспорт DOCX/OMML: `crates/exporter-docx/AGENTS.md`.
- API: `services/api/AGENTS.md`.
- Веб-интерфейс: `apps/web/AGENTS.md`.
- Тесты и fixtures: `tests/AGENTS.md`.
- Автоматически прогоняй unit, integration и component тесты после изменений; изменения тестовых файлов выполняются только в отдельном согласованном этапе.

## Инварианты проекта

- Сохраняй конвейер: ввод -> parser -> Mathcad AST -> семантика -> Document IR -> exporter.
- Слои parser и math-engine не должны зависеть от Word, HTTP или кода UI.
- Поддерживаемые уравнения остаются редактируемыми структурами; неподдерживаемое содержимое создаёт явную диагностику вместо незаметной потери.
- При работе с загрузкой файлов, parsing, аутентификацией, хранением или криптографией прочитай соответствующий раздел `docs/SECURITY.md`.
- Необязательные hooks и фрагменты MCP остаются отключёнными до явной проверки и включения.

Проверяй структуру и контекст командой `python scripts/validate_project.py`. Загружай только относящийся к задаче документ или раздел SPEC; никогда не загружай заранее всю библиотеку prompts, дерево правил или `LEARNING_LOG.md`.
