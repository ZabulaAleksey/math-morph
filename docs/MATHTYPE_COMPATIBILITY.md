# MathType — матрица совместимости Presentation MathML

**Этап:** 093
**Дата evidence snapshot:** 2026-08-20
**Общий статус:** `UNVERIFIED`

## 1. Область и смысл статуса

Этот документ сопоставляет 17 принятых golden payloads из [`exporter-mathml`](../crates/exporter-mathml/tests/golden/) с официально документированным WIRIS coverage и зафиксированными статусами smoke-проверок.

Общий статус `UNVERIFIED` означает, что MathMorph детерминированно создаёт проверенный Presentation MathML subset, но в текущей среде не выполнен ни один live import/edit round trip в MathType Web или MathType 7 desktop SDK. Документированная поддержка элемента не является доказательством успешного импорта конкретного payload.

Общий envelope всех cases:

```xml
<math xmlns="http://www.w3.org/1998/Math/MathML" display="block">…</math>
```

WIRIS MathType Web coverage перечисляет `math`, `mrow`, `mn`, `mi`, `mo`, `mfrac`, `msub`, `msup` и `msqrt`, а также `math@xmlns` и `math@display`. Это только static surface evidence. В публичной таблице не найден `mo@fence`, поэтому grouping case имеет `PARTIAL`.

## 2. Evidence snapshot среды

| Surface | Version / environment | Evidence | Status |
|---|---|---|---|
| MathMorph renderer | repository stage 091 corpus, 17 exact snapshots | byte-for-byte regression через production `MathMlRenderer` | `PASS` |
| MathType Web static coverage | публичная WIRIS coverage reference, доступ 2026-08-20 | элементы/атрибуты из официальной таблицы; не live import | `DOCUMENTED` / `PARTIAL` |
| MathType Web live editor | версия не определена | интерактивный browser runner недоступен в текущей среде | `NOT_RUN` |
| MathType 7 desktop SDK | Windows development host; MathType command, типовые install paths и uninstall registry entries отсутствуют | продукт/SDK не установлен, SDK license не проверялась | `NOT_RUN` |
| Microsoft Word + MathType | Word отдельно доступен по evidence этапа 088; MathType add-in/SDK отсутствует | MathType import/edit path отсутствует | `NOT_RUN` |

Локальный probe был только read-only. MathType, SDK, trial, add-in, DLL/WLL и license автоматически не устанавливались и не активировались.

## 3. Источники и область утверждений

- [WIRIS MathML coverage by MathType Web](https://www.wiris.net/demo/editor/docs/mathml-coverage/) — перечисляет поддерживаемые MathType Web Presentation MathML elements/attributes. Не доказывает desktop SDK/Word import и не является результатом выполнения наших payloads. Доступ: 2026-08-20.
- [WIRIS: Converting equations](https://docs.wiris.com/mathtype-sdk-documentation/converting-equations) — документирует MathML text file как допустимый input MathType SDK conversion. Не утверждает поддержку каждого MathMorph case. Доступ: 2026-08-20.
- [WIRIS: Getting started to MathType's API](https://docs.wiris.com/en_US/mathtype-sdk-technical-documentation/mathtype-api-documentation) — указывает, что SDK требует активированный MathType 7 с SDK license, и описывает Windows DLL/WLL/Word boundary. Доступ: 2026-08-20.
- [WIRIS: programmatically inserting a MathML equation](https://docs.wiris.com/en_US/mathtype-sdk-documentation/using-mfc-to-access-mathtypes-ole-subsystem) — описывает отдельный OLE import path через clipboard format `MathML`. MathMorph его не реализует. Доступ: 2026-08-20.
- [WIRIS: MathML in files and clipboard](https://docs.wiris.com/mathtype-sdk-documentation/how-mathml-is-stored-in-files-and-the-clipboard) — описывает Presentation MathML/UTF-8 и proprietary MTEF/translator context. Наличие нашего standalone MathML не означает наличие re-editable MTEF. Доступ: 2026-08-20.

## 4. Vocabulary evidence

- `DOCUMENTED` — все элементы и атрибуты case перечислены в выбранной официальной static coverage reference; token semantics и live import этим не проверены.
- `PARTIAL` — хотя бы один элемент или атрибут case не найден в выбранной static coverage reference.
- `NOT_DOCUMENTED` — static coverage для case не найдено.
- `PASS` — case фактически импортирован/отредактирован на записанной поверхности и версии.
- `FAIL` — фактический smoke выполнен и обнаружил несовместимость.
- `NOT_RUN` — фактический smoke не выполнен; результат остаётся `UNVERIFIED`.

Общий статус равен `UNVERIFIED`, пока присутствует хотя бы один `NOT_RUN`; `INCOMPATIBLE`, если все стадии выполнены, но присутствует `FAIL`; и `VERIFIED` только когда все live/edit cells имеют `PASS` с полными evidence records.

## 5. Матрица

| Case | Generated shape | Static coverage | MathType Web live import | MathType 7 desktop SDK import | Edit round trip |
|---|---|---|---|---|---|
| `add.mathml` | `mrow(mi, mo(+), mn)` | `DOCUMENTED` | `NOT_RUN` | `NOT_RUN` | `NOT_RUN` |
| `divide.mathml` | `mfrac(mi, mn)` | `DOCUMENTED` | `NOT_RUN` | `NOT_RUN` | `NOT_RUN` |
| `grouping.mathml` | `mrow(mo[fence=true], mi, mo[fence=true])` | `PARTIAL` | `NOT_RUN` | `NOT_RUN` | `NOT_RUN` |
| `identifier.mathml` | `mi` | `DOCUMENTED` | `NOT_RUN` | `NOT_RUN` | `NOT_RUN` |
| `identifier-escaped.mathml` | `mi` with escaped XML text | `DOCUMENTED` | `NOT_RUN` | `NOT_RUN` | `NOT_RUN` |
| `identifier-subscript.mathml` | `msub(mi, mi)` | `DOCUMENTED` | `NOT_RUN` | `NOT_RUN` | `NOT_RUN` |
| `multiply-dot.mathml` | `mrow(mi, mo(U+00B7), mi)` | `DOCUMENTED` | `NOT_RUN` | `NOT_RUN` | `NOT_RUN` |
| `multiply-no-space.mathml` | `mrow(mi, mi)` | `DOCUMENTED` | `NOT_RUN` | `NOT_RUN` | `NOT_RUN` |
| `multiply-thin-space.mathml` | `mrow(mi, mo(U+2009), mi)` | `DOCUMENTED` | `NOT_RUN` | `NOT_RUN` | `NOT_RUN` |
| `multiply-x.mathml` | `mrow(mi, mo(U+00D7), mi)` | `DOCUMENTED` | `NOT_RUN` | `NOT_RUN` | `NOT_RUN` |
| `numeric-binary.mathml` | `mn(1010)` | `DOCUMENTED` | `NOT_RUN` | `NOT_RUN` | `NOT_RUN` |
| `numeric-decimal.mathml` | `mn(-12.5e+2)` | `DOCUMENTED` | `NOT_RUN` | `NOT_RUN` | `NOT_RUN` |
| `numeric-hexadecimal.mathml` | `mn(FF)` | `DOCUMENTED` | `NOT_RUN` | `NOT_RUN` | `NOT_RUN` |
| `numeric-octal.mathml` | `mn(755)` | `DOCUMENTED` | `NOT_RUN` | `NOT_RUN` | `NOT_RUN` |
| `power.mathml` | `msup(mi, mn)` | `DOCUMENTED` | `NOT_RUN` | `NOT_RUN` | `NOT_RUN` |
| `square-root.mathml` | `msqrt(mi)` | `DOCUMENTED` | `NOT_RUN` | `NOT_RUN` | `NOT_RUN` |
| `subtract.mathml` | `mrow(mi, mo(U+2212), mn)` | `DOCUMENTED` | `NOT_RUN` | `NOT_RUN` | `NOT_RUN` |

Особые случаи:

- `grouping.mathml`: `mo` документирован, но `fence="true"` отсутствует в выбранной public coverage table; до live smoke это `PARTIAL`.
- `identifier-escaped.mathml`: XML escaping проверено локально, но static element coverage не доказывает, что importer сохранит decoded text `a<&>` без нормализации.
- numeric base cases используют визуальные lexemes без отдельной MathML annotation о системе счисления; live smoke должен проверять именно отображение и редактирование lexeme, а не предполагать semantic base recovery.

## 6. Воспроизводимый live smoke

### Preconditions

1. Использовать только synthetic payloads из `crates/exporter-mathml/tests/golden/` на зафиксированном Git commit.
2. Для MathType Web записать видимую product/build version или release identifier и browser/OS.
3. Для desktop записать MathType `Help → About`, OS, Word version, способ активации SDK license и import route. License key в отчёт не копировать.
4. Не включать macro security bypass, неподписанные templates или proprietary binaries в repository.

### Case protocol

Для каждого из 17 cases:

1. Прочитать exact UTF-8/LF payload без ручной нормализации.
2. Импортировать через документированный surface-specific MathML route.
3. Проверить отсутствие error, видимую структуру, operator glyphs, grouping, subscript/root/power и literal text.
4. Изменить один leaf token, сохранить, закрыть и повторно открыть equation.
5. Экспортировать MathML, если surface это поддерживает; записать структурные изменения отдельно от визуального результата.
6. Записать `PASS`, `FAIL` или `NOT_RUN`, точную версию, дату, route, краткое redacted наблюдение и путь/хэш разрешённого evidence artifact.

### Stop conditions

- Отсутствие продукта, license, documented import route или version capture → `NOT_RUN`.
- Crash, importer error, потеря структуры или невозможность edit/save/reopen → соответствующая стадия `FAIL`; другой backend автоматически не выбирается.
- Успешный render без edit/save/reopen не повышает `Edit round trip` до `PASS`.

## 7. Versioned live evidence records

Каждый будущий `PASS` или `FAIL` в матрице обязан иметь ровно одну строку ниже. Разрешённые surface keys: `WEB_IMPORT`, `DESKTOP_IMPORT`, `EDIT_ROUND_TRIP`. Поля проверяются fail closed:

- product/version: `MathType Web <semver>` либо `MathType 7 <version>` согласно surface;
- platform: `<OS name/version> / <browser name/version>` для Web либо `Windows <version> / Word <version>` для desktop;
- method: `WEB_SET_MATHML`, `SDK_TEXT_FILE`, `OLE_MATHML_CLIPBOARD`, `WEB_EDIT_SAVE_REOPEN_EXPORT` или `DESKTOP_EDIT_SAVE_REOPEN_EXPORT` согласно surface;
- date: календарно корректная ISO date `YYYY-MM-DD`;
- evidence: существующий repository-relative файл `tests/evidence/mathtype/<artifact>#sha256=<64 lowercase hex>` с совпадающим SHA-256.

License keys, пользовательские формулы и proprietary binaries запрещены.

| Case | Surface | Status | Product version | Platform | Date | Import method | Evidence |
|---|---|---|---|---|---|---|---|

На дату snapshot records отсутствуют, потому что все live/edit cells имеют `NOT_RUN`.

## 8. Runtime boundary и handoff

`EquationBackend::MathType` остаётся typed unavailable в `exporter-docx`. Этот документ не активирует OLE/COM, SDK, cloud services или automatic fallback. Этап 094 может рассматривать feature gate только после появления versioned `PASS` evidence для явно выбранной поверхности; неподтверждённые поверхности остаются fail closed.

Критерии и machine-check contract определены в [`mathtype-compatibility-evidence.spec.md`](../specs/features/mathtype-compatibility-evidence.spec.md).
