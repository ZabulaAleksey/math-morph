# SPEC: Расширенный OMML и конфигурация backend экспорта

**Статус:** accepted
**Версия:** 1.0.0
**Дата:** 2026-08-14
**Область:** этапы 077–089

## 1. Цель и границы

Расширить редактируемый экспорт формул Word OMML, построенный этапами 055–076, степенями, корнями, индексами, вызовами функций, скобками, матрицами и ограниченным математическим анализом. Одновременно зафиксировать публичный выбор backend для DOCX.

Эта SPEC дополняет, а не заменяет `document-ir-docx-omml.spec.md`. Все инварианты Document IR V1, детерминированного DOCX/OPC subset, отсутствия raw XML в IR, allowlist-валидации, ограничений ресурсов и redacted typed errors из этапов 055–076 обязательны без ослаблений.

Конвейер остаётся неизменным:

```text
Mathcad AST -> MathExpression -> Document IR -> EquationExporter -> DOCX/OMML
```

Ни parser, ни `math-engine` не получают зависимости от Word, OPC, OMML или backend-конфигурации. Поддерживаемые формулы должны оставаться структурно редактируемыми в Word; неизвестная или неоднозначная математика не заменяется похожим текстом либо неявным fallback. На этапах 077–089 `WordEquationExporter` и `DocxExporter` работают all-or-nothing: unsupported node завершает экспорт typed error без частичного DOCX и без fallback.

## 2. Общие правила OMML

### FR-OMML-004 — каноническая структурная сериализация

Каждая новая форма сериализуется только builder/API exporter-а в детерминированный разрешённый OMML subset. Порядок дочерних элементов, обязательные контейнеры и `m:*`-shape фиксированы этой SPEC и проверяются как snapshot/структурными тестами, так и `DocxValidator`.

`m:r`/`m:t` применяются только для безопасных текстовых листьев и математических glyphs, когда это прямо требуется данным контрактом. Raw XML, строковая конкатенация XML и passthrough фрагментов запрещены. Все текстовые данные проходят XML escaping и XML 1.0 character validation до упаковки; недопустимые code point, некорректный UTF-8 и нарушения лимитов возвращают typed redacted error.

### FR-OMML-005 — fail closed

Если форма AST не соответствует точно одному поддержанному правилу, `WordEquationExporter` и `DocxExporter` возвращают явную typed error и не создают частичный DOCX. В частности запрещены: преобразование не-identifier callee в функцию, угадывание размеров матрицы, автоматическое спаривание одинокой группировки, подстановка текстового `^`, `sqrt`, `d/dx` или изображения вместо требуемой OMML-структуры.

Это fail-closed правило относится к 077–089 и не отменяет будущий отдельный путь partial conversion этапов 143–147: если такой путь когда-либо создаёт fallback, он обязан быть явным и сопровождаться conversion warning. Silent loss запрещён всегда; в текущих этапах fallback не создаётся вовсе.

Ошибки не содержат raw formula, XML, архивные данные, пути или секреты. Диагностика может включать стабильный код, безопасный вид узла и счётчики/лимиты.

## 3. Поддерживаемые выражения 077–086

### FR-OMML-006 — степень (077)

`Power { base, exponent }` сериализуется как `m:sSup`: `base` находится в `m:e`, `exponent` — в `m:sup`. Оба потомка экспортируются рекурсивно и не теряют собственную структуру. Не допускается текстовая запись с символом `^` вместо `m:sSup`.

### FR-OMML-007 — квадратный корень (078)

`SquareRoot { radicand }` сериализуется как `m:rad`, содержащий явные `m:radPr/m:degHide` и пустой `m:deg`, а radicand — в `m:e`. `m:degHide` обязан явно обозначать скрытую степень; отсутствие `m:deg`, попытка представить квадратный корень как текст либо неявно интерпретировать другой вид корня недопустимы.

### FR-OMML-008 — литеральный нижний индекс identifier (079)

Идентификатор с литеральным подстрочным индексом сериализуется как `m:sSub`: исходный identifier — в `m:e`, литеральный индекс — в `m:sub`. Подстрочный индекс не должен деградировать до обычного run или быть присоединён к имени строковой конкатенацией.

### FR-OMML-009 — степень над индексированным identifier (080)

Если `Power` имеет в качестве base identifier с литеральным subscript, он сериализуется канонически одним `m:sSubSup`, а не вложенными `m:sSup(m:sSub(...))`. Identifier помещается в `m:e`, literal subscript — в `m:sub`, exponent — в `m:sup`. Другие сочетания скриптов поддерживаются только если для них существует отдельное точное правило; они не нормализуются эвристически в `m:sSubSup`.

### FR-OMML-010 — вызов функции (081)

Поддерживаемый `FunctionCall` обязан иметь callee типа identifier и не менее одного аргумента. Точная последовательность детей `m:func`: опциональный `m:funcPr`, затем `m:fName`, затем обязательный единственный `m:e`. Этот `m:e` содержит ровно одну parenthesized `m:d` для аргументов.

У этой `m:d` сначала находится `m:dPr` с `m:begChr` значением `(`, `m:sepChr` значением `,` и `m:endChr` значением `)`, после чего находятся один или более `m:e` — строго по одному на аргумент. Аргументы сохраняют порядок и рекурсивно экспортируются. Любой иной ребёнок, второй контейнер, неканонический разделитель или пропуск одного из обязательных delimiter properties невалиден.

Нулевое число аргументов, callee не-identifier, неразрешимая/двусмысленная форма вызова и неподдерживаемая разновидность function notation завершаются typed fail-closed error. Exporter не принимает product, indexed expression или grouping за function call по внешнему сходству.

### FR-OMML-011 — парная группировка (082)

Явная парная `Grouping` сериализуется в `m:d` с круглыми скобками и одним `m:e`, содержащим рекурсивно экспортированное выражение. Только логически парная группировка может быть сериализована. Одиночные opening/closing delimiters и прочие явные непарные delimiter формы являются unsupported и должны быть отклонены; exporter не добавляет недостающую скобку.

### FR-OMML-012 — вектор и матрица (083)

`Vector` и `Matrix` экспортируются как `m:m` с упорядоченными `m:mr` (rows) и `m:e` (cells), заключённый в square `m:d`. Вектор является частным случаем прямоугольной матрицы согласно модели и получает тот же редактируемый контейнер: `Row` создаёт одну `m:mr` с N ячейками `m:e` (размерность `1×N`), а `Column` — N `m:mr`, каждая с одной `m:e` (размерность `N×1`). Empty vector отклоняется.

До создания OMML exporter валидирует, что размерности объявлены/выводимы однозначно, количество строк и ячеек соответствует размерностям, каждая строка имеет одинаковое число ячеек и все клетки экспортируемы. Пустые, ragged, переполненные, несогласованные или иным образом невалидные формы отклоняются целиком. Заполнять отсутствующие клетки, обрезать лишние либо печатать матрицу текстом запрещено.

### FR-OMML-013 — интеграл (084)

`Integral` сериализуется через `m:nary` с `m:naryPr/m:chr` для integral glyph. Нижний и верхний пределы присутствуют только если заданы моделью и располагаются в соответствующих `m:sub`/`m:sup`. `m:e` содержит структурную композицию integrand, ordinary `m:r` с glyph `d` и экспортированного bound variable.

Интеграл — семантическая запись, а не presentational shortcut: `bound_variable` обязателен и имеет тип `Identifier` (с literal subscript, если он поддерживается renderer-ом); его значение не выводится из текста integrand. Незаданный/невалидный bound variable, не-identifier bound variable, неподдерживаемый вид differential или предел, который нельзя экспортировать, приводит к typed error; интеграл не превращается в строку с символом `∫`.

### FR-OMML-014 — производная (085)

`Derivative` формируется стандартной композицией `m:f` и существующих `m:sSup`/`m:sSubSup`/run nodes; новый фиктивный элемент `m:derivative` не вводится. Числитель содержит glyph `d` либо partial derivative glyph по стилю производной, при заданной degree — этот glyph в `m:sSup`, затем expression. Знаменатель содержит тот же glyph, затем `bound_variable`; при заданной degree bound variable также рендерится через `m:sSup`. Degree, если задана, является рекурсивно поддерживаемым выражением.

`bound_variable` обязан быть `Identifier` (literal subscript разрешён только через поддерживаемый renderer). Неизвестный стиль производной, невалидная/неэкспортируемая degree, отсутствующая или не-identifier переменная либо комбинация, которой нельзя однозначно сопоставить указанную композицию, отклоняется fail closed. Exporter не заменяет производную текстом вида `d/dx`.

### FR-OMML-015 — агрегат sum/product (086)

`Aggregate` для sum/product экспортируется как `m:nary` с `m:naryPr/m:chr`, равным соответственно sigma или product glyph. `bound_variable` обязан быть `Identifier` (literal subscript разрешён через поддерживаемый renderer). Когда lower отсутствует, `m:sub` содержит только bound variable; когда lower задан, `m:sub` содержит bound variable, ordinary equals run и lower. `m:sup` создаётся только для заданного upper; тело агрегата находится в `m:e`.

Вид агрегата, bound variable, lower/upper и тело валидируются до сериализации. Неизвестный aggregate kind, не-identifier bound variable, некорректный binding, отсутствующее тело или неподдерживаемый предел возвращают typed error. Пределы нельзя подменять текстовыми подписью/надписью или silently discard.

## 4. Ограничения, валидатор и совместимость (087)

### FR-OMML-016 — bounded nested equations

Глубоко вложенные комбинации форм 077–086 должны быть покрыты regression fixture и корректно сериализоваться до достижения разрешённых бюджетов. Единственный источник значений этих бюджетов — `OmmlLimits`, передаваемый в `DocxLimits` через поля equation limits и далее в renderer и `DocxValidator` без повторной интерпретации. Если вызывающий код не задал более строгий лимит, defaults равны: maximum equation depth `256`, maximum equation nodes `100000`, maximum equation output bytes `4 MiB`.

Проверка выполняется в renderer/exporter до выделения неограниченных ресурсов и повторно в `DocxValidator` для сгенерированного пакета с теми же значениями, которые выбрал вызывающий код. Превышение любого бюджета не частично выдаёт DOCX и возвращает redacted typed error. Существующие более строгие лимиты этапов 055–076 сохраняют приоритет.

`DocxValidator` расширяется строгой allowlist без неизвестных elements, attributes или attribute values. Разрешены только следующие ordered shapes (указанный порядок обязателен):

- `m:sSup(m:e, m:sup)`;
- `m:rad(m:radPr(m:degHide[@m:val="1"]), m:deg, m:e)`;
- `m:sSub(m:e, m:sub)`;
- `m:sSubSup(m:e, m:sub, m:sup)`;
- `m:func([m:funcPr], m:fName, m:e)`;
- `m:d(m:dPr(m:begChr[@m:val="("], m:sepChr[@m:val=","], m:endChr[@m:val=")"]), m:e+)`;
- matrix delimiter `m:d(m:dPr(m:begChr[@m:val="["], m:endChr[@m:val="]"]), m:e(m:m(m:mr(m:e+)+)))`;
- `m:nary(m:naryPr(m:chr[@m:val in {"∫", "∑", "∏"}]), [m:sub], [m:sup], m:e)`;
- derivative composition only from the existing allowed `m:f`, `m:sSup`, `m:sSubSup` and run shapes.

Квадратные delimiters допускают только указанные значения `[` и `]`; иной delimiter element/value не принимается. XML, который лишь well-formed, но не соответствует allowlist/порядку/лимитам, не считается валидным.

## 5. Reference artifact и Microsoft Word smoke (088)

### FR-OMML-017 — воспроизводимое внешнее доказательство

Репозиторий содержит воспроизводимую процедуру подготовки reference DOCX, включающую входной fixture/команду генерации, именование артефакта, запуск structural validation и способ сохранить результат smoke-проверки. Процедура не подменяет автоматические structural tests ручной проверкой.

При наличии Microsoft Word на выполняющей проверку машине создаётся evidence: версия Word, дата, идентификатор/reference artifact, успешное открытие, возможность выделить/изменить формулу как editable equation и отсутствие repair prompt. Evidence не включает пользовательские документы, локальные абсолютные пути, содержимое формул из внешних данных или иные чувствительные данные.

Отсутствие Word в CI/среде не меняет статус автоматической структурной валидации и не создаёт ложного заявления об editability. В таком окружении smoke отмечается как unavailable с причиной; его выполнение остаётся ручной воспроизводимой проверкой при доступности Word.

## 6. Публичный backend-контракт (089)

### FR-OMML-018 — `EquationBackend` и `DocxExportConfig`

`exporter-docx` публикует runtime Rust API `EquationBackend { WordOmml, MathType }` и `DocxExportConfig { equation_backend }` как явную конфигурацию выбора equation exporter. Значение по умолчанию — `EquationBackend::WordOmml`; существующий `DocxExporter::new(limits)` сохраняет совместимое default-поведение. Публичный API также содержит `DocxExporter::with_config(limits, config)` и accessor для конфигурации.

`EquationBackend::MathType` является зарезервированным, но недоступным вариантом. При наличии equation его явный выбор возвращает `DocxError::EquationBackendUnavailable` до генерации equation fragment. Он никогда не переключается на `WordOmml` молча и не создаёт текстовый или растровый fallback.

`DocxExportConfig` — только runtime Rust API: он не сериализуется в Document IR и не меняет schema/wire contract. В рамках 089 не реализуются MathType, MathML, OLE, сторонние зависимости, feature flags и адаптеры. Никакая публичная конфигурация не должна создавать скрытую зависимость от Microsoft Word, MathType либо установленного Office.

## 7. Критерии приёмки

| ID | Критерий |
|---|---|
| AC-077 | `Power` имеет точную форму `m:sSup(m:e, m:sup)` и не использует текстовый `^`. |
| AC-078 | `SquareRoot` создаёт `m:rad` с явным `m:degHide`, пустым `m:deg` и radicand в `m:e`. |
| AC-079 | Identifier с литеральным subscript создаёт `m:sSub` с отдельными base и subscript. |
| AC-080 | Степень над subscripted identifier создаёт единый канонический `m:sSubSup`; запрещён вложенный substitute shape. |
| AC-081 | Только identifier callee и >=1 argument дают точный `m:func([funcPr], fName, e(d(dPr beg/sep/end, e+)))`; все отклонённые формы возвращают typed error. |
| AC-082 | Только парная `Grouping` даёт parenthesized `m:d`; одиночный delimiter отклоняется. |
| AC-083 | Vector/Matrix создают square-delimited `m:m` с точными rows/cells; `Row` = `1×N`, `Column` = `N×1`, empty/ragged/invalid dimensions/counts fail closed. |
| AC-084 | Integral создаёт `m:nary` с integral glyph, optional bounds, integrand, ordinary `d` run и identifier bound variable; алгоритм не является текстовой подстановкой. |
| AC-085 | Derivative использует `m:f` и существующие `sSup`/`sSubSup`/run nodes с корректным `d`/partial glyph, recursively supported degree и identifier bound variable, без `m:derivative`. |
| AC-086 | Sum/Product создают `m:nary` с верным glyph, identifier bound либо `bound=lower`, optional upper и body. |
| AC-087 | Defaults depth=256/nodes=100000/output=4 MiB и caller-selected единые limits проверены в exporter и `DocxValidator`; validator reject-ит shape/allowlist нарушения. |
| AC-088 | Reference artifact procedure воспроизводима; при доступности Word сохранено smoke evidence editability, а structural validation остаётся отдельной обязательной проверкой. |
| AC-089 | `EquationBackend { WordOmml, MathType }`, `DocxExportConfig { equation_backend }`, `DocxExporter::new`/`with_config`/accessor имеют `WordOmml` default; `MathType` с equation выдаёт `DocxError::EquationBackendUnavailable` до fragment generation, без fallback и без реализации исключённых технологий. |

## 8. Проверка и отображение тестов

| Acceptance criteria | Обязательное автоматическое доказательство | Дополнительное доказательство |
|---|---|---|
| AC-077..AC-080 | unit/snapshot tests OMML builder-а для power, root, subscript и canonical sub-sup; negative AST cases | DOCX integration: equation XML проходит `DocxValidator` |
| AC-081..AC-083 | unit/snapshot tests function/grouping/matrix/vector, включая `Row`=`1×N` и `Column`=`N×1`; negative tests zero-arg, non-identifier callee, unpaired, empty vector, ragged/dimension mismatch | DOCX integration с несколькими аргументами и nested cells |
| AC-084..AC-086 | unit/snapshot tests integral, derivative и aggregate с пределами/degree/styles; negative semantic cases | DOCX integration с composition существующих fraction/script/run nodes |
| AC-087 | regression fixture глубокой вложенности, boundary tests byte/node/depth и malformed OMML validator tests | property/fuzz-style bounded inputs, если такой harness уже принят в проекте |
| AC-088 | scripted generation reference artifact и structural validation в репозитории/CI | documented Microsoft Word open-and-edit smoke evidence при доступности среды |
| AC-089 | public API/default/backend selection tests; `MathType` typed-error/no-fallback tests | compatibility test существующего default DOCX export path |

Минимальный набор команд проверки после реализации: `cargo test --workspace --locked`, целевые OMML/DOCX tests, `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `python -B scripts/validate_project.py` и `git diff --check`. Word smoke не заменяет ни одну из этих проверок.

## 9. Вне области

Этапы 090+ и любые дополнительные формы формул; полноценный MathType adapter; MathML; OLE; внешние зависимости; feature flags; автоматизация Word/Office; изменения parser/math-engine архитектуры; UI/API/CLI; raw XML import/export; генерация уравнений в растровое изображение как fallback в рамках 077–089. Будущий explicit fallback с conversion warning относится к отдельному контракту partial conversion 143–147.
