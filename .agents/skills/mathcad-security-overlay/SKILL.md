---
name: mathcad-security-overlay
description: Add Mathcad-specific OWASP Top 10:2025 checks to an existing AI Dev Team security review, focusing on hostile XMCD/MCDX, conversion privacy and document-processing boundaries.
---

This is an overlay, not a second full security review. Prefer the installed AI Dev Team security workflow for generic application security.

1. Read `docs/SECURITY.md` only for affected Mathcad trust boundaries.
2. Check hostile XML/ZIP/container parsing, file-size/decompression limits and exceptional conditions.
3. Check conversion/report/logging paths for document or formula leakage.
4. Check zero-knowledge/admin boundaries if storage or support-sharing changed.
5. For Mathcad-related dependency/format-library changes, note supply-chain provenance and pinning.
6. Return only Mathcad-specific delta findings not already covered by the global review.
