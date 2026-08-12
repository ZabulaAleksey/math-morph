# Test Rules

- Bug fix → regression test/fixture.
- Do not approve a new golden output merely because implementation changed it.
- Include malformed, boundary and security cases for parser/upload/auth changes.
- Verify observable behavior; avoid brittle tests of internal implementation details.
- Privacy tests must prove sensitive plaintext does not appear in DB/storage/logs when the architecture claims it should not.
- Keep fixture metadata/manifest consistent.
