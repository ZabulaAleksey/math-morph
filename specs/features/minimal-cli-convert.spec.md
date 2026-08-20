# SPEC: Minimal CLI convert

**Статус:** accepted
**Версия:** 1.0.0
**Дата:** 2026-08-20
**Область:** этап 148

## 1. Цель и границы

Предоставить первую реальную локальную команду:

```text
mathmorph convert <input.xmcd> --to docx
```

CLI является тонким filesystem/argument adapter над `conversion-core` и не дублирует detector, parser, transformation, IR или exporter logic. Команды inspect, дополнительные formats/options и стабильный JSON report относятся к этапам 149–153.

## 2. Требования

### FR-CLI-148 — команда convert

Команда принимает ровно один input и обязательный target `docx`. Выход по умолчанию — соседний `<stem>.docx`; optional `--output <path>` разрешён для тестируемого явного назначения. Unknown flags, missing values и unsupported target дают usage error без чтения input.

CLI вызывает production `ConversionPipeline` с `AllowSafePartial`, печатает redacted summary/diagnostic codes и возвращает:

- `0` — completed или completed-with-warnings;
- `2` — usage;
- `3` — invalid/unsupported input;
- `4` — conversion/export failure;
- `5` — filesystem I/O.

Эти коды являются контрактом этапа 148; machine-readable report body относится к этапу 153.

### FR-CLI-OUTPUT-148 — безопасная запись

Существующий output не перезаписывается. Input и output не могут ссылаться на один и тот же путь. Result сначала записывается в уникальный `create_new` temporary file в каталоге output, синхронизируется и затем атомарно переименовывается. При сбое удаляется только созданный этой операцией temp-файл; final artifact не появляется.

### NFR-CLI-148 — bounded I/O

Input читается с hard maximum до передачи core. CLI не загружает файл повторно после limit failure и не содержит retry/fallback. Абсолютные пути, document content и formula values не попадают в stderr или `Debug`.

### SEC-CLI-148 — файловая граница

Symlink/reparse/race и existing-output случаи завершаются fail closed настолько, насколько позволяет стандартная библиотека текущей платформы. CLI не удаляет и не изменяет input, существующий output или неизвестные temporary files.

## 3. Совместимость

- Поддержанный путь этапа 148: legacy `.xmcd` worksheet30 → `.docx` с text и supported editable OMML equations.
- `.mcdx` безопасно определяется, но возвращает `MCDX_CONTENT_UNSUPPORTED` и exit `3` без output.
- MathType не подключается; используется `WordOmml`.

## 4. Критерии приёмки

- `AC-CLI-148-001`: настоящий binary создаёт structurally valid DOCX из supported XMCD.
- `AC-CLI-148-002`: DOCX содержит editable OMML для supported equation.
- `AC-CLI-148-003`: existing output не изменяется.
- `AC-CLI-148-004`: invalid input/MCDX/security failure возвращают non-zero и не создают output.
- `AC-CLI-148-005`: mixed supported/unsupported input возвращает `0`, создаёт output и печатает warning code.
- `AC-CLI-148-006`: stderr/Debug не раскрывают absolute path или document/formula payload.

## 5. Связь с тестами

| Требование | Проверка |
|---|---|
| FR-CLI-148 | process-level integration test настоящего binary |
| FR-CLI-OUTPUT-148 | existing output, same path, cleanup tests |
| NFR-CLI-148, SEC-CLI-148 | oversized/invalid input и redaction regressions |
| AC-CLI-148-001/002 | generated DOCX + `DocxValidator` + OMML structural assertion |

## 6. История

- 1.0.0 — принят минимальный CLI contract этапа 148.
