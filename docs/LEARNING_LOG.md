# Учебный журнал

## 2026-08-14 — Безопасная граница Mathcad input

### Что и зачем изменено

- Введены content-based XMCD/MCDX detection, bounded ZIP inspection и ограниченное чтение XML root metadata.
- Fixture corpus получил versioned manifest и fail-closed validator, чтобы regression evidence оставался воспроизводимым.
- ZIP и XML рассматриваются как недоверенный ввод; содержательное чтение worksheet оставлено этапу 027.

### Ключевой поток данных / управления

1. `FormatDetector` определяет формат по байтам, а расширение использует только для диагностического сравнения.
2. `SafeMcdxReader` проверяет central/local ZIP metadata, имена, collisions, лимиты и фактический распакованный размер без записи на диск.
3. XML inspector принимает только UTF-8, запрещает DTD/entities и возвращает ограниченный root metadata envelope.

### Команды и проверки

```text
python -B scripts/validate_project.py
python -B scripts/validate_fixtures.py
python -B -m unittest discover -s tests -p "test_*.py" -v
cargo fmt --all -- --check
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo audit
```

### Решения и trade-offs

- Поддерживаемая `zip 8.6.0` потребовала Rust 1.88; это предпочтено неподдерживаемой archive dependency и собственной ZIP-реализации.
- Manifest инвентаризирует parts, но не извлекает их и не доверяет CRC32 как признаку целостности.
- Schema URI сохраняются только как metadata; сетевые обращения отсутствуют.

### Проблемы и способы исправления

- Review выявил drive-relative ZIP path, unchecked offset arithmetic, неполную XML attribute validation и неточный namespace limit error mapping.
- Все случаи закрыты fail-closed проверками и отдельными regression-тестами.

### Как повторить самостоятельно

1. Запустить оба Python validator и unit-тесты из корня репозитория.
2. Выполнить Rust format/test/clippy на toolchain 1.88 с `--locked`.
3. Запустить `cargo audit` по актуальной RustSec advisory DB.
4. Сопоставить этапы 002–026 с verified-строками в `docs/TRACEABILITY.md`.
