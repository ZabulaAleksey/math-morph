# Спецификация видимой публичной Next.js-оболочки

Статус: Принято к реализации
Версия: 1.0
Этап: 154 — `Next.js shell`

## 1. Назначение и область

Этап создаёт первую видимую публичную страницу MathMorph по маршруту `/`. Страница объясняет назначение продукта и текущее состояние MVP, следует `docs/DESIGN.md` и остаётся честной относительно ещё не подключённого converter flow.

## 2. Вне области

- выбор, drag-and-drop и загрузка `.xmcd`/`.mcdx`;
- клиентская или серверная валидация файлов;
- параметры и запуск конвертации;
- API, аутентификация, оплата и реальные тарифы;
- production i18n routing и выбор локали;
- подключение MathType или обход блокировки этапа 094.

## 3. Функциональные требования

### FR-WEB-SHELL-001 Публичная структура

Маршрут `/` должен отображать семантическую публичную оболочку: header с навигацией, hero, возможности, этапы будущего conversion flow, объяснение конфиденциальности, краткие блоки API и тарифов, финальный CTA и footer с юридической оговоркой и статусом MVP.

### FR-WEB-SHELL-002 Честное состояние converter flow

CTA может перемещать пользователя к preview-секции, но не должен имитировать загрузку или конвертацию. Рядом с preview должно быть явно указано, что интерактивный конвертер подключается отдельным следующим этапом.

### FR-WEB-SHELL-003 Контент вне компонентов

Пользовательские строки должны поступать из отдельного украинского каталога. Полноценные каталоги и locale routing остаются этапами 162–165.

### FR-WEB-SHELL-004 Тема

Оболочка должна учитывать `system`, `light` и `dark`. Начальное состояние `system` определяется до первой отрисовки, явный выбор сохраняется локально, а переключатель имеет доступное имя.

### FR-WEB-SHELL-005 Адаптивная навигация

Навигация и основные действия должны оставаться доступными с клавиатуры и при ширине viewport от 320 CSS px. На compact-ширине допустима нативная раскрывающаяся навигация без обязательного JavaScript.

## 4. Нефункциональные требования

### NFR-WEB-SHELL-001 Дизайн и доступность

UI использует scoped namespace `[data-design-system="cbui"]`, семантические tokens Calm Blue, заметный `focus-visible`, touch target не меньше 44 px, корректные landmarks и `prefers-reduced-motion`. Целевой уровень — WCAG 2.2 AA.

### NFR-WEB-SHELL-002 Границы frontend

Страница остаётся статически рендеримой. В клиентский bundle попадает только переключатель темы; математическая и бизнес-семантика в React-компоненты не переносится.

### SEC-WEB-SHELL-001 Чувствительный контент

Макет не использует пользовательские документы, секреты или недостоверные compatibility/security claims. Иллюстрации содержат только синтетические формулы и статусы.

## 5. Ошибки и граничные случаи

- При недоступном `localStorage` страница сохраняет рабочую тему `system`.
- При отключённом JavaScript системная тема и навигационные ссылки остаются работоспособными.
- Длинный текст и viewport 320 px не должны создавать горизонтальную прокрутку страницы.
- `forced-colors` не должен скрывать focus, границы или статус MVP.

## 6. Критерии приёмки

- **AC-WEB-SHELL-001:** `/` содержит один `h1`, семантические `header`, `main`, именованные секции и `footer` из FR-WEB-SHELL-001.
- **AC-WEB-SHELL-002:** CTA ведёт к `#converter`, где явно показано staged/unavailable состояние без file input и сетевого запроса.
- **AC-WEB-SHELL-003:** CSS использует Calm Blue tokens и изолирован под `[data-design-system="cbui"]`; light/dark/system и reduced motion проверены.
- **AC-WEB-SHELL-004:** desktop и compact layout визуально проверены; keyboard navigation и focus-visible доступны.
- **AC-WEB-SHELL-005:** unit, component, integration, typecheck и production build проходят.

## 7. Связь с тестами

| Требование | Проверка |
|---|---|
| FR-WEB-SHELL-001, AC-WEB-SHELL-001 | component render test и browser smoke |
| FR-WEB-SHELL-002, AC-WEB-SHELL-002 | integration render test и browser smoke |
| FR-WEB-SHELL-003 | unit test каталога контента |
| FR-WEB-SHELL-004, AC-WEB-SHELL-003 | unit test theme resolver, browser light/dark smoke |
| FR-WEB-SHELL-005, NFR-WEB-SHELL-001 | component test, keyboard/manual browser QA, compact screenshot |
| NFR-WEB-SHELL-002 | Next.js production build и bundle boundary review |
| SEC-WEB-SHELL-001 | source review и static render assertions |

## 8. Открытые вопросы

Нет блокирующих вопросов. Реальный upload CTA, locale routing и backend path намеренно отложены до соответствующих этапов roadmap.

## 9. История изменений

| Версия | Дата | Изменение |
|---|---|---|
| 1.0 | 2026-08-20 | Создан контракт этапа 154 для первой видимой публичной оболочки |
