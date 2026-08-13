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

Текущая реализованная входная граница `mathcad-parser` разделена на четыре узких модуля:

- `format` сопоставляет заявленное расширение с форматом, подтверждённым содержимым;
- `mcdx` выполняет bounded ZIP preflight, перечисление и классификацию частей без извлечения на диск;
- `xml_metadata` читает только безопасный root envelope, namespaces и schema URI без загрузки схем;
- `diagnostic` содержит только scoped-коды этапов 015–026 до появления общего collector.

Эта граница намеренно заканчивается до worksheet metadata, regions и AST. Неизвестные безопасные MCDX parts сохраняются в `ContainerManifest`, а не исчезают и не интерпретируются эвристически.

### Exporters

- `WordEquationExporter`: AST/Display AST → уравнение OMML/Word.
- `MathTypeExporter`: отдельный adapter, потенциально через MathML.
- `DOCXExporter`: Document IR → DOCX/OOXML.
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

- Тонкий локальный адаптер конвертации и безопасной проверки формата.
- Переиспользует core Rust, контракты exporters и общую модель диагностики.
- Не владеет семантикой parser и математики и не обходит ограничения ввода и безопасности.
- Может выводить понятный человеку результат и стабильный машиночитаемый отчёт для автоматизации.

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
