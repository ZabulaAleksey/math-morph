# Спецификация определения формата и безопасного чтения входа

Статус: реализована и проверена
Версия: 0.1
Этапы дорожной карты: 011–026

## 1. Назначение

Создать минимальную проверяемую границу недоверенного ввода для Mathcad: классифицировать fixtures, определять XMCD/MCDX по содержимому, безопасно инспектировать контейнер MCDX и извлекать только metadata корневого XML. Спецификация уточняет разделы 2, 4 и 18 `system.spec.md`, не заменяя их.

## 2. Область

- taxonomy, JSON manifest и validator репозиторных fixtures;
- стартовые synthetic fixtures повреждённого и опасного XML;
- `InputFormat::{Xmcd, Mcdx, Unknown}` и content-based detector;
- диагностика несовпадения расширения и содержимого;
- ограниченная инспекция ZIP-контейнера MCDX без записи на диск;
- классификация worksheet part, embedded resources и неизвестных частей;
- namespace/schema metadata корневого XML-элемента.

## 3. Вне области

- worksheet metadata, regions, AST и любое содержательное чтение worksheet (этап 027+);
- извлечение контейнера на файловую систему;
- декодирование embedded resources;
- общий `DiagnosticsCollector` и общая severity model (этапы 144–145);
- API, CLI, UI, DOCX и бизнес-логика конвертации;
- реальные пользовательские документы, закрытые образцы и reverse engineering неизвестных версий.

## 4. Граница доверия

Все байты XMCD/MCDX, XML declarations, namespace/schema URI, имена ZIP entries, metadata размеров и compression method считаются недоверенными. Код не выполняет сетевые запросы, не разрешает внешние сущности, не извлекает файлы на диск и не включает содержимое документа в тексты ошибок.

## 5. Fixtures

### FR-FIX-001 — Taxonomy

`tests/fixtures/` содержит группы `xmcd`, `mcdx`, `formulas`, `complex`, `plots`, `diagrams`, `mixed`, `corrupted`, `security`, `compatibility`. Пустые будущие группы могут содержать только `.gitkeep`.

### FR-FIX-002 — Manifest

`tests/fixtures/manifest.json` имеет `schema_version: 1` и массив `fixtures`. Каждая запись содержит уникальные `id` и относительный `path`, `format`, непустую `version`, уникальный массив `features` и `expected_status` (`accepted`, `rejected` или `unsupported`). Неизвестные поля запрещены.

### FR-FIX-003 — Validator

Validator обязан fail-closed проверять JSON schema, taxonomy, уникальность, отсутствие абсолютных/родительских путей, существование файлов и двустороннее соответствие manifest всем видимым fixture-файлам. Dotfiles и сам manifest fixtures не считаются fixture-файлами.

### FR-FIX-004 — Starter fixtures

Репозиторий содержит как минимум один минимальный XMCD, один обрезанный XML и один XML с `DOCTYPE`. Fixtures являются синтетическими, не содержат пользовательских данных и явно описаны в manifest.

## 6. Определение формата

### FR-FMT-001 — Публичный контракт

Rust crate `mathcad-parser` предоставляет `InputFormat`, `FormatDetector`, `FormatDetection` и scoped `FormatDiagnostic`. Detector получает байты и необязательное имя файла и возвращает заявленный расширением формат, обнаруженный по содержимому формат и диагностики.

### FR-FMT-002 — XMCD

XMCD распознаётся только после безопасного чтения XML root: local name равен `worksheet`, а namespace соответствует семейству `http://schemas.mathsoft.com/worksheet<digits>`. Одно расширение `.xmcd` не является доказательством формата.

### FR-FMT-003 — MCDX

MCDX распознаётся только как корректно проинспектированный ограниченный ZIP с канонической частью `mathcad/worksheet.xml`. Одна ZIP signature или расширение `.mcdx` не являются доказательством формата.

### FR-FMT-004 — Несовпадение

Если расширение уверенно заявляет XMCD/MCDX, содержимое уверенно определено как другой поддерживаемый формат, detector возвращает одну диагностику с кодом `FILE_EXTENSION_MISMATCH`. Неизвестное расширение или неизвестное содержимое само по себе эту диагностику не создаёт.

## 7. Контейнер MCDX

### FR-ZIP-001 — Safe inspection

`SafeMcdxReader` принимает байты и `ContainerLimits`, перечисляет entries через индекс, проверяет и полностью дренирует каждую обычную entry через bounded reader, но не пишет данные на диск и не разбирает worksheet XML.

### FR-ZIP-002 — Path policy

Отвергаются NUL, backslash, абсолютные/drive-prefixed пути, пустые внутренние сегменты, `.`/`..`, слишком длинные пути, небезопасный результат `enclosed_name`, точные дубли и ASCII case-insensitive коллизии. Имена никогда не используются как путь назначения.

### FR-ZIP-003 — Limits

Без явной настройки действуют следующие fail-closed лимиты:

| Ограничение | Значение |
|---|---:|
| размер входного архива | 64 MiB |
| entries | 4096 |
| распакованный размер одной entry | 64 MiB |
| суммарный распакованный размер | 256 MiB |
| отношение uncompressed/compressed | 100:1 |
| длина имени entry в UTF-8 bytes | 1024 |

Все суммы используют checked arithmetic. Ненулевая распакованная entry с нулевым compressed size отвергается. Разрешены только `Stored` и `Deflated`; encrypted entries и symlinks отвергаются. Metadata лимиты подтверждаются фактически прочитанными распакованными байтами.

### FR-ZIP-004 — Container manifest

`ContainerManifest` хранит входной размер и упорядоченный список `ContainerPart`: индекс, каноническое имя, флаг directory, compressed/uncompressed size, CRC32 как metadata и `ContainerPartKind`. CRC32 не используется как признак доверия или аутентичности.

### FR-ZIP-005 — Worksheet discovery

Ровно `mathcad/worksheet.xml` классифицируется как `Worksheet`. Manifest предоставляет способ получить эту часть. Несколько worksheet-кандидатов не выбираются эвристически; нестандартные пути остаются неизвестными.

### FR-ZIP-006 — Embedded resources

Без чтения payload известные безопасные расширения изображений/данных классифицируются как `EmbeddedResource` и получают best-effort media type. Это metadata, а не разрешение отображать или исполнять содержимое.

### FR-ZIP-007 — Unknown parts

Все остальные безопасные entries остаются в manifest как `Unknown`; они не исчезают, не вызывают panic и не читаются как worksheet.

## 8. Namespace/schema metadata

### FR-XML-001 — Root metadata

`inspect_xml_metadata` потоково читает только пролог и первый root element и возвращает local name, resolved root namespace, namespace bindings корня, пары `xsi:schemaLocation` и `xsi:noNamespaceSchemaLocation`. URI сохраняются как metadata и никогда не загружаются.

Дополнительные ограничения:

- максимальный XML input — 32 MiB;
- максимум namespace declarations на одном element — 64;
- максимум root attributes — 256;
- разрешена только UTF-8 XML declaration; UTF-16 BOM и иные declarations отвергаются;
- `DOCTYPE`, неизвестный namespace prefix, нечётное число tokens в `schemaLocation`, duplicate/malformed attributes и отсутствие root завершаются контролируемой ошибкой.

Функция не читает worksheet metadata, regions или дочерние элементы после определения root.

## 9. Ошибки и конфиденциальность

Ошибки являются типизированными и различают как минимум нераспознанный/повреждённый XML или ZIP, нарушение пути, duplicate/collision, неподдерживаемое сжатие, encryption/symlink и каждый класс лимитов. `Display` не содержит payload, исходного имени пользовательского файла, полного entry name или внутренних абсолютных путей; допустим безопасный индекс entry.

## 10. Совместимость и зависимости

- Минимальная версия Rust повышается с 1.85 до 1.88, чтобы использовать поддерживаемую ветку `zip` вместо неподдерживаемой ветки 7.x.
- `zip` фиксируется на `=8.6.0` с отключёнными default features и только поддержкой `Stored`/Deflate.
- `quick-xml` фиксируется на `=0.41.0` без optional features; более ранние версии с известными DoS-проблемами не допускаются.
- Изменение не обещает распознавание всех исторических MCDX/XMCD без реальных законно доступных fixtures; неизвестные варианты обрабатываются явно.

## 11. Критерии приёмки

- AC-011-014: taxonomy/manifest/validator и synthetic starter fixtures проходят positive и negative unit tests.
- AC-015: enum/detector существует, пустой и произвольный ввод дают `Unknown` без panic.
- AC-016: минимальный XMCD определяется по root+namespace; одно расширение не достаточно; DTD отклоняется.
- AC-017: MCDX определяется только по ограниченно проверенному ZIP с `mathcad/worksheet.xml`; произвольный ZIP остаётся `Unknown`.
- AC-018: реальный XMCD с `.mcdx` и реальный MCDX с `.xmcd` дают ровно `FILE_EXTENSION_MISMATCH`.
- AC-019-021: безопасный ZIP перечисляется без извлечения; traversal, duplicate/case collision, symlink/encryption/unsupported compression и каждый лимит покрыты тестом либо явно недоступны через test writer и проверены инспекцией API.
- AC-022-025: manifest детерминирован, worksheet/resource/unknown классифицируются без потерь и parsing worksheet.
- AC-026: namespace/schema metadata извлекаются; DTD, encoding, prefix, attribute/namespace limits и malformed schema locations отклоняются.
- AC-SCOPE-001: в diff отсутствуют worksheet metadata/regions/AST/API/UI/exporter функции этапа 027+.

## 12. Связь с тестами

| Требование | Доказательство |
|---|---|
| FR-FIX-001..004 | `tests/test_validate_fixtures.py`, `tests/fixtures/manifest.json` |
| FR-FMT-001..004 | Rust integration tests `format_detection` |
| FR-ZIP-001..007 | Rust integration tests `safe_container` |
| FR-XML-001 | Rust integration tests `xml_metadata` |
| AC-SCOPE-001 | project validator, diff review и traceability review |

## 13. Откат

Изменения не создают данные и внешнее состояние. Откат выполняется отменой stage-коммитов. Если зависимость `zip 8.6.0` или MSRV 1.88 несовместима с целевой средой, реализация блокируется и пересматривается ADR; fallback на неподдерживаемую ветку dependency не выполняется автоматически.

## 14. Открытые вопросы

- Дополнительные исторические namespace и нестандартные MCDX part paths остаются TBD до появления проверенных, законно доступных fixtures.
- Точный allow-list embedded resource paths остаётся TBD; текущий этап классифицирует только по безопасному имени и известному расширению, не доверяя media type.

## 15. История изменений

- 0.1, 2026-08-14 — первоначальный контракт этапов 011–026.
