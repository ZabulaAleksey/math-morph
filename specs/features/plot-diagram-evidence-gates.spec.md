# SPEC: Plot/diagram preservation и evidence gates

**Статус:** accepted  
**Версия:** 1.0.0  
**Дата:** 2026-08-21  
**Область:** подтверждённые части этапов 123–142

## Цель

Сохранять только доказанную семантику plot/diagram, поддержать явно предоставленный безопасный raster preview и не угадывать axes, series, shapes, connectors или container mapping без versioned fixtures/schema evidence.

## Требования

- `FR-PLOT-123`: parser сохраняет подтверждённые `PlotRegion.item_idref`, `disable_calc`, span и region provenance; malformed boolean завершается typed error.
- `FR-PLOT-124`: plot metadata проходит в versioned Document IR. V1 остаётся byte-compatible; исторически отклонённая версия 2 остаётся unsupported; новая схема имеет `schema_version = 3`, strict sidecar metadata, bounded references и no-loss V1 projection.
- `FR-PREVIEW-126/134`: `PlotIr.preview` и `DiagramIr.preview` экспортируются в DOCX только как explicit `FallbackRendered` PNG/JPEG через существующий bounded `AssetResolver`. Preview без размера, неверный media type, missing/rejected asset или неверная fidelity завершается typed fail-closed error.
- `FR-FALLBACK-127`: отсутствие preview не создаёт placeholder и не игнорируется молча. Strict conversion завершается ошибкой; safe partial сохраняет unsupported item/diagnostic и исключает его только из DOCX projection.
- `SEC-PLOT-DIAGRAM-001`: raw XML/SVG/metafile/OLE/ActiveX, external relationships и payload-derived guesses запрещены.
- `NFR-PLOT-DIAGRAM-001`: V1 golden не меняется; V3 deterministic round-trip проверяет referential integrity, canonical order, limits и redacted Debug.

## Evidence-gated scope

- 125 preview extraction, 128–132 ChartIR/Excel, 133 diagram detection и 136 shape forensics заблокированы до легальных versioned fixtures и документированной mapping schema.
- 135 и 137–142 требуют отдельной versioned DiagramIR/VSDX SPEC, resource limits и live Visio editability evidence; synthetic package не является live compatibility proof.
- Компонентный DOCX preview exporter не означает, что preview уже извлекается из реального Mathcad input.

## Критерии приёмки

- V1 golden/read contract остаётся неизменным, `schema_version = 2` по-прежнему отклоняется.
- V3 round-trip сохраняет `item_idref` presence/value и `disable_calc`; wrong/missing/oversized metadata отклоняется.
- Mixed text+plot safe partial создаёт валидный DOCX и возвращает V3 IR с unsupported warning; all-plot не создаёт artifact.
- Явные plot/diagram previews создают structurally valid DOCX; отсутствие preview и неверная fidelity fail closed.
- Workspace tests, fmt, Clippy и project validator проходят.
