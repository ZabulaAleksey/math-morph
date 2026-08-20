# MathMorph — project-specific Fallback Policy

## Наследование

Этот документ является project-specific delta к:

`~/codex-workspace/rules/fallback-policy.md`

Общие правила retry, idempotency, degraded mode, provenance,
security fail-closed и evidence здесь не дублируются.

## 1. Определение входного формата

Primary:
- проверка signature и структуры XMCD/MCDX.

Fallback:
- безопасное распознавание container/format без содержательного parsing.

Fail closed:
- неизвестный, конфликтующий или небезопасный формат отклоняется.

Extension и MIME не являются основанием для ослабления проверки.

## 2. MCDX / ZIP container

Primary:
- bounded ZIP validation и parsing.

Retry:
- deterministic malformed archive не retryable.

Fallback:
- только безопасное распознавание container metadata, если оно не требует обхода limits.

Fail closed:
- zip bomb;
- path traversal;
- conflicting entries;
- unsafe compression ratio;
- превышение resource limits;
- integrity violation.

Запрещено повторять parsing с отключёнными limits.

## 3. XML / XMCD

Primary:
- bounded parser для подтверждённой schema/version.

Fallback:
- неизвестный безопасный узел сохраняется как Unsupported/opaque source-backed representation,
  если это разрешено контрактом слоя.

Fail closed:
- DTD;
- external entity;
- unsafe URI resolution;
- превышение depth/node/byte limits;
- malformed security-sensitive structure.

## 4. Неподтверждённая версия Mathcad

Primary:
- schema/version-specific parser.

Fallback:
- format/container recognition;
- compatibility diagnostic;
- preservation безопасно доступного provenance.

Fail closed:
- содержательная семантика неизвестной версии не угадывается.

## 5. Неизвестная математическая семантика

Primary:
- подтверждённое отображение Mathcad AST.

Fallback:
- preserve Original AST/source provenance;
- Unsupported diagnostic.

Fail closed:
- запрещено заменять неизвестную операцию приблизительно похожей семантикой.

Неизвестный оператор не становится автоматически умножением,
function call, assignment или иной известной конструкцией.

## 6. Resource limits

Primary:
- bounded parsing/export.

Fallback отсутствует.

При превышении security/resource budget:

Fail closed.

Запрещено:
- повторять без limits;
- автоматически увеличивать hard limit;
- переключаться на менее безопасный parser.

## 7. DOCX / OMML

Primary:
- редактируемый Word OMML для поддерживаемой семантики.

Fallback:
- только заранее разрешённый backend или representation,
  если SPEC явно допускает потерю конкретной возможности.

Degraded mode:
- должен явно сообщать, что именно перестало быть редактируемым
  или семантически эквивалентным.

Fail closed:
- если безопасного и разрешённого представления нет.

Запрещён silent fallback:

editable equation → screenshot/text/OLE/external content.

## 8. MathType backend

Primary:
- выбранный и доступный MathType backend.

Fallback:
- Word OMML только если configuration/product contract явно допускает этот переход.

Fail closed:
- `EquationBackendUnavailable`, если requested backend обязателен.

Нельзя тихо подменять requested MathType другим backend.

## 9. Assets

Primary:
- allowlisted и проверенные PNG/JPEG.

Fallback:
- omission/partial conversion только если SPEC явно допускает
  `completed_with_warnings`.

Fail closed:
- unsafe active content;
- invalid structure;
- SVG/OLE/macros/HTML/external relationship,
  если отдельный review не разрешил конкретный тип.

## 10. Local/WASM и server processing

Primary:
- выбранный пользователем trust mode.

Переход:

local/WASM → server processing

не является технически нейтральным fallback.

Он разрешён только после явного product-level решения
и информирования пользователя.

Запрещён silent privacy fallback с локальной обработки на серверную.

## 11. API job и потеря соединения

Primary:
- продолжить существующую operation по stable job ID / idempotency key.

При disconnect:

reconcile existing state
→ resume
→ retry безопасной операции при необходимости.

Запрещено:
- автоматически создавать новую conversion job только из-за network disconnect.

## 12. Worker / queue / storage

Transient infrastructure failure:
- bounded retry;
- timeout;
- backoff.

После exhaustion:
- reconciliation;
- failed/DLQ согласно архитектуре.

Deterministic invalid input:
- retry запрещён.

## 13. Partial conversion

Partial conversion разрешена только если конкретный export contract это допускает.

Статус:

`completed_with_warnings`

обязателен.

Conversion report должен объяснять потерянные элементы.

Partial result нельзя выдавать за полный success.

## 14. Web UI

UI различает:

- loading;
- error;
- no data;
- stale;
- partial;
- unverified;
- completed_with_warnings.

Cached/previous state после backend failure помечается `stale`.

Partial conversion помечается `partial`/`completed_with_warnings`.

Недостаточное compatibility evidence помечается `unverified`.

## 15. Security/QA reviewer unavailable

Project-specific agent или Skill может выполнить дополнительный локальный анализ.

Он не заменяет обязательный global QA/security gate.

При недоступном обязательном review:

`SECURITY_REVIEW_UNVERIFIED`
или
`QA_REVIEW_UNVERIFIED`

Security-sensitive изменение не считается полностью проверенным.

## 16. Dependency/backend replacement

Primary:
- утверждённая pinned dependency/backend.

Fallback:
- заранее проверенная совместимая альтернатива.

Fail closed:
- если безопасной подтверждённой альтернативы нет.

Случайный package или неподдерживаемая версия зависимости
не являются fallback.

## 17. Локальная публикация CLI

Primary:
- bounded single-read input;
- same-directory `create_new` temp;
- sync содержимого;
- atomic no-replace hard-link publication.

Retry/fallback отсутствуют:
- deterministic input/conversion failure не повторяется;
- hard-link unavailable не переключается на replacing `rename` или прямую запись final output;
- security/identity uncertainty не ослабляет component/reparse/ownership checks.

Commit point:
- успешное создание final hard link означает completed side effect;
- сбой последующего cleanup собственного temp даёт redacted warning, но не false failure и не retry;
- неизвестный replacement temp не удаляется.

Fail closed до commit point:
- existing output или race-created destination;
- input/output identity conflict;
- symlink/reparse/Windows network или device namespace;
- input/output/temp identity uncertainty;
- filesystem без безопасной no-replace публикации.

## Tests

Для значимых цепочек проверяются:

- primary success;
- retry success;
- retry exhaustion;
- fallback success;
- degraded result;
- fail closed;
- resource limit;
- reconciliation после неизвестного side effect;
- отсутствие silent fallback;
- отсутствие утечки пользовательского payload.
