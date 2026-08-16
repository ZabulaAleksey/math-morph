# Самостоятельное выполнение этапов с помощью ChatGPT

## 1. Назначение

Этот документ помогает владельцу MathMorph самостоятельно выполнять этапы `docs/ROADMAP.md`, привлекая ChatGPT как помощника, но сохраняя проверяемость результата.

Текущее состояние на 2026-08-15:

- этапы 001–091 имеют статус `verified`;
- следующий этап — 092, `experimental MathType adapter`;
- всего в roadmap 304 этапа;
- пользовательский web-интерфейс начинается с этапа 154, первый прикладной компонент `dropzone` — с этапа 156.

Главное правило: один этап — одна ограниченная задача — отдельная ветка — целевые тесты — review — commit — обновлённый статус. Не просите ChatGPT реализовать сразу несколько следующих этапов: так сложнее заметить ошибку и безопасно откатить изменение.

## 2. Роли человека и ChatGPT

### Что решает человек

- какой этап выполнять;
- устраивает ли наблюдаемое поведение;
- можно ли принимать архитектурный компромисс;
- можно ли добавлять dependency, внешний сервис или платный продукт;
- можно ли делать merge и удалять ветку;
- являются ли реальные Mathcad-файлы законными и безопасными для fixture corpus.

### Что поручать ChatGPT

- найти относящиеся к этапу файлы и требования;
- составить ограниченный план;
- реализовать один этап;
- написать позитивные, негативные и граничные тесты;
- запустить проверки и объяснить ошибки;
- провести read-only review diff;
- обновить только действительно затронутую документацию;
- подготовить commit и временную ветку для push.

### Два режима помощи

**Codex/агент с доступом к репозиторию** может сам читать разрешённые файлы, изменять рабочую ветку и запускать команды. Всё равно требуйте фактический список команд и результаты.

**Обычный ChatGPT без доступа к компьютеру** не видит Git, файлы и терминал. В этом режиме вы сами запускаете команды, прикладываете только относящиеся фрагменты и просите ChatGPT подготовить небольшой patch или объяснение. Никогда не считайте тест выполненным, если ChatGPT лишь написал ожидаемый результат.

Безопасный пакет контекста для обычного ChatGPT:

```powershell
git status -sb
git log -1 --oneline
git diff --stat
git diff -- path/to/relevant/file
```

Дополнительно передайте текст одного stage-prompt, относящиеся acceptance criteria, нужный `AGENTS.md` и полный безопасный вывод упавшей команды. Не прикладывайте `.env`, tokens, секреты или конфиденциальные Mathcad-документы.

### Что нельзя принимать на веру

- сообщение «тесты должны пройти», если нет фактического вывода команды;
- новый формат XML/Mathcad, придуманный без SPEC или подтверждённой схемы;
- автоматически обновлённый golden snapshot только потому, что тест упал;
- утверждение о совместимости с Word, браузером или API без соответствующего smoke/integration test;
- статус `verified`, если review не завершён или существенные замечания не исправлены.

## 3. Подготовка рабочего места на Windows

Откройте PowerShell и перейдите в корень репозитория:

```powershell
cd ~/codex-workspace/projects/math-morph
Test-Path Cargo.toml
git status -sb
git branch --show-current
```

`Test-Path Cargo.toml` должен вернуть `True`. Перед Rust-командами проверьте инструменты:

```powershell
rustup show active-toolchain
rustc --version
cargo --version
```

Проект закрепляет Rust 1.88 через `rust-toolchain.toml`. Для MSVC нужен Visual Studio Build Tools с workload `Desktop development with C++`; VS Code сам по себе не устанавливает `link.exe`.

PowerShell может блокировать `pnpm.ps1`, поэтому в командах ниже используется `pnpm.cmd`.

## 4. Начало каждого этапа

### 4.1. Обновите `main`

Сначала убедитесь, что незакоммиченных изменений нет. Если они есть, не удаляйте их: завершите, закоммитьте или отдельно сохраните осознанным способом.

```powershell
git switch main
git pull --ff-only origin main
git status -sb
```

Создайте ветку одного этапа:

```powershell
git switch -c feature/stage-092-mathtype-adapter
```

Для следующего этапа заменяйте номер и короткое имя. Не работайте непосредственно в `main`.

### 4.2. Найдите точное задание

Используйте три разных источника по назначению:

1. `docs/ROADMAP.md` — номер, порядок и название этапа.
2. Соответствующий раздел `docs/PROMPTS.md` — исполняемое задание для ChatGPT.
3. SPEC из `specs/README.md` — стабильные требования и acceptance criteria.

`ROADMAP` и `PROMPTS` не заменяют SPEC. Если для изменения наблюдаемого поведения нет достаточного требования или acceptance criteria, сначала остановитесь и попросите ChatGPT подготовить или уточнить SPEC без реализации кода.

### 4.3. Загрузите только необходимый контекст

Минимальный порядок чтения:

1. корневой `AGENTS.md`;
2. ближайший модульный `AGENTS.md` для изменяемого каталога;
3. `docs/AI_STATUS.md`;
4. `docs/AI_PLAN.md`;
5. относящийся раздел SPEC;
6. текущие исходники и тесты затрагиваемого модуля;
7. `docs/SECURITY.md`, если изменяется недоверенный ввод, XML/ZIP, сеть, загрузка, auth, secrets или хранение.

Не отправляйте ChatGPT целиком все 304 prompts, весь `LEARNING_LOG` или все SPEC: это расходует контекст и повышает риск смешать этапы.

## 5. Первый prompt для ChatGPT

Скопируйте шаблон и заполните квадратные скобки:

```text
Проект: math-morph.
Репозиторий: ~/codex-workspace/projects/math-morph.
Текущая ветка: [feature/stage-NNN-short-name].
Выполняем только этап [NNN — название] из docs/ROADMAP.md и соответствующий раздел docs/PROMPTS.md.

Перед реализацией:
1. Прочитай корневой и ближайший модульный AGENTS.md.
2. Прочитай AI_STATUS, AI_PLAN и только относящийся SPEC через specs/README.md.
3. Проверь git status и не трогай чужие/незакоммиченные изменения.
4. Сначала перечисли acceptance criteria, изменяемые файлы, риски и целевые тесты.
5. Если требований недостаточно, остановись на уточнении SPEC и не придумывай формат или API.

Реализация:
- реализуй только этот этап без функций последующих этапов;
- добавь позитивный тест и минимум один относящийся негативный или граничный тест;
- сохраняй архитектурные границы parser → model/engine → Document IR → exporter;
- ошибки на недоверенном вводе должны быть typed/fail-closed и не раскрывать payload;
- обнови только затронутую документацию и LEARNING_LOG;
- проведи review, исправь существенные замечания и повтори проверки;
- сделай commit и push только во временную ветку; не делай merge без моего отдельного разрешения.

В финале сообщи: статус, файлы, тесты, фактически выполненные команды, ограничения, commit, branch и следующий этап.
```

Перед подтверждением плана убедитесь, что ChatGPT назвал один этап, конкретный SPEC и конечный набор файлов. Если в плане уже появились функции следующего этапа, попросите сократить scope.

## 6. Цикл реализации одного этапа

### Шаг 1. Зафиксировать контракт

До кода должны быть понятны:

- входы и выходы;
- публичные типы или наблюдаемое поведение;
- точные acceptance criteria;
- ошибки и неподдержанные случаи;
- ограничения размера, глубины, количества узлов или времени, если обрабатывается недоверенный ввод;
- что намеренно не входит в этап.

Если ChatGPT предлагает поменять архитектуру, новый публичный API, dependency или wire format, попросите записать решение и последствия в `docs/DECISIONS.md` до реализации.

### Шаг 2. Записать план

`docs/AI_PLAN.md` должен содержать только текущий этап:

- статус `planned`, `in progress` или `completed`;
- номер и название;
- ветку;
- выбранную SPEC;
- 3–7 проверяемых шагов;
- non-goals;
- тесты и способ отката.

### Шаг 3. Реализовать минимальный вертикальный срез

Хороший срез даёт проверяемый результат от публичного входа до публичного выхода, но не добавляет будущие возможности. После небольшого рабочего блока сразу запускайте целевой тест — не ждите конца большого refactor.

### Шаг 4. Добавить тесты

Для каждого нового поведения нужны:

- позитивный тест ожидаемого результата;
- негативный или граничный тест;
- regression test для исправленного бага;
- deterministic/golden test только там, где exact output является контрактом;
- limit/security test, если вход контролирует пользователь.

Тест должен падать без реализации или при возвращении исправленной ошибки. Не проверяйте только внутренний helper, если пользователь наблюдает поведение через публичный API.

### Шаг 5. Провести review

Попросите ChatGPT выполнить отдельный read-only review. По возможности используйте новую сессию или отдельного reviewer-агента: ему нужен исходный SPEC и готовый diff, но не нужно защищать решения предыдущего исполнителя.

```text
Проведи read-only review текущего diff этапа [NNN]. Не меняй файлы.
Сверь реализацию с acceptance criteria и архитектурными границами.
Ищи correctness regressions, scope leakage, неполные негативные тесты,
неограниченную работу на недоверенном вводе и раскрытие payload.
Для каждого finding укажи severity, файл/строку, воспроизведение и минимальное исправление.
Если существенных findings нет, явно напиши PASS и перечисли выполненные проверки.
```

Для XML/ZIP, загрузки файлов, auth, network, storage, secrets или внешних документов дополнительно нужен security review. Для горячих вычислительных путей — performance review.

### Шаг 6. Исправить findings и повторить проверки

Finding считается закрытым только после изменения, regression test и повторной проверки. Не ослабляйте assertion или limit, чтобы получить зелёный тест.

### Шаг 7. Обновить статус и сделать checkpoint

После завершения обновите `AI_STATUS`, `AI_PLAN`, `TRACEABILITY` и относящиеся документы по правилам раздела 9. Затем сделайте commit и push временной ветки.

## 7. Где и какие тесты запускать

Все команды выполняются из корня репозитория, если таблица не говорит обратного.

### 7.1. Быстрые проверки во время разработки

| Область | Команда | Когда запускать |
|---|---|---|
| Структура проекта | `python -B scripts/validate_project.py` | после manifests, docs, workspace или dependency changes |
| Fixture corpus | `python -B scripts/validate_fixtures.py` | после любого изменения `tests/fixtures/` или manifest |
| Python validators | `python -B -m unittest discover -s tests -p "test_*.py" -v` | после validators/fixtures/docs contracts |
| Один Rust crate | `cargo test -p CRATE_NAME --locked` | после локального Rust-изменения; замените `CRATE_NAME` |
| Один Rust integration test | `cargo test -p CRATE_NAME --test TEST_NAME --locked` | на каждом коротком цикле; замените оба имени |
| Rust formatting | `cargo fmt --all -- --check` | перед commit; при ошибке выполнить `cargo fmt --all` |
| Rust lint | `cargo clippy -p CRATE_NAME --all-targets --locked -- -D warnings` | после компиляции целевого crate |
| Frontend types | `pnpm.cmd --filter @math-morph/web typecheck` | после TypeScript/React changes |
| Frontend build | `pnpm.cmd --filter @math-morph/web build` | перед завершением frontend-этапа |
| Python package | `uv build --project services/api` | после package/config changes API |
| Git whitespace | `git diff --check` | перед каждым commit |

Имена текущих Rust crates:

- `mathcad-parser`;
- `math-model`;
- `math-engine`;
- `document-ir`;
- `exporter-docx`;
- `exporter-mathml`.

### 7.2. Обязательные финальные проверки Rust-этапа

```powershell
cargo fmt --all -- --check
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
python -B scripts/validate_project.py
git diff --check
```

Если этап меняет fixtures, добавьте:

```powershell
python -B scripts/validate_fixtures.py
python -B -m unittest discover -s tests -p "test_*.py" -v
```

### 7.3. Parser, XMCD и MCDX

Основные тесты находятся в `crates/mathcad-parser/tests/`:

- `input_boundary.rs` — detection, ZIP/container, XML metadata и resource limits;
- `worksheet_structure.rs` — metadata, regions, layout и opaque content;
- `math_ast*.rs` — математические узлы и ошибки структуры.

Минимальный набор:

```powershell
cargo test -p mathcad-parser --locked
python -B scripts/validate_fixtures.py
```

Для нового пользовательского ввода проверьте corrupted input, неправильный QName/arity, превышение limit и отсутствие исходного payload в `Debug`/error.

### 7.4. Document IR, DOCX и MathML

```powershell
cargo test -p document-ir --locked
cargo test -p exporter-docx --locked
cargo test -p exporter-mathml --locked
```

Для DOCX дополнительно можно сгенерировать reference artifact:

```powershell
cargo run -p exporter-docx --example advanced_omml_reference
Invoke-Item target/word-reference/advanced-omml-reference.docx
```

Ручная проверка Word дополняет, но не заменяет автоматические structural tests. Golden MathML находятся в `crates/exporter-mathml/tests/golden/`; не обновляйте их без намеренного изменения SPEC.

### 7.5. Frontend

Один раз после изменения lockfile или нового checkout:

```powershell
pnpm.cmd install --frozen-lockfile
```

Проверки:

```powershell
pnpm.cmd run typecheck
pnpm.cmd run build:web
pnpm.cmd run dev:web
```

Откройте `http://localhost:3000` и вручную проверьте desktop/mobile ширину, keyboard navigation, focus, loading/error/empty states, `light`/`dark`/`system`, `forced-colors` и reduced motion в пределах этапа. Автоматический test/E2E script запускайте только после того, как он реально появится в `apps/web/package.json`; не придумывайте несуществующую команду.

### 7.6. API

Сейчас API является package-каркасом без HTTP endpoints:

```powershell
uv sync --project services/api --locked
uv build --project services/api
uv run --project services/api python -c "import math_morph_api; print(math_morph_api.__doc__)"
```

После появления FastAPI/test dependencies используйте команды, закреплённые в `services/api/pyproject.toml` и CI. До этого не заявляйте, что HTTP server или endpoint протестирован.

### 7.7. Документационный этап

```powershell
python -B scripts/validate_project.py
python -B -m unittest discover -s tests -p "test_*.py" -v
git diff --check
```

Дополнительно найдите устаревшие номера, ветки и статусы через `rg`.

## 8. Definition of Done одного этапа

Этап можно назвать `verified`, только если выполнены все применимые пункты.

### Требования и scope

- [ ] Реализован ровно один номер этапа.
- [ ] Выбрана каноническая SPEC; acceptance criteria перечислены и выполнены.
- [ ] Более поздние этапы не реализованы заранее.
- [ ] Неизвестное поведение не было придумано без решения владельца/SPEC.

### Реализация

- [ ] Публичное поведение соответствует SPEC и архитектурным границам.
- [ ] Неподдержанный случай возвращает явную typed ошибку/diagnostic.
- [ ] Недоверенный ввод ограничен по применимым ресурсам и не вызывает panic/crash.
- [ ] Ошибки, `Debug`, logs и telemetry не раскрывают документы, формулы, пути, secrets или ключи.
- [ ] Новая dependency действительно необходима, закреплена lockfile и проверена по политике проекта.

### Тесты

- [ ] Добавлен позитивный тест.
- [ ] Добавлен негативный или граничный тест.
- [ ] Для bugfix добавлен regression test.
- [ ] Целевые тесты проходят.
- [ ] Все относящиеся lint/typecheck/build проверки проходят.
- [ ] Workspace/repository gates проходят перед завершением.
- [ ] Ручной smoke выполнен там, где результат визуальный или зависит от Word/browser.

### Review и документация

- [ ] Выполнен независимый read-only review, существенные findings закрыты.
- [ ] Security/performance review выполнен, если менялась соответствующая граница.
- [ ] `docs/AI_PLAN.md` показывает фактически завершённый план.
- [ ] `docs/AI_STATUS.md` обновлён.
- [ ] `docs/TRACEABILITY.md` содержит реализацию, тесты и статус `verified`.
- [ ] `docs/TESTING.md` обновлён, если изменился постоянный набор тестов.
- [ ] `docs/LEARNING_LOG.md` объясняет изменение, команды, проблемы и способы повторения.
- [ ] `ARCHITECTURE`, `DECISIONS`, `DESIGN` и `SECURITY` обновлены только при реальном изменении их контрактов.

### Git и handoff

- [ ] `git diff --check` проходит.
- [ ] В commit нет чужих или случайных файлов.
- [ ] Commit message описывает один логический результат.
- [ ] Временная ветка отправлена в origin без force push.
- [ ] Записаны branch, commit hash, результаты проверок, ограничения и следующий этап.

Если хотя бы один обязательный пункт не выполнен, используйте статус `in progress` или `implemented, not verified`, но не `verified`.

## 9. Какие документы обновлять

| Документ | Когда обновлять |
|---|---|
| `docs/AI_PLAN.md` | в начале этапа и после фактического завершения |
| `docs/AI_STATUS.md` | после существенного этапа или при появлении blocker/ограничения |
| `docs/ROADMAP.md` | когда этап становится `verified` или меняется утверждённый порядок |
| `docs/TRACEABILITY.md` | когда появились код и доказательства acceptance criteria |
| `docs/TESTING.md` | когда добавился постоянный test suite, fixture class или quality gate |
| `docs/LEARNING_LOG.md` | для воспроизводимого объяснения существенного этапа |
| `docs/ARCHITECTURE.md` | только при изменении границ/потока/зависимостей |
| `docs/DECISIONS.md` | при существенном техническом решении и trade-off |
| `docs/DESIGN.md` | при изменении канонического UI/UX contract |
| `docs/SECURITY.md` | при изменении trust boundary, угроз, secrets, auth, network или storage |

Не превращайте `AI_STATUS` в длинный журнал: он должен оставаться компактным снимком для следующей сессии.

## 10. Git workflow этапа

Проверить изменения:

```powershell
git status --short
git diff --stat
git diff --check
```

Добавляйте только относящиеся файлы, а не автоматически весь рабочий каталог:

```powershell
git add path/to/file1 path/to/file2
git diff --cached --check
git commit -m "feat(scope): complete stage NNN"
git push -u origin feature/stage-NNN-short-name
```

Для документации используйте `docs: ...`, для bugfix — `fix(scope): ...`, для тестового контракта — `test(scope): ...`.

Не выполняйте `git reset --hard`, force push, merge в `main` или удаление веток без явного понимания цели и подтверждения. Перед merge убедитесь, что review и все обязательные проверки завершены.

## 11. Если лимит ChatGPT заканчивается

Не начинайте следующий этап. Сначала оставьте воспроизводимый checkpoint:

1. обновите `AI_STATUS` и `AI_PLAN` честным статусом;
2. выполните доступные целевые тесты;
3. закоммитьте только рабочее состояние, если оно собирается и представляет полезную точку отката;
4. отправьте временную ветку;
5. сохраните краткий handoff по шаблону ниже.

```text
Проект: math-morph
Этап: [NNN — название]
Статус: [planned / in progress / implemented, not verified / verified]
Ветка: [branch]
Commit: [hash или «нет»]
Реализовано: [кратко]
Осталось: [конкретные пункты]
Изменённые файлы: [список]
Пройденные проверки: [команда → PASS]
Падающие проверки: [команда → краткая ошибка]
Review findings: [открытые пункты]
Blocker/решение владельца: [если есть]
Следующее безопасное действие: [один пункт]
```

В новой сессии передайте этот handoff и попросите сначала сверить его с `AI_STATUS`, `AI_PLAN`, Git status/log и фактическими тестами, а не начинать реализацию заново.

## 12. Как просить ChatGPT исправить ошибку

Передавайте полный текст ошибки, команду, рабочую директорию и последние относящиеся изменения:

```text
Проект math-morph, ветка [branch], этап [NNN].
Команда: [точная команда].
Рабочая директория: [путь].
Фактическая ошибка:
[полный безопасный вывод без secrets/payload]

Сначала воспроизведи и локализуй причину. Не меняй код, пока не объяснишь,
какой контракт нарушен. Затем предложи минимальное исправление и regression test.
Не ослабляй существующие проверки и не реализуй следующий этап.
```

Не отправляйте содержимое конфиденциального Mathcad-документа. Для диагностики используйте минимальный synthetic пример или отредактированный fixture без персональных данных.

## 13. Финальный отчёт ChatGPT

Требуйте ответ в следующем формате:

```text
Этап: NNN — название
Статус: verified / implemented, not verified / blocked
Ветка и commit:
SPEC и выполненные acceptance criteria:
Изменённые файлы:
Добавленные позитивные тесты:
Добавленные негативные/граничные тесты:
Фактически выполненные команды и результаты:
Review/security/performance findings и их статус:
Документация:
Известные ограничения:
Намеренно не реализовано:
Следующий этап:
```

Если отчёт не содержит точных команд и результатов, этап ещё нельзя считать доказанно завершённым.

## 14. Короткий ежедневный сценарий

1. `git status -sb` и проверка ветки.
2. Прочитать один этап, один prompt и относящийся SPEC.
3. Согласовать с ChatGPT acceptance criteria и тесты.
4. Реализовать маленький срез.
5. Запустить targeted test.
6. Добавить negative/regression test.
7. Запустить финальные gates затронутой области.
8. Провести read-only review и исправить findings.
9. Обновить status/traceability/learning.
10. Commit и push временной ветки.
11. Оставить handoff; только затем переходить к следующему этапу.
