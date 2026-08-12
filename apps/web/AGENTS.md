# Web UI Rules

- If `docs/DESIGN.md` is non-empty, it is the UI contract.
- If DESIGN.md is empty, use minimal neutral patterns; do not invent a large design system.
- All user-facing strings go through i18n.
- Explicit idle/loading/warning/error/empty states.
- Error Boundary around converter and other independent critical surfaces.
- Client validation improves UX but is never a security boundary.
- Never expose secrets in browser bundle/logs.
- Math/business semantics belongs in shared core/services, not React components.
