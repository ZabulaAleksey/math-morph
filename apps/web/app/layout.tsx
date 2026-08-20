import type { Metadata } from "next";
import type { ReactNode } from "react";

import "./globals.css";

import { landingContent } from "./content";

export const metadata: Metadata = {
  title: "MathMorph — Mathcad у редагований DOCX",
  description: "Публічна оболонка MathMorph для прозорої конвертації Mathcad у редагований Word.",
};

const themeBootstrap = `(() => {
  try {
    const value = localStorage.getItem("mathmorph-theme");
    document.documentElement.dataset.theme = ["light", "dark", "system"].includes(value) ? value : "system";
  } catch {
    document.documentElement.dataset.theme = "system";
  }
})();`;

export default function RootLayout({ children }: Readonly<{ children: ReactNode }>) {
  return (
    <html data-theme="system" lang="uk" suppressHydrationWarning>
      <head>
        <script dangerouslySetInnerHTML={{ __html: themeBootstrap }} />
      </head>
      <body data-design-system="cbui">
        <a className="cbui-skip-link" href="#main-content">
          {landingContent.skipToContent}
        </a>
        {children}
      </body>
    </html>
  );
}
