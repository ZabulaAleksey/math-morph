# Статус проекта

## Снимок состояния

- **Статус:** этапы 001–026 проверены; этапы 027–051 выполняются на ветке `feature/stages-027-051`.
- **Текущий этап:** specification/architecture для worksheet parser и Math AST завершены; реализация ещё не отмечена выполненной.
- **В работе:** этапы 027–051 по `specs/features/worksheet-structure-and-ast.spec.md`.
- **Blockers:** нет. Официальный контракт legacy XML подтверждён локальными XSD Mathcad 15.

## Уже реализовано и проверено

- Воспроизводимый Cargo/uv/pnpm monorepo и минимальные Rust/Python/Next.js каркасы.
- Project overlay, canonical docs и fail-closed project/fixture validators.
- Versioned synthetic fixture corpus.
- Content-based XMCD/MCDX detection и `FILE_EXTENSION_MISMATCH`.
- Bounded ZIP inspection без filesystem extraction, безопасная path policy и ordered container manifest.
- UTF-8 XML root-envelope inspection namespaces/schema metadata с запретом DTD/entities.
- Rust 1.88, `zip = 8.6.0`, `quick-xml = 0.41.0`; предыдущий security/code review прошёл.

## Принятый контракт следующего блока

- Поддерживаемое legacy XML подмножество: worksheet30 3.0.3 + math30 3.0.2.
- Table/program/vector трактуются по реальной XSD-модели, а не по неточной короткой формулировке ROADMAP.
- Stages 027–051 заканчиваются structural comparisons. Boolean AST/evaluation, units, generic `UnsupportedNode`, IR/export/API/UI не входят.

## Известные ограничения

- Совместимость с реальными вариантами подтверждается постепенно легально доступными образцами; в Git хранятся только synthetic fixtures.
- MCDX container инспектируется безопасно, но его внутренний Prime worksheet ещё не имеет подтверждённого content schema contract.
- API и web остаются каркасами; приложение пока не предоставляет пользовательский conversion flow.
- На Windows Rust MSVC требует Visual Studio Build Tools с workload `Desktop development with C++` и запуск из Developer PowerShell либо окружения с доступным `link.exe`.

## Проверки последнего verified блока

- Python validators и unit tests: PASS, 14/14.
- Rust 1.88: format, 17/17 integration tests и Clippy `-D warnings`: PASS.
- `cargo-audit 0.22.2`: уязвимости в 23 locked dependencies не найдены.

## Следующие действия

1. Реализовать и проверить worksheet block 027–035.
2. Реализовать AST блоками 036–037, 038–044 и 045–051.
3. Провести limit/security regression suite и независимые reviews.
4. Только после доказательств обновить stages 027–051 до `verified` в `TRACEABILITY.md`.
