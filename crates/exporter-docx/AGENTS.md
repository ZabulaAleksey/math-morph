# DOCX / Word Rules

- Read architecture before changing exporter contracts.
- Supported equations must be editable Word/OMML structures, not screenshots.
- Word-specific XML belongs in exporter layer, never Mathcad AST.
- DOCX package/relationships/XML must be structurally validated in tests.
- External relationships and embedded active content require explicit security review.
- Unsupported equation uses explicit fallback + conversion warning; never silent loss.
- MathType remains a separate backend/adapter.
