# Architecture

## 1. Цель

Платформа разбирает Mathcad-документы, сохраняет математическую и документную семантику, выполняет преобразования и экспортирует результат в редактируемые форматы. Архитектура должна масштабироваться от локального browser/WASM conversion до server-side API/worker processing.

## 2. High-level flow

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
  - TextBlock
  - EquationBlock
  - ImageBlock
  - PlotBlock / ChartIR
  - DiagramBlock / DiagramIR
        |
        +------------------------------+
        |              |               |
        v              v               v
DOCX/OMML          Markdown/...     Future Excel/Visio
```

## 3. Основные boundaries

### Rust core

Владеет:
- format detection;
- safe MCDX/XMCD parsing;
- AST;
- semantics;
- transformations/evaluation;
- Document IR;
- browser WASM-compatible core where practical.

Не владеет:
- user sessions;
- billing;
- HTTP authentication;
- React UI.

### Exporters

- `WordEquationExporter`: AST/Display AST → OMML/Word equation.
- `MathTypeExporter`: отдельный adapter, потенциально через MathML.
- `DOCXExporter`: Document IR → DOCX/OOXML.
- Future: Markdown, LaTeX, PDF, HTML, JSON.
- `ChartExporter`: текущий raster path + future Excel path.
- `DiagramExporter`: текущий raster path + future VSDX path.

### Web

Next.js/React/TypeScript:
- upload/dropzone;
- settings;
- job state;
- account/dashboard;
- API key UI;
- billing/admin surfaces;
- WASM local conversion orchestration.

Математическая семантика не реализуется в React components.

### API

FastAPI/Pydantic/SQLAlchemy:
- authentication integration;
- authorization;
- API keys/scopes;
- conversion job orchestration;
- metadata/history/preferences;
- signed download/upload flows;
- billing/usage integration.

Conversion semantics делегируется общему core.

### CLI

- Thin local adapter for conversion and safe format inspection.
- Reuses Rust core, exporter contracts and the common diagnostics model.
- Does not own parser/math semantics and does not bypass input/security limits.
- May emit human-readable output plus a stable machine-readable report for automation.

### Worker layer

RabbitMQ + Celery:
- server-side conversions;
- controlled retries;
- timeout/cancellation where possible;
- dead-letter/error flow;
- independent horizontal scaling.

### Data

- PostgreSQL: metadata, users/profile refs, jobs, settings, usage, subscriptions, audit/security events.
- S3-compatible storage/MinIO: encrypted/temporary objects; не использовать PostgreSQL для крупных файлов.
- Redis: cache/rate-limit/ephemeral coordination only, не system of record.

## 4. Privacy model

Предпочтительный secure path:

```text
Browser
  -> Rust/WASM processing where supported
  -> client-side authenticated encryption
  -> ciphertext storage
```

Server-side conversion является отдельным trust mode: нельзя одновременно обещать серверную plaintext-обработку и абсолютное zero-knowledge без дополнительной confidential-compute architecture.

Account recovery и document-key recovery — разные процессы.

## 5. API model

Versioned API `/api/v1`.

Typical async flow:

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

API conversion history следует user save preferences/explicit request policy.

## 6. Error model

Система различает:
- warning;
- recoverable error;
- fatal error.

Partial conversion допускается только когда результат не вводит пользователя в заблуждение. Unsupported structure не должна silently disappear.

## 7. Scaling

Stage 1: Docker Compose, single API + worker.
Stage 2: stateless API replicas + multiple workers + external storage/DB.
Stage 3: Kubernetes/Helm только после подтверждённой необходимости.

## 8. Future extensibility

- MathType backend не влияет на parser.
- Excel chart reconstruction идёт через ChartIR.
- Visio reconstruction идёт через DiagramIR и генерирует редактируемые shapes/connectors, а не одно изображение.
- Новые output formats реализуют exporter contract.

## 9. Запрещённые shortcut-архитектуры

- XML → DOCX напрямую без AST/IR.
- parser, который знает о React/API/Word.
- формулы-картинки как основной path.
- admin endpoint для чтения privacy-protected документов.
- локальная in-memory очередь как production source of truth.
