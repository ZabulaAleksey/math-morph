# Трассировка требований

## Назначение

Поддерживать компактный путь от утверждённых требований продукта к этапам дорожной карты, реализации и тестам без создания второй спецификации.

Канонические источники:

- требования продукта: `specs/system.spec.md`;
- порядок реализации и стабильные номера этапов: `docs/ROADMAP.md`;
- исполняемые части этапов: соответствующий раздел `docs/PROMPTS.md`;
- архитектурные ограничения и решения: `docs/ARCHITECTURE.md` и `docs/DECISIONS.md`;
- доказательства проверки: закоммиченный код, тесты, fixtures и результаты review.

Текущие ссылки уровня продукта используют стабильные номера разделов спецификации, например `SPEC-02`. Новые предметные или функциональные specs могут добавлять ID `REQ-*`, если трассировка на уровне разделов недостаточно точна; такие ID должны ссылаться на каноническое требование продукта.

## Исходное соответствие

Все этапы имеют статус `planned` до появления реализации и доказательств проверки.

| Область требований | Этапы дорожной карты | Ожидаемая реализация / доказательства проверки | Статус |
|---|---:|---|---|
| NFR-FOUNDATION-001/002 | 001 | workspace manifests/lockfiles, пустые каркасы, `scripts/validate_project.py`, `tests/test_validate_project.py`, сборки Python/Next.js и `cargo check --workspace --locked` | verified |
| NFR-CONTEXT-001/002 | 002–010 | канонические документы, project overlay и модульные `AGENTS.md`; `scripts/validate_project.py`, `tests/test_validate_project.py` | verified |
| SPEC-02 fixture corpus | 011–014 | `tests/fixtures/manifest.json`, taxonomy directories, `scripts/validate_fixtures.py`, `tests/test_validate_fixtures.py` | verified |
| SPEC-02 определение формата | 015–018 | `crates/mathcad-parser/src/format.rs`, `diagnostic.rs`, detector/mismatch tests в `tests/input_boundary.rs` | verified |
| SPEC-02 безопасный MCDX container | 019–025 | `crates/mathcad-parser/src/mcdx.rs`, path/collision/limits/manifest tests в `tests/input_boundary.rs` | verified |
| SPEC-02 XML root metadata | 026 | `crates/mathcad-parser/src/xml_metadata.rs`, UTF-8/DTD/namespace/schema/limits tests в `tests/input_boundary.rs` | verified |
| SPEC-02 worksheet structure | 027–035 | worksheet metadata, regions, ordering, version compatibility и regression coverage | planned |
| SPEC-04 архитектура конвертации | 036–061, 143–147 | AST, семантические границы, Document IR, тесты контракта конвейера | planned |
| SPEC-05 экспорт редактируемых формул | 062–094 | `crates/exporter-docx/`, тесты OMML и пакета, smoke-эталоны Word | planned |
| SPEC-06 преобразования и точность | 095–111 | `crates/math-engine/`, тесты сохранения семантики и trace | planned |
| SPEC-07 комплексные числа | 112–122 | round-trip проверки алгебраического и полярного представления и граничные тесты | planned |
| SPEC-08 графики | 123–132 | PlotIR/ChartIR, fallback предпросмотра и fixtures восстановления | planned |
| SPEC-09 схемы | 133–142 | DiagramIR, растровый fallback и доказательства редактируемого POC VSDX | planned |
| SPEC-13 локальный адаптер CLI | 148–153 | команды общего core `convert` и `inspect` и тесты структурированных отчётов | planned |
| SPEC-10–11 веб-страницы и состояния конвертации | 154–161 | `apps/web/`, тесты компонентов, E2E, доступности и состояний ошибок | planned |
| SPEC-16 интернационализация | 162–165 | внешние каталоги и проверка отсутствующих ключей в CI | planned |
| SPEC-12 аутентификация и восстановление | 173–186 | граница аутентификации, тесты replay, brute force и восстановления | planned |
| SPEC-13 API и ключи API | 166–172, 191–217 | `services/api/`, тесты контракта, авторизации, задач и идемпотентности | planned |
| SPEC-14 сохранённые документы | 187–190, 200–217 | границы метаданных и объектного хранилища, тесты срока хранения и удаления | planned |
| SPEC-15 направление конфиденциальности | 218–232 | ADR конфиденциальности, прототипы WASM и шифрования и тесты границ | planned |
| SPEC-17 оплата и монетизация | 245–256 | абстракция provider, тесты прав и жизненного цикла | planned |
| SPEC-10 граница администрирования и конфиденциальности | 257–270 | RBAC, области метрик и тесты запрета открытого текста для администратора | planned |
| SPEC-18 нефункциональные требования | 233–244, 271–304 | доказательства безопасности, наблюдаемости, benchmarks, CI и масштабирования | planned |
| SPEC-19 критерии успеха MVP | совокупно | end-to-end набор приёмки, охватывающий полный путь MVP | planned |

## Правила обновления

- Обновляй матрицу в том же изменении, которое принимает изменение соответствия требования этапу или проверяет этап.
- Используй только `planned`, `in progress`, `blocked` или `verified`.
- `verified` требует закоммиченной реализации, успешных относящихся к ней тестов и завершённого review; письменного prompt, плана или каркаса недостаточно.
- После появления кода указывай точные пути к коду и тестам. Не придумывай будущие модули и не заявляй о невыполненных проверках.
- Если реализация намеренно меняет архитектурную границу, обнови или добавь ADR вместо незаметного изменения соответствия.
