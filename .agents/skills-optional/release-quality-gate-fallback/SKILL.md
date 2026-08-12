---
name: mathcad-release-quality-gate-fallback
description: Optional full project quality gate only when the installed AI Dev Team does not already provide an equivalent release/merge validation workflow.
---

DO NOT use when a global AI Dev Team release/quality gate is available.

1. Inspect changed stacks.
2. Run relevant lint/typecheck/build/tests.
3. Run Mathcad conversion regression and DOCX validation when applicable.
4. Run configured security/dependency/secret checks.
5. Return PASS/FAIL with exact commands/results.
