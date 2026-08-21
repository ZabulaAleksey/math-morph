# SPEC: CLI inspection, exporter registry и machine-readable reports

**Статус:** accepted
**Версия:** 1.0.0
**Дата:** 2026-08-21
**Область:** этапы 149–153

## 1. Цель и границы

Расширить локальный `mathmorph` безопасными командами анализа и стабильными JSON-артефактами, не дублируя detector/parser/conversion logic. Реальным production exporter остаётся DOCX; остальные зарегистрированные форматы не имитируются и возвращают typed unavailable error.

## 2. Требования

- `FR-CLI-149`: `inspect <file>` выполняет bounded format detection и worksheet parsing через общий core, не экспортирует документ и возвращает versioned JSON summary без source payload/path.
- `FR-CLI-150`: allow-list форматов содержит `docx`, `markdown`, `latex`, `html`, `pdf`, `json`, `typst`; `--format` является alias `--to`. Неизвестное имя даёт `UNSUPPORTED_TARGET`, известный backend без реализации — `EXPORTER_UNAVAILABLE` без чтения input.
- `FR-CLI-151`: `--complex-mode algebraic|polar|both` валидируется и передаётся в `ConversionOptions`; значение не расширяет подтверждённый parser subset и не включает heuristic evaluation.
- `FR-CLI-152`: `--precision <1..1000>` создаёт одинаковую computation/display `PrecisionPolicy`; invalid/duplicate value даёт usage error до I/O.
- `FR-CLI-153`: `validate <file>` выполняет полный общий conversion/validation path без публикации DOCX и печатает versioned JSON report. `export-ir <file> [--output <path>]` сериализует versioned Document IR с bounded output; file publication использует существующий safe no-replace путь.

## 3. JSON contracts

Все JSON envelopes содержат `schema_version: 1`, используют стабильные machine strings и детерминированный порядок. Conversion report содержит status, counts, ordered diagnostics и items; inspect report — detected format, region count и diagnostic codes. Filename, absolute path, document text и formula values в report diagnostics отсутствуют. Document IR по своему контракту содержит пользовательское содержимое и поэтому выводится только по явной команде `export-ir`.

## 4. Ограничения и безопасность

- Все команды переиспользуют stage-148 bounded read и path/reparse проверки.
- Unknown/MCDX/malformed/oversized input завершается fail closed.
- JSON serialization имеет hard output cap и checked accounting.
- `inspect` и `validate` не создают файлы; `export-ir` не перезаписывает существующий output.
- Options не являются разрешением на неподтверждённую complex evaluation или exporter fallback.

## 5. Критерии приёмки

- Process tests покрывают успешные `inspect`, `validate`, `export-ir` и parseable versioned JSON.
- Unknown/known-unavailable format, invalid precision/complex mode и damaged input покрыты negative tests.
- Existing stage-148 convert behavior и exit codes не регрессируют.
- Workspace tests, fmt, Clippy и project validator проходят.
