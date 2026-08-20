# Правила API

- При каждом изменении аутентификации, авторизации, ключей API, загрузки или хранения прочитай `docs/SECURITY.md`.
- Версионируй endpoints под `/api/v1`, если ADR не меняет эту политику.
- Аутентифицируй И авторизуй каждый защищённый объект и действие.
- Используй структурированный код ошибки и ID запроса; не возвращай исходные stack traces или данные документа.
- Секрет ключа API показывается один раз; хранится только hash или verifier.
- Web/API/CLI должны переиспользовать core конвертации, а не дублировать семантику.
- Обеспечивай серверную проверку размера, квоты, частоты и параметров.
- Создание задачи должно быть идемпотентным там, где возможны повторные запросы.

## Retry и fallback

Следуй `docs/FALLBACKS.md` и глобальной Fallback Policy.

Retryable:
- transient network/service/storage failure;
- timeout только при безопасной идемпотентности;
- документированный temporary provider failure.

Non-retryable:
- invalid/corrupted input;
- unsupported version;
- authentication/authorization failure;
- hard quota;
- integrity/signature failure;
- resource/security limit.

Если предыдущая mutation имеет неизвестный результат:

reconcile → retry

а не:

retry → возможно создать дубликат.

Создание job/resource должно использовать stable operation ID
или idempotency key там, где возможен повтор.