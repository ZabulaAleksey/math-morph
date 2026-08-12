---
name: mathcad-format-forensics
description: Analyze an unfamiliar XMCD/MCDX fixture or Mathcad version, identify structure/unsupported nodes, and produce a compatibility report without changing production parser code.
---

1. Read root and parser AGENTS.
2. Inspect only the target fixture and minimal parser code needed for comparison.
3. Identify format/container, version/schema/namespaces, region types and unknown nodes.
4. Classify findings: already supported / partially supported / unknown / malformed.
5. Note security-relevant container/XML anomalies without executing embedded content.
6. Return a concise report with evidence and recommended fixtures/tests.
7. Do not modify production code or golden outputs.
