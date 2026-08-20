---
name: mathcad-conversion-regression
description: Запускает целевой регрессионный workflow Mathcad parser, math-engine, DocumentIR и DOCX без дублирования глобальных release или QA-проверок AI Dev Team.
---

Используй этот Skill только для специфичной для Mathcad регрессионной работы. Если глобальная AI Dev Team уже запускает универсальные проверки lint, typecheck, release и безопасности, не повторяй их здесь.

1. Определи изменённые модули Mathcad по git diff.
2. Сначала выбери только затронутые группы fixtures; расширяй охват только при наличии оснований.
3. Запусти тесты модулей и относящиеся к задаче тесты конвертации.
4. Для изменений DOCX проверь структуру пакета, XML, relationships и редактируемых уравнений.
5. Сравни snapshots и эталонные результаты; никогда не принимай различие автоматически.
6. Сообщи ID fixture, ожидаемое и фактическое поведение и вероятный слой.
7. Заверши результатом PASS/FAIL и перечисли только фактически выполненные команды.

Локальный Mathcad regression PASS не заменяет global QA/release gate.

Если обязательный global gate недоступен:

- завершить project-specific regression;
- пометить отсутствующий gate как `QA_REVIEW_UNVERIFIED`;
- не повышать локальный PASS до полного release evidence.
