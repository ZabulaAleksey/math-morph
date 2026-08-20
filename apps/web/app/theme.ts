export const themeModes = ["system", "light", "dark"] as const;

export type ThemeMode = (typeof themeModes)[number];

export function resolveStoredTheme(value: string | null): ThemeMode {
  return themeModes.includes(value as ThemeMode) ? (value as ThemeMode) : "system";
}

export function nextThemeMode(mode: ThemeMode): ThemeMode {
  const currentIndex = themeModes.indexOf(mode);
  return themeModes[(currentIndex + 1) % themeModes.length];
}
