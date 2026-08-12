# Parser Rules

- Read root `AGENTS.md` plus `docs/SECURITY.md` sections A05/A06/A10 for input-handling changes.
- Parser knows Mathcad format/AST, not Word/HTTP/React.
- Never parse structured XML with regex as the primary parser.
- Disable/avoid external entity expansion; enforce size/depth/container limits.
- Preserve source/layout metadata needed for evaluation order and future exporters.
- Unknown nodes become controlled `Unsupported*` + diagnostics, not panic.
- Every fixed malformed/compatibility bug gets a regression fixture.
- Prefer streaming/bounded parsing for large input.
