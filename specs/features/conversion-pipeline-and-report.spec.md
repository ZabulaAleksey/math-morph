# SPEC: Conversion pipeline, diagnostics и partial result

**Статус:** accepted
**Версия:** 1.0.0
**Дата:** 2026-08-20
**Область:** этапы 143–147

## 1. Цель и границы

Создать общий application core для детерминированной конвертации подтверждённого legacy XMCD worksheet30 в DOCX через существующие parser, transformation pipeline, Document IR и DOCX exporter.

Первая версия поддерживает безопасно отображаемые text и parsed math regions. Prime MCDX content parsing, binary asset extraction, plots/charts/diagrams, evaluation, API, UI и persistent JSON report находятся вне scope. MCDX определяется, но завершается typed unsupported failure без попытки передать Prime XML legacy parser.

## 2. Требования

### FR-CONVERT-143 — общий ConversionPipeline

Pipeline выполняет `detect → parse → transform → Worksheet→Document IR → DOCX export → DOCX validate` и не зависит от CLI, HTTP или UI. Подтверждённое содержимое имеет приоритет над расширением; mismatch создаёт warning.

Text regions отображаются в `TextBlockIr`, parsed supported math — в `FormulaIr { original, display }`. Порядок блоков соответствует `Worksheet::visual_order()`. Metadata, region ID, source ordinal и безопасная provenance сохраняются. Неподтверждённые единицы layout не угадываются и получают `approximate` fidelity.

### FR-DIAG-144 — DiagnosticsCollector

Collector имеет детерминированный порядок, caller-configurable hard cap и стабильные machine codes. При превышении cap pipeline завершается typed fatal error; diagnostics не содержат document content, formula values, filename или absolute paths.

### FR-SEVERITY-145 — severity model

Поддерживаются `Warning`, `RecoverableError`, `FatalError`. Security/integrity/limit/parser boundary failures всегда fatal. Unsupported region при разрешённой safe partial policy является recoverable и обязан попасть в report.

### FR-REPORT-146 — ConversionReport

Report содержит итоговый `Completed` или `CompletedWithWarnings`, агрегированные counts, ordered diagnostics и per-item fidelity: `Exact`, `Approximate`, `Unsupported`, `FallbackRendered`. В этапе 146 report является in-memory Rust contract; стабильный JSON/CLI report относится к этапу 153.

### FR-PARTIAL-147 — safe partial conversion

При `AllowSafePartial` комбинация supported и unsupported regions создаёт DOCX и `CompletedWithWarnings`; каждый пропуск имеет `Unsupported` record и diagnostic. Если безопасно экспортируемых blocks нет, conversion fails и не создаёт artifact. `FallbackRendered` не создаётся без реального production fallback.

`Strict` policy завершает конвертацию при первом recoverable unsupported item. Security failure никогда не переводится в partial/fallback и не повторяется с ослабленными limits.

### NFR-CONVERT-001 — детерминизм

Одинаковые bytes, filename и options создают byte-identical DOCX и равный report.

### NFR-CONVERT-002 — ресурсные ограничения

Pipeline переиспользует parser/IR/exporter limits и дополнительно ограничивает diagnostics/items. Checked arithmetic применяется до роста коллекций.

### SEC-CONVERT-001 — доверенная граница

Unknown input, DTD/entities, malformed XML, container attack, limit violation, invalid provenance и exporter validation failure завершаются fail closed без artifact. Partial conversion не обходит ни один security control.

## 3. Публичная граница

```text
ConversionPipeline
ConversionRequest { bytes, file_name, target, options }
ConversionOptions { partial_policy, limits }
ConversionOutcome { artifact, report }
ConversionFailure { code, diagnostics }
```

Новый `conversion-core` является библиотечным application crate. `Document IR V1` не меняется. Для первой версии target равен `Docx`, equation backend — `WordOmml`.

## 4. Ошибки и ограничения

- MCDX: `MCDX_CONTENT_UNSUPPORTED`, без DOCX.
- Unknown/empty input: fatal invalid input.
- Invalid/unsupported math region: recoverable только при `AllowSafePartial`.
- Picture/plot/table/program/opaque region без production mapper: explicit unsupported record.
- All-unsupported worksheet: fatal no-exportable-content.
- MathType backend: unavailable; автоматического fallback на него или с него нет.

## 5. Критерии приёмки

- `AC-CONVERT-143`: synthetic supported XMCD проходит полный core path до structurally valid DOCX.
- `AC-DIAG-144`: ordering/cap/redaction collector проверены unit tests.
- `AC-SEVERITY-145`: security failures fatal, unsupported content recoverable только по policy.
- `AC-REPORT-146`: все fidelity/status variants покрыты model tests; production outcome не заявляет неиспользованный fallback.
- `AC-PARTIAL-147`: mixed worksheet даёт artifact + warnings; all-unsupported и strict policy не дают artifact.
- `AC-CONVERT-DET`: повторная конвертация byte-identical и report-equal.

## 6. Связь с тестами

| Требование | Проверка |
|---|---|
| FR-CONVERT-143 | integration XMCD→IR→DOCX→DocxValidator |
| FR-DIAG-144, FR-SEVERITY-145 | collector/severity unit tests |
| FR-REPORT-146 | report/fidelity unit tests |
| FR-PARTIAL-147 | mixed, strict, all-unsupported integration tests |
| SEC-CONVERT-001 | DTD, malformed, limits, MCDX negative tests |
| NFR-CONVERT-001 | deterministic bytes/report regression |

## 7. История

- 1.0.0 — принят application-core контракт этапов 143–147.
