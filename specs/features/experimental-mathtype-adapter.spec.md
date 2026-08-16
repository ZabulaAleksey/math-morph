# SPEC: Экспериментальный MathType adapter через Presentation MathML

**Статус:** accepted
**Версия:** 1.0.0
**Дата:** 2026-08-17
**Область:** этап 092

## 1. Цель

Добавить отдельную экспериментальную границу между backend-neutral математическим AST MathMorph и будущей интеграцией MathType. Этап 092 не запускает MathType и не встраивает уравнение в DOCX. Он только преобразует поддерживаемое `math-model::MathExpression` в opaque bounded Presentation MathML payload, созданный production renderer-ом этапов 090–091.

Канонический поток этапа:

```text
MathExpression
  -> exporter-mathml::MathMlRenderer
  -> exporter-mathtype::MathTypeAdapter
  -> opaque MathTypePayload (application/mathml+xml)
```

`MathTypePayload` является подготовленным входом для будущего отдельно проверяемого bridge. Наличие payload не является заявлением, что конкретная версия MathType, Word, SDK или web integration импортирует каждую поддержанную MathMorph-форму без изменения.

## 2. Архитектурные границы

### FR-MATHTYPE-001 — отдельный adapter crate

Добавляется crate `exporter-mathtype`. Он зависит только от внутренних crates `math-model`, `document-ir` и `exporter-mathml`. Parser, `math-engine`, Document IR schema и `exporter-docx` не получают обратной зависимости от MathType.

### FR-MATHTYPE-002 — только сгенерированный Presentation MathML

`MathTypeAdapter` принимает borrowed `MathExpression` и делегирует генерацию `MathMlRenderer`. Он не принимает `String`, raw XML, произвольный namespace, DTD, entity declaration, URL, файл или сетевой ответ.

Успешный результат — opaque `MathTypePayload`:

- format: `MathTypePayloadFormat::PresentationMathMl`;
- media type: `application/mathml+xml`;
- read-only access: `as_mathml()`, `as_bytes()`, `byte_len()`;
- публичного конструктора из произвольного XML нет.

### FR-MATHTYPE-003 — общий exporter port

`MathTypeAdapter` реализует `document_ir::ports::EquationExporter<Output = MathTypePayload, Error = MathTypeError>`. Это не подключает adapter к `DocxExporter` и не меняет `EquationBackend::MathType`.

### FR-MATHTYPE-004 — fail closed и redaction

Любая ошибка `MathMlRenderer` преобразуется в typed `MathTypeError` без частичного payload и fallback. `Display`, `Debug` и error source не содержат identifier, literal, формулу, XML payload, путь или данные документа. `Debug` успешного payload показывает только format и byte length.

### FR-MATHTYPE-005 — отсутствие runtime-интеграции

На этапе 092 запрещены:

- MathType SDK, WIRIS cloud/self-hosted services и license keys;
- COM, OLE, VBA, Word automation и platform-specific DLL/WLL;
- MTEF generation или reverse engineering;
- HTTP, filesystem, registry и clipboard access;
- изменение DOCX package или включение `EquationBackend::MathType`;
- compatibility claims, version matrix или автоматический fallback.

## 3. Ограничения ресурсов и безопасность

### NFR-MATHTYPE-001 — единые MathML budgets

Adapter не создаёт второй набор лимитов. `MathTypeAdapter::new(MathMlLimits)` использует те же maximum depth, node count, cumulative input/output byte accounting и iterative traversal, что `MathMlRenderer`. Defaults остаются depth `256`, nodes `100000`, output `4 MiB`.

### SEC-MATHTYPE-001 — отсутствие raw injection boundary

Payload может появиться только после allowlist renderer-а, XML 1.0 validation, escaping и budget checks. Adapter не выполняет повторный parsing и не предоставляет API для подмешивания raw markup.

### SEC-MATHTYPE-002 — отсутствие новых привилегий

Crate не получает network, filesystem, registry, process-launch или Office permissions. Добавление такой возможности требует отдельной SPEC, dependency/license review, threat-model update и явного решения владельца.

## 4. Публичный Rust API

Crate публикует:

- `MathTypeAdapter::new(MathMlLimits)` и `Default`;
- `MathTypeAdapter::limits()` и `adapt_expression()`;
- `MathTypePayload`, `MathTypePayloadFormat`;
- `MATHTYPE_MATHML_MEDIA_TYPE`;
- `MathTypeError::mathml_error()`;
- реализацию backend-neutral `EquationExporter`.

API является экспериментальным внутренним workspace-контрактом. Он не сериализуется в Document IR и не является wire/API/CLI contract.

## 5. Критерии приёмки

| ID | Критерий |
|---|---|
| AC-092-001 | Поддерживаемое scalar expression создаёт deterministic opaque Presentation MathML payload byte-for-byte равный production `MathMlRenderer`. |
| AC-092-002 | Payload сообщает format `PresentationMathMl`, media type `application/mathml+xml`, byte length и read-only bytes/string без публичного raw constructor. |
| AC-092-003 | Adapter реализует общий `EquationExporter`; `exporter-docx` и `EquationBackend::MathType` не изменяются. |
| AC-092-004 | Unsupported/invalid AST и depth/node/output limits возвращают typed `MathTypeError` без частичного payload и fallback. |
| AC-092-005 | `Debug` payload/error не раскрывает identifier или formula text. |
| AC-092-006 | Workspace/lockfile/project validator фиксируют новый crate и только три разрешённые внутренние зависимости. |

## 6. Тесты и проверка

Обязательные доказательства:

- positive exact-output test;
- generic `EquationExporter` port test;
- unsupported-expression negative test;
- output-limit boundary test;
- payload/error redaction test;
- project-validator dependency-scope regression;
- существующий DOCX regression продолжает доказывать, что `EquationBackend::MathType` unavailable.

Команды:

```powershell
cargo test -p exporter-mathtype --locked
cargo test -p exporter-docx --locked
cargo test --workspace --locked
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
python -B scripts/validate_project.py
python -B -m unittest discover -s tests -p "test_*.py" -v
git diff --check
```

## 7. Вне области

- этап 093: документированная compatibility matrix и реальные импорт/smoke evidence;
- этап 094: feature-gated DOCX backend selection;
- расширение MathML AST coverage;
- MTEF/OLE object generation;
- сетевой rendering/conversion service;
- лицензирование или поставка proprietary SDK;
- UI/API/CLI configuration.

## 8. Основание и ограничения совместимости

Официальная документация WIRIS описывает MathML как текстовый вход для conversion SDK и как формат хранения/обмена MathType integrations. Та же документация отдельно указывает SDK license и platform-specific native integration. Поэтому этап 092 использует MathML только как изолированный payload contract и не включает proprietary runtime:

- <https://docs.wiris.com/mathtype-sdk-documentation/converting-equations>
- <https://docs.wiris.com/en_US/mathtype-sdk-technical-documentation/mathtype-api-documentation>
- <https://docs.wiris.com/en_US/technical-references-folder/mathml-coverage-reference>

Фактическая совместимость конкретных generated shapes остаётся предметом этапа 093.
