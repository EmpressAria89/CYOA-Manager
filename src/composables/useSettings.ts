import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { ref, watch } from "vue";
import type { Theme } from "../types";
import type { OversizeDefaultAction } from "../types";

const STORAGE_KEY = "cyoa-manager-settings";

interface Settings {
  managerFont: string;
  managerFontSize: number;
  cyoaFont: string;
  useAuthorFonts: boolean;
  archiveLimit: number;
  defaultViewer: string | null;
  theme: Theme;
  cheatsEnabled: boolean;
  downloadSizeLimitMb: number;
  oversizeDefaultAction: OversizeDefaultAction;
}

function load(): Settings {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (raw) {
      const loaded = { ...defaults(), ...JSON.parse(raw) };
      loaded.managerFontSize = normalizeFontSize(loaded.managerFontSize);
      if (loaded.theme === "dark") loaded.theme = "mocha";
      if (loaded.theme === "light") loaded.theme = "latte";
      if (!["system", "latte", "frappe", "macchiato", "mocha"].includes(loaded.theme)) loaded.theme = "system";
      return loaded;
    }
  } catch {
    /* ignore */
  }
  return defaults();
}

function defaults(): Settings {
  return {
    managerFont: "system-ui",
    managerFontSize: 16,
    cyoaFont: "",
    useAuthorFonts: false,
    archiveLimit: 5,
    defaultViewer: null,
    theme: "system",
    cheatsEnabled: true,
    downloadSizeLimitMb: 200,
    oversizeDefaultAction: "ask",
  };
}

const settings = ref<Settings>(load());
void listen<{cyoaFont:string;useAuthorFonts:boolean}>("manager-reader-defaults-changed", ({payload}) => {
  settings.value.cyoaFont=payload.cyoaFont;
  settings.value.useAuthorFonts=payload.useAuthorFonts;
}).catch(console.error);

function normalizeFontSize(value: unknown): number {
  const size = Number(value);
  return Number.isFinite(size) && size > 0 ? Math.round(Math.min(24, Math.max(12, size))) : 16;
}

watch(
  settings,
  (val) => {
    try { localStorage.setItem(STORAGE_KEY, JSON.stringify(val)); } catch (error) { console.error("Could not persist manager settings:", error); }
    useSettings().applyTheme();

  },
  { deep: true }
);

let pendingPreferences: Record<string, unknown> | null = null;
let writingPreferences = false;
let persistedPreferences = "";
async function persistPreferences(value: Record<string, unknown>) {
  pendingPreferences = value;
  if (writingPreferences) return;
  writingPreferences = true;
  try {
    while (pendingPreferences) {
      const current = pendingPreferences;
      pendingPreferences = null;
      const encoded = JSON.stringify(current);
      if (encoded === persistedPreferences) continue;
      try { await invoke("set_preferences", {preferences: current}); persistedPreferences = encoded; }
      catch (error) { console.error("Could not persist reader defaults:", error); }
    }
  } finally { writingPreferences = false; }
}

export function useSettings() {
  function applyTheme() {
    const font=settings.value.managerFont||"system-ui";
    let face=document.getElementById("manager-font-face");if(!face){face=document.createElement("style");face.id="manager-font-face";document.head.append(face);}
    const faceCss=`@font-face{font-family:'CYOA Manager Regular';src:url('cyoafont://localhost/${encodeURIComponent(font)}');font-weight:400;font-style:normal;font-display:swap;}`;
    if (face.textContent !== faceCss) face.textContent = faceCss;
    document.documentElement.style.setProperty("--manager-font", "'CYOA Manager Regular'");
    const managerFontSize = normalizeFontSize(settings.value.managerFontSize);
    document.documentElement.style.setProperty("--manager-font-size", `${managerFontSize}px`);

    const theme = settings.value.theme;
    const prefersDark = window.matchMedia("(prefers-color-scheme: dark)").matches;
    const flavor = theme === "system" ? (prefersDark ? "mocha" : "latte")
      : theme === "dark" ? "mocha" : theme === "light" ? "latte" : theme;
    void persistPreferences({managerFont: settings.value.managerFont, managerFontSize, cyoaFont: settings.value.cyoaFont, useAuthorFonts: settings.value.useAuthorFonts, archiveLimit: settings.value.archiveLimit, theme: flavor});
    document.documentElement.dataset.theme = flavor;
    document.documentElement.classList.toggle("dark", flavor !== "latte");
  }

  return { settings, applyTheme };
}
