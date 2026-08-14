# Правила `document-ir`

- Crate владеет версионируемым backend-neutral Document IR и малыми exporter ports.
- Не добавляй зависимости от Mathcad XML/parser, ZIP, Word/OOXML, HTTP, filesystem paths или frontend.
- Wire schema меняется только версионно; V1 использует bounded JSON, stable `snake_case` names и strict unknown-field rejection.
- Layout и physical sizes хранятся как integer micrometres; `f32/f64` в wire contract запрещены.
- Binary assets не сериализуются: IR хранит `AssetRefIr`, bytes предоставляет `AssetResolver`.
- `Debug` и ошибки не раскрывают text, formula payload, metadata или asset IDs.
