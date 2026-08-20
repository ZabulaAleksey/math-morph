# Дорожная карта — 304 атомарных этапа

> Каждый этап должен допускать независимое тестирование. Исполняемый текст prompts находится в `docs/PROMPTS.md`.

**Текущий статус:** этапы 001–093, 095–100 и 143–148, а также независимый frontend-этап 154 реализованы и проверены в stacked feature-ветке. Этап 094 заблокирован до versioned MathType import/edit `PASS`; этапы 101–142 остаются planned и не объявляются выполненными. Первый локальный legacy XMCD→DOCX CLI path доступен на этапе 148. Подробные доказательства находятся в `docs/TRACEABILITY.md`.

## Основа — 001–010

- **001** — `monorepo`
- **002** — `canonical docs`
- **003** — `minimal design contract`
- **004** — `root AGENTS`
- **005** — `parser AGENTS`
- **006** — `math-engine AGENTS`
- **007** — `DOCX AGENTS`
- **008** — `frontend AGENTS`
- **009** — `API AGENTS`
- **010** — `tests AGENTS`

## Fixtures — 011–014

- **011** — `fixture taxonomy`
- **012** — `fixture manifest`
- **013** — `fixture validator`
- **014** — `corrupted/security starter fixtures`

## Определение формата — 015–018

- **015** — `InputFormat/FormatDetector`
- **016** — `XMCD content detection`
- **017** — `MCDX content detection`
- **018** — `extension/content mismatch diagnostics`

## Контейнер MCDX — 019–025

- **019** — `safe container open`
- **020** — `path traversal defense`
- **021** — `ZIP limits`
- **022** — `container manifest`
- **023** — `worksheet part discovery`
- **024** — `embedded resources metadata`
- **025** — `unknown part handling`

## Структура XMCD/XML — 026–035

- **026** — `namespace/schema metadata`
- **027** — `worksheet metadata`
- **028** — `region discovery`
- **029** — `region coordinates/layout`
- **030** — `deterministic ordering`
- **031** — `text region`
- **032** — `math region`
- **033** — `plot region`
- **034** — `image region`
- **035** — `table/program/unknown regions`

## Математическое AST — 036–054

- **036** — `minimal scalar/operators AST`
- **037** — `AST snapshots`
- **038** — `Definition`
- **039** — `Evaluation`
- **040** — `FunctionCall`
- **041** — `FunctionDefinition`
- **042** — `unary operators`
- **043** — `grouping`
- **044** — `index/subscript`
- **045** — `matrix`
- **046** — `vector`
- **047** — `range`
- **048** — `integral`
- **049** — `derivative`
- **050** — `sum/product`
- **051** — `comparisons`
- **052** — `booleans`
- **053** — `units`
- **054** — `UnsupportedNode`

## Document IR — 055–061

- **055** — `versioned DocumentIR/PageIR/TableIR/MetadataIR`
- **056** — `TextBlock`
- **057** — `EquationBlock`
- **058** — `ImageBlock`
- **059** — `PlotBlock`
- **060** — `DiagramBlock`
- **061** — `layout model`

## Основы DOCX — 062–069

- **062** — `minimal valid DOCX`
- **063** — `single text paragraph`
- **064** — `multiple paragraphs`
- **065** — `basic formatting`
- **066** — `image block`
- **067** — `image sizing`
- **068** — `page dimensions`
- **069** — `DOCX structural validator`

## Уравнения Word — 070–089

- **070** — `EquationExporter`
- **071** — `WordEquationExporter`
- **072** — `number`
- **073** — `variable`
- **074** — `add/subtract`
- **075** — `multiply`
- **076** — `fraction`
- **077** — `powers` — `verified`
- **078** — `roots` — `verified`
- **079** — `subscripts` — `verified`
- **080** — `sub+sup` — `verified`
- **081** — `functions` — `verified`
- **082** — `brackets` — `verified`
- **083** — `matrices` — `verified`
- **084** — `integrals` — `verified`
- **085** — `derivatives` — `verified`
- **086** — `sum/product` — `verified`
- **087** — `nested equations regression` — `verified`
- **088** — `manual Word reference validation` — `verified`

## Подготовка MathType — 089–094

- **089** — `backend enum/config` — `verified`
- **090** — `MathML renderer` — `verified`
- **091** — `MathML snapshots` — `verified`
- **092** — `experimental MathType adapter` — `verified`
- **093** — `compatibility doc` — `verified` (`live compatibility: UNVERIFIED`)
- **094** — `feature-gated backend selection` — `blocked by versioned live evidence`

## Преобразование нотации — 095–099

- **095** — `transformation pipeline` — `verified`
- **096** — `Definition presentation rule` — `verified`
- **097** — `SymbolMappingRegistry` — `verified`
- **098** — `NotationProfile` — `verified`
- **099** — `semantic-preservation regression` — `verified`

## Граф зависимостей — 100–105

- **100** — `SymbolTable` — `verified`
- **101** — `variable references`
- **102** — `dependency graph`
- **103** — `worksheet evaluation order`
- **104** — `undefined-variable diagnostic`
- **105** — `circular dependency diagnostic`

## Подстановка — 106–111

- **106** — `simple substitution`
- **107** — `recursive substitution`
- **108** — `depth limit`
- **109** — `EvaluationTrace`
- **110** — `display modes`
- **111** — `PrecisionPolicy`

## Комплексные числа — 112–122

- **112** — `complex value`
- **113** — `algebraic representation`
- **114** — `polar representation`
- **115** — `algebraic→polar`
- **116** — `polar→algebraic`
- **117** — `multiplication trace`
- **118** — `division trace`
- **119** — `addition`
- **120** — `subtraction`
- **121** — `output modes`
- **122** — `edge-case suite`

## Графики — 123–127

- **123** — `plot region semantics`
- **124** — `PlotIR metadata`
- **125** — `preview extraction`
- **126** — `plot image→DOCX`
- **127** — `plot fallback`

## Будущее ChartIR/Excel — 128–132

- **128** — `ChartIR`
- **129** — `series expression extraction`
- **130** — `reconstruction fixtures`
- **131** — `ChartExporter`
- **132** — `experimental Excel chart POC`

## Будущее диаграмм и Visio — 133–142

- **133** — `diagram detection`
- **134** — `diagram preview→DOCX`
- **135** — `DiagramIR`
- **136** — `shape-forensics report`
- **137** — `primitive shapes`
- **138** — `connector graph`
- **139** — `grouping`
- **140** — `DiagramExporter`
- **141** — `VSDX POC`
- **142** — `DiagramIR→VSDX POC`

## Единый конвейер — 143–147

- **143** — `ConversionPipeline` — `verified`
- **144** — `DiagnosticsCollector` — `verified`
- **145** — `severity model` — `verified`
- **146** — `ConversionReport fidelity statuses` — `verified`
- **147** — `partial conversion` — `verified`

## CLI — 148–153

- **148** — `minimal convert command` — `verified`
- **149** — `inspect command`
- **150** — `--format/exporter registry including Typst`
- **151** — `--complex-mode`
- **152** — `--precision`
- **153** — `JSON/Document IR report`

## Веб-интерфейс — 154–161

- **154** — `Next.js shell` — verified в `feature/stage-154-visible-nextjs-shell`
- **155** — `design compliance checklist`
- **156** — `dropzone`
- **157** — `file validation UI`
- **158** — `conversion settings UI`
- **159** — `conversion states`
- **160** — `converter Error Boundary`
- **161** — `localized error mapping`

## i18n — 162–165

- **162** — `next-intl infrastructure`
- **163** — `catalog split`
- **164** — `missing-key CI`
- **165** — `Ukrainian/Russian/English locale coverage`

## Backend FastAPI — 166–172

- **166** — `FastAPI skeleton with uv`
- **167** — `health endpoint`
- **168** — `PostgreSQL`
- **169** — `Alembic`
- **170** — `API error model`
- **171** — `request ID`
- **172** — `/api/v1 versioning`

## Аутентификация — 173–186

- **173** — `auth integration boundary`
- **174** — `registration`
- **175** — `password login`
- **176** — `email verification`
- **177** — `OIDC provider 1`
- **178** — `OIDC provider 2`
- **179** — `TOTP`
- **180** — `recovery codes`
- **181** — `passkeys`
- **182** — `email recovery`
- **183** — `phone recovery`
- **184** — `Telegram linking`
- **185** — `Telegram recovery`
- **186** — `auth abuse suite`

## Профиль — 187–190

- **187** — `basic profile`
- **188** — `conversion preferences`
- **189** — `save preferences`
- **190** — `conversion history metadata`

## Ключи API — 191–199

- **191** — `API-key model`
- **192** — `secure generation`
- **193** — `show secret once`
- **194** — `hashed storage`
- **195** — `revoke`
- **196** — `expiry`
- **197** — `scopes`
- **198** — `rate limiting`
- **199** — `usage statistics`

## Публичный API — 200–209

- **200** — `POST conversions`
- **201** — `durable async jobs/progress/correlation`
- **202** — `status`
- **203** — `result`
- **204** — `save/temporary TTL preference integration`
- **205** — `OpenAPI descriptions`
- **206** — `API docs page`
- **207** — `curl examples`
- **208** — `JavaScript example`
- **209** — `Python example`

## RabbitMQ и workers — 210–217

- **210** — `RabbitMQ broker`
- **211** — `Celery test worker`
- **212** — `conversion task`
- **213** — `retry policy`
- **214** — `dead-letter strategy`
- **215** — `timeout`
- **216** — `cancellation model`
- **217** — `idempotency/reconnect recovery`

## Конфиденциальность и zero knowledge — 218–232

- **218** — `threat model`
- **219** — `privacy ADR`
- **220** — `local WASM POC`
- **221** — `WASM parser/core`
- **222** — `browser conversion POC`
- **223** — `Web Worker`
- **224** — `client-side encryption`
- **225** — `encrypted storage`
- **226** — `privacy marker regression`
- **227** — `metadata minimization`
- **228** — `filename protection`
- **229** — `account vs key recovery`
- **230** — `Document Recovery Key`
- **231** — `explicit support sharing`
- **232** — `privacy claims checklist`

## Безопасность — 233–244

- **233** — `file size limits`
- **234** — `XML attack defenses`
- **235** — `ZIP bomb tests`
- **236** — `malformed corpus`
- **237** — `cargo-fuzz target`
- **238** — `auth rate limits`
- **239** — `API rate limits`
- **240** — `security headers`
- **241** — `CSP`
- **242** — `dependency audit`
- **243** — `secret scanning`
- **244** — `security regression suite`

## Оплата — 245–256

- **245** — `BillingProvider`
- **246** — `plans model`
- **247** — `quota engine`
- **248** — `Free quota`
- **249** — `Pro quota`
- **250** — `API accounting`
- **251** — `provider sandbox`
- **252** — `webhook verification`
- **253** — `subscription lifecycle`
- **254** — `failed payment behavior`
- **255** — `refund workflow`
- **256** — `billing history`

## Администрирование — 257–270

- **257** — `admin auth boundary`
- **258** — `role/permission model`
- **259** — `user list metadata`
- **260** — `conversion stats`
- **261** — `worker health`
- **262** — `queue stats`
- **263** — `parser error stats`
- **264** — `unsupported-node stats`
- **265** — `API usage metrics`
- **266** — `billing admin`
- **267** — `feature flags`
- **268** — `language metadata`
- **269** — `security event viewer`
- **270** — `admin privacy integration test`

## Наблюдаемость — 271–278

- **271** — `structured logging`
- **272** — `log redaction`
- **273** — `OpenTelemetry traces`
- **274** — `technical and product metrics without content`
- **275** — `conversion duration`
- **276** — `parser failure metrics`
- **277** — `worker metrics`
- **278** — `Sentry with sanitization`

## Производительность — 279–284

- **279** — `benchmark fixture classes`
- **280** — `parser benchmark`
- **281** — `DOCX benchmark`
- **282** — `memory benchmark`
- **283** — `regression thresholds`
- **284** — `parallel conversion benchmark`

## CI/CD — 285–295

- **285** — `lint CI`
- **286** — `Rust tests CI`
- **287** — `Python tests CI`
- **288** — `frontend tests CI`
- **289** — `integration CI`
- **290** — `fixture regression CI`
- **291** — `security CI`
- **292** — `artifact validation`
- **293** — `Docker build`
- **294** — `staging deployment`
- **295** — `production release gate`

## Масштабирование — 296–304

- **296** — `Docker Compose baseline`
- **297** — `stateless API verification`
- **298** — `multiple workers`
- **299** — `multiple API replicas`
- **300** — `load test`
- **301** — `storage abstraction`
- **302** — `benchmark-driven Redis cache`
- **303** — `Kubernetes POC`
- **304** — `worker autoscaling POC`
