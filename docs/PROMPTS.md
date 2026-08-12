# Codex Prompt Library — 304 atomic stages

## Usage

Do **not** load this entire file for a normal task. `AGENTS.md` explicitly requires reading only the current stage/section. Run one stage, review, fix, commit, then continue.

## Base prompt

```text
Before starting this stage:

1. Read root `AGENTS.md`.
2. Read only the local `AGENTS.md` files and canonical docs relevant to the files you may change.
3. Reuse the already installed global AI Dev Team for generic architect/QA/security/frontend/backend/DevOps/release/Git capabilities. Do not create or run a duplicate generic project agent.
4. Use Mathcad project subagents only when domain expertise materially helps. Total global+project subagent budget follows `AGENTS.md`.
5. Implement only this stage; do not pre-build later stages.
6. Add targeted tests and a relevant negative/boundary case. A bug fix requires a regression test/fixture.
7. Run only relevant checks during development; do not duplicate a global release gate.
8. Update `PROGRESS.md` for meaningful completion and update DECISIONS/ARCHITECTURE/SECURITY only if their contracts changed.
9. Never log document content, formulas, secrets or encryption keys.
10. End with: Completed / Files changed / Tests added / Validation performed / Test results / Architecture decisions / Known limitations / Next stage / Not implemented intentionally.
```

# Foundation

## Prompt 001 — monorepo

```text
Create the baseline monorepo/workspace directories and buildable empty application/service/crate shells according to TECH_STACK. Do not add business logic.
```

## Prompt 002 — canonical docs

```text
Ensure all canonical docs exist and contain only known requirements; unknown decisions remain TBD.
```

## Prompt 003 — empty design contract

```text
Keep docs/DESIGN.md completely empty. Do not invent design tokens or UI style.
```

## Prompt 004 — root AGENTS

```text
Create/maintain root AGENTS.md as a compact router/overlay compatible with the existing global AI Dev Team.
```

## Prompt 005 — parser AGENTS

```text
Implement only roadmap stage **005 — parser AGENTS**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 006 — math-engine AGENTS

```text
Implement only roadmap stage **006 — math-engine AGENTS**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 007 — DOCX AGENTS

```text
Implement only roadmap stage **007 — DOCX AGENTS**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 008 — frontend AGENTS

```text
Implement only roadmap stage **008 — frontend AGENTS**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 009 — API AGENTS

```text
Implement only roadmap stage **009 — API AGENTS**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 010 — tests AGENTS

```text
Implement only roadmap stage **010 — tests AGENTS**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

# Fixtures

## Prompt 011 — fixture taxonomy

```text
Implement only roadmap stage **011 — fixture taxonomy**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 012 — fixture manifest

```text
Implement only roadmap stage **012 — fixture manifest**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 013 — fixture validator

```text
Implement only roadmap stage **013 — fixture validator**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 014 — corrupted/security starter fixtures

```text
Implement only roadmap stage **014 — corrupted/security starter fixtures**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

# Format detection

## Prompt 015 — InputFormat/FormatDetector

```text
Implement InputFormat and FormatDetector with Xmcd, Mcdx and Unknown; detection must not rely only on extension.
```

## Prompt 016 — XMCD content detection

```text
Implement only roadmap stage **016 — XMCD content detection**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 017 — MCDX content detection

```text
Implement only roadmap stage **017 — MCDX content detection**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 018 — extension/content mismatch diagnostics

```text
Implement only roadmap stage **018 — extension/content mismatch diagnostics**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

# MCDX container

## Prompt 019 — safe container open

```text
Implement safe MCDX container listing only; no worksheet parsing yet.
```

## Prompt 020 — path traversal defense

```text
Implement only roadmap stage **020 — path traversal defense**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 021 — ZIP limits

```text
Implement only roadmap stage **021 — ZIP limits**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 022 — container manifest

```text
Implement only roadmap stage **022 — container manifest**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 023 — worksheet part discovery

```text
Implement only roadmap stage **023 — worksheet part discovery**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 024 — embedded resources metadata

```text
Implement only roadmap stage **024 — embedded resources metadata**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 025 — unknown part handling

```text
Implement only roadmap stage **025 — unknown part handling**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

# XMCD/XML structure

## Prompt 026 — namespace/schema metadata

```text
Implement only roadmap stage **026 — namespace/schema metadata**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 027 — worksheet metadata

```text
Implement only roadmap stage **027 — worksheet metadata**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 028 — region discovery

```text
Implement only roadmap stage **028 — region discovery**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 029 — region coordinates/layout

```text
Implement only roadmap stage **029 — region coordinates/layout**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 030 — deterministic ordering

```text
Implement only roadmap stage **030 — deterministic ordering**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 031 — text region

```text
Implement only roadmap stage **031 — text region**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 032 — math region

```text
Implement only roadmap stage **032 — math region**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 033 — plot region

```text
Implement only roadmap stage **033 — plot region**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 034 — image region

```text
Implement only roadmap stage **034 — image region**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 035 — unknown region

```text
Implement only roadmap stage **035 — unknown region**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

# Math AST

## Prompt 036 — minimal scalar/operators AST

```text
Implement the minimal Mathcad AST for real values, variables and arithmetic operators. No evaluator yet.
```

## Prompt 037 — AST snapshots

```text
Implement only roadmap stage **037 — AST snapshots**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 038 — Definition

```text
Implement only roadmap stage **038 — Definition**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 039 — Evaluation

```text
Implement only roadmap stage **039 — Evaluation**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 040 — FunctionCall

```text
Implement only roadmap stage **040 — FunctionCall**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 041 — FunctionDefinition

```text
Implement only roadmap stage **041 — FunctionDefinition**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 042 — unary operators

```text
Implement only roadmap stage **042 — unary operators**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 043 — grouping

```text
Implement only roadmap stage **043 — grouping**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 044 — index/subscript

```text
Implement only roadmap stage **044 — index/subscript**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 045 — matrix

```text
Implement only roadmap stage **045 — matrix**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 046 — vector

```text
Implement only roadmap stage **046 — vector**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 047 — range

```text
Implement only roadmap stage **047 — range**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 048 — integral

```text
Implement only roadmap stage **048 — integral**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 049 — derivative

```text
Implement only roadmap stage **049 — derivative**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 050 — sum/product

```text
Implement only roadmap stage **050 — sum/product**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 051 — comparisons

```text
Implement only roadmap stage **051 — comparisons**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 052 — booleans

```text
Implement only roadmap stage **052 — booleans**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 053 — units

```text
Implement only roadmap stage **053 — units**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 054 — UnsupportedNode

```text
Implement only roadmap stage **054 — UnsupportedNode**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

# Document IR

## Prompt 055 — DocumentIR

```text
Introduce DocumentIR independent of Mathcad XML, DOCX, HTTP and frontend.
```

## Prompt 056 — TextBlock

```text
Implement only roadmap stage **056 — TextBlock**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 057 — EquationBlock

```text
Implement only roadmap stage **057 — EquationBlock**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 058 — ImageBlock

```text
Implement only roadmap stage **058 — ImageBlock**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 059 — PlotBlock

```text
Implement only roadmap stage **059 — PlotBlock**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 060 — DiagramBlock

```text
Implement only roadmap stage **060 — DiagramBlock**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 061 — layout model

```text
Implement only roadmap stage **061 — layout model**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

# DOCX basics

## Prompt 062 — minimal valid DOCX

```text
Generate the smallest structurally valid DOCX and verify package/XML relationships automatically.
```

## Prompt 063 — single text paragraph

```text
Implement only roadmap stage **063 — single text paragraph**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 064 — multiple paragraphs

```text
Implement only roadmap stage **064 — multiple paragraphs**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 065 — basic formatting

```text
Implement only roadmap stage **065 — basic formatting**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 066 — image block

```text
Implement only roadmap stage **066 — image block**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 067 — image sizing

```text
Implement only roadmap stage **067 — image sizing**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 068 — page dimensions

```text
Implement only roadmap stage **068 — page dimensions**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 069 — DOCX structural validator

```text
Implement only roadmap stage **069 — DOCX structural validator**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

# Word equations

## Prompt 070 — EquationExporter

```text
Create the replaceable EquationExporter interface without implementing MathType.
```

## Prompt 071 — WordEquationExporter

```text
Implement only roadmap stage **071 — WordEquationExporter**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 072 — number

```text
Implement only roadmap stage **072 — number**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 073 — variable

```text
Implement only roadmap stage **073 — variable**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 074 — add/subtract

```text
Implement only roadmap stage **074 — add/subtract**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 075 — multiply

```text
Implement only roadmap stage **075 — multiply**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 076 — fraction

```text
Implement only roadmap stage **076 — fraction**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 077 — powers

```text
Implement only roadmap stage **077 — powers**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 078 — roots

```text
Implement only roadmap stage **078 — roots**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 079 — subscripts

```text
Implement only roadmap stage **079 — subscripts**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 080 — sub+sup

```text
Implement only roadmap stage **080 — sub+sup**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 081 — functions

```text
Implement only roadmap stage **081 — functions**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 082 — brackets

```text
Implement only roadmap stage **082 — brackets**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 083 — matrices

```text
Implement only roadmap stage **083 — matrices**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 084 — integrals

```text
Implement only roadmap stage **084 — integrals**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 085 — derivatives

```text
Implement only roadmap stage **085 — derivatives**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 086 — sum/product

```text
Implement only roadmap stage **086 — sum/product**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 087 — nested equations regression

```text
Implement only roadmap stage **087 — nested equations regression**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 088 — manual Word reference validation

```text
Implement only roadmap stage **088 — manual Word reference validation**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

# MathType preparation

## Prompt 089 — backend enum/config

```text
Implement only roadmap stage **089 — backend enum/config**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 090 — MathML renderer

```text
Implement only roadmap stage **090 — MathML renderer**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 091 — MathML snapshots

```text
Implement only roadmap stage **091 — MathML snapshots**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 092 — experimental MathType adapter

```text
Implement only roadmap stage **092 — experimental MathType adapter**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 093 — compatibility doc

```text
Implement only roadmap stage **093 — compatibility doc**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 094 — feature-gated backend selection

```text
Implement only roadmap stage **094 — feature-gated backend selection**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

# Notation transformation

## Prompt 095 — transformation pipeline

```text
Introduce Original AST → transformation rules → Display AST without mutating original semantics.
```

## Prompt 096 — Definition presentation rule

```text
Implement only roadmap stage **096 — Definition presentation rule**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 097 — SymbolMappingRegistry

```text
Implement only roadmap stage **097 — SymbolMappingRegistry**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 098 — NotationProfile

```text
Implement only roadmap stage **098 — NotationProfile**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 099 — semantic-preservation regression

```text
Implement only roadmap stage **099 — semantic-preservation regression**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

# Dependency graph

## Prompt 100 — SymbolTable

```text
Create SymbolTable from definitions without substitution.
```

## Prompt 101 — variable references

```text
Implement only roadmap stage **101 — variable references**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 102 — dependency graph

```text
Implement only roadmap stage **102 — dependency graph**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 103 — worksheet evaluation order

```text
Implement only roadmap stage **103 — worksheet evaluation order**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 104 — undefined-variable diagnostic

```text
Implement only roadmap stage **104 — undefined-variable diagnostic**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 105 — circular dependency diagnostic

```text
Implement only roadmap stage **105 — circular dependency diagnostic**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

# Substitution

## Prompt 106 — simple substitution

```text
Implement only roadmap stage **106 — simple substitution**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 107 — recursive substitution

```text
Implement only roadmap stage **107 — recursive substitution**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 108 — depth limit

```text
Implement only roadmap stage **108 — depth limit**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 109 — EvaluationTrace

```text
Implement only roadmap stage **109 — EvaluationTrace**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 110 — display modes

```text
Implement only roadmap stage **110 — display modes**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 111 — PrecisionPolicy

```text
Implement only roadmap stage **111 — PrecisionPolicy**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

# Complex numbers

## Prompt 112 — complex value

```text
Introduce complex-number value representation separate from display mode.
```

## Prompt 113 — algebraic representation

```text
Implement only roadmap stage **113 — algebraic representation**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 114 — polar representation

```text
Implement only roadmap stage **114 — polar representation**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 115 — algebraic→polar

```text
Implement only roadmap stage **115 — algebraic→polar**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 116 — polar→algebraic

```text
Implement only roadmap stage **116 — polar→algebraic**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 117 — multiplication trace

```text
Implement only roadmap stage **117 — multiplication trace**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 118 — division trace

```text
Implement only roadmap stage **118 — division trace**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 119 — addition

```text
Implement only roadmap stage **119 — addition**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 120 — subtraction

```text
Implement only roadmap stage **120 — subtraction**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 121 — output modes

```text
Implement only roadmap stage **121 — output modes**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 122 — edge-case suite

```text
Implement only roadmap stage **122 — edge-case suite**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

# Plots

## Prompt 123 — plot region semantics

```text
Recognize plot regions and preserve semantics/metadata; do not reconstruct Excel charts yet.
```

## Prompt 124 — PlotIR metadata

```text
Implement only roadmap stage **124 — PlotIR metadata**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 125 — preview extraction

```text
Implement only roadmap stage **125 — preview extraction**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 126 — plot image→DOCX

```text
Implement only roadmap stage **126 — plot image→DOCX**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 127 — plot fallback

```text
Implement only roadmap stage **127 — plot fallback**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

# ChartIR/Excel future

## Prompt 128 — ChartIR

```text
Introduce experimental ChartIR only; Excel export remains future/feature-gated.
```

## Prompt 129 — series expression extraction

```text
Implement only roadmap stage **129 — series expression extraction**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 130 — reconstruction fixtures

```text
Implement only roadmap stage **130 — reconstruction fixtures**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 131 — ChartExporter

```text
Implement only roadmap stage **131 — ChartExporter**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 132 — experimental Excel chart POC

```text
Implement only roadmap stage **132 — experimental Excel chart POC**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

# Diagrams/Visio future

## Prompt 133 — diagram detection

```text
Recognize diagram/graphics regions when format evidence supports it; do not infer semantics that are not present.
```

## Prompt 134 — diagram preview→DOCX

```text
Implement only roadmap stage **134 — diagram preview→DOCX**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 135 — DiagramIR

```text
Introduce DiagramIR skeleton with shapes/connectors/groups/text/styles/coordinates/bounds. No Visio exporter yet.
```

## Prompt 136 — shape-forensics report

```text
Implement only roadmap stage **136 — shape-forensics report**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 137 — primitive shapes

```text
Implement only roadmap stage **137 — primitive shapes**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 138 — connector graph

```text
Implement only roadmap stage **138 — connector graph**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 139 — grouping

```text
Implement only roadmap stage **139 — grouping**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 140 — DiagramExporter

```text
Implement only roadmap stage **140 — DiagramExporter**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 141 — VSDX POC

```text
Create a standalone editable VSDX proof of concept from synthetic shapes, not Mathcad input.
```

## Prompt 142 — DiagramIR→VSDX POC

```text
Implement only roadmap stage **142 — DiagramIR→VSDX POC**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

# Unified pipeline

## Prompt 143 — ConversionPipeline

```text
Compose the existing stages into one ConversionPipeline without introducing HTTP.
```

## Prompt 144 — DiagnosticsCollector

```text
Implement only roadmap stage **144 — DiagnosticsCollector**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 145 — severity model

```text
Implement only roadmap stage **145 — severity model**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 146 — ConversionReport

```text
Implement only roadmap stage **146 — ConversionReport**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 147 — partial conversion

```text
Implement only roadmap stage **147 — partial conversion**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

# CLI

## Prompt 148 — minimal convert command

```text
Create a CLI that calls the shared conversion core rather than duplicating parser logic.
```

## Prompt 149 — inspect command

```text
Implement only roadmap stage **149 — inspect command**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 150 — --format

```text
Implement only roadmap stage **150 — --format**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 151 — --complex-mode

```text
Implement only roadmap stage **151 — --complex-mode**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 152 — --precision

```text
Implement only roadmap stage **152 — --precision**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 153 — JSON report

```text
Implement only roadmap stage **153 — JSON report**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

# Web UI

## Prompt 154 — Next.js shell

```text
Create the Next.js application shell. If DESIGN.md is empty, use minimal neutral structure only.
```

## Prompt 155 — design compliance checklist

```text
Implement only roadmap stage **155 — design compliance checklist**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 156 — dropzone

```text
Implement only roadmap stage **156 — dropzone**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 157 — file validation UI

```text
Implement only roadmap stage **157 — file validation UI**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 158 — conversion settings UI

```text
Implement only roadmap stage **158 — conversion settings UI**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 159 — conversion states

```text
Implement only roadmap stage **159 — conversion states**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 160 — converter Error Boundary

```text
Implement only roadmap stage **160 — converter Error Boundary**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 161 — localized error mapping

```text
Implement only roadmap stage **161 — localized error mapping**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

# i18n

## Prompt 162 — next-intl infrastructure

```text
Implement only roadmap stage **162 — next-intl infrastructure**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 163 — catalog split

```text
Implement only roadmap stage **163 — catalog split**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 164 — missing-key CI

```text
Implement only roadmap stage **164 — missing-key CI**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 165 — second locale

```text
Implement only roadmap stage **165 — second locale**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

# FastAPI backend

## Prompt 166 — FastAPI skeleton with uv

```text
Create a FastAPI skeleton using uv; no conversion endpoint yet.
```

## Prompt 167 — health endpoint

```text
Implement only roadmap stage **167 — health endpoint**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 168 — PostgreSQL

```text
Implement only roadmap stage **168 — PostgreSQL**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 169 — Alembic

```text
Implement only roadmap stage **169 — Alembic**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 170 — API error model

```text
Implement only roadmap stage **170 — API error model**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 171 — request ID

```text
Implement only roadmap stage **171 — request ID**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 172 — /api/v1 versioning

```text
Implement only roadmap stage **172 — /api/v1 versioning**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

# Authentication

## Prompt 173 — auth integration boundary

```text
Create the authentication integration boundary around the chosen identity provider; do not write custom password crypto unnecessarily.
```

## Prompt 174 — registration

```text
Implement only roadmap stage **174 — registration**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 175 — password login

```text
Implement only roadmap stage **175 — password login**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 176 — email verification

```text
Implement only roadmap stage **176 — email verification**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 177 — OIDC provider 1

```text
Implement only roadmap stage **177 — OIDC provider 1**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 178 — OIDC provider 2

```text
Implement only roadmap stage **178 — OIDC provider 2**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 179 — TOTP

```text
Implement only roadmap stage **179 — TOTP**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 180 — recovery codes

```text
Implement only roadmap stage **180 — recovery codes**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 181 — passkeys

```text
Implement only roadmap stage **181 — passkeys**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 182 — email recovery

```text
Implement only roadmap stage **182 — email recovery**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 183 — phone recovery

```text
Implement only roadmap stage **183 — phone recovery**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 184 — Telegram linking

```text
Implement Telegram account linking only through one-time confirmed association; not recovery yet.
```

## Prompt 185 — Telegram recovery

```text
Add Telegram recovery only for previously linked accounts with expiry, replay protection, rate limiting and audit events.
```

## Prompt 186 — auth abuse suite

```text
Implement only roadmap stage **186 — auth abuse suite**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

# Profile

## Prompt 187 — basic profile

```text
Implement only roadmap stage **187 — basic profile**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 188 — conversion preferences

```text
Implement only roadmap stage **188 — conversion preferences**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 189 — save preferences

```text
Implement only roadmap stage **189 — save preferences**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 190 — conversion history metadata

```text
Implement only roadmap stage **190 — conversion history metadata**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

# API keys

## Prompt 191 — API-key model

```text
Create API-key model with secure lifecycle metadata; never store plaintext secret.
```

## Prompt 192 — secure generation

```text
Implement only roadmap stage **192 — secure generation**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 193 — show secret once

```text
Implement only roadmap stage **193 — show secret once**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 194 — hashed storage

```text
Implement only roadmap stage **194 — hashed storage**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 195 — revoke

```text
Implement only roadmap stage **195 — revoke**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 196 — expiry

```text
Implement only roadmap stage **196 — expiry**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 197 — scopes

```text
Implement only roadmap stage **197 — scopes**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 198 — rate limiting

```text
Implement only roadmap stage **198 — rate limiting**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 199 — usage statistics

```text
Implement only roadmap stage **199 — usage statistics**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

# Public API

## Prompt 200 — POST conversions

```text
Implement the first versioned conversion endpoint using the shared ConversionPipeline.
```

## Prompt 201 — async jobs

```text
Implement only roadmap stage **201 — async jobs**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 202 — status

```text
Implement only roadmap stage **202 — status**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 203 — result

```text
Implement only roadmap stage **203 — result**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 204 — save preference integration

```text
Implement only roadmap stage **204 — save preference integration**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 205 — OpenAPI descriptions

```text
Implement only roadmap stage **205 — OpenAPI descriptions**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 206 — API docs page

```text
Implement only roadmap stage **206 — API docs page**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 207 — curl examples

```text
Implement only roadmap stage **207 — curl examples**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 208 — JavaScript example

```text
Implement only roadmap stage **208 — JavaScript example**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 209 — Python example

```text
Implement only roadmap stage **209 — Python example**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

# RabbitMQ/workers

## Prompt 210 — RabbitMQ broker

```text
Add RabbitMQ as broker only; keep conversion core independent from Celery.
```

## Prompt 211 — Celery test worker

```text
Implement only roadmap stage **211 — Celery test worker**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 212 — conversion task

```text
Implement only roadmap stage **212 — conversion task**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 213 — retry policy

```text
Implement only roadmap stage **213 — retry policy**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 214 — dead-letter strategy

```text
Implement only roadmap stage **214 — dead-letter strategy**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 215 — timeout

```text
Implement only roadmap stage **215 — timeout**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 216 — cancellation model

```text
Implement only roadmap stage **216 — cancellation model**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 217 — idempotency

```text
Implement only roadmap stage **217 — idempotency**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

# Privacy/zero knowledge

## Prompt 218 — threat model

```text
Write the project threat model before adding more cryptography.
```

## Prompt 219 — privacy ADR

```text
Implement only roadmap stage **219 — privacy ADR**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 220 — local WASM POC

```text
Implement only roadmap stage **220 — local WASM POC**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 221 — WASM parser/core

```text
Implement only roadmap stage **221 — WASM parser/core**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 222 — browser conversion POC

```text
Implement only roadmap stage **222 — browser conversion POC**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 223 — Web Worker

```text
Implement only roadmap stage **223 — Web Worker**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 224 — client-side encryption

```text
Implement client-side authenticated encryption according to SECURITY/PRIVACY; do not invent a custom crypto primitive.
```

## Prompt 225 — encrypted storage

```text
Implement only roadmap stage **225 — encrypted storage**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 226 — privacy marker regression

```text
Add a privacy regression marker proving plaintext does not appear in DB/object storage/logs/error tracking for encrypted-storage mode.
```

## Prompt 227 — metadata minimization

```text
Implement only roadmap stage **227 — metadata minimization**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 228 — filename protection

```text
Implement only roadmap stage **228 — filename protection**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 229 — account vs key recovery

```text
Implement only roadmap stage **229 — account vs key recovery**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 230 — Document Recovery Key

```text
Implement only roadmap stage **230 — Document Recovery Key**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 231 — explicit support sharing

```text
Implement only roadmap stage **231 — explicit support sharing**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 232 — privacy claims checklist

```text
Implement only roadmap stage **232 — privacy claims checklist**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

# Security

## Prompt 233 — file size limits

```text
Enforce authoritative backend file-size limits and matching UX limits.
```

## Prompt 234 — XML attack defenses

```text
Implement only roadmap stage **234 — XML attack defenses**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 235 — ZIP bomb tests

```text
Implement only roadmap stage **235 — ZIP bomb tests**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 236 — malformed corpus

```text
Implement only roadmap stage **236 — malformed corpus**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 237 — cargo-fuzz target

```text
Implement only roadmap stage **237 — cargo-fuzz target**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 238 — auth rate limits

```text
Implement only roadmap stage **238 — auth rate limits**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 239 — API rate limits

```text
Implement only roadmap stage **239 — API rate limits**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 240 — security headers

```text
Implement only roadmap stage **240 — security headers**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 241 — CSP

```text
Implement only roadmap stage **241 — CSP**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 242 — dependency audit

```text
Implement only roadmap stage **242 — dependency audit**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 243 — secret scanning

```text
Implement only roadmap stage **243 — secret scanning**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 244 — security regression suite

```text
Implement only roadmap stage **244 — security regression suite**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

# Billing

## Prompt 245 — BillingProvider

```text
Create BillingProvider abstraction without coupling core to a concrete provider.
```

## Prompt 246 — plans model

```text
Implement only roadmap stage **246 — plans model**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 247 — quota engine

```text
Implement only roadmap stage **247 — quota engine**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 248 — Free quota

```text
Implement only roadmap stage **248 — Free quota**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 249 — Pro quota

```text
Implement only roadmap stage **249 — Pro quota**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 250 — API accounting

```text
Implement only roadmap stage **250 — API accounting**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 251 — provider sandbox

```text
Implement only roadmap stage **251 — provider sandbox**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 252 — webhook verification

```text
Implement only roadmap stage **252 — webhook verification**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 253 — subscription lifecycle

```text
Implement only roadmap stage **253 — subscription lifecycle**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 254 — failed payment behavior

```text
Implement only roadmap stage **254 — failed payment behavior**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 255 — refund workflow

```text
Implement only roadmap stage **255 — refund workflow**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 256 — billing history

```text
Implement only roadmap stage **256 — billing history**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

# Admin

## Prompt 257 — admin auth boundary

```text
Create a separate admin authentication boundary; normal users cannot access admin surfaces.
```

## Prompt 258 — role/permission model

```text
Implement only roadmap stage **258 — role/permission model**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 259 — user list metadata

```text
Implement only roadmap stage **259 — user list metadata**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 260 — conversion stats

```text
Implement only roadmap stage **260 — conversion stats**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 261 — worker health

```text
Implement only roadmap stage **261 — worker health**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 262 — queue stats

```text
Implement only roadmap stage **262 — queue stats**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 263 — parser error stats

```text
Implement only roadmap stage **263 — parser error stats**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 264 — unsupported-node stats

```text
Implement only roadmap stage **264 — unsupported-node stats**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 265 — API usage metrics

```text
Implement only roadmap stage **265 — API usage metrics**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 266 — billing admin

```text
Implement only roadmap stage **266 — billing admin**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 267 — feature flags

```text
Implement only roadmap stage **267 — feature flags**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 268 — language metadata

```text
Implement only roadmap stage **268 — language metadata**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 269 — security event viewer

```text
Implement only roadmap stage **269 — security event viewer**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 270 — admin privacy integration test

```text
Prove through integration tests that admin cannot retrieve privacy-protected plaintext, encryption keys, passwords or full API secrets.
```

# Observability

## Prompt 271 — structured logging

```text
Introduce structured logging with safe fields only.
```

## Prompt 272 — log redaction

```text
Implement only roadmap stage **272 — log redaction**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 273 — OpenTelemetry traces

```text
Implement only roadmap stage **273 — OpenTelemetry traces**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 274 — metrics

```text
Implement only roadmap stage **274 — metrics**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 275 — conversion duration

```text
Implement only roadmap stage **275 — conversion duration**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 276 — parser failure metrics

```text
Implement only roadmap stage **276 — parser failure metrics**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 277 — worker metrics

```text
Implement only roadmap stage **277 — worker metrics**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 278 — Sentry with sanitization

```text
Implement only roadmap stage **278 — Sentry with sanitization**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

# Performance

## Prompt 279 — benchmark fixture classes

```text
Implement only roadmap stage **279 — benchmark fixture classes**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 280 — parser benchmark

```text
Implement only roadmap stage **280 — parser benchmark**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 281 — DOCX benchmark

```text
Implement only roadmap stage **281 — DOCX benchmark**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 282 — memory benchmark

```text
Implement only roadmap stage **282 — memory benchmark**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 283 — regression thresholds

```text
Implement only roadmap stage **283 — regression thresholds**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 284 — parallel conversion benchmark

```text
Implement only roadmap stage **284 — parallel conversion benchmark**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

# CI/CD

## Prompt 285 — lint CI

```text
Add the first CI lint job without duplicating the global AI Dev Team release workflow.
```

## Prompt 286 — Rust tests CI

```text
Implement only roadmap stage **286 — Rust tests CI**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 287 — Python tests CI

```text
Implement only roadmap stage **287 — Python tests CI**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 288 — frontend tests CI

```text
Implement only roadmap stage **288 — frontend tests CI**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 289 — integration CI

```text
Implement only roadmap stage **289 — integration CI**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 290 — fixture regression CI

```text
Implement only roadmap stage **290 — fixture regression CI**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 291 — security CI

```text
Implement only roadmap stage **291 — security CI**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 292 — artifact validation

```text
Implement only roadmap stage **292 — artifact validation**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 293 — Docker build

```text
Implement only roadmap stage **293 — Docker build**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 294 — staging deployment

```text
Implement only roadmap stage **294 — staging deployment**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 295 — production release gate

```text
Implement only roadmap stage **295 — production release gate**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

# Scaling

## Prompt 296 — Docker Compose baseline

```text
Create a production-like Docker Compose baseline with health checks, but do not introduce Kubernetes yet.
```

## Prompt 297 — stateless API verification

```text
Implement only roadmap stage **297 — stateless API verification**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 298 — multiple workers

```text
Implement only roadmap stage **298 — multiple workers**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 299 — multiple API replicas

```text
Implement only roadmap stage **299 — multiple API replicas**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 300 — load test

```text
Implement only roadmap stage **300 — load test**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 301 — storage abstraction

```text
Implement only roadmap stage **301 — storage abstraction**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 302 — benchmark-driven Redis cache

```text
Implement only roadmap stage **302 — benchmark-driven Redis cache**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

## Prompt 303 — Kubernetes POC

```text
Create Kubernetes only as a proof of concept after validating current deployment assumptions; do not replace the normal deployment path.
```

## Prompt 304 — worker autoscaling POC

```text
Implement only roadmap stage **304 — worker autoscaling POC**. Preserve existing architecture, add the smallest correct implementation, targeted tests, one relevant negative/boundary case, and update only affected documentation. Do not implement later-stage functionality.
```

# Reusable review prompts

## Stage quality review

```text
Review only the just-completed stage. Do not add features. Check SPECIFICATION/ARCHITECTURE scope, tests and negative cases, security/privacy regression, DESIGN compliance for UI, and PROGRESS/DECISIONS updates. Return PASS or FAIL with concrete findings. Prefer existing global AI Dev Team reviewer roles; do not run duplicate generic reviewers.
```

## Fix review findings

```text
Fix only the findings from the latest stage review. Add regression coverage for every corrected bug, rerun relevant checks, and update PROGRESS.md. Do not broaden scope.
```

## Next-stage readiness

```text
Do not implement the next stage. Check that the previous stage is PASS, targeted tests are green, blockers are documented, required fixtures/dependencies exist, and repository state is understandable. Return READY or NOT READY with reasons.
```
