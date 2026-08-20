---
version: alpha
name: MathMorph Calm Blue UI
description: "Спокойная светлая дизайн-система MathMorph для безопасной конвертации Mathcad в редактируемый DOCX/OMML, кабинета, истории задач и API-сценариев."
colors:
  primary: "#4767F5"
  primary-hover: "#3858E8"
  primary-pressed: "#2F4BCB"
  primary-soft: "#F4F6FF"
  on-primary: "#FFFFFF"
  secondary: "#626262"
  on-surface: "#090909"
  surface: "#FFFFFF"
  surface-page: "#FAFAFA"
  surface-subtle: "#F6F7F9"
  border: "#E4E4E4"
  border-strong: "#C9CDD5"
  disabled: "#929292"
  success: "#13B97A"
  warning: "#F59E2F"
  error: "#D64545"
  info: "#4767F5"
  overlay: "#0909097A"
  focus: "#4767F552"
  dark-primary: "#9BAEFF"
  dark-primary-hover: "#B3C0FF"
  dark-primary-pressed: "#7F95FF"
  dark-primary-soft: "#202A52"
  dark-on-primary: "#101631"
  dark-secondary: "#B2BAC9"
  dark-on-surface: "#F4F6FB"
  dark-surface: "#171B24"
  dark-surface-page: "#0F1218"
  dark-surface-subtle: "#202631"
  dark-surface-elevated: "#252C39"
  dark-border: "#303847"
  dark-border-strong: "#4A566A"
  dark-disabled: "#737D8E"
  dark-success: "#4DD6A0"
  dark-warning: "#F6B84A"
  dark-error: "#FF8585"
  dark-info: "#9BAEFF"
  dark-overlay: "#000000B8"
  dark-focus: "#9BAEFF66"
typography:
  headline-display:
    fontFamily: "Inter, ui-sans-serif, system-ui, sans-serif"
    fontSize: 40px
    fontWeight: 700
    lineHeight: 1.2
    letterSpacing: -0.02em
  headline-lg:
    fontFamily: "Inter, ui-sans-serif, system-ui, sans-serif"
    fontSize: 32px
    fontWeight: 700
    lineHeight: 1.25
    letterSpacing: -0.01em
  headline-md:
    fontFamily: "Inter, ui-sans-serif, system-ui, sans-serif"
    fontSize: 24px
    fontWeight: 700
    lineHeight: 1.3
  headline-sm:
    fontFamily: "Inter, ui-sans-serif, system-ui, sans-serif"
    fontSize: 20px
    fontWeight: 600
    lineHeight: 1.35
  body-lg:
    fontFamily: "Inter, ui-sans-serif, system-ui, sans-serif"
    fontSize: 18px
    fontWeight: 400
    lineHeight: 1.45
  body-md:
    fontFamily: "Inter, ui-sans-serif, system-ui, sans-serif"
    fontSize: 16px
    fontWeight: 400
    lineHeight: 1.5
  body-sm:
    fontFamily: "Inter, ui-sans-serif, system-ui, sans-serif"
    fontSize: 14px
    fontWeight: 400
    lineHeight: 1.45
  label-lg:
    fontFamily: "Inter, ui-sans-serif, system-ui, sans-serif"
    fontSize: 16px
    fontWeight: 600
    lineHeight: 1.25
  label-md:
    fontFamily: "Inter, ui-sans-serif, system-ui, sans-serif"
    fontSize: 14px
    fontWeight: 600
    lineHeight: 1.4
  label-sm:
    fontFamily: "Inter, ui-sans-serif, system-ui, sans-serif"
    fontSize: 12px
    fontWeight: 500
    lineHeight: 1.35
rounded:
  none: 0px
  sm: 8px
  md: 14px
  lg: 20px
  full: 9999px
spacing:
  none: 0
  xxs: 4px
  xs: 8px
  sm: 12px
  md: 16px
  lg: 20px
  xl: 24px
  xxl: 32px
  xxxl: 40px
  huge: 48px
  section: 64px
components:
  page:
    backgroundColor: "{colors.surface-page}"
    textColor: "{colors.on-surface}"
  card:
    backgroundColor: "{colors.surface}"
    textColor: "{colors.on-surface}"
    rounded: "{rounded.lg}"
    padding: "{spacing.xl}"
  button-primary:
    backgroundColor: "{colors.primary}"
    textColor: "{colors.on-primary}"
    typography: "{typography.label-lg}"
    rounded: "{rounded.md}"
    height: 48px
    padding: "{spacing.md}"
  button-primary-hover:
    backgroundColor: "{colors.primary-hover}"
    textColor: "{colors.on-primary}"
  button-primary-pressed:
    backgroundColor: "{colors.primary-pressed}"
    textColor: "{colors.on-primary}"
  button-secondary:
    backgroundColor: "{colors.surface}"
    textColor: "{colors.primary}"
    typography: "{typography.label-lg}"
    rounded: "{rounded.md}"
    height: 48px
    padding: "{spacing.md}"
  input:
    backgroundColor: "{colors.surface}"
    textColor: "{colors.on-surface}"
    typography: "{typography.body-md}"
    rounded: "{rounded.md}"
    height: 48px
    padding: "{spacing.md}"
  selected-row:
    backgroundColor: "{colors.primary-soft}"
    textColor: "{colors.primary-pressed}"
    rounded: "{rounded.md}"
  metadata:
    textColor: "{colors.secondary}"
    typography: "{typography.body-sm}"
  disabled-label:
    textColor: "{colors.disabled}"
  surface-subtle:
    backgroundColor: "{colors.surface-subtle}"
    textColor: "{colors.on-surface}"
  divider:
    backgroundColor: "{colors.border}"
    height: 1px
  divider-strong:
    backgroundColor: "{colors.border-strong}"
    height: 1px
  status-success:
    backgroundColor: "{colors.success}"
    textColor: "{colors.on-surface}"
  status-warning:
    backgroundColor: "{colors.warning}"
    textColor: "{colors.on-surface}"
  status-error:
    backgroundColor: "{colors.error}"
    textColor: "{colors.on-surface}"
  status-info:
    backgroundColor: "{colors.info}"
    textColor: "{colors.on-primary}"
  overlay:
    backgroundColor: "{colors.overlay}"
  focus-ring:
    backgroundColor: "{colors.focus}"
  dark-page:
    backgroundColor: "{colors.dark-surface-page}"
    textColor: "{colors.dark-on-surface}"
  dark-card:
    backgroundColor: "{colors.dark-surface}"
    textColor: "{colors.dark-on-surface}"
    rounded: "{rounded.lg}"
    padding: "{spacing.xl}"
  dark-floating-surface:
    backgroundColor: "{colors.dark-surface-elevated}"
    textColor: "{colors.dark-on-surface}"
    rounded: "{rounded.lg}"
  dark-button-primary:
    backgroundColor: "{colors.dark-primary}"
    textColor: "{colors.dark-on-primary}"
    typography: "{typography.label-lg}"
    rounded: "{rounded.md}"
    height: 48px
    padding: "{spacing.md}"
  dark-button-primary-hover:
    backgroundColor: "{colors.dark-primary-hover}"
    textColor: "{colors.dark-on-primary}"
  dark-button-primary-pressed:
    backgroundColor: "{colors.dark-primary-pressed}"
    textColor: "{colors.dark-on-primary}"
  dark-button-secondary:
    backgroundColor: "{colors.dark-surface}"
    textColor: "{colors.dark-primary}"
    typography: "{typography.label-lg}"
    rounded: "{rounded.md}"
    height: 48px
    padding: "{spacing.md}"
  dark-input:
    backgroundColor: "{colors.dark-surface}"
    textColor: "{colors.dark-on-surface}"
    typography: "{typography.body-md}"
    rounded: "{rounded.md}"
    height: 48px
    padding: "{spacing.md}"
  dark-selected-row:
    backgroundColor: "{colors.dark-primary-soft}"
    textColor: "{colors.dark-primary}"
    rounded: "{rounded.md}"
  dark-metadata:
    textColor: "{colors.dark-secondary}"
    typography: "{typography.body-sm}"
  dark-disabled-label:
    textColor: "{colors.dark-disabled}"
  dark-surface-subtle:
    backgroundColor: "{colors.dark-surface-subtle}"
    textColor: "{colors.dark-on-surface}"
  dark-divider:
    backgroundColor: "{colors.dark-border}"
    height: 1px
  dark-divider-strong:
    backgroundColor: "{colors.dark-border-strong}"
    height: 1px
  dark-status-success:
    backgroundColor: "{colors.dark-success}"
    textColor: "{colors.dark-on-primary}"
  dark-status-warning:
    backgroundColor: "{colors.dark-warning}"
    textColor: "{colors.dark-on-primary}"
  dark-status-error:
    backgroundColor: "{colors.dark-error}"
    textColor: "{colors.dark-on-primary}"
  dark-status-info:
    backgroundColor: "{colors.dark-info}"
    textColor: "{colors.dark-on-primary}"
  dark-overlay:
    backgroundColor: "{colors.dark-overlay}"
  dark-focus-ring:
    backgroundColor: "{colors.dark-focus}"
---

# MathMorph — Calm Blue UI

## Overview

MathMorph использует Calm Blue UI как утверждённый канонический visual baseline web-интерфейса. Первый продуктовый путь — безопасная загрузка `.xmcd`/`.mcdx`, настройка конвертации и получение редактируемого DOCX/OMML. Этап 154 реализует публичную landing shell; полный UI flow ещё не реализован: интерактивные upload/converter states остаются последующими этапами. Система основана на desktop-референсе светлого интерфейса с боковой навигацией и карточками объектов размером 1800 × 822 px, но не наследует его бренд, предметную область или локализацию.

Интерфейс должен ощущаться спокойным, доброжелательным, надёжным и современным, но не демонстративно технологичным. Светлые поверхности, заметное количество воздуха и ясная иерархия помогают пользователю понять экран без предварительного обучения. В каждом локальном контексте должно быть одно визуально доминирующее действие.

Дизайн лучше всего подходит продуктам с содержательными рабочими сценариями. Для игр, иммерсивных промосайтов, luxury-брендов, экспериментальных медиа и сверхплотных профессиональных терминалов потребуется отдельная визуальная адаптация. Базовые токены, правила доступности и контракт интеграции при этом можно сохранить.

### Продуктовое применение MathMorph

Визуальная система применяется к следующим зонам продукта:

- public landing с кратким объяснением конвертации, privacy, API и тарифов;
- authentication и account recovery;
- основной converter flow: выбор файла, безопасная валидация, настройки, запуск и progress;
- diagnostics неподдерживаемых или частично преобразованных элементов;
- result screen со скачиванием, отчётом и правилами хранения;
- история документов и задач конвертации;
- account, retention, locale и security settings;
- usage и API dashboard, keys, SDK examples и будущие webhooks;
- support, status, changelog и юридическая информация.

Landing использует компактную top navigation, а аутентифицированное приложение — application shell с sidebar. Все пользовательские строки проходят через i18n; целевые локали — украинская, русская и английская. Содержимое документов не используется как декоративный материал и не попадает в telemetry, логи или analytics UI.

### Нормативные уровни

- **Core** — обязательное ядро: семантические цвета, типографика, интервалы, состояния и доступность.
- **Pattern** — рекомендуемый переиспользуемый компонент или способ компоновки.
- **Recipe** — необязательная композиция для конкретного типа экрана.

Проект может не использовать ненужный `Pattern` или `Recipe`. Отклонение от `Core` должно быть осознанно зафиксировано в локальном дизайн-контракте проекта.

### Визуальные принципы

- Иерархия строится размером, весом и интервалами, а затем цветом.
- Белые поверхности отделяются тонкой границей, а не тяжёлой тенью.
- Связанные элементы стоят ближе друг к другу, чем соседние смысловые группы.
- Декоративная графика не конкурирует с содержимым.
- Интерфейс не уплотняется только ради размещения большего количества данных.
- Доменная семантика не зашивается в универсальный компонент.
- Цвет никогда не является единственным носителем смысла.

### Контракт интеграции без конфликтов

В MathMorph действует следующий приоритет:

1. прямые требования пользователя и канонические требования MathMorph;
2. этот `docs/DESIGN.md`;
3. существующие доступные компоненты и семантические токены web-приложения;
4. переносимый Calm Blue UI baseline, из которого создан этот контракт;
5. исходный визуальный референс.

Если MathMorph уже имеет эквивалентный token или компонент, его нужно сопоставить с этим документом, а не создавать дубликат. При конфликте реализация web-приложения приводится к этому контракту либо отклонение явно фиксируется в локальной документации.

Машинные токены во frontmatter используют имена, предусмотренные спецификацией `DESIGN.md`. В коде их runtime-представление изолируется namespace `cbui`:

```css
[data-design-system="cbui"] {
  --cbui-color-primary: #4767f5;
  --cbui-color-on-surface: #090909;
  --cbui-color-surface: #ffffff;
  --cbui-radius-control: 14px;
}
```

- Не объявлять runtime-токены глобально в `:root`, если у приложения уже есть собственная тема.
- Не использовать глобальные селекторы `button`, `input`, `a`, `h1` и аналогичные.
- Не добавлять глобальный CSS reset вместе с дизайном.
- Ограничивать стили контейнером `[data-design-system="cbui"]`, CSS Modules, Shadow DOM или локальным механизмом проекта.
- При наличии проектной дизайн-системы выполнять token mapping вместо копирования значений.
- Не переопределять сторонние компоненты по внутренним class names.
- Имена публичных компонентов при необходимости делать уникальными: `CbuiButton`, `CbuiCard`, `CbuiSidebar`.
- Логотип, тексты, изображения и доменные статусы всегда принадлежат целевому проекту.
- Экспортированные CLI-переменные нельзя слепо подключать глобально; их нужно сопоставить или поместить в область темы проекта.

### Карта адаптации MathMorph

| Вопрос | Решение MathMorph |
|---|---|
| Контейнер темы | `[data-design-system="cbui"]` в web-root |
| Основной accent | `{colors.primary}` |
| Шрифт | Нормативный font stack из frontmatter; webfont Inter необязателен |
| Радиусы | Шкала `rounded` из frontmatter |
| Иконки | Один outline-набор; конкретная библиотека выбирается при UI-этапе |
| Application shell | Top navigation для landing; sidebar для authenticated app |
| Локаль | i18n: украинская, русская и английская |
| Color theme | `light`, `dark`, `system`; `system` следует настройке OS |
| Sensitive content | Никогда не использовать содержимое документов в telemetry или декоративных previews |

### Архитектура системы

```text
Core
├── colors, typography, spacing and shapes
├── elevation and motion
└── accessibility

Patterns
├── buttons and form controls
├── navigation, cards and data rows
├── badges, feedback and overlays
└── media and icons

Recipes
├── application shell
├── entity catalog and detail page
├── form flow
└── dashboard

Project adapter
├── brand, logo and locale
├── domain content and statuses
├── framework bindings
└── token mapping
```

## Colors

Палитра строится на почти чёрном тексте, белых поверхностях, мягком сером фоне и одном ярком сине-фиолетовом accent. Машинные значения во frontmatter нормативны; описание ниже объясняет их роли.

- **Primary {colors.primary}:** главное действие, активная навигация, ссылки и смысловые акценты.
- **Primary hover {colors.primary-hover}:** hover основного действия.
- **Primary pressed {colors.primary-pressed}:** pressed и active.
- **Primary soft {colors.primary-soft}:** выбранные строки и мягкие badges.
- **On primary {colors.on-primary}:** текст и иконки поверх основного accent.
- **On surface {colors.on-surface}:** основной текст и заголовки.
- **Secondary {colors.secondary}:** metadata и вторичные подписи.
- **Surface {colors.surface}:** карточки, sidebar и controls.
- **Page surface {colors.surface-page}:** фон рабочей области.
- **Subtle surface {colors.surface-subtle}:** нейтральные badges и вложенные зоны.
- **Border {colors.border}:** обычные границы и разделители.
- **Strong border {colors.border-strong}:** усиленная граница в hover или выделенном состоянии.
- **Success {colors.success}:** подтверждённый успешный статус.
- **Warning {colors.warning}:** внимание или срочность.
- **Error {colors.error}:** ошибка или разрушительное действие.
- **Info {colors.info}:** информационное состояние.
- **Overlay {colors.overlay}:** подложка modal и drawer.
- **Focus {colors.focus}:** внешнее focus-ring.

### Dark theme

Тёмная тема использует холодные графитово-синие поверхности вместо чистого чёрного. Это снижает резкий контраст больших областей, сохраняя высокую читаемость текста, формул и статусов.

- **Dark primary {colors.dark-primary}:** главное действие, активная навигация и ссылки.
- **Dark primary hover {colors.dark-primary-hover}:** hover основного действия.
- **Dark primary pressed {colors.dark-primary-pressed}:** pressed и active.
- **Dark primary soft {colors.dark-primary-soft}:** выбранные строки и мягкие badges.
- **Dark on primary {colors.dark-on-primary}:** тёмный текст поверх светлого accent.
- **Dark on surface {colors.dark-on-surface}:** основной текст и формулы.
- **Dark secondary {colors.dark-secondary}:** metadata и вторичные подписи.
- **Dark surface {colors.dark-surface}:** карточки, sidebar и controls.
- **Dark page {colors.dark-surface-page}:** основной фон страницы.
- **Dark subtle {colors.dark-surface-subtle}:** вложенные нейтральные зоны.
- **Dark elevated {colors.dark-surface-elevated}:** dropdown, popover, modal и drawer.
- **Dark border {colors.dark-border}:** обычные границы и разделители.
- **Dark strong border {colors.dark-border-strong}:** hover и усиленная граница.
- **Dark success {colors.dark-success}:** подтверждённый успешный результат.
- **Dark warning {colors.dark-warning}:** ограничения и неподдерживаемые элементы.
- **Dark error {colors.dark-error}:** ошибка или разрушительное действие.
- **Dark info {colors.dark-info}:** ход безопасно продолжающейся операции.
- **Dark overlay {colors.dark-overlay}:** подложка modal и drawer.
- **Dark focus {colors.dark-focus}:** внешнее focus-ring.

В тёмной теме preview пользовательского документа может сохранять собственный светлый «бумажный» canvas, если это честно отражает экспортируемый DOCX. Такой canvas отделяется border и подписью, а не перекрашивается автоматически. Формулы, diagnostics и application chrome используют dark tokens.

### Выбор темы и предотвращение flash

```css
[data-design-system="cbui"][data-theme="dark"] {
  color-scheme: dark;
  --cbui-color-primary: #9baeff;
  --cbui-color-on-surface: #f4f6fb;
  --cbui-color-surface: #171b24;
  --cbui-color-surface-page: #0f1218;
  --cbui-color-border: #303847;
}
```

- Пользователю доступны `light`, `dark` и `system`; начальное значение — `system`.
- `system` следует `prefers-color-scheme` и реагирует на его изменение без перезагрузки.
- Явный выбор пользователя имеет приоритет над OS и сохраняется локально; для авторизованного пользователя он может синхронизироваться с account settings.
- Theme attribute и `color-scheme` устанавливаются до первой отрисовки, чтобы не было flash неправильной темы.
- Иконка переключателя дополняется текстом или accessible name и сообщает текущее состояние.
- Переключение не сбрасывает форму, upload, conversion progress, filters или scroll position.
- При первой загрузке цветовая смена не анимируется; последующее переключение использует только короткий transition и уважает reduced motion.

### Дополнительные режимы вместо множества тем

Цветовая тема — только одна ось персонализации. Более продуманная система использует независимые режимы, которые можно комбинировать:

| Ось | Значения | Назначение |
|---|---|---|
| Color theme | `light`, `dark`, `system` | Цветовые поверхности и семантические роли |
| Contrast | `normal`, `high`, OS `forced-colors` | Читаемость и различимость controls |
| Density | `comfortable`, `compact` | Количество данных без изменения font scale |
| Motion | `normal`, `reduced` | Интенсивность переходов и skeleton |
| Workspace | `standard`, `focus` | Полный application chrome или сосредоточенная конвертация |
| Output preview | `application`, `paper` | Интерфейсная поверхность или честный preview документа |

Не создавать отдельные темы вроде «dark compact focus». Комбинация собирается из независимых attributes и media preferences:

```text
data-theme="dark"
data-contrast="normal"
data-density="compact"
data-workspace-mode="focus"
prefers-reduced-motion: reduce
```

Обязательный первый набор: `light` / `dark` / `system`, безопасное поведение в `forced-colors` и `prefers-reduced-motion`. `compact` полезен для history и API dashboard; `focus` — для converter flow. Эти режимы добавляются только после проверки реальной пользы и не должны менять business semantics.

Правила использования:

- Статус сопровождается текстом, иконкой или accessible name.
- Белый текст на accent разрешён только после проверки contrast.
- Brand color проекта может заменить `{colors.primary}`, если hover, pressed и focus пересчитаны вместе с ним.
- Dark mode использует отдельную семантическую таблицу и не создаётся простой инверсией.
- Значения `error`, `overlay` и `focus` добавлены как системные, даже если не показаны на исходном референсе.
- В MathMorph `success` означает завершённую операцию, `warning` — ограничения или неподдерживаемые элементы, `error` — неуспешную валидацию или конвертацию, а `info` — ход безопасно продолжающейся операции. Status всегда сопровождается стабильным кодом и локализованным текстом.

## Typography

Интерфейс использует нейтральный современный sans-serif. Нормативный font stack задан в frontmatter. Если проект уже использует читаемый sans-serif, он имеет приоритет через карту адаптации. Wordmark и декоративный display-font не заменяют интерфейсный шрифт.

| Token | Роль |
|---|---|
| `{typography.headline-display}` | Редкий крупный заголовок |
| `{typography.headline-lg}` | Заголовок страницы |
| `{typography.headline-md}` | Заголовок секции или карточки |
| `{typography.headline-sm}` | Подсекция |
| `{typography.body-lg}` | Выделенный основной текст |
| `{typography.body-md}` | Базовый текст и controls |
| `{typography.body-sm}` | Вторичная информация |
| `{typography.label-lg}` | Кнопки и активная навигация |
| `{typography.label-md}` | Labels и badges |
| `{typography.label-sm}` | Короткие подписи |

Правила:

- Основной текст выравнивается по левому краю.
- Обычный регистр предпочтительнее капса.
- Строки основного текста ограничиваются примерно `45–75` символами.
- Масштабирование текста до `200%` не скрывает функции.
- Длинные пользовательские значения не обрезаются без пути к полной версии.
- На одном экране обычно достаточно двух-трёх font weights.

## Layout

Layout использует 4px micro-step и согласованную шкалу интервалов из frontmatter. Смысловые группы отделяются пространством, а не лишними рамками. Для ближайшего наблюдаемого размера выбирается ближайший token: например, визуальный отступ `30px` обычно реализуется как `{spacing.xxl}` (`32px`), если pixel-perfect совпадение не является отдельным требованием.

### Базовые размеры

- Минимальная область нажатия: `44 × 44px`.
- Обычная высота control: `48px`.
- Крупное основное действие: `56px`.
- Обычная icon button: `48 × 48px`.
- Крупная icon button рядом с CTA: `56 × 56px`.
- Минимальный промежуток между независимыми touch targets: `{spacing.xs}`.
- Внутренний отступ карточки: `{spacing.xl}`–`{spacing.xxl}` на desktop и `{spacing.md}`–`{spacing.lg}` на compact.

### Application shell — Recipe

Исходный характер хорошо передаёт двухчастный shell:

- sidebar на desktop: `272–280px`;
- рабочая область: `{colors.surface-page}`;
- основной контент: `max-width: 960px` для ленты и `1200–1280px` для dashboard или grid;
- горизонтальный page gutter: `clamp(16px, 4vw, 48px)`;
- вертикальный интервал между крупными секциями: `{spacing.lg}`–`{spacing.xxl}`;
- sidebar использует белую поверхность без тяжёлой тени.

```text
┌────────────────────┬───────────────────────────────────────────────┐
│ Brand              │ Page header                                   │
│                    │                                               │
│ Primary navigation │ Main content                                  │
│                    │ ┌───────────────────────────────────────────┐ │
│ Secondary group    │ │ Section / card / data view                │ │
│                    │ └───────────────────────────────────────────┘ │
│ Account / support  │                                               │
└────────────────────┴───────────────────────────────────────────────┘
```

В MathMorph public landing использует top navigation, а authenticated converter, history, settings и API dashboard — sidebar. Sidebar не переносится на landing автоматически. На compact-экранах он заменяется menu или drawer без потери текущего раздела и primary action.

### Адаптивность

Стартовые ориентиры могут быть сопоставлены с существующей системой проекта:

```text
compact: < 640px
medium:  640–959px
wide:    960–1279px
x-wide:  >= 1280px
```

- Layout перестраивается, когда содержимое перестаёт помещаться, а не по названию устройства.
- Sidebar на medium может сворачиваться, а на compact заменяется menu, drawer или bottom navigation.
- Многоколоночная карточка превращается в одну колонку.
- Primary action на compact обычно занимает доступную ширину.
- Группы метрик переносятся целиком.
- Горизонтальная прокрутка всей страницы не допускается при viewport от `320px`.
- Декоративный паттерн можно скрывать на compact.
- Container queries предпочтительны для переиспользуемых карточек, если стек их поддерживает.

## Elevation & Depth

Глубина создаётся тональными слоями и границами. Фон страницы использует `{colors.surface-page}`, основной контент — `{colors.surface}`, а вложенные нейтральные зоны — `{colors.surface-subtle}`. Обычные карточки и sidebar не используют заметную тень.

```text
flat surface:      no shadow; 1px solid {colors.border}
floating surface: 0 10px 30px #0909091A
overlay surface:  0 20px 60px #09090929
backdrop:         {colors.overlay}
```

- `floating surface` предназначена для dropdown и popover.
- `overlay surface` предназначена для modal и drawer.
- Elevation не используется как замена ясной иерархии.
- Hover не меняет размеры или положение поверхности.

## Shapes

Форма элементов мягкая, но не капсульная. Нормативные радиусы находятся в группе `rounded` во frontmatter.

- `{rounded.sm}` — небольшие badges и вложенные элементы.
- `{rounded.md}` — buttons, inputs, navigation rows и media preview.
- `{rounded.lg}` — cards, panels, modal и drawer.
- `{rounded.full}` — avatar, status dot и действительно капсульные badges.
- `{rounded.none}` — таблицы и полноширинные разделители, если этого требует композиция.

Правила:

- Не смешивать случайные радиусы на одном экране.
- Изменение толщины границы в hover не должно менять геометрию.
- Квадратное media preview использует `object-fit: cover` и `{rounded.md}`.
- Avatar обычно круглый.
- Controls и cards сохраняют отличимые уровни radius: `md` и `lg`.

## Components

### Buttons

Обязательные варианты:

- `primary` — залитый `{colors.primary}`, одно главное действие в локальном контексте;
- `secondary` — белая поверхность и primary-border;
- `tertiary` — текстовое действие без отдельной поверхности;
- `danger` — только для подтверждённо разрушительных действий;
- `icon` — всегда имеет accessible name.

Все варианты поддерживают `default`, `hover`, `focus-visible`, `pressed`, `disabled` и `loading`. Во время loading ширина кнопки не меняется. Иконка перед текстом имеет размер `20–24px`.

### Form controls

Input, textarea, select, checkbox, radio и switch используют единый набор состояний:

- label находится снаружи и не заменяется placeholder;
- helper text и error message занимают предсказуемое место;
- обычная высота input и select — `48px`;
- фон — `{colors.surface}`, граница — `{colors.border}`, radius — `{rounded.md}`;
- focus показывает primary-border и внешнее focus-ring;
- error показывает error-border и текстовую причину;
- disabled имеет корректный нативный атрибут;
- валидация не зависит только от цвета.

### Cards

Универсальная карточка использует белую поверхность, границу `1px`, `{rounded.lg}`, внутренний отступ `{spacing.xl}`–`{spacing.xxl}` и не имеет тени в основном потоке. Header, content и actions создаются только при наличии соответствующих данных.

Карточка не считается ссылкой автоматически. Если кликабельна вся поверхность, вложенные действия остаются корректными для клавиатуры и assistive technologies.

### Navigation

- Пункт содержит иконку и текст либо только текст.
- Активное состояние использует `{colors.primary-soft}` и primary-текст.
- Hover слабее активного состояния.
- Группировка отмечается интервалом, заголовком или разделителем.
- Иконка без текста допустима при понятном значении и наличии tooltip или accessible name.
- Активность определяется не только цветом, но также фоном, маркером или весом текста.

### Chips, badges и statuses

- Chip может выражать выбор, фильтр или короткое действие.
- Badge короткий и не содержит основное действие.
- Нейтральный badge использует `{colors.surface-subtle}`.
- Смысловой badge использует мягкий фон и текст достаточного contrast.
- Status dot дублируется доступным текстом.
- Значения статусов определяет проект: зелёный индикатор не означает «онлайн» сам по себе.

### Lists и data display

- Метрики объединяются в компактные смысловые группы.
- List items используют согласованные divider и leading/trailing zones.
- Таблица используется для сравнения однотипных структурированных данных.
- Карточки используются, когда у объектов есть изображение, описание и несколько действий.
- На узком экране таблица прокручивается с явным affordance или превращается в строки-карточки без потери labels.
- Pagination, sorting и filters сохраняют состояние предсказуемо.

### Feedback и overlays

Самодостаточная реализация предусматривает:

- inline error рядом с источником проблемы;
- alert для сообщения уровня секции;
- toast для краткого результата завершённого действия;
- skeleton для первичной загрузки структуры;
- progress indicator для длительного действия;
- empty state с объяснением и одним следующим шагом;
- modal для короткого блокирующего решения;
- drawer для вспомогательного контекста;
- tooltip только для краткого пояснения, не для критичной информации.

Modal и drawer удерживают keyboard focus, закрываются предсказуемо и возвращают focus инициатору.

### Media и iconography

- Meaningful image получает содержательный `alt`, декоративное — пустой `alt`.
- Иконки выбираются из одного outline-набора с округлыми окончаниями.
- Базовый размер иконки: `20–24px`, в навигации допустим `24–28px`.
- Не смешивать outline, filled и duotone внутри одной функциональной группы.
- Логотип и wordmark всегда заменяются активами целевого проекта.
- Декоративный outline-паттерн допустим только при низком contrast и тематическом соответствии.

### MathMorph product patterns

**File upload zone**

- Ясно перечисляет поддерживаемые форматы и ограничения до выбора файла.
- Поддерживает click, keyboard activation и drag-and-drop без обязательности drag gesture.
- Показывает имя, размер и локальную validation status без чтения содержимого в analytics.
- Отделяет client-side UX validation от серверной security validation.
- При потере сети сохраняет понятный recovery path и не обещает завершённую загрузку.

**Conversion settings**

- Группирует параметры DOCX, equation handling, graphics и retention по смыслу.
- Default values безопасны и объяснимы.
- Параметр, влияющий на редактируемость или потерю данных, содержит постоянное пояснение, а не только tooltip.
- Запуск конвертации остаётся единственным primary action секции.

**Job progress**

- Показывает стабильный status, progress, текущий этап и возможность безопасной отмены, если она поддерживается.
- Не симулирует точный процент, когда backend его не предоставляет.
- Retry и resume не создают визуально дублирующую задачу.
- Потеря сети, timeout и истёкшая download URL имеют отдельные recovery actions.

**Diagnostics**

- Unsupported и partial-conversion outcomes показываются явно и не маскируются зелёным success-state.
- Каждая диагностика имеет severity, стабильный code, локализованное объяснение и при возможности location/context.
- Длинный отчёт поддерживает фильтрацию и скачивание; критичные предупреждения не скрываются в collapsed area по умолчанию.

**Result**

- Главное действие — скачать проверенный результат.
- Рядом видны формат, размер, время готовности, срок хранения и checksum, если он доступен.
- Partial success визуально отличается от полного success.
- Удаление документа и повторная конвертация являются отдельными secondary или danger actions.

**History и API dashboard**

- История использует data rows или таблицу, а не карточки с портретным media.
- API keys показываются полностью только один раз после создания; далее отображаются безопасно сокращённо.
- Usage metrics не содержат содержимое документов и не создают ложной точности.

### Motion

```text
interactive feedback: 120ms
content transition:   180ms
large transition:     240ms
easing:               cubic-bezier(0.2, 0, 0, 1)
```

- Motion объясняет смену состояния, а не украшает интерфейс.
- Hover и press не перемещают layout.
- Skeleton не должен агрессивно мерцать.
- `prefers-reduced-motion: reduce` отключает необязательные перемещения и сокращает переходы.
- Autoplay со звуком запрещён.

### Каталог объектов — Recipe из референса

Recipe подходит специалистам, товарам, курсам, объявлениям, кандидатам, помещениям и другим сравнимым сущностям.

- Одна колонка карточек с интервалом около `{spacing.lg}`.
- Media слева, информация справа.
- Квадратное preview около `256–272px`.
- Gap между preview и содержимым: `{spacing.xl}`.
- Последовательность: title, optional badge, metadata, metrics, description, contextual action.
- Описание ограничивается по высоте, но полная версия доступна на detail page.
- Действия: contextual value, secondary action, primary action.
- Высота карточки определяется содержимым и не фиксируется жёстко.

Доменная модель остаётся внешней:

| Пример референса | Универсальная роль |
|---|---|
| Имя объекта | `entity.title` |
| Тип и metadata | `entity.metadata` |
| Цена, рейтинг, количество | `entity.metrics[]` |
| Ближайшее значение | `entity.contextualValue` |
| Вторичное действие | `entity.secondaryAction` |
| Главное действие | `entity.primaryAction` |
| Бейдж | `entity.badge` |
| Status dot | `entity.status` с проектной семантикой |

Компонент не содержит слова, валюту, формат дат, иконки или business rules конкретной предметной области.

### Дополнительные Recipes

**Detail page:** page header с title и primary action; summary card для ключевых атрибутов; основной текст комфортной длины; secondary data в отдельных секциях.

**Form flow:** одна логическая группа на секцию; видимые labels; primary action после полей; ошибки у поля и summary сверху для длинной формы.

**Dashboard:** одинаковые summary cards; главный insight сильнее вторичных метрик; charts имеют title, legend и текстовый эквивалент значений.

### Состояния страниц

Экран с асинхронными данными проектируется минимум в состояниях:

- `loading` — структура узнаваема, действия не скачут после загрузки;
- `content` — обычное рабочее состояние;
- `empty-first-use` — объяснение назначения и первый шаг;
- `empty-filtered` — изменение или сброс фильтров;
- `partial` — доступные данные полезны при частичной ошибке;
- `error-recoverable` — понятная причина и повтор действия;
- `offline`, если продукт зависит от постоянной сети;
- `permission-denied`, если присутствуют роли и доступы.

Ошибка вторичного блока не должна скрывать всю страницу.

### Контент и локализация

- Язык, валюта, часовой пояс, формат даты и plural rules задаются проектом.
- UI copy хранится отдельно от компонентов.
- Компоненты выдерживают расширение текста примерно до `30–40%`.
- Даты и деньги форматируются средствами локали, а не конкатенацией строк.
- Направление `RTL` учитывается через logical CSS properties, если входит в аудиторию проекта.
- Подписи действий называют результат: «Сохранить изменения», а не «ОК».

Язык, валюта и названия исходного изображения не являются требованиями baseline.

### Accessibility

- Целевой уровень — WCAG 2.2 AA.
- Интерфейс полностью доступен с клавиатуры.
- Порядок focus соответствует визуальному порядку.
- `focus-visible` заметен на всех поверхностях.
- Минимальная область нажатия — `44 × 44px`.
- Семантические HTML-элементы предпочтительнее имитации через `div`.
- Icon-only controls имеют accessible name.
- Images, charts и statuses имеют текстовый эквивалент.
- Формы связывают label, helper и error корректными атрибутами.
- Видео с речью предоставляет captions и не запускается автоматически со звуком.
- Контент работает при zoom `200%` и reflow на ширине `320 CSS px`.
- Contrast проверяется после изменения primary или status colors.

### Технологическая независимость

Дизайн не требует React, Vue, Svelte, Tailwind, shadcn/ui или конкретной icon library.

- Существующий primitive переиспользуется, если удовлетворяет контракту.
- CSS-in-JS, utility classes и обычный CSS равноправны.
- Системные tokens хранятся в одном каноническом месте.
- Magic numbers допустимы для уникальной композиции, но не вместо системных tokens.
- Сторонняя библиотека адаптируется через публичный theme API без хрупких внутренних селекторов.

## Do's and Don'ts

### Do

- Использовать `{colors.primary}` для одного главного действия в локальном контексте.
- Группировать контент интервалами и тонкими границами.
- Сохранять согласованные радиусы, spacing и размеры controls.
- Проектировать loading, empty, error, disabled и permission states.
- Проверять keyboard navigation, focus, contrast, zoom и reflow.
- Изолировать реализацию через namespace или token mapping.
- Заменять brand, locale и business semantics данными целевого проекта.
- Выполнять визуальное сравнение с утверждённым проектным макетом.
- Показывать неподдерживаемые элементы и partial conversion как явную диагностику.
- Сохранять различие между upload, validation, conversion, result и retention states.
- Проверять каждую семантическую пару отдельно в light и dark themes.
- Уважать OS preferences для color scheme, contrast, forced colors и reduced motion.

### Don't

- Не использовать цвет как единственный носитель смысла.
- Не смешивать случайные радиусы, icon styles и тяжёлые тени.
- Не создавать несколько визуально равных primary actions рядом.
- Не зашивать язык, валюту, формат дат или business rules в универсальный компонент.
- Не подключать экспортированные CSS tokens глобально без проверки конфликтов.
- Не переопределять чужие компоненты по внутренним class names.
- Не скрывать критичную информацию только в tooltip.
- Не считать sidebar или каталог обязательной структурой любого проекта.
- Не показывать partial conversion как безусловный success.
- Не использовать preview содержимого пользовательского документа как фоновый или маркетинговый декор.
- Не создавать отдельную именованную тему для каждой комбинации color, density и workspace mode.
- Не затемнять screenshots или document previews CSS-фильтром, если это искажает результат.

### Проверка перед внедрением

- [ ] Заполнена карта адаптации.
- [ ] Выбраны только нужные Patterns и Recipes.
- [ ] Проверены существующие tokens и компоненты проекта.
- [ ] Определён уникальный runtime namespace или token mapping.
- [ ] Определены logo, font, icons и locale; проверены `light`, `dark` и `system`.
- [ ] Проверены `forced-colors`, `prefers-reduced-motion` и theme flash до hydration.
- [ ] Проверены viewport от `320px` до целевого desktop.
- [ ] Проверены длинные строки, пустые данные и ошибки.
- [ ] Нет глобальных style overrides, затрагивающих чужие экраны.
- [ ] Пройден официальный `@google/design.md` lint.

### Известные границы

Документ самодостаточен как дизайн-контракт и основа реализации, но не заменяет уникальную бренд-айдентику, продуктовые требования, пользовательские исследования, точные макеты сложных экранов, отраслевые юридические требования, браузерную проверку и отдельные спецификации email, native mobile или print.

Эти части добавляются в локальный дизайн-контракт выбранного проекта без изменения переносимого ядра, если в этом нет необходимости.

### История изменений

| Версия системы | Дата | Изменение |
|---|---|---|
| `0.1` | 2026-08-14 | Зафиксирован исходный desktop-референс каталога |
| `1.0` | 2026-08-14 | Дизайн отделён от бренда и предметной области; добавлены Core, Patterns, Recipes и бесконфликтная интеграция |
| `1.1` | 2026-08-14 | Добавлен нормативный YAML frontmatter и документ приведён к структуре Google Labs `design.md` specification `alpha` |
| `1.2` | 2026-08-14 | Calm Blue UI принят для MathMorph; добавлены product flows, status semantics, безопасные upload/conversion patterns и проектная карта адаптации |
| `1.3` | 2026-08-14 | Добавлена полноценная dark palette, выбор `light` / `dark` / `system` и независимые contrast, density, motion, workspace и preview modes |
| `1.4` | 2026-08-20 | Зафиксирована реализация stage 154 public landing shell и её section-level visual concepts; upload/converter flow оставлен последующим этапам |
