<script setup lang="ts">
import { invoke } from "@tauri-apps/api/core";
import { ref, onMounted, computed } from "vue";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { useLibrary } from "../composables/useLibrary";
import { useSettings } from "../composables/useSettings";
import { normalizeTags } from "../catalogTags";
import VersionHistory from "../components/VersionHistory.vue";
import ProjectCard from "../components/ProjectCard.vue";
import AddProjectDialog from "../components/AddProjectDialog.vue";
import BulkImportDialog from "../components/BulkImportDialog.vue";
import DownloadProjectDialog from "../components/DownloadProjectDialog.vue";
import EditProjectDialog from "../components/EditProjectDialog.vue";
import RelinkDialog from "../components/RelinkDialog.vue";
import type { Project, ProjectPatch, SortKey } from "../types";

const {
  projects,
  viewers,
  loading,
  error,
  loadLibrary,
  takeLibraryMigrationNotice,
  removeProject,
  removeProjectFromDisk,
  startOverwriteCatalogEntry,
  setProjectFavorite,
  updateProject,
  openViewer,
} = useLibrary();

const { settings } = useSettings();

const search = ref("");
const tagFilter = ref("");
const authorFilter = ref("");
const fandomFilter = ref("");
const modFilter = ref("");
const modderFilter = ref("");
const statusFilter = ref("");
const searchHint = computed(() => search.value ? `${search.value}\nUse -word to exclude matches.` : "Search title, author, modder, fandom, description or tags. Use -word to exclude matches.");
const allTags = computed(() => normalizeTags(projects.value.flatMap(project => project.tags)));
function metadataOptions(field: "author" | "fandom" | "modder") {
  return computed(() => [...new Set(projects.value.map(project => project[field]?.trim()).filter((value): value is string => Boolean(value)))].sort((a,b) => a.localeCompare(b)));
}
const authors = metadataOptions("author");
const fandoms = metadataOptions("fandom");
const modders = metadataOptions("modder");
const sort = ref<SortKey>("favorite_date_added");

const showAdd = ref(false);
const showBulk = ref(false);
const showDownload = ref(false);
const editTarget = ref<Project | null>(null);
const relinkTarget = ref<Project | null>(null);
const removeFromDiskTarget = ref<Project | null>(null);
const removingFromDisk = ref(false);
const migrationNotice = ref<string | null>(null);
const duplicates = ref<{kind:string;projects:Project[]}[]>([]);
const duplicatesOpen = ref(false);
async function scanDuplicates() { duplicates.value=await invoke("find_duplicates"); duplicatesOpen.value=true; }
const historyProject = ref<Project | null>(null);
const redownloadingProjectId = ref<string | null>(null);
const redownloadStatus = ref<string | null>(null);
let redownloadProgressUnlisten: UnlistenFn | null = null;
let activeRedownloadTaskId = "";

type CatalogProgressPayload = {
  taskId: string;
  phase: string;
  current: number;
  total: number;
  message: string;
  done: boolean;
  success: boolean;
  error?: string | null;
};

type LibrarySearchToken = {
  value: string;
  excluded: boolean;
};

const displayedList = computed(() => {
  let list = [...projects.value];
  const tokens = parseLibrarySearchTokens(search.value);
  if (tokens.length > 0) {
    list = list.filter((project) => matchesLibrarySearch(project, tokens));
  }
  if (tagFilter.value) list = list.filter(project => normalizeTags(project.tags).includes(tagFilter.value));
  if (authorFilter.value) list = list.filter(project => project.author === authorFilter.value);
  if (fandomFilter.value) list = list.filter(project => project.fandom === fandomFilter.value);
  if (modFilter.value) list = list.filter(project => Boolean(project.is_mod) === (modFilter.value === "modded"));
  if (modFilter.value === "modded" && modderFilter.value) list = list.filter(project => project.modder === modderFilter.value);
  if (statusFilter.value) list = list.filter(project => Boolean(project.build_count) === (statusFilter.value === "completed"));
  if (sort.value === "name") {
    list.sort((a, b) => a.name.localeCompare(b.name));
  } else if (sort.value === "favorite_date_added") {
    list.sort((a, b) => {
      if (a.favorite !== b.favorite) {
        return Number(b.favorite) - Number(a.favorite);
      }
      return new Date(b.date_added).getTime() - new Date(a.date_added).getTime();
    });
  } else {
    list.sort(
      (a, b) => new Date(b.date_added).getTime() - new Date(a.date_added).getTime()
    );
  }
  return list;
});

function parseLibrarySearchTokens(raw: string): LibrarySearchToken[] {
  return raw
    .trim()
    .toLowerCase()
    .split(/\s+/)
    .map((part) => part.trim())
    .filter(Boolean)
    .map((part) => {
      const excluded = part.startsWith("-");
      const value = excluded ? part.slice(1).trim() : part;
      return {
        value,
        excluded,
      };
    })
    .filter((token) => token.value.length > 0);
}

function buildLibrarySearchHaystack(project: Project): string[] {
  return [project.name, project.title, project.author, project.source_author, project.modder, project.fandom, project.description, project.is_mod ? "mod" : "", project.build_count ? `Completed(${project.build_count})` : "", ...normalizeTags(project.tags)]
    .filter((value): value is string => Boolean(value))
    .map((value) => value.toLowerCase());
}

function matchesLibrarySearch(project: Project, tokens: LibrarySearchToken[]): boolean {
  const haystack = buildLibrarySearchHaystack(project);

  return tokens.every((token) => {
    const matched = haystack.some((value) => value.includes(token.value));
    return token.excluded ? !matched : matched;
  });
}

onMounted(async () => {
  await loadLibrary();
  migrationNotice.value = await takeLibraryMigrationNotice();
});

async function reloadLibrary() {
  await loadLibrary(true);
  if(await invoke<number>("enrich_catalog_metadata")) await loadLibrary(true);
}

function closeMigrationNotice() {
  migrationNotice.value = null;
}

async function clearRedownloadListener() {
  if (!redownloadProgressUnlisten) {
    return;
  }

  await redownloadProgressUnlisten();
  redownloadProgressUnlisten = null;
}

async function onRemove(project: Project) {
  if (!confirm(`Remove "${project.name}" from library?`)) return;
  await removeProject(project.id);
}

function removableDiskTargetPath(project: Project) {
  const normalized = project.file_path.replace(/\\/g, "/");
  if (normalized.toLowerCase().endsWith("/project.json")) {
    return normalized.slice(0, -"/project.json".length);
  }
  return project.file_path;
}

function onRequestRemoveFromDisk(project: Project) {
  removeFromDiskTarget.value = project;
}

function cancelRemoveFromDisk() {
  if (removingFromDisk.value) {
    return;
  }

  removeFromDiskTarget.value = null;
}

async function confirmRemoveFromDisk() {
  if (!removeFromDiskTarget.value || removingFromDisk.value) {
    return;
  }

  removingFromDisk.value = true;
  try {
    await removeProjectFromDisk(removeFromDiskTarget.value.id);
    removeFromDiskTarget.value = null;
  } finally {
    removingFromDisk.value = false;
  }
}

async function onEdit(project: Project, patch: ProjectPatch) {
  await updateProject(project.id, patch);
  editTarget.value = null;
}

async function onRelink(project: Project, patch: ProjectPatch) {
  await updateProject(project.id, patch);
  relinkTarget.value = null;
}

async function onOpen(project: Project, viewerId: string) {
  await openViewer(project, viewerId);
}

async function onToggleFavorite(project: Project) {
  await setProjectFavorite(project.id, !project.favorite);
}

async function onForceUpdate(project: Project, patch: ProjectPatch) {
  if (activeRedownloadTaskId) { alert("A re-download is already in progress."); return; }
  if (!confirm("Force update this card from its source? Your current edition will be archived first. This can replace an older restored edition with the latest release. The edits in this form will be saved.")) return;
  try { const updated=await updateProject(project.id,patch);editTarget.value=null;await onRedownload(updated,true); } catch(e) { alert(String(e)); }
}
async function onRedownload(project: Project, forceUpdate = false) {
  if (project.restored_from_archive && !forceUpdate) { alert("This is a restored archive edition. Re-download the current main card instead."); return; }
  if(project.kind==='website' && project.source_url){try{const result=await invoke<{unavailable:string[]}>("download_website",{url:project.source_url,title:project.title||project.name,author:project.source_author||project.author||"",fandom:project.fandom||"",description:project.description,maxSizeMb:settings.value.downloadSizeLimitMb,existingProjectId:project.id,forceUpdate});await loadLibrary(true);if(result.unavailable.length)alert(`${result.unavailable.length} website resources could not be saved. Use Website (online) if the saved copy does not work.`);}catch(e){alert(String(e));}return;}

  const redownloadUrl = project.project_json_url?.trim() || project.source_url?.trim() || "";
  if (!redownloadUrl) {
    return;
  }

  if (activeRedownloadTaskId) {
    alert("A re-download is already in progress.");
    return;
  }

  const taskId = crypto.randomUUID();
  activeRedownloadTaskId = taskId;
  redownloadingProjectId.value = project.id;
  redownloadStatus.value = "Preparing...";

  await clearRedownloadListener();

  try {
    redownloadProgressUnlisten = await listen<CatalogProgressPayload>("download-catalog-progress", async (event) => {
      const payload = event.payload;
      if (payload.taskId !== activeRedownloadTaskId) {
        return;
      }

      redownloadStatus.value = payload.message;

      if (!payload.done) {
        return;
      }

      activeRedownloadTaskId = "";
      redownloadingProjectId.value = null;
      await clearRedownloadListener();

      if (payload.success) {
        redownloadStatus.value = null;
        await invoke("enrich_catalog_metadata").catch(console.error);
        await loadLibrary(true);
        return;
      }

      const message = payload.error || payload.message || "Re-download failed.";
      redownloadStatus.value = null;
      alert(message);
    });

    await startOverwriteCatalogEntry(
      taskId,
      project.id,
      redownloadUrl,
      "",
      project.name,
      settings.value.downloadSizeLimitMb,
      forceUpdate,
    );
  } catch (redownloadError) {
    activeRedownloadTaskId = "";
    redownloadingProjectId.value = null;
    redownloadStatus.value = null;
    await clearRedownloadListener();
    alert(redownloadError instanceof Error ? redownloadError.message : String(redownloadError));
  }
}

</script>

<template>
  <div v-if="duplicatesOpen" class="duplicate-overlay" @click.self="duplicatesOpen=false">
    <section class="duplicate-panel" role="dialog" aria-label="Duplicate finder">
      <h2>Duplicate finder</h2><button class="btn-secondary" @click="duplicatesOpen=false">Close</button>
      <p v-if="!duplicates.length">No duplicate files or matching sources found.</p>
      <section v-for="(group,i) in duplicates" :key="i"><h3>{{group.kind}}</h3>
        <div v-for="project in group.projects" :key="project.id"><span>{{project.title || project.name}}</span>
          <button class="btn-secondary" @click="historyProject=project;duplicatesOpen=false">Compare / archive copies</button>
        </div>
      </section>
      <p>Versions and mods can intentionally differ. No files are changed by this scan.</p>
    </section>
  </div>
  <VersionHistory v-if="historyProject" :project="historyProject" @close="historyProject = null" @restored="loadLibrary(true)" />
  <div class="library-view">
    <!-- Toolbar -->
    <div class="toolbar">
      <input
        v-model="search"
        class="search"
        type="text"
        placeholder="Search projects"
        :title="searchHint"
        aria-label="Search projects"
      />

      <select v-model="authorFilter" class="filter-select" title="Filter by original author" aria-label="Author">
        <option value="">Author</option><option v-for="value in authors" :key="value" :value="value">{{ value }}</option>
      </select>
      <select v-model="fandomFilter" class="filter-select" title="Filter by fandom" aria-label="Fandom">
        <option value="">Fandom</option><option v-for="value in fandoms" :key="value" :value="value">{{ value }}</option>
      </select>
      <select v-model="modFilter" class="filter-select" title="Filter adaptations" aria-label="Mods">
        <option value="">Mods: all</option><option value="original">Originals</option><option value="modded">Modded</option>
      </select>
      <select v-if="modFilter === 'modded'" v-model="modderFilter" class="filter-select" title="Filter by modder" aria-label="Modder">
        <option value="">Modder</option><option v-for="value in modders" :key="value" :value="value">{{ value }}</option>
      </select>
      <select v-model="statusFilter" class="filter-select" title="Your saved builds" aria-label="My status">
        <option value="">My status</option><option value="completed">Completed</option><option value="not-completed">Not completed</option>
      </select>
      <select v-model="tagFilter" class="filter-select" title="Optional extra tags" aria-label="Extra tags">
        <option value="">Extra tags</option><option v-for="tag in allTags" :key="tag" :value="tag">{{ tag }}</option>
      </select>

      <select v-model="sort" class="filter-select" title="Sort">
        <option value="favorite_date_added">Favorites first</option>
        <option value="date_added">Newest first</option>
        <option value="name">Name A–Z</option>
      </select>

      <div class="project-count">
        {{ projects.length }} {{ projects.length === 1 ? "project" : "projects" }}
      </div>

      <div class="toolbar-spacer" />

      <button class="btn-secondary" @click="scanDuplicates">Find duplicates</button>
      <button class="btn-secondary" @click="showBulk = true">Import folder</button>
      <button class="btn-secondary" @click="showDownload = true">Download Project</button>
      <button class="btn-primary" @click="showAdd = true">+ Add Project</button>
    </div>

    <!-- Loading -->
    <div v-if="loading && !projects.length" class="center-msg">Loading library…</div>

    <!-- Error -->
    <div v-else-if="error" class="center-msg error">{{ error }}</div>

    <!-- Empty state -->
    <div v-else-if="projects.length === 0" class="empty-state">
      <div class="empty-icon">📚</div>
      <h2>Your library is empty</h2>
      <p>Add your first <code>project.json</code> to get started.</p>
      <div class="empty-actions">
        <button class="btn-secondary large" @click="showDownload = true">Download Project</button>
        <button class="btn-primary large" @click="showAdd = true">+ Add Project</button>
      </div>
    </div>

    <!-- No results -->
    <div
      v-else-if="displayedList.length === 0"
      class="center-msg"
    >
      No projects match your filter.
    </div>

    <!-- Card grid -->
    <div v-else class="grid">
      <ProjectCard
        v-for="p in displayedList"
        :key="p.id"
        :project="p"
        :viewers="viewers"
        :default-viewer="settings.defaultViewer"
        :redownload-busy="redownloadingProjectId === p.id"
        :redownload-label="redownloadingProjectId === p.id ? redownloadStatus : null"
        @open="(vid) => onOpen(p, vid)"
        @toggle-favorite="onToggleFavorite(p)"
        @redownload="onRedownload(p)"
        @history="historyProject = p"
        @remove="onRemove(p)"
        @remove-disk="onRequestRemoveFromDisk(p)"
        @edit="editTarget = p"
        @relink="relinkTarget = p"
      />
    </div>
  </div>

  <!-- Dialogs -->
  <AddProjectDialog
    v-if="showAdd"
    @close="showAdd = false"
    @added="reloadLibrary"
  />
  <BulkImportDialog
    v-if="showBulk"
    @close="showBulk = false"
    @added="reloadLibrary"
  />
  <DownloadProjectDialog
    v-if="showDownload"
    @close="showDownload = false"
    @added="reloadLibrary"
  />
  <EditProjectDialog
    v-if="editTarget"
    :project="editTarget"
    :viewers="viewers"
    @save="(patch) => onEdit(editTarget!, patch)"
    @force-update="(patch) => onForceUpdate(editTarget!, patch)"
    @close="editTarget = null"
  />
  <RelinkDialog
    v-if="relinkTarget"
    :project="relinkTarget"
    @save="(patch) => onRelink(relinkTarget!, patch)"
    @close="relinkTarget = null"
  />

  <div v-if="migrationNotice" class="overlay" @click.self="closeMigrationNotice">
    <div class="dialog migration-dialog">
      <h2>Library Migrated</h2>
      <p>{{ migrationNotice }}</p>
      <div class="dialog-actions">
        <button class="btn-primary" @click="closeMigrationNotice">OK</button>
      </div>
    </div>
  </div>

  <div v-if="removeFromDiskTarget" class="overlay" @click.self="cancelRemoveFromDisk">
    <div class="dialog migration-dialog">
      <h2>Remove From Disk?</h2>
      <p>
        This will remove <strong>{{ removeFromDiskTarget.name }}</strong> from the library and permanently delete it from disk.
      </p>
      <p class="disk-path">{{ removableDiskTargetPath(removeFromDiskTarget) }}</p>
      <div class="dialog-actions">
        <button class="btn-secondary" :disabled="removingFromDisk" @click="cancelRemoveFromDisk">Cancel</button>
        <button class="btn-danger" :disabled="removingFromDisk" @click="confirmRemoveFromDisk">
          {{ removingFromDisk ? "Removing..." : "Remove from disk" }}
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.duplicate-overlay {position:fixed;inset:0;background:#0008;z-index:1100;display:grid;place-items:center;}
.duplicate-panel {background:var(--dialog-bg);border:1px solid var(--border);border-radius:12px;padding:24px;width:min(800px,95vw);max-height:90vh;overflow:auto;}
.duplicate-panel section div {display:flex;justify-content:space-between;gap:12px;padding:8px;}
.library-view {
  flex: 1;
  display: flex;
  flex-direction: column;
  height: 100%;
  min-height: 0;
  overflow-y: auto;
}

.toolbar {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 8px;
  padding: 12px 20px;
  border-bottom: 1px solid var(--border);
  flex-shrink: 0;
  position: sticky;
  top: 0;
  z-index: 2;
  background: var(--bg);
}

.search {
  flex: 1 1 210px;
  min-width: 160px;
  max-width: 360px;
  padding: 7px 12px;
  background: var(--input-bg);
  border: 1px solid var(--border);
  border-radius: 8px;
  color: var(--text);
  font-size: 0.9rem;
  outline: none;
  transition: border-color 0.15s;
}
.search:focus { border-color: var(--accent); }

.filter-select {
  flex: 0 1 auto;
  min-width: 100px;
  max-width: 170px;
  text-overflow: ellipsis;
  padding: 7px 10px;
  background: var(--input-bg);
  border: 1px solid var(--border);
  border-radius: 8px;
  color: var(--text);
  font-size: 0.875rem;
  outline: none;
  cursor: pointer;
}

.project-count {
  flex: 0 0 auto;
  padding: 7px 10px;
  border: 1px solid var(--border);
  border-radius: 8px;
  color: var(--muted);
  font-size: 0.875rem;
  white-space: nowrap;
}

.toolbar-spacer { flex: 1; }

.grid {
  flex: none;
  min-height: auto;
  overflow: visible;
  padding: 20px;
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(220px, 1fr));
  gap: 16px;
  align-content: start;
}

.center-msg {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--muted);
  font-size: 0.95rem;
}
.center-msg.error { color: #e55; }

.empty-state {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 12px;
  color: var(--muted);
  text-align: center;
  padding: 40px;
}
.empty-icon { font-size: 4rem; }
.empty-state h2 { margin: 0; color: var(--text); }
.empty-state p { margin: 0; font-size: 0.95rem; }
.empty-state code { color: var(--accent); }
.empty-actions {
  display: flex;
  gap: 10px;
  flex-wrap: wrap;
  justify-content: center;
}
.btn-primary.large { padding: 12px 28px; font-size: 1rem; margin-top: 8px; }

.overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.6);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 200;
}

.dialog {
  width: min(480px, calc(100vw - 32px));
  background: var(--dialog-bg);
  border: 1px solid var(--border);
  border-radius: 12px;
  padding: 22px;
  box-shadow: 0 16px 40px rgba(0, 0, 0, 0.35);
}

.migration-dialog h2 {
  margin: 0 0 10px;
}

.migration-dialog p {
  margin: 0;
  color: var(--muted);
  line-height: 1.5;
}

.disk-path {
  margin-top: 10px;
  padding: 10px 12px;
  background: var(--input-bg);
  border: 1px solid var(--border);
  border-radius: 8px;
  color: var(--text);
  font-family: Consolas, "Courier New", monospace;
  font-size: 0.82rem;
  word-break: break-all;
}

.dialog-actions {
  display: flex;
  justify-content: flex-end;
  margin-top: 18px;
}
</style>
