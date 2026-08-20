import assert from "node:assert/strict";
import test from "node:test";

import { renderToStaticMarkup } from "react-dom/server";

import HomePage from "../app/page";
import { landingContent } from "../app/content";
import { nextThemeMode, resolveStoredTheme } from "../app/theme";

test("unit: theme values fail closed to system and cycle predictably", () => {
  assert.equal(resolveStoredTheme(null), "system");
  assert.equal(resolveStoredTheme("unexpected"), "system");
  assert.equal(resolveStoredTheme("dark"), "dark");
  assert.equal(nextThemeMode("system"), "light");
  assert.equal(nextThemeMode("light"), "dark");
  assert.equal(nextThemeMode("dark"), "system");
});

test("component: landing page exposes semantic public sections", () => {
  const markup = renderToStaticMarkup(<HomePage />);

  assert.match(markup, /<main id="main-content">/);
  assert.match(markup, /<h1 id="hero-title">/);
  assert.match(markup, /id="features"/);
  assert.match(markup, /id="privacy"/);
  assert.match(markup, /id="api"/);
  assert.match(markup, /id="pricing"/);
  assert.match(markup, /<footer[^>]+id="status"/);
  assert.match(markup, /<details class="cbui-mobile-nav">/);
  assert.match(markup, new RegExp(landingContent.footer.status));
});

test("integration: converter CTA resolves to an honest staged state", () => {
  const markup = renderToStaticMarkup(<HomePage />);

  assert.match(markup, /href="#converter"/);
  assert.match(markup, /id="converter"/);
  assert.match(markup, new RegExp(landingContent.converter.status));
  assert.doesNotMatch(markup, /<input[^>]+type="file"/);
  assert.doesNotMatch(markup, /MathType.{0,30}(verified|підтверджено)/i);
});
