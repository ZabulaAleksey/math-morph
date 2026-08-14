# Политика форматов

## Входные форматы

### XMCD
Устаревшее семейство worksheets Mathcad на основе XML. Выполняй безопасный parsing, сохраняй namespaces, метаданные версии, координаты областей и неизвестные узлы как диагностику или неподдерживаемые структуры.

### MCDX
Семейство контейнеров Mathcad Prime. Считай входной архив или контейнер недоверенным. До parsing содержащихся XML и ресурсов применяй защиту от path traversal и ограничения количества записей, распакованного размера, степени сжатия и вложенности.

## Определение

Никогда не доверяй только расширению. Записывай заявленное расширение и определённый формат содержимого. Ошибка `FILE_EXTENSION_MISMATCH` может быть восстанавливаемой, если содержимое уверенно распознано и политика разрешает продолжение.

Текущий detector подтверждает XMCD по root `worksheet` и namespace семейства `http://schemas.mathsoft.com/worksheet<version>`. MCDX подтверждается только корректно проинспектированным ZIP с точной частью `mathcad/worksheet.xml`; generic ZIP и один magic header не достаточны.

Текущий MCDX reader ничего не извлекает на диск. Он применяет лимиты feature-SPEC, отвергает traversal, абсолютные/drive/backslash paths, duplicate/case-conflicting имена, symlinks, encryption и неподдерживаемое сжатие, затем сохраняет worksheet, resource и unknown metadata в детерминированном manifest. CRC32 является только metadata повреждения, не доказательством доверия.

XML metadata reader разрешает только UTF-8, запрещает `DOCTYPE` и возвращает root namespace bindings и `xsi:schemaLocation` как строки без сетевой загрузки.

## Поддерживаемое содержимое legacy XMCD

`WorksheetParser` содержательно читает только явно подтверждённый contract:

- root `{http://schemas.mathsoft.com/worksheet30}worksheet`, `version="3.0.3"`;
- math namespace `http://schemas.mathsoft.com/math30`;
- XML prefixes произвольны, сравниваются expanded QName;
- metadata, recursive `area/region`, обязательный layout, text runs, math, plot/picture references и opaque fallbacks;
- синтаксический Math AST: real/id/arithmetic, definitions/evaluation/functions, unary/grouping/index, matrix/vector, range, calculus и comparisons.

Парсер не является полным runtime XSD validator и не выполняет формулы. `table` сохраняется как opaque `resultFormat` reference, `ml:program` — unsupported math expression, vector определяется как `1×N`/`N×1` matrix. Plot, picture и table binary payload не декодируется.

Другие worksheet/math namespace и версии не маскируются под worksheet30. Prime MCDX content parsing не заявляется: контейнер безопасно инспектируется, но внутренний worksheet ждёт отдельного schema contract.

## Выходные форматы

### DOCX — MVP
Текст в виде абзацев и runs Word, поддерживаемые уравнения как редактируемые Office Math/OMML, изображения и графики с сохранением геометрии, где это возможно.

### В будущем
Markdown, PDF, LaTeX, HTML, JSON, Typst и веб-просмотр через контракты exporters поверх `DocumentIR`.

`DocumentIR` версионируется и сериализуется; schema evolution сохраняет возможность прочитать ранее созданные артефакты либо возвращает явную ошибку несовместимости.

### Диаграммы
Текущий растровый путь должен сосуществовать с `PlotIR` и будущим `ChartIR`, чтобы будущий экспорт Excel мог создавать редактируемые диаграммы.

### Схемы
Текущий растровый путь должен сосуществовать с `DiagramIR`, чтобы будущий экспорт VSDX мог создавать редактируемые shapes, connectors и groups.

## Неподдерживаемые конструкции

Неизвестные или неподдерживаемые узлы никогда не должны исчезать незаметно. Создавай структурированную диагностику и, если это безопасно, частичную конвертацию с явным предупреждением или заполнителем.

Отчёт классифицирует точность каждого результата как `exact`, `approximate`, `unsupported` или `fallback-rendered`.
