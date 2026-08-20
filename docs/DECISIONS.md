# Архитектурные решения

## ADR-0001 — Прогрессивная загрузка контекста

**Статус:** принято.

**Решение:** сохранять корневой AGENTS компактным; предметные правила находятся во вложенных AGENTS, Skills и целевых документах.

**Причина:** уменьшить дублирование контекста, сохраняя точные инструкции рядом с затронутым кодом.

## ADR-0002 — Hooks являются защитными ограничителями, а не безопасностью приложения

**Статус:** принято.

**Решение:** hooks обеспечивают только детерминированные проверки workflow разработчика. Безопасность приложения и CI остаётся авторитетной.

## ADR-0003 — Субагенты наследуют родительскую модель, если явно не требуется другое

**Статус:** принято.

**Решение:** проектные TOML-файлы агентов не фиксируют имена моделей. Это предотвращает устаревание конфигурации модели и позволяет политике родителя или сессии выбирать модель.

## ADR-0004 — Трассировка требований без их дублирования

**Статус:** принято.

**Решение:** `docs/TRACEABILITY.md` сопоставляет разделы канонической спецификации этапам дорожной карты и доказательствам проверки. Он не повторяет требования продукта.

**Причина:** будущей реализации нужен стабильный путь от требования к коду и тестам, а дублирующие файлы SPEC создавали бы конфликты контекста.

**Последствия:** обновлять соответствие при изменении принятого требования или проверке этапа. Пункт дорожной карты или prompt сам по себе никогда не доказывает реализацию.

## ADR-0005 — Версионируемый Document IR является канонической границей exporters

**Статус:** принято.

**Контекст:** будущие DOCX, HTML, Markdown, LaTeX, PDF, JSON, Typst, Excel и Visio adapters не должны повторно разбирать Mathcad или зависеть от внутренней XML-модели.

**Решение:** `DocumentIR` включает version marker и сериализуемые page, text, formula, table, image, plot/chart, diagram и metadata structures. Schema evolution имеет явную миграцию либо диагностируемую несовместимость.

**Последствия:** parser и semantics сохраняют source/layout provenance; exporters зависят от IR-контракта. Round-trip и compatibility fixtures обязательны.

**Связанные требования:** SPEC-04, SPEC-08, SPEC-09.

## ADR-0006 — Поддерживаемая ZIP-зависимость важнее сохранения MSRV 1.85

**Статус:** принято 2026-08-14.

**Контекст:** последняя ветка `zip`, совместимая с Rust 1.85, больше не поддерживается upstream. MCDX является границей недоверенного архива.

**Варианты:** остаться на `zip 7.3.0`; реализовать ZIP самостоятельно; отложить MCDX; повысить MSRV и использовать поддерживаемую ветку.

**Решение:** повысить workspace MSRV до Rust 1.88 и точно зафиксировать `zip = 8.6.0` с отключёнными default features и только Deflate backend. `quick-xml = 0.41.0` также фиксируется точно.

**Причина:** неподдерживаемая archive dependency или собственная ZIP-реализация создают больший security/supply-chain риск, чем контролируемое повышение MSRV.

**Последствия:** среда Rust должна быть не ниже 1.88; `Cargo.lock` обязателен; dependency review повторяется при обновлении.

**Fallback / rollback:** отменить MCDX-реализацию либо отдельно пересмотреть ADR; тихий downgrade на неподдерживаемую ветку запрещён.

**Проверка:** locked build/test на Rust 1.88 и security review A03/A05/A06/A10.

**Связанные требования:** FR-ZIP-001..007, раздел 10 feature-SPEC входных форматов.

## ADR-0007 — Worksheet parser следует подтверждённому XSD-подмножеству

**Статус:** принято 2026-08-14.

**Контекст:** этапы 027–051 требуют содержательного чтения legacy XMCD, но короткие формулировки ROADMAP не задают QName, arity и реальную вложенность table/program/vector. Попытка вывести их из названий этапов создала бы несовместимый выдуманный формат.

**Варианты:** угадывать XML mapping по дорожной карте; копировать vendor XSD и выполнять runtime validation; реализовать ограниченный streaming parser по подтверждённым XSD-формам с synthetic fixtures.

**Решение:** поддержать явно заявленное подмножество `worksheet30` 3.0.3 и `math30` 3.0.2, сверенное с официальной локальной установкой Mathcad 15. Сравнивать expanded QName, хранить source spans/opaque fragments, строго проверять arity и limits. Vendor XSD и содержимое официальных worksheet в репозиторий не копировать; runtime XSD resolver не добавлять.

`ws:resultFormat/ws:table` остаётся opaque table reference, `ml:program` — неподдержанное math expression, vector — семантическая специализация `ml:matrix`. Prime MCDX worksheet parsing не заявляется без отдельного schema contract.

**Причина:** подход даёт проверяемую совместимость и безопасную границу без copyright, filesystem resolver, case-sensitivity и supply-chain рисков runtime XSD validation.

**Последствия:** новые версии/namespaces подключаются через явный dispatch и новые fixtures; неизвестные nodes сохраняются source-backed и диагностируются. Полная XSD-validity не обещается, проверяется только стабильный продуктовый контракт.

**Fallback / rollback:** отключить содержательный parser для неподтверждённой версии, сохранив уже проверенную format/container boundary этапов 015–026.

**Проверка:** `AC-027..051`, negative namespace/version/arity/limit tests, security review и отсутствие runtime network/filesystem schema resolution.

**Связанные требования:** FR-WS-001..004, FR-REG-001..005, FR-AST-001..016, NFR-PARSE-001..003.

## ADR-0008 — AST и Document IR получают нейтральных владельцев

**Статус:** принято 2026-08-14.

**Контекст:** `mathcad-parser` уже владеет source AST, а этап 055 требует общий сериализуемый IR. Размещение IR в `math-engine` связало бы exporters с будущим evaluator; размещение в `exporter-docx` сделало бы Word владельцем общего контракта.

**Варианты:** оставить AST в parser и дублировать formula model; поместить IR в `math-engine`; поместить IR в exporter; выделить нейтральные crates.

**Решение:** добавить `math-model` для source-neutral AST и `document-ir` для wire schema/export ports. `mathcad-parser` сохраняет совместимые re-export; `exporter-docx` не зависит от parser.

**Причина:** единая formula model не дублируется, а dependency DAG сохраняет границы XML, semantics и Word.

**Последствия:** workspace и validator получают два новых crate; AST-типы становятся сериализуемыми, но `Debug` остаётся redacted.

**Fallback / rollback:** вернуть re-exported AST в parser до публикации внешнего API; не переносить IR в exporter/evaluator как shortcut.

**Проверка:** workspace dependency inspection, compile tests без циклов и AC-052–061.

**Связанные требования:** FR-AST2-001..007, FR-IR-001..007, ADR-0005.

## ADR-0009 — Document IR V1 использует строгий JSON и integer micrometres

**Статус:** принято 2026-08-14.

**Контекст:** exporter boundary должна быть сохраняемой и совместимой, а layout с `f64` допускает NaN, platform-dependent rounding и неоднозначные единицы. Binary assets нельзя встраивать в основной IR без раздувания и утечки путей.

**Варианты:** неверсированный in-memory Rust API; JSON с floating-point layout и bytes; строгий versioned envelope с asset references.

**Решение:** V1 сериализуется в bounded UTF-8 JSON с `schema_version = 1`, exact field set и integer micrometres. Изображения представлены `AssetRefIr`; bytes предоставляет отдельный `AssetResolver`.

**Причина:** wire contract детерминирован, проверяем и независим от filesystem/Word; физические conversions используют checked integer arithmetic.

**Последствия:** unknown version/field требует явной migration; сериализация содержимого остаётся сознательной операцией, а не способом логирования.

**Fallback / rollback:** V1 reader сохраняется после появления artifacts; новая несовместимая модель получает V2.

**Проверка:** golden/round-trip/version/geometry/privacy tests и AC-055–061.

**Связанные требования:** FR-IR-001..007, ADR-0005.

## ADR-0010 — DOCX exporter производит bounded deterministic OPC subset

**Статус:** принято 2026-08-14.

**Контекст:** ручная генерация произвольного OOXML создаёт риск broken relationships, active content и недетерминированных artifacts. Полный универсальный DOCX validator не входит в текущий этап.

**Варианты:** Office COM/Open XML SDK runtime; сторонний high-level DOCX stack; ограниченный Rust writer/validator на закреплённых ZIP/XML dependencies.

**Решение:** `exporter-docx` генерирует только заявленный Transitional OPC/WordprocessingML/OMML subset, только internal relationships и проверенные PNG/JPEG. Package/XML writer и validator ограничены ресурсными лимитами и используют детерминированные IDs/order/timestamps.

**Причина:** core остаётся кроссплатформенным, воспроизводимым и не зависит от установленного Office или внешнего schema resolver.

**Последствия:** неподдерживаемые sections/assets/equations дают typed error; validator гарантирует только generated subset и не рекламируется как универсальная очистка чужих DOCX.

**Fallback / rollback:** отключить соответствующий exporter layer, сохранив Document IR и parser; не переходить на внешний runtime без отдельного dependency/security ADR.

**Проверка:** AC-062–076, deterministic bytes, negative OPC/XML/image tests и security review.

**Связанные требования:** FR-DOCX-001..006, FR-OMML-001..003, SPEC-05.

## ADR-0011 — Расширенный OMML остаётся каноническим Word backend

**Статус:** принято 2026-08-14.

**Контекст:** этапы 077–089 добавляют редактируемые powers, roots, scripts, function calls, grouping, vector/matrix и calculus shapes, а публичному DOCX API нужен явный выбор backend. MathType/MathML/OLE в текущем scope не реализуются.

**Варианты:** продолжить неявный выбор exporter; сделать `WordOmml` и резервный MathType fallback; ввести typed backend config с fail-closed для недоступных backend.

**Решение:** `EquationBackend::WordOmml` является default через `DocxExportConfig`. `WordEquationExporter` строит только bounded canonical OMML и делит equation byte/node/depth quotas с `DocxValidator`. `EquationBackend::MathType` остаётся зарезервированным и возвращает `EquationBackendUnavailable` без текстового, MathML или OLE fallback.

**Причина:** сохраняется редактируемая семантика Word, deterministic output и явная граница доверия; недоступность backend не скрывает потерю структуры.

**Последствия:** новые формы проходят snapshot/negative tests и Word/Open XML SDK evidence; MathML/MathType потребуют отдельной SPEC, dependency и compatibility review начиная с этапа 090.

**Fallback / rollback:** выбрать `WordOmml` или отклонить export typed error; не добавлять скрытую картинку или внешний backend.

**Проверка:** `cargo test --workspace --locked` (92 Rust tests), `cargo clippy --workspace --all-targets --locked -- -D warnings`, structural validator, Word 16.0 open/edit smoke, Open XML SDK 2.5.4728 (0 errors), independent review и security review.

**Связанные требования:** AC-077..AC-089, SPEC-05.

## ADR-0012 — MathML является отдельным backend-neutral exporter

**Статус:** принято 2026-08-15.

**Контекст:** этап 090 требует Presentation MathML, но подключение MathType к DOCX относится к более позднему экспериментальному adapter. Размещение renderer внутри `exporter-docx` связало бы стандартный MathML с Word/OPC и сделало зарезервированный backend доступным преждевременно.

**Варианты:** добавить MathML в `exporter-docx`; встроить renderer в `document-ir`; создать отдельный exporter crate.

**Решение:** `exporter-mathml` зависит только от `math-model`, `document-ir` и закреплённого `thiserror`, реализует общий `EquationExporter` и выдаёт opaque bounded `MathMlFragment`. Этап 090 ограничен принятым scalar Presentation MathML Core subset; `EquationBackend::MathType` остаётся unavailable.

**Причина:** стандартный output format не зависит от Word, parser или будущего proprietary adapter; exact shapes, XML trust boundary и resource budgets тестируются самостоятельно.

**Последствия:** расширение AST coverage требует явного изменения SPEC; snapshot corpus и MathType-specific normalization остаются этапами 091–093. Renderer принимает уже bounded borrowed AST, а lifecycle caller-owned recursive структуры не входит в его ownership boundary.

**Fallback / rollback:** удалить новый crate и workspace registration; существующие Document IR и DOCX/OMML contracts не изменяются.

**Проверка:** AC-090-001..006, workspace tests/Clippy, project validator, independent review и security review.

**Связанные требования:** FR-MATHML-001..004, NFR-MATHML-001..002, SEC-MATHML-001.

## ADR-0013 — MathType adapter остаётся pure payload boundary

**Статус:** принято 2026-08-17.

**Контекст:** этап 092 должен подготовить будущую интеграцию MathType, но прямое подключение proprietary SDK, OLE/COM или DOCX backend одновременно добавило бы platform/license/runtime trust boundary и преждевременно реализовало этапы 093–094. `exporter-mathml` уже создаёт bounded allowlist Presentation MathML.

**Варианты:** вызвать native MathType SDK; встроить raw MathML/OLE непосредственно в DOCX; добавить сетевой WIRIS service; создать отдельный pure adapter над production MathML renderer.

**Решение:** добавить `exporter-mathtype`, который принимает `MathExpression`, делегирует `MathMlRenderer` и возвращает opaque `MathTypePayload` с media type `application/mathml+xml`. Публичного raw XML constructor, network/filesystem/process/registry access и proprietary dependency нет. `EquationBackend::MathType` остаётся unavailable.

**Причина:** минимальный срез проверяет архитектурную границу и переиспользует существующие XML/resource guarantees, не создавая ложной совместимости и новых привилегий.

**Последствия:** actual importer/version evidence переносится в 093, а feature-gated DOCX wiring — в 094. Новый crate получает только internal dependencies и может быть удалён без изменения parser, IR, MathML или OMML contracts.

**Fallback / rollback:** удалить crate, workspace/lockfile registration и SPEC 092; оставить `MathMlRenderer` и fail-closed DOCX backend без изменений.

**Проверка:** AC-092-001..006, targeted/workspace tests, dependency-scope validator, fmt/Clippy, independent architecture/security review.

**Связанные требования:** `specs/features/experimental-mathtype-adapter.spec.md`, SPEC-05.

## ADR-0014 — Project-specific fallback catalog имеет один канонический источник

**Статус:** принято 2026-08-20.

**Контекст:** в MathMorph правила отказа и деградации исторически появились
в `SECURITY.md`, feature SPEC, отдельных ADR, `ARCHITECTURE.md`,
локальных `AGENTS.md`, Skills и кодовых контрактах.

Эти документы описывают разные аспекты системы, но без отдельного
project-specific каталога существует риск drift, silent fallback
и появления нескольких конкурирующих описаний одной цепочки.

**Решение:** общий контракт retry/fallback/degraded/fail-closed
наследуется из AI Dev Team:

`~/codex-workspace/rules/fallback-policy.md`

MathMorph хранит только предметную delta в:

`docs/FALLBACKS.md`

Роли документов разделяются следующим образом:

- `docs/FALLBACKS.md` — конкретные MathMorph fallback/deny-path цепочки;
- `docs/SECURITY.md` — security invariants и случаи обязательного fail closed;
- `docs/ARCHITECTURE.md` — компоненты, state boundaries, idempotency,
  reconciliation и recovery interfaces;
- `docs/DECISIONS.md` — причины существенных fallback/rollback решений;
- feature SPEC — требуемое продуктовое поведение и acceptance criteria;
- `AGENTS.md` / Skills / subagents — краткая маршрутизация и локальные guardrails;
- tests — фактическое evidence поведения.

**Причина:** один project-specific каталог устраняет расхождение правил,
не превращая `docs/FALLBACKS.md` в копию общей AI Dev Team policy.

**Последствия:**

- глобальный fallback contract в MathMorph не копируется;
- существующие ADR `Fallback / rollback` сохраняются;
- конкретное архитектурное поведение может оставаться описанным
  в `ARCHITECTURE.md`, если оно необходимо для понимания компонента;
- локальные `AGENTS.md` не должны содержать полную копию fallback-цепочек;
- существенный новый fallback добавляется в `docs/FALLBACKS.md`
  и связывается с соответствующими SPEC/ADR/tests;
- silent fallback запрещён.

**Fallback / rollback:** если отдельный `docs/FALLBACKS.md` перестанет
давать пользу, решение пересматривается отдельным ADR; возврат
к нескольким независимым конкурирующим источникам истины не допускается.

**Проверка:** context compatibility audit, review ссылок из `AGENTS.md`,
`SECURITY.md`, `ARCHITECTURE.md` и tests; отсутствие полной копии
глобальной `rules/fallback-policy.md` внутри проекта.

## Шаблон ADR

Используй только для значимых архитектурных или технических решений.

### ADR-XXXX — Название

**Статус:** предложено / принято / заменено.

**Контекст:** проблема, ограничения и затронутые границы доверия или модулей.

**Варианты:** рассмотренные жизнеспособные альтернативы.

**Решение:** выбранный подход.

**Причина:** почему он лучше всего удовлетворяет ограничениям.

**Последствия:** компромиссы, миграция и влияние на эксплуатацию.

**Fallback / rollback:** безопасный путь отмены.

**Проверка:** тесты или benchmarks, подтверждающие решение.

**Связанные требования:** ссылки на канонические разделы SPEC или стабильные ID требований.
