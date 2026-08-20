# SPEC: Доказательства совместимости MathType

**Статус:** accepted
**Версия:** 1.0.0
**Дата:** 2026-08-20
**Область:** этап 093

## 1. Цель

Создать проверяемый compatibility contract для Presentation MathML payload этапов 090–092. Контракт обязан отделять локальную детерминированность renderer-а, документированное WIRIS coverage и фактически выполненный import/edit smoke для конкретной поверхности и версии MathType.

Канонический результат этапа — [`docs/MATHTYPE_COMPATIBILITY.md`](../../docs/MATHTYPE_COMPATIBILITY.md). Наличие MathML payload или упоминание элемента в официальной документации не считается доказательством успешного импорта, визуальной эквивалентности или редактируемого round trip.

## 2. Область

- 17 принятых `.mathml` snapshots `exporter-mathml`;
- официальные WIRIS sources с датой доступа и точной областью применимости;
- отдельные статусы для static coverage, live import и edit round trip;
- локальный environment probe и воспроизводимый manual smoke protocol;
- machine-checkable inventory и evidence vocabulary.

## 3. Вне области

- установка или поставка MathType, SDK, license key, WLL/DLL/VBA/OLE artifacts;
- вызов cloud/self-hosted WIRIS services;
- изменение `MathTypeAdapter`, `MathMlRenderer`, `DocxExporter` или Document IR;
- включение `EquationBackend::MathType` или fallback на Word OMML;
- расширение поддерживаемого MathML/AST subset;
- объявление совместимости с поверхностью или версией, для которой smoke не выполнен.

## 4. Требования

### FR-MTCOMP-001 — каноническая матрица

`docs/MATHTYPE_COMPATIBILITY.md` содержит ровно по одной строке для каждого принятого golden snapshot. Имя case является стабильным ключом; новые или удалённые cases требуют согласованного изменения snapshot contract и этой SPEC.

### FR-MTCOMP-002 — независимые уровни evidence

Для каждого case отдельно фиксируются:

- `static coverage`: `DOCUMENTED`, `PARTIAL` или `NOT_DOCUMENTED`;
- `MathType Web live import`: `PASS`, `FAIL` или `NOT_RUN`;
- `MathType 7 desktop SDK import`: `PASS`, `FAIL` или `NOT_RUN`;
- `edit round trip`: `PASS`, `FAIL` или `NOT_RUN`.

`PASS` допустим только с указанием product surface, точной версии, platform, даты, метода импорта и сохранённого результата проверки. `NOT_RUN` обязан содержать причину и означает `UNVERIFIED`, а не предполагаемый успех.

Каждый `PASS` или `FAIL` имеет отдельную machine-checkable evidence record. Record использует surface-specific product/version, platform и allowlisted import/edit method, календарно корректную ISO date и существующий repository-relative evidence artifact с совпадающим SHA-256. Общий статус уникален и равен `UNVERIFIED` при любом `NOT_RUN`, `INCOMPATIBLE` при полностью выполненной матрице с хотя бы одним `FAIL` и `VERIFIED` только при `PASS` во всех live/edit cells.

### FR-MTCOMP-003 — границы официального coverage

Официальная документация WIRIS подтверждает только явно указанную capability. MathType Web coverage нельзя переносить на desktop SDK/Word; заявление SDK о приёме MathML input нельзя переносить на конкретный generated payload или round trip.

### FR-MTCOMP-004 — воспроизводимый smoke protocol

Документ содержит последовательность для повторного прогона всех 17 cases: prerequisites, version capture, import action, визуальная проверка, edit/save/reopen check, запись результата и cleanup. Невыполненная стадия не заменяется более слабым evidence.

### FR-MTCOMP-005 — fail-closed backend boundary

Этап 093 не меняет runtime. `EquationBackend::MathType` остаётся unavailable; отсутствие compatibility evidence не запускает автоматический fallback и не разрешает этап 094.

### NFR-MTCOMP-001 — проверяемость и provenance

Матрица использует стабильный vocabulary, относительные repository paths, UTC-даты формата `YYYY-MM-DD` и прямые ссылки на официальные sources. Автоматическая проверка обнаруживает missing/duplicate cases, неизвестные статусы и ложное общее `PASS` при `NOT_RUN`.

### SEC-MTCOMP-001 — отсутствие новых привилегий

В repository не сохраняются license keys, proprietary binaries, registry dumps, macro templates или пользовательские формулы. Smoke использует только synthetic golden cases. Установка или запуск SDK является отдельным явно разрешённым действием.

## 5. Ошибки и граничные случаи

- Отсутствующий MathType/runtime/license → `NOT_RUN`, итог `UNVERIFIED`.
- Недоступный browser/demo → `NOT_RUN`; static documentation не повышается до live evidence.
- Документированный element при недокументированном attribute/value → `PARTIAL`.
- Успешный import без edit/save/reopen → import может быть `PASS`, но round trip остаётся `NOT_RUN`.
- Поведение MathType Web не является evidence для MathType 7 desktop SDK/Word.
- Любая нормализация payload должна быть отражена как наблюдаемое отличие; byte equality после round trip не предполагается без отдельного доказательства.

## 6. Критерии приёмки

| ID | Критерий |
|---|---|
| AC-093-001 | Канонический compatibility-документ и эта SPEC проиндексированы и проходят проверку Markdown links. |
| AC-093-002 | Матрица содержит exact inventory всех 17 golden cases без duplicates и использует только разрешённые статусы. |
| AC-093-003 | Каждое официальное утверждение имеет WIRIS source, дату доступа и surface scope; static coverage не выдано за import evidence. |
| AC-093-004 | При отсутствии установленного MathType все desktop import/edit результаты остаются `NOT_RUN`, а общий результат — `UNVERIFIED`. |
| AC-093-005 | Добавленная автоматическая проверка отклоняет missing case, duplicate case, неизвестный status, `PASS/FAIL` без полной provenance record и ложный verified summary. |
| AC-093-006 | Runtime-код и backend selection не изменены; `EquationBackend::MathType` продолжает fail closed в существующих regression tests. |

## 7. Связь требований с проверками

| Требования | Проверка |
|---|---|
| FR-MTCOMP-001..003, NFR-MTCOMP-001 | `tests/test_mathtype_compatibility.py` и `python -B scripts/validate_project.py` |
| FR-MTCOMP-004 | ручной review smoke protocol и machine-check required sections |
| FR-MTCOMP-005 | существующий `cargo test -p exporter-docx --locked` |
| SEC-MTCOMP-001 | diff/dependency review; отсутствие новых runtime dependencies и proprietary artifacts |
| AC-093-001..006 | Python unit suite, targeted Rust regressions, workspace tests, Clippy, project validator и independent review |

## 8. Открытые вопросы

- MathType Web live import остаётся `UNVERIFIED`, пока доступный browser runtime не выполнит smoke.
- MathType 7 desktop SDK/Word остаётся `UNVERIFIED`, пока не доступны установленный продукт и соответствующая SDK license.
- Решение о минимальной подтверждённой версии и feature gate относится к этапу 094 после появления versioned `PASS` evidence.

## 9. История изменений

- 1.0.0 (2026-08-20) — первоначальный контракт этапа 093.
