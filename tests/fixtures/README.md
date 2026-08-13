# Fixtures MathMorph

## Taxonomy

- `xmcd` — минимальные и совместимые XML worksheets;
- `mcdx` — проверенные контейнеры Mathcad Prime;
- `formulas`, `complex`, `plots`, `diagrams`, `mixed` — будущие предметные corpus-группы;
- `corrupted` — синтаксически или структурно повреждённый ввод;
- `security` — синтетические abuse cases без пользовательских данных;
- `compatibility` — законно доступные межверсионные regression cases.

Каждый видимый fixture-файл перечисляется в `manifest.json`. Dotfiles сохраняют пустые будущие категории и fixtures не считаются. Реальные пользовательские документы и данные с неясными правами в репозиторий не добавляются.
