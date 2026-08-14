# Стратегия тестирования и критерии готовности

## Обязательные уровни тестирования

### Parser/core

- unit-тесты для каждого узла и функции;
- snapshot и эталонные AST/IR там, где они стабильны;
- round-trip сериализации каждой версии `DocumentIR` и проверка schema migration/несовместимости;
- явное покрытие таблиц, программных блоков, metadata и неизвестных новых constructs;
- тесты повреждённого ввода и неизвестных узлов;
- property-based тесты инвариантов;
- fuzz-цели для границ XML, контейнера и parser;
- ограничения памяти, размера и рекурсии.

### Математический движок

- тесты зависимостей и символов;
- traces подстановки;
- разделение точности и округления;
- round-trip проверки комплексного алгебраического↔полярного представления с допуском;
- случаи квадрантов, нуля, деления и нормализации угла.

### DOCX/OMML

- проверка структуры созданного пакета;
- корректность relationships, типов содержимого и XML;
- тесты структуры редактируемых уравнений;
- ручной smoke-набор эталонных DOCX для открытия и редактирования в Word;
- регрессионные тесты вложенных уравнений.

### API/backend

- тесты авторизации и scopes ключей API;
- жизненный цикл и идемпотентность асинхронных задач;
- progress/correlation, cancellation и восстановление состояния после reconnect;
- chunked upload/checksum/atomic finalize и Range/resume download;
- семантика предпочтений сохранения;
- границы квот и rate limit;
- повторяемые и неповторяемые сбои;
- тесты сбоя и timeout хранилища.

### Frontend/E2E

- загрузка и drag-and-drop;
- состояния проверки файла;
- состояния конвертации и восстановление после обрыва сети;
- смена Wi‑Fi/IP/VPN, истёкший signed URL и прерванный download;
- локализованные структурированные ошибки;
- flows аутентификации, 2FA и восстановления;
- dashboard, ключи API и настройки конфиденциальности.

### Безопасность

- атаки XML;
- ZIP bomb и path traversal;
- вредоносные имена файлов, метаданные и SVG;
- XSS и injection;
- brute force и replay для аутентификации и восстановления;
- скрытие секретов в журналах;
- отсутствие документа, формул и имён файлов в traces, metrics и product events;
- тесты границы конфиденциальности администратора;
- review цепочки поставки зависимостей, MCP, hooks и Skills.

## Структура fixtures

`tests/fixtures/` groups: xmcd, mcdx, formulas, complex, plots, diagrams, mixed, corrupted, security, compatibility.

Каждый fixture включается в манифест с форматом, версией, функциями и ожидаемым статусом. Исправленная ошибка parser получает постоянный регрессионный fixture, если это допустимо юридически и технически.

## Текущее покрытие parser

- `tests/input_boundary.rs`: format detection, MCDX path/container/limit policy и XML root metadata;
- `tests/worksheet_structure.rs`: AC-027–035, metadata/regions/layout/order/text/math/plot/picture/opaque и worksheet resource limits;
- `tests/math_ast.rs`: AC-036–037, core AST/radix/arity/node limits и canonical test-only snapshots;
- `tests/math_ast_forms.rs`: AC-038–044, definitions/evaluation/functions/unary/grouping/index;
- `tests/math_ast_advanced.rs`: AC-045–051, matrix/vector/range/calculus/comparisons и shape/limit regressions.

Небольшие synthetic XML cases находятся inline рядом с Rust integration tests: так grammar edge case виден в одном месте и не раздувает corpus manifest. Постоянный cross-language или full-document regression fixture по-прежнему обязан входить в `tests/fixtures/manifest.json`.

Snapshot renderer существует только в tests и выдаёт канонический S-expression. Production serialization dependency для этапа 037 не добавлялась.

## Правило эталонных данных

Не обновляй эталонный fixture только потому, что после изменения реализации тест упал. Сначала докажи, что желаемое поведение изменилось намеренно.

## DoD этапа

- объём реализован без неотносящихся к нему будущих функций;
- целевые тесты добавлены и проходят;
- включён относящийся к задаче негативный или граничный тест;
- lint, typecheck и build затронутой области проходят;
- parser не вызывает panic или crash на пути пользовательского ввода;
- `docs/AI_STATUS.md` обновлён для существенного этапа;
- DECISIONS, ARCHITECTURE и SECURITY обновлены только при изменении их контракта;
- итоговый отчёт перечисляет фактически выполненные, а не предполагаемые проверки.

Универсальные полные release-проверки должны выполняться уже установленной глобальной AI Dev Team, если она их предоставляет; не дублируй их локально.
