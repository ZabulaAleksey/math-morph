# Архитектура

## 1. Цель

Платформа разбирает Mathcad-документы, сохраняет математическую и документную семантику, выполняет преобразования и экспортирует результат в редактируемые форматы. Архитектура должна масштабироваться от локальной конвертации в браузере/WASM до серверной обработки API/workers.

## 2. Общий поток

```text
Input (.xmcd/.mcdx)
        |
        v
Format Detector / Safe Container Reader
        |
        v
Mathcad Parser
        |
        v
Mathcad AST + layout/source metadata
        |
        v
Semantic Analyzer
  - symbol table
  - dependency graph
  - evaluation order
        |
        v
Transformation / Evaluation Engine
  - notation profiles
  - substitutions
  - complex-number traces
  - precision policy
        |
        v
Document IR
  - PageIR / MetadataIR
  - TextBlock
  - EquationBlock
  - TableIR
  - ImageBlock
  - PlotBlock / ChartIR
  - DiagramBlock / DiagramIR
        |
        +------------------------------+
        |              |               |
        v              v               v
DOCX/OMML          Markdown/...     Future Excel/Visio
```

## 3. Основные границы

### Core Rust

Владеет:
- определение формата;
- безопасный parsing MCDX/XMCD;
- AST;
- семантика;
- преобразования и вычисление;
- Document IR;
- версионирование и сериализация Document IR;
- совместимый с браузерным WASM core, где это практично.

Не владеет:
- пользовательские сессии;
- оплата;
- аутентификация HTTP;
- React UI.

Текущая реализованная граница `mathcad-parser` разделена на слои:

- `format` сопоставляет заявленное расширение с форматом, подтверждённым содержимым;
- `mcdx` выполняет bounded ZIP preflight, перечисление и классификацию частей без извлечения на диск;
- `xml_metadata` читает только безопасный root envelope, namespaces и schema URI без загрузки схем;
- `source` владеет immutable XML bytes и безопасными `SourceSpan`/opaque fragments;
- `xml_worksheet` выполняет bounded namespace-aware разбор worksheet30 и строит worksheet/region model;
- `worksheet` и `region` отделяют metadata, layout, source/visual/z order и typed/opaque region content;
- `math_xml` отображает подтверждённое подмножество math30 в синтаксический `ast` с проверкой QName, arity, shape и limits;
- `diagnostic` содержит scoped machine-readable codes до появления общего collector.

`WorksheetParser` поддерживает legacy XMCD contract worksheet30 3.0.3 + math30 3.0.2. Он сохраняет metadata, recursive regions, source provenance и синтаксический AST до сравнений, но ничего не вычисляет. `MathParseOutcome` различает parsed, typed invalid и unsupported math; неизвестные region/inline fragments остаются source-backed. Source order, visual order и z-order представлены отдельно.

Безопасный MCDX container reader по-прежнему заканчивается на manifest. Внутренний Prime worksheet не передаётся legacy parser без отдельного подтверждённого schema contract. Неизвестные MCDX parts сохраняются в `ContainerManifest`, а не интерпретируются эвристически.

Реальная последовательность текущего parser:

```text
bytes
  -> FormatDetector / SafeMcdxReader / XML root inspector
  -> WorksheetParser (только подтверждённый legacy worksheet30)
  -> Worksheet + Regions + source-backed opaque fragments
  -> structural Math AST (без evaluator)
```

Функциональный путь этапов 095–148 добавляет поверх parser две production-границы:

- `math-engine`: bounded immutable `Original AST → Display AST` presentation pipeline и backend-neutral semantic foundation `SymbolTable`; таблица хранит ordered revisions и не выполняет evaluation/substitution;
- `conversion-core`: application orchestration `detect → parse → transform → Document IR → DOCX export → DOCX validate`, bounded diagnostics/report и explicit `Strict`/`AllowSafePartial` policy.

Текущий живой поток:

```text
mathmorph CLI
  -> conversion-core
  -> FormatDetector + WorksheetParser (legacy XMCD)
  -> math-engine TransformationPipeline
  -> DocumentIrV1 producer
  -> DocxExporter(WordOmml)
  -> DocxValidator
  -> atomic no-replace local publication
```

Prime MCDX распознаётся безопасно, но не передаётся legacy parser и завершается typed `MCDX_CONTENT_UNSUPPORTED`.

### Shared model и Document IR

Этапы 052–061 выделили две нейтральные границы:

- `math-model` владеет source-neutral `MathExpression`, `SourceSpan`, boolean/unit/unsupported nodes и стабильным Serde contract;
- `document-ir` владеет versioned wire envelope V1, pages/blocks/layout/provenance/fidelity и портами `EquationExporter`/`AssetResolver`.

Dependency DAG не допускает обратной связи exporter → parser:

```text
math-model <--- mathcad-parser
     ^
     +------ math-engine
     +------ document-ir <--- exporter-docx <--- conversion-core <--- mathmorph-cli
                        <--- exporter-mathml <--- exporter-mathtype
```

`FormulaIr` хранит immutable optional `original` и обязательный `display`; exporter читает только `display`. Binary assets, filesystem paths и URLs не входят в Document IR JSON и выдаются адаптеру только через `AssetResolver`.

`math-engine` отделяет синтаксис от будущего вычисления. `SymbolTable` хранит ordered scalar/function revisions и разрешает только definitions, видимые строго до source ordinal. `ReferenceAnalyzer` поверх того же immutable AST извлекает свободные variable/function references с typed identity, lexical binder scopes и детерминированной per-site дедупликацией. `DependencyGraph` связывает concrete revision IDs рёбрами `dependent → dependency`, не создаёт guessed nodes для unresolved references и не переставляет worksheet. `EvaluationPlan` разворачивает эти рёбра bounded Kahn-проходом и выдаёт dependency-first порядок со stable source tie-break; numeric evaluator ещё отсутствует.

### Exporters

- `WordEquationExporter`: `MathExpression` → bounded editable OMML subset для numbers, identifiers, add/subtract, multiplication, fractions, powers, roots, scripts, typed function calls, paired grouping, vectors/matrices, integrals, derivatives и sum/product. Renderer выдаёт только канонические `m:*` shapes, проверяемые строгим `DocxValidator`.
- `EquationBackend`/`DocxExportConfig`: публичный выбор backend для DOCX; `WordOmml` — default, а зарезервированный `MathType` завершается typed `EquationBackendUnavailable` без fallback.
- `MathMlRenderer`: независимый от Word bounded exporter `MathExpression` → standalone Presentation MathML Core для принятого scalar subset. Он реализует общий `EquationExporter` и не зависит от parser/DOCX.
- `MathTypeAdapter`: отдельный experimental crate поверх `MathMlRenderer`; выдаёт opaque `application/mathml+xml` payload, не принимает raw XML, не выполняет I/O и не подключён к `DocxExporter`.
- Compatibility matrix этапа 093 хранится в `docs/MATHTYPE_COMPATIBILITY.md`: static WIRIS coverage отделено от versioned live import/edit evidence, которое пока `UNVERIFIED`. Будущий MathType bridge/SDK/OLE/service остаётся отдельным этапом; в текущем backend enum `MathType` по-прежнему fail closed.
- `DocxExporter`: validated Document IR → deterministic single-page DOCX/OOXML с text/styles, internal PNG/JPEG и equations.
- `DocxValidator`: fail-closed validator только генерируемого subset, а не универсальный validator произвольного DOCX.
- В будущем: Markdown, LaTeX, PDF, HTML, JSON, Typst.
- `ChartExporter`: текущий растровый путь + будущий путь Excel.
- `DiagramExporter`: текущий растровый путь + будущий путь VSDX.

### Web

Next.js/React/TypeScript:
- загрузка и dropzone;
- settings;
- состояние задачи;
- учётная запись и dashboard;
- API key UI;
- области оплаты и администрирования;
- координация локальной конвертации WASM.

Математическая семантика не реализуется в компонентах React.

### API

FastAPI/Pydantic/SQLAlchemy:
- интеграция аутентификации;
- авторизация;
- ключи и scopes API;
- координация задач конвертации;
- метаданные, история и предпочтения;
- подписанные flows скачивания и загрузки;
- интеграция оплаты и учёта использования.

Семантика конвертации делегируется общему core.

### CLI

- Реализованный binary `mathmorph` предоставляет stage-148 команду `convert <input.xmcd> --to docx [--output <path>]`.
- Переиспользует core Rust, контракты exporters и общую модель диагностики.
- Не владеет семантикой parser и математики и не обходит ограничения ввода и безопасности.
- Выполняет bounded single-read input, redacted вывод и атомарную no-replace публикацию из same-directory `create_new` temp через hard link; replacing-rename fallback запрещён.
- Успешная публикация является commit point; последующая cleanup-проблема выводится warning и не превращает готовый artifact в false failure.
- Стабильный машиночитаемый JSON report, `inspect` и дополнительные options относятся к этапам 149–153.

### Слой workers

RabbitMQ + Celery:
- серверные конвертации;
- контролируемые повторы;
- durable job ID, progress и correlation ID;
- timeout и отмена, где это возможно;
- идемпотентность и восстановление состояния после reconnect;
- flow dead-letter и ошибок;
- независимое горизонтальное масштабирование.

### Data

- PostgreSQL: метаданные, ссылки на пользователей и профили, задачи, настройки, использование, подписки, события аудита и безопасности.
- S3-совместимое хранилище/MinIO: зашифрованные и временные объекты, short-lived signed URLs, retention/delete и изоляция владельца/workspace; не использовать PostgreSQL для крупных файлов.
- Redis: только кэш, rate limit и временная координация, не авторитетное хранилище.

## 4. Модель конфиденциальности

Предпочтительный безопасный путь:

```text
Browser
  -> Rust/WASM processing where supported
  -> client-side authenticated encryption
  -> ciphertext storage
```

Серверная конвертация является отдельным режимом доверия: нельзя одновременно обещать серверную обработку открытого текста и абсолютный zero knowledge без дополнительной архитектуры конфиденциальных вычислений.

Восстановление учётной записи и восстановление ключа документа — разные процессы.

## 5. Модель API

Версионируемый API `/api/v1`.

Типичный асинхронный поток:

```text
POST conversion
 -> validate/authz/quota
 -> create job
 -> enqueue
 -> worker
 -> result/report
 -> encrypted/policy-controlled storage
 -> status/download
```

История конвертаций API следует пользовательским предпочтениям сохранения и политике явного запроса.

## 6. Модель ошибок

Система различает:
- предупреждение;
- восстанавливаемая ошибка;
- фатальная ошибка.

Частичная конвертация допускается только тогда, когда результат не вводит пользователя в заблуждение. Неподдерживаемая структура не должна исчезать незаметно.

### 6.1 Recovery, retry и degraded mode

Общий контракт retry/fallback/degraded/fail-closed наследуется из
`~/codex-workspace/rules/fallback-policy.md`.

MathMorph-specific цепочки являются каноническими в:

`docs/FALLBACKS.md`

Этот архитектурный документ не дублирует полный fallback contract.
Его задача — определить компоненты и интерфейсы, необходимые для:

- typed failures;
- idempotency;
- reconciliation после неизвестного результата mutation;
- cancellation;
- retry state;
- recovery после disconnect;
- observable degraded state;
- partial-result reporting.

Правило для операций с неизвестным side effect:

`unknown side effect → reconciliation → retry/fallback`

а не:

`unknown side effect → повторить mutation`.

Security-sensitive failure не переводится в менее безопасный режим:
authentication, authorization, integrity, crypto, security validation
и hard resource limits завершаются fail closed согласно `docs/SECURITY.md`.

Конкретные product-specific цепочки parser, Mathcad version handling,
DOCX/MathType backend, assets, partial conversion, local/WASM,
server processing и API recovery определены только в `docs/FALLBACKS.md`.

## 7. Scaling

Этап 1: Docker Compose, один API и worker.
Этап 2: stateless-реплики API, несколько workers и внешние хранилище и база данных.
Этап 3: Kubernetes/Helm только после подтверждённой необходимости.

## 8. Будущая расширяемость

- Backend MathType не влияет на parser.
- Восстановление диаграмм Excel идёт через ChartIR.
- Восстановление Visio идёт через DiagramIR и создаёт редактируемые shapes и connectors, а не одно изображение.
- Новые выходные форматы реализуют контракт exporter.
- REST, CLI, GUI/SDK, workers и будущий MCP являются adapters к общим Application Services и не дублируют parser или бизнес-логику задач.

## 9. Запрещённые shortcut-архитектуры

- XML → DOCX напрямую без AST/IR.
- parser, который знает о React/API/Word.
- формулы-картинки как основной путь.
- административный endpoint для чтения защищённых конфиденциальностью документов.
- локальная очередь в памяти как production-источник истины.
