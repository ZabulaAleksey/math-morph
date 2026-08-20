# Правила DOCX / Word

- Перед изменением контрактов exporter прочитай архитектуру.
- Поддерживаемые уравнения должны быть редактируемыми структурами Word/OMML, а не снимками экрана.
- Специфичный для Word XML относится к слою exporter, но никогда не к Mathcad AST.
- Структура пакета DOCX, relationships и XML должна проверяться тестами.
- Внешние relationships и встроенное активное содержимое требуют явного review безопасности.
- Для неподдерживаемого уравнения используй явный fallback и предупреждение о конвертации; не допускай незаметной потери.
- MathType остаётся отдельным backend или адаптером.
- Ordered fallback для equation export определяется `docs/FALLBACKS.md`.
- Requested backend нельзя тихо заменять другим.
- Для неподдерживаемой формулы допускается только заранее разрешённый
  semantically-equivalent или явно degraded representation.
- Screenshot, plain text, OLE, MathML или внешний backend не являются
  неявным fallback для редактируемого OMML.
- Если разрешённого fallback нет — typed error / fail closed.