# Testing Strategy and Definition of Done

## Required test layers

### Parser/core

- unit tests per node/feature;
- snapshot/golden AST/IR where stable;
- malformed input and unknown-node tests;
- property tests for invariants;
- fuzz targets for XML/container/parser boundaries;
- memory/size/recursion limits.

### Mathematical engine

- dependency/symbol tests;
- substitution traces;
- precision vs rounding separation;
- complex algebraic↔polar round-trips with tolerance;
- quadrant/zero/division/angle normalization cases.

### DOCX/OMML

- generated package structural validation;
- relationship/content-type/XML validity;
- editable equation structure tests;
- reference DOCX manual smoke set for Word opening/editing;
- regression tests for nested equations.

### API/backend

- authz/API-key scope tests;
- async job lifecycle/idempotency;
- save-preference semantics;
- quota/rate-limit boundaries;
- retry vs non-retryable failures;
- storage failure/timeout tests.

### Frontend/E2E

- upload and drag/drop;
- file validation states;
- conversion states and recovery after network interruption;
- localized structured errors;
- auth/2FA/recovery flows;
- dashboard/API keys/privacy settings.

### Security

- XML attacks;
- ZIP bomb/path traversal;
- malicious filenames/metadata/SVG;
- XSS/injection;
- auth/recovery brute force/replay;
- secret/log redaction;
- admin privacy-boundary tests;
- dependency/MCP/hook/Skill supply-chain review.

## Fixture layout

`tests/fixtures/` groups: xmcd, mcdx, formulas, complex, plots, diagrams, mixed, corrupted, security, compatibility.

Every fixture belongs in a manifest with format/version/features/expected status. A corrected parser bug gets a permanent regression fixture when legally/technically possible.

## Golden rule

Do not update a golden fixture merely because a test failed after implementation changes. First prove that the desired behavior changed intentionally.

## DoD for a stage

- scope implemented without unrelated future features;
- targeted tests added/passing;
- relevant negative/boundary test included;
- lint/typecheck/build for touched area pass;
- no parser panic/crash on user input path;
- docs/PROGRESS updated for meaningful stage;
- DECISIONS/ARCHITECTURE/SECURITY updated only if their contract changed;
- final report lists checks actually executed, not assumed checks.

Generic full release gates should be owned by the already installed global AI Dev Team when it provides them; do not duplicate them locally.
