"use client";

import { useEffect, useState } from "react";

import { nextThemeMode, resolveStoredTheme, type ThemeMode } from "./theme";

type ThemeControlProps = Readonly<{
  labels: Readonly<Record<ThemeMode, string>>;
}>;

function applyTheme(mode: ThemeMode) {
  document.documentElement.dataset.theme = mode;
}

export function ThemeControl({ labels }: ThemeControlProps) {
  const [mode, setMode] = useState<ThemeMode>("system");

  useEffect(() => {
    let storedMode: ThemeMode = "system";

    try {
      storedMode = resolveStoredTheme(window.localStorage.getItem("mathmorph-theme"));
    } catch {
      storedMode = "system";
    }

    setMode(storedMode);
    applyTheme(storedMode);
  }, []);

  function changeTheme() {
    const nextMode = nextThemeMode(mode);
    setMode(nextMode);
    applyTheme(nextMode);

    try {
      window.localStorage.setItem("mathmorph-theme", nextMode);
    } catch {
      // The selected theme still applies for the current page when storage is unavailable.
    }
  }

  return (
    <button
      aria-label={labels[mode]}
      className="cbui-theme-control"
      onClick={changeTheme}
      title={labels[mode]}
      type="button"
    >
      <svg aria-hidden="true" className="cbui-icon" viewBox="0 0 24 24">
        <circle cx="12" cy="12" r="3.25" />
        <path d="M12 2.5v2M12 19.5v2M4.5 12h-2M21.5 12h-2M5.3 5.3l1.4 1.4M17.3 17.3l1.4 1.4M18.7 5.3l-1.4 1.4M6.7 17.3l-1.4 1.4" />
      </svg>
      <span>{labels[mode]}</span>
    </button>
  );
}
