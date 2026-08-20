import { landingContent as content } from "./content";
import { ArrowIcon, BrandMark, CheckIcon, Icon } from "./icons";
import { MobileNavigation } from "./mobile-navigation";
import { ThemeControl } from "./theme-control";

export default function HomePage() {
  return (
    <div className="cbui-page">
      <header className="cbui-header">
        <div className="cbui-container cbui-header__inner">
          <a aria-label={content.brand} className="cbui-brand" href="#top">
            <BrandMark />
            <span>{content.brand}</span>
          </a>

          <nav aria-label={content.navigationLabel} className="cbui-nav cbui-nav--desktop">
            {content.navigation.map((item) => (
              <a href={item.href} key={item.href}>{item.label}</a>
            ))}
          </nav>

          <div className="cbui-header__actions">
            <ThemeControl labels={content.theme} />
            <MobileNavigation
              items={content.navigation}
              label={content.navigationLabel}
              menuLabel={content.mobileNavigationLabel}
            />
          </div>
        </div>
      </header>

      <main id="main-content">
        <section className="cbui-hero" id="top" aria-labelledby="hero-title">
          <div className="cbui-hero__motif" aria-hidden="true" />
          <div className="cbui-container cbui-hero__grid">
            <div className="cbui-hero__copy">
              <h1 id="hero-title">{content.hero.title}</h1>
              <p>{content.hero.body}</p>
              <div className="cbui-actions">
                <a className="cbui-button cbui-button--primary" href="#converter">
                  {content.hero.primaryAction}
                  <ArrowIcon />
                </a>
                <a className="cbui-button cbui-button--secondary" href="#process">
                  {content.hero.secondaryAction}
                </a>
              </div>
              <p className="cbui-support-note">
                <span aria-hidden="true">i</span>
                {content.hero.supportNote}
              </p>
            </div>

            <div aria-label={content.hero.workflowLabel} className="cbui-workflow-card" role="img">
              <div className="cbui-workflow-card__rail">
                {content.hero.workflow.map((item, index) => (
                  <div className="cbui-workflow-node" key={item}>
                    <span><Icon name={index === 0 ? "document" : index === 1 ? "shield" : "download"} /></span>
                    <strong>{item}</strong>
                  </div>
                ))}
              </div>
              <div className="cbui-equation-grid">
                <div>
                  <span>{content.hero.sourcePreview}</span>
                  <div className="cbui-equation">f(x) = √(x² + 1)</div>
                </div>
                <div>
                  <span>{content.hero.resultPreview}</span>
                  <div className="cbui-equation cbui-equation--result">f(x) = √(x² + 1)<i aria-hidden="true" /></div>
                </div>
              </div>
              <div className="cbui-workflow-status">
                <CheckIcon />
                <span>{content.hero.resultStatus}</span>
              </div>
            </div>
          </div>

          <div className="cbui-container">
            <div className="cbui-trust-strip">
              {content.trustFacts.map((item) => (
                <article key={item.title}>
                  <span className="cbui-icon-box"><Icon name={item.icon} /></span>
                  <div>
                    <h2>{item.title}</h2>
                    <p>{item.body}</p>
                  </div>
                </article>
              ))}
            </div>
          </div>
        </section>

        <section aria-labelledby="features-title" className="cbui-section" id="features">
          <div className="cbui-container">
            <div className="cbui-section-heading cbui-section-heading--center">
              <h2 id="features-title">{content.features.title}</h2>
              <p>{content.features.body}</p>
            </div>
            <div className="cbui-card-grid">
              {content.features.items.map((item) => (
                <article className="cbui-feature-card" key={item.title}>
                  <span className="cbui-icon-box"><Icon name={item.icon} /></span>
                  <h3>{item.title}</h3>
                  <p>{item.body}</p>
                </article>
              ))}
            </div>
          </div>
        </section>

        <section aria-labelledby="process-title" className="cbui-section cbui-section--subtle" id="process">
          <div className="cbui-container">
            <div className="cbui-section-heading cbui-section-heading--center">
              <h2 id="process-title">{content.process.title}</h2>
            </div>
            <ol className="cbui-process-list">
              {content.process.steps.map((step, index) => (
                <li key={step.title}>
                  <span className="cbui-process-number">{index + 1}</span>
                  <span className="cbui-icon-box"><Icon name={step.icon} /></span>
                  <div>
                    <h3>{step.title}</h3>
                    <p>{step.body}</p>
                  </div>
                </li>
              ))}
            </ol>
            <aside className="cbui-converter-note" id="converter">
              <span className="cbui-converter-note__icon" aria-hidden="true">i</span>
              <div>
                <h3>{content.converter.title}</h3>
                <p>{content.converter.body}</p>
                <strong>{content.converter.status}</strong>
              </div>
            </aside>
          </div>
        </section>

        <section aria-labelledby="privacy-title" className="cbui-section" id="privacy">
          <div className="cbui-container">
            <div className="cbui-privacy-card">
              <div>
                <h2 id="privacy-title">{content.privacy.title}</h2>
                <p>{content.privacy.body}</p>
                <ul>
                  {content.privacy.checks.map((item) => (
                    <li key={item}><CheckIcon />{item}</li>
                  ))}
                </ul>
              </div>
              <div aria-label={content.privacy.illustrationLabel} className="cbui-privacy-visual" role="img">
                <span className="cbui-privacy-browser-dots" aria-hidden="true"><i /><i /><i /></span>
                <span className="cbui-shield-outline"><Icon name="shield" /></span>
                <span className="cbui-privacy-line cbui-privacy-line--one" aria-hidden="true" />
                <span className="cbui-privacy-line cbui-privacy-line--two" aria-hidden="true" />
              </div>
            </div>

            <div className="cbui-future-grid">
              {content.future.map((item) => (
                <article className="cbui-future-card" id={item.id} key={item.id}>
                  <span className="cbui-icon-box"><Icon name={item.icon} /></span>
                  <h3>{item.title}</h3>
                  <p>{item.body}</p>
                  <a href={item.href}>{item.link}<ArrowIcon /></a>
                </article>
              ))}
            </div>

            <div className="cbui-final-callout">
              <div>
                <h2>{content.finalCallout.title}</h2>
                <p>{content.finalCallout.body}</p>
              </div>
              <a className="cbui-button cbui-button--primary" href="#converter">
                {content.finalCallout.action}
                <ArrowIcon />
              </a>
            </div>
          </div>
        </section>
      </main>

      <footer className="cbui-footer" id="status">
        <div className="cbui-container cbui-footer__main">
          <div>
            <a aria-label={content.brand} className="cbui-brand" href="#top">
              <BrandMark />
              <span>{content.brand}</span>
            </a>
            <p>{content.brandTagline}</p>
          </div>
          <nav aria-label={content.footer.navigationLabel}>
            {content.footer.links.map((item) => (
              <a href={item.href} key={item.href}>{item.label}</a>
            ))}
          </nav>
          <p className="cbui-status"><span aria-hidden="true" />{content.footer.status}</p>
        </div>
        <div className="cbui-container cbui-footer__legal">
          <p>{content.footer.legal}</p>
          <p>{content.footer.copyright}</p>
        </div>
      </footer>
    </div>
  );
}
