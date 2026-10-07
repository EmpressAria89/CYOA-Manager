<script setup lang="ts">
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { computed, onMounted, ref } from "vue";
import { RouterView, RouterLink } from "vue-router";
import { useAuthorAliases } from "./composables/useAuthorAliases";
import { useSettings } from "./composables/useSettings";
import { useLibrary } from "./composables/useLibrary";
import { resolveViewerId } from "./viewers";

const GITHUB_RELEASES_LATEST_URL = "https://api.github.com/repos/EmpressAria89/CYOA-Manager/releases/latest";

const { loadAuthorAliases } = useAuthorAliases();
const { settings, applyTheme } = useSettings();
const { projects, viewers, loadLibrary, loadViewers, openViewer } = useLibrary();
interface ChangeReport { projectName: string; diff: { changed: boolean; added: string[]; removed: string[]; edited: string[]; other_changes: string[] } }
const maintenanceError=ref("");
const changeReport = ref<ChangeReport|null>(null);
const latestReleaseVersion = ref("");
const latestReleaseUrl = ref("");

const randomCandidates = computed(() =>
  projects.value.filter((project) => !project.file_missing)
);

const canOpenRandom = computed(
  () => randomCandidates.value.length > 0 && viewers.value.length > 0
);

const updateReleaseUrl = computed(() => {
  if (!latestReleaseUrl.value || !latestReleaseVersion.value) {
    return "";
  }

  return compareReleaseVersions(latestReleaseVersion.value, __APP_VERSION__) > 0
    ? latestReleaseUrl.value
    : "";
});

onMounted(async () => {
  applyTheme();
  void listen("manager-build-saved", () => loadLibrary(true)).catch(console.error);
  void listen<ChangeReport>("project-update-result", event => changeReport.value=event.payload).catch(console.error);
  window.matchMedia("(prefers-color-scheme: dark)").addEventListener("change", applyTheme);
  void checkForAppUpdate();
  await loadAuthorAliases().catch(e=>maintenanceError.value=`Author aliases: ${String(e)}`);
  await loadLibrary();
  await loadViewers();
  void (async () => { await invoke("enrich_library_metadata"); await invoke("reconcile_imported_archives"); await invoke("optimize_storage"); await loadLibrary(true); await invoke("enrich_catalog_metadata"); await loadLibrary(true); })().catch(e=>maintenanceError.value=`Storage maintenance: ${String(e)}`);
});

async function openRandomProject() {
  if (!canOpenRandom.value) {
    return;
  }

  const candidates = randomCandidates.value;
  const project = candidates[Math.floor(Math.random() * candidates.length)];
  const viewerId = resolveViewerId(
    viewers.value,
    project.viewer_preference,
    settings.value.defaultViewer,
  );

  if (!viewerId) {
    return;
  }

  await openViewer(project, viewerId);
}

async function checkForAppUpdate() {
  try {
    const response = await fetch(GITHUB_RELEASES_LATEST_URL, {
      headers: {
        Accept: "application/vnd.github+json",
      },
    });

    if (!response.ok) {
      return;
    }

    const payload = await response.json() as { tag_name?: string; html_url?: string };
    const version = normalizeReleaseVersion(payload.tag_name);
    if (!version || !payload.html_url) {
      return;
    }

    latestReleaseVersion.value = version;
    latestReleaseUrl.value = payload.html_url;
  } catch {
    latestReleaseVersion.value = "";
    latestReleaseUrl.value = "";
  }
}

function normalizeReleaseVersion(version: string | undefined): string {
  return (version || "").trim().replace(/^v/i, "");
}

function compareReleaseVersions(left: string, right: string): number {
  const leftParts = splitReleaseVersion(left);
  const rightParts = splitReleaseVersion(right);
  const maxLength = Math.max(leftParts.length, rightParts.length);

  for (let index = 0; index < maxLength; index += 1) {
    const leftPart = leftParts[index] || 0;
    const rightPart = rightParts[index] || 0;

    if (leftPart !== rightPart) {
      return leftPart - rightPart;
    }
  }

  return 0;
}

function splitReleaseVersion(version: string): number[] {
  return normalizeReleaseVersion(version)
    .split(".")
    .map((part) => {
      const match = part.match(/^\d+/);
      return match ? Number(match[0]) : 0;
    });
}
</script>

<template>
  <div v-if="changeReport" class="changes-overlay" @click.self="changeReport=null"><section class="changes-panel" role="dialog" aria-label="Update differences">
    <h2>{{changeReport.projectName}} — Update differences</h2>
    <p v-if="!changeReport.diff.changed">No content changes found. Kept the current edition.</p>
    <p v-else>The previous edition is archived under this CYOA. {{changeReport.diff.added.length}} choices added, {{changeReport.diff.removed.length}} removed, {{changeReport.diff.edited.length}} changed.</p>
    <template v-for="(entries,category) in {Added:changeReport.diff.added,Removed:changeReport.diff.removed,Changed:changeReport.diff.edited,Other:changeReport.diff.other_changes}" :key="category"><h3 v-if="entries.length">{{category}}</h3><ul><li v-for="entry in entries" :key="entry">{{entry}}</li></ul></template>
    <button class="btn-primary" @click="changeReport=null">Close</button>
  </section></div>
  <div class="app">
    <nav class="sidebar">
      <div class="brand">CYOA<br /><span>Manager</span></div>
      <RouterLink to="/" class="nav-link" active-class="active">
        📚 Library
      </RouterLink>
      <RouterLink to="/authors" class="nav-link" active-class="active">Author aliases</RouterLink>
      <RouterLink to="/archives" class="nav-link" active-class="active">Archives</RouterLink>
      <RouterLink to="/settings" class="nav-link" active-class="active">
        ⚙️ Settings
      </RouterLink>
      <RouterLink to="/perks" class="nav-link" active-class="active">
        🔎 All Perks
      </RouterLink>
      <RouterLink to="/catalog" class="nav-link" active-class="active">
        🌐 Infaera Catalog
      </RouterLink>
      <button class="nav-link nav-button" :disabled="!canOpenRandom" @click="openRandomProject">
        🎲 Random
      </button>
      <a
        class="nav-link sidebar-external github-link"
        href="https://github.com/EmpressAria89/CYOA-Manager"
        target="_blank"
        rel="noreferrer"
      >
        SOURCE ON GITHUB
      </a>
      <a
        class="nav-link sidebar-external patreon-link"
        href="https://www.patreon.com/interactiveapps"
        target="_blank"
        rel="noreferrer"
      >
        SUPPORT ON PATREON
      </a>
      <a
        v-if="updateReleaseUrl"
        class="nav-link update-link"
        :href="updateReleaseUrl"
        target="_blank"
        rel="noreferrer"
      >
        RELEASE ({{ latestReleaseVersion }})
      </a>
    </nav>
    <main class="main">
      <p v-if="maintenanceError" role="alert">{{ maintenanceError }}</p>
      <RouterView />
    </main>
  </div>
</template>

<style>
 .changes-overlay {position:fixed;inset:0;z-index:1400;background:#0008;display:grid;place-items:center;}
.changes-panel {padding:24px;background:var(--dialog-bg);border:1px solid var(--border);border-radius:12px;width:min(880px,95vw);max-height:90vh;overflow:auto;}
.changes-panel li {overflow-wrap:anywhere;}
/* ── CSS Variables ──────────────────────────────────────────── */
:root, :root[data-theme="mocha"] {
  --bg: #1e1e2e;
  --sidebar-bg: #181825;
  --card-bg: #1e1e2e;
  --dialog-bg: #1e1e2e;
  --menu-bg: #181825;
  --input-bg: #313244;
  --border: #45475a;
  --text: #cdd6f4;
  --muted: #a6adc8;
  --accent: #89b4fa;
  --accent-hover: #b4befe;
  --hover: #313244;
  --tag-bg: #313244;
  --tag-color: #89b4fa;
  --cover-placeholder: #313244;
  --danger: #f38ba8;
  --accent-text: #1e1e2e;
  color-scheme: dark;
}
:root[data-theme="latte"] {
  --bg: #eff1f5;
  --sidebar-bg: #e6e9ef;
  --card-bg: #eff1f5;
  --dialog-bg: #eff1f5;
  --menu-bg: #e6e9ef;
  --input-bg: #ccd0da;
  --border: #bcc0cc;
  --text: #4c4f69;
  --muted: #6c6f85;
  --accent: #1e66f5;
  --accent-hover: #7287fd;
  --hover: #ccd0da;
  --tag-bg: #ccd0da;
  --tag-color: #1e66f5;
  --cover-placeholder: #ccd0da;
  --danger: #d20f39;
  --accent-text: #eff1f5;
  color-scheme: light;
}
:root[data-theme="frappe"] {
  --bg: #303446;
  --sidebar-bg: #292c3c;
  --card-bg: #303446;
  --dialog-bg: #303446;
  --menu-bg: #292c3c;
  --input-bg: #414559;
  --border: #51576d;
  --text: #c6d0f5;
  --muted: #a5adce;
  --accent: #8caaee;
  --accent-hover: #babbf1;
  --hover: #414559;
  --tag-bg: #414559;
  --tag-color: #8caaee;
  --cover-placeholder: #414559;
  --danger: #e78284;
  --accent-text: #303446;
  color-scheme: dark;
}
:root[data-theme="macchiato"] {
  --bg: #24273a;
  --sidebar-bg: #1e2030;
  --card-bg: #24273a;
  --dialog-bg: #24273a;
  --menu-bg: #1e2030;
  --input-bg: #363a4f;
  --border: #494d64;
  --text: #cad3f5;
  --muted: #a5adcb;
  --accent: #8aadf4;
  --accent-hover: #b7bdf8;
  --hover: #363a4f;
  --tag-bg: #363a4f;
  --tag-color: #8aadf4;
  --cover-placeholder: #363a4f;
  --danger: #ed8796;
  --accent-text: #24273a;
  color-scheme: dark;
}

*, *::before, *::after { box-sizing: border-box; }
html { font-size: var(--manager-font-size, 16px); }

html, body, #app {
  height: 100%;
  margin: 0;
  padding: 0;
}

body {
  font-family: var(--manager-font, system-ui), sans-serif;
  font-size: 1rem;
  font-weight:400;
  background: var(--bg);
  color: var(--text);
  -webkit-font-smoothing: antialiased;
  user-select: none;
}

/* ── Shared button styles ──────────────────────────────────── */
.btn-primary {
  background: var(--accent);
  color: var(--accent-text);
  border: none;
  border-radius: 8px;
  padding: 7px 16px;
  font-size: 0.875rem;
  font-weight: 400;
  cursor: pointer;
  transition: background 0.15s;
}
.btn-primary:hover:not(:disabled) { background: var(--accent-hover); }
.btn-primary:disabled { opacity: 0.45; cursor: not-allowed; }

.btn-secondary {
  background: transparent;
  color: var(--text);
  border: 1px solid var(--border);
  border-radius: 8px;
  padding: 7px 16px;
  font-size: 0.875rem;
  font-weight: 400;
  cursor: pointer;
  transition: background 0.15s;
}
.btn-secondary:hover { background: var(--hover); }

.btn-ghost {
  background: none;
  border: none;
  color: var(--accent);
  font-size: 0.8rem;
  cursor: pointer;
  padding: 2px 4px;
}
.btn-ghost:hover { text-decoration: underline; }

select, select option { background-color: var(--input-bg) !important; color: var(--text) !important; color-scheme: inherit; }
select { appearance: none; -webkit-appearance: none; background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='12' height='8'%3E%3Cpath d='m1 1 5 5 5-5' fill='none' stroke='%2389b4fa' stroke-width='2'/%3E%3C/svg%3E"); background-repeat: no-repeat; background-position: right 10px center; padding-right: 30px !important; }
input, textarea, button, select { font-family: inherit; }
input, textarea, button, select, label {font-weight:400 !important;}
h1,h2,h3 {font-weight:600;}
input, textarea { user-select: text; }

/* ── Scrollbar ─────────────────────────────────────────────── */
::-webkit-scrollbar { width: 6px; height: 6px; }
::-webkit-scrollbar-track { background: transparent; }
::-webkit-scrollbar-thumb { background: var(--border); border-radius: 3px; }
</style>

<style scoped>
.app {
  display: flex;
  height: 100vh;
  overflow: hidden;
}
.sidebar {
  width: 180px;
  flex-shrink: 0;
  background: var(--sidebar-bg);
  border-right: 1px solid var(--border);
  display: flex;
  flex-direction: column;
  padding: 20px 0;
  gap: 4px;
}
.brand {
  font-size: 1.1rem;
  font-weight: 400;
  line-height: 1.2;
  padding: 0 20px 20px;
  color: var(--text);
}
.brand span { color: var(--accent); }
.nav-link {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 9px 20px;
  color: var(--muted);
  text-decoration: none;
  font-size: 0.875rem;
  border-radius: 0;
  transition: background 0.12s, color 0.12s;
}
.nav-link:hover { background: var(--hover); color: var(--text); }
.nav-link.active {
  background: var(--hover);
  color: var(--accent);
  font-weight: 400;
}
.nav-button {
  width: 100%;
  background: none;
  border: none;
  text-align: left;
  font: inherit;
}
.nav-button:disabled {
  opacity: 0.45;
  cursor: not-allowed;
}
.nav-button:disabled:hover {
  background: transparent;
  color: var(--muted);
}
.sidebar-external {
  margin-top: auto;
}
.main {
  flex: 1;
  min-height: 0;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.main :deep(.library-view) {
  flex: 1;
  min-height: 0;
}

.patreon-link {
  background: var(--danger);
  color: var(--accent-text);
  border-radius: 8px;
  padding: 8px 20px;
  font-weight: 400;
  margin: 10px ;
  font-size: 0.6875rem;
}

.github-link {
  background: var(--input-bg);
  color: var(--text);
  border-radius: 8px;
  padding: 8px 20px;
  font-weight: 400;
  margin: 0px 10px ;
  font-size: 0.6875rem;
}

.update-link {
  margin: 8px 10px 0;
  font-size: 0.6875rem;
  color: var(--accent);
  justify-content: center;
}
</style>
