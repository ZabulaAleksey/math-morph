# Privacy Model

## Principles

- Minimize document/content exposure and retained metadata.
- Never log worksheet text, formulas, decrypted filenames, passwords, tokens, API secrets or encryption keys.
- Client-side processing/encryption is preferred when functionality allows it.
- Marketing/privacy claims must be technically true for the selected trust mode.

## Processing modes

### Local/WASM mode

Browser uses shared Rust core compiled to WebAssembly. The source document can remain on the client for supported operations.

### Server conversion mode

Plaintext may be processed by controlled backend workers. Do not describe this mode as absolute zero-knowledge unless a separate confidential-compute architecture actually provides that property.

## Saved objects

For zero-knowledge-compatible storage, encrypt client-side with authenticated encryption before upload. Server stores ciphertext + minimized metadata and does not possess a usable plaintext recovery key.

## Recovery

Account login recovery and encrypted-document key recovery are distinct. Resetting an account password must not silently grant the server access to encrypted documents.

## Support

Support access to protected content must require explicit, scoped, time-limited user sharing. Admin panels do not include a hidden plaintext bypass.

## Deletion/retention

Retention is policy/plan-configurable. Deletion removes metadata and schedules physical object/temp cleanup. Backups and deletion lag must be documented honestly.
