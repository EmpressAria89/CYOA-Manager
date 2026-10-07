import { invoke } from "@tauri-apps/api/core";
import { ref, watch } from "vue";
import type { Theme } from "../types";
import type { OversizeDefaultAction } from "../types";

const STORAGE_KEY = "cyoa-manager-settings";

interface Settings {
  managerFont: string;
  managerFontSize: number;
  cyoaFont: string;
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
    archiveLimit: 5,
    defaultViewer: null,
    theme: "system",
    cheatsEnabled: true,
    downloadSizeLimitMb: 200,
    oversizeDefaultAction: "ask",
  };
}

const settings = ref<Settings>(load());

function normalizeFontSize(value: unknown): number {
  const size = Number(value);
  return Number.isFinite(size) && size > 0 ? Math.round(Math.min(24, Math.max(12, size))) : 16;
}

watch(
  settings,
  (val) => {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(val));
    useSettings().applyTheme();

  },
  { deep: true }
);

export function useSettings() {
  function applyTheme() {
    const font=settings.value.managerFont||"system-ui";
    let face=document.getElementById("manager-font-face");if(!face){face=document.createElement("style");face.id="manager-font-face";document.head.append(face);}
    face.textContent=`@font-face{font-family:'CYOA Manager Regular';src:url('cyoafont://localhost/${encodeURIComponent(font)}');font-weight:400;font-style:normal;font-display:swap;}`;
    document.documentElement.style.setProperty("--manager-font", "'CYOA Manager Regular'");
    const managerFontSize = normalizeFontSize(settings.value.managerFontSize);
    document.documentElement.style.setProperty("--manager-font-size", `${managerFontSize}px`);

    const theme = settings.value.theme;
    const prefersDark = window.matchMedia("(prefers-color-scheme: dark)").matches;
    const flavor = theme === "system" ? (prefersDark ? "mocha" : "latte")
      : theme === "dark" ? "mocha" : theme === "light" ? "latte" : theme;
    void invoke("set_preferences", {preferences: {...settings.value, managerFontSize, theme: flavor}}).catch(console.error);
    document.documentElement.dataset.theme = flavor;
    document.documentElement.classList.toggle("dark", flavor !== "latte");
  }

  return { settings, applyTheme };
}
