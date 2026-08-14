# Правила `math-model`

- Crate владеет source-neutral Math AST и общими provenance-типами.
- Не добавляй зависимости от XML, ZIP, evaluator, Word/OOXML, HTTP или frontend.
- `ExpressionOrigin::Source` хранит реальный `SourceSpan`; derived-узлам не назначай фиктивные span.
- Публичный `Debug` не раскрывает identifier, literal, unit, QName или иной пользовательский payload.
- Изменения сериализуемого AST должны сохранять явный Serde contract и round-trip tests.
