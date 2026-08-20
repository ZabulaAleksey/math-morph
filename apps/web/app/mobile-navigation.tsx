"use client";

import { useRef } from "react";

import { MenuIcon } from "./icons";

type MobileNavigationProps = Readonly<{
  label: string;
  menuLabel: string;
  items: ReadonlyArray<Readonly<{ label: string; href: string }>>;
}>;

export function MobileNavigation({ items, label, menuLabel }: MobileNavigationProps) {
  const detailsRef = useRef<HTMLDetailsElement>(null);
  const summaryRef = useRef<HTMLElement>(null);

  function closeMenu() {
    detailsRef.current?.removeAttribute("open");
    window.requestAnimationFrame(() => summaryRef.current?.focus());
  }

  return (
    <details className="cbui-mobile-nav" ref={detailsRef}>
      <summary aria-label={menuLabel} ref={summaryRef}>
        <MenuIcon />
      </summary>
      <nav aria-label={label}>
        {items.map((item) => (
          <a href={item.href} key={item.href} onClick={closeMenu}>{item.label}</a>
        ))}
      </nav>
    </details>
  );
}
