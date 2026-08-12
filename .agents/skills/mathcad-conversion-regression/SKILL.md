---
name: mathcad-conversion-regression
description: Run a focused Mathcad parser/math-engine/DocumentIR/DOCX regression workflow without duplicating the global AI Dev Team release or QA gate.
---

Use this Skill only for Mathcad-specific regression work. If the global AI Dev Team already runs generic lint/typecheck/release/security gates, do not repeat them here.

1. Determine changed Mathcad modules from git diff.
2. Select only affected fixture groups first; broaden only with evidence.
3. Run module tests and relevant conversion tests.
4. For DOCX changes, validate package/XML/relationships/editable equation structures.
5. Compare snapshots/golden outputs; never auto-accept a difference.
6. Report fixture ID, expected/actual behavior and likely layer.
7. End with PASS/FAIL and only commands actually executed.
