# Статус проекта

## Снимок состояния

- **Статус:** этапы 001–026 реализованы и проверены на ветке `feature/stages-002-026`.
- **Текущий этап:** этап 026 завершён; ветка готова к пользовательской проверке и merge.
- **В работе:** нет.
- **Blockers:** нет.

## Завершено

- Этап 001: воспроизводимый Cargo/uv/pnpm monorepo и минимальные каркасы Rust, Python API и Next.js.
- Этапы 002–010: project overlay, канонические документы и модульные `AGENTS.md` защищены fail-closed validator и отрицательными contract tests.
- Этапы 011–014: создан versioned fixture manifest, taxonomy и synthetic corpus с отдельным fixture validator.
- Этапы 015–018: реализованы `InputFormat`, content-based `FormatDetector`, XMCD/MCDX detection и `FILE_EXTENSION_MISMATCH`.
- Этапы 019–025: реализована bounded ZIP-инспекция без filesystem extraction, безопасная path policy, лимиты, ordered `ContainerManifest` и классификация частей.
- Этап 026: реализована UTF-8 XML root-envelope инспекция namespace/schema metadata с запретом DTD и внешних сущностей.
- MSRV повышен до Rust 1.88; `zip = 8.6.0` и `quick-xml = 0.41.0` точно зафиксированы по ADR-0006.
- Независимые code review и security review завершены без открытых существенных замечаний.

## Проверки

- `python -B scripts/validate_project.py` — PASS.
- `python -B scripts/validate_fixtures.py` — PASS.
- `python -B -m unittest discover -s tests -p "test_*.py" -v` — PASS, 14/14.
- Rust 1.88: `cargo fmt --all -- --check`, `cargo test --workspace --locked` — PASS, 17/17 integration tests; `cargo clippy --workspace --all-targets --locked -- -D warnings` — PASS.
- `cargo-audit 0.22.2` загрузил актуальную RustSec advisory DB и просканировал 23 зависимости из `Cargo.lock` без найденных уязвимостей.

## Известные ограничения

- Corpus пока содержит открытые synthetic fixtures; совместимость с реальными историческими вариантами Mathcad должна подтверждаться легально доступными образцами по мере их появления.
- Worksheet metadata, regions и содержательное чтение worksheet не реализованы: это область этапа 027 и далее.
- Parser не извлекает ZIP entries на диск и не загружает URI из XML metadata.

## Следующий разумный шаг

После пользовательской проверки и merge приступить только к этапу 027 (`worksheet metadata`) по отдельной SPEC/плану, сохраняя текущую границу недоверенного ввода.

## Завершение этапа

Этап дорожной карты считается проверенным только после ограниченной реализации, относящихся к ней тестов, review и обновления затронутых канонических документов. Текст prompt или дорожной карты сам по себе не является доказательством завершения.
