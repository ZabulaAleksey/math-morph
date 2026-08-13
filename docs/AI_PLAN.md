# Текущий план AI — этапы 002–026

**Статус:** завершён и проверен 2026-08-14.

## Цель

Формально проверить существующий context foundation этапов 002–010 и реализовать ограниченную границу недоверенного Mathcad input этапов 011–026. Содержательное чтение worksheet начинается только с этапа 027 и в этот план не входит.

## Режим и источники

- сложность: `COMPLEX`;
- режим: production;
- SDLC: specification → architecture → implementation → testing → security/review;
- домен/стек: document conversion, hostile XML/ZIP, Rust 1.88;
- SPEC: `specs/system.spec.md` и `specs/features/input-formats-and-containers.spec.md`.

## Порядок

1. **002–010:** расширить project validator каноническими документами и всеми локальными `AGENTS.md`; добавить отрицательные contract tests; провести review и отметить этапы только после доказательства.
2. **011–014:** создать taxonomy, versioned fixture manifest, fail-closed validator и synthetic corrupted/security fixtures.
3. **015–018:** реализовать content-based `InputFormat`/`FormatDetector`, XMCD/MCDX detection и `FILE_EXTENSION_MISMATCH`.
4. **019–025:** реализовать ограниченную ZIP-инспекцию, path policy, лимиты, manifest и классификацию worksheet/resource/unknown.
5. **026:** реализовать namespace/schema root metadata без worksheet parsing.
6. Выполнить format/lint/test/build, project/fixture validators, security review, независимый code review и проверку соответствия SPEC.
7. Обновить `AI_STATUS.md`, `TRACEABILITY.md`, затронутые архитектурные решения и зафиксировать завершённые логические блоки коммитами.

## Контрольные точки

- context contracts — verified;
- fixtures — verified;
- detector — verified;
- safe container — verified;
- XML metadata — verified;
- итоговые security/reviewer verdict — PASS без открытых существенных замечаний.

## Результат

- Этапы 002–010 подтверждены validator и отрицательными contract tests.
- Этапы 011–026 реализованы в пределах `specs/features/input-formats-and-containers.spec.md`; worksheet parsing этапа 027+ не начат.
- Python validators и 14 unit-тестов прошли.
- На Rust 1.88 прошли format, 17 integration tests и Clippy с запретом warnings.
- `Cargo.lock` просканирован `cargo-audit 0.22.2` по актуальной RustSec advisory DB: уязвимости не найдены.
- Code review и security review завершены; найденные обходы path/XML validation и расхождения manifest/typed-error contract закрыты regression-тестами.

## Откат

Каждый логический блок оформляется отдельным коммитом. Миграций данных и внешнего состояния нет; откат выполняется отменой соответствующего коммита. Повышение MSRV и выбор ZIP dependency фиксируются отдельным ADR.
