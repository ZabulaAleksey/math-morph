# Requirement Traceability

## Purpose

Keep a lightweight path from approved product requirements to roadmap stages, implementation and tests without creating a second specification.

Canonical sources:

- product requirements: `docs/SPECIFICATION.md`;
- implementation order and stable stage numbers: `docs/ROADMAP.md`;
- executable stage slices: the matching section of `docs/PROMPTS.md`;
- architecture constraints and decisions: `docs/ARCHITECTURE.md` and `docs/DECISIONS.md`;
- verification evidence: committed code, tests, fixtures and review results.

Current product-level references use stable specification section numbers such as `SPEC-02`. New feature/domain specs may add `REQ-*` IDs when section-level traceability is not precise enough; those IDs must point back to the canonical product requirement.

## Initial mapping

All stages are `planned` until implementation and verification evidence exist.

| Requirement area | Roadmap stages | Expected implementation / verification evidence | Status |
|---|---:|---|---|
| SPEC-18 foundation and project boundaries | 001–010 | monorepo layout, canonical context documents and context-pack validation | planned |
| SPEC-02 input formats and safe detection | 011–035 | `crates/mathcad-parser/`, format fixtures, malformed/container tests | planned |
| SPEC-04 conversion architecture | 036–061, 143–147 | AST, semantic boundaries, Document IR, pipeline contract tests | planned |
| SPEC-05 editable formula export | 062–094 | `crates/exporter-docx/`, OMML/package tests, Word smoke references | planned |
| SPEC-06 transformations and precision | 095–111 | `crates/math-engine/`, semantic-preservation and trace tests | planned |
| SPEC-07 complex numbers | 112–122 | algebraic/polar round-trips and edge-case tests | planned |
| SPEC-08 plots | 123–132 | PlotIR/ChartIR, preview fallback and reconstruction fixtures | planned |
| SPEC-09 diagrams | 133–142 | DiagramIR, raster fallback and editable VSDX POC evidence | planned |
| SPEC-13 local CLI adapter | 148–153 | shared-core `convert`/`inspect` commands and structured-report tests | planned |
| SPEC-10–11 web pages and conversion states | 154–161 | `apps/web/`, component/E2E/accessibility/error-state tests | planned |
| SPEC-16 internationalization | 162–165 | externalized catalogs and missing-key CI | planned |
| SPEC-12 authentication and recovery | 173–186 | auth boundary, replay/brute-force and recovery tests | planned |
| SPEC-13 API and API keys | 166–172, 191–217 | `services/api/`, contract/authz/job/idempotency tests | planned |
| SPEC-14 saved documents | 187–190, 200–217 | metadata/object-storage boundaries, retention/delete tests | planned |
| SPEC-15 privacy direction | 218–232 | privacy ADR, WASM/encryption prototypes and boundary tests | planned |
| SPEC-17 billing and monetization | 245–256 | provider abstraction, entitlement and lifecycle tests | planned |
| SPEC-10 admin/privacy boundary | 257–270 | RBAC, metrics surfaces and admin plaintext-denial tests | planned |
| SPEC-18 non-functional requirements | 233–244, 271–304 | security, observability, benchmarks, CI and scaling evidence | planned |
| SPEC-19 MVP success criteria | cumulative | end-to-end acceptance suite covering the complete MVP path | planned |

## Update rules

- Update this matrix in the same change that accepts a requirement-to-stage mapping change or verifies a stage.
- Use only `planned`, `in progress`, `blocked` or `verified`.
- `verified` requires committed implementation, passing relevant tests and completed review; a written prompt, plan or scaffold is insufficient.
- Link exact code/test paths once they exist. Do not invent future modules or claim unexecuted checks.
- If implementation intentionally changes an architectural boundary, update or add an ADR instead of silently changing this mapping.
