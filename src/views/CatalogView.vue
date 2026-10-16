<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { resolveAuthor, useAuthorAliases } from "../composables/useAuthorAliases";
import { normalizeCatalogSnapshot, catalogUrl, catalogKey, indexCatalogEntry, parseCatalogSearch, matchesCatalogSearch, type CatalogSnapshot } from "../catalogDiscovery";
import CatalogDownloadDialog from "../components/CatalogDownloadDialog.vue";
import { catalogMetadataPatch } from "../catalogImport";
import { openUrl } from "@tauri-apps/plugin-opener";
import type { CatalogEntry, Project } from "../types";
import { useLibrary } from "../composables/useLibrary";
import { useSettings } from "../composables/useSettings";

const LOCAL_CATALOG_URL = "/zip_link_catalog_data.js";

const { projects, loadLibrary, setProjectFavorite, updateProject } = useLibrary();
const { settings } = useSettings();

const entries = ref<CatalogEntry[]>([]);
const loading = ref(false);
const addingLink = ref<string | null>(null);
const error = ref<string | null>(null);
const selectedDownloadEntry = ref<CatalogEntry | null>(null);
const selectedDownloadProjects = computed(() => selectedDownloadEntry.value ? matchingLibraryProjects(selectedDownloadEntry.value) : []);
const selectedDownloadProject = computed(() => {
  const matches = selectedDownloadProjects.value, current = matches.filter(p => !p.restored_from_archive);
  return current.length === 1 ? current[0] : matches.length === 1 ? matches[0] : null;
});
const { authors, loadAuthorAliases } = useAuthorAliases();
const snapshot = ref<CatalogSnapshot | null>(null);
const advancedFilters = ref(false);
const page = ref(1);
const PAGE_SIZE = 48;
const membershipFilter = ref('');
const statusFilter = ref('');
const sourceFilter = ref('');
const showDiscoveries = ref(false);
const stopBatch = ref(false);
const failedKeys = ref<string[]>([]);
const search = ref("");
const sort = ref<"name" | "date">("date");
const authorFilter = ref("");
const universeFilter = ref("");
const importerFilter = ref("");
const typeFilter = ref("");
const povFilter = ref("");
const lengthFilter = ref("");
const tagFilter = ref("");
const favoriteFilter = ref<"all" | "favorites">("all");
const successMessage = ref<string | null>(null);
const addStatus = ref("");
const selectedCatalogKeys = ref<string[]>([]);
const batchDownloadInProgress = ref(false);
const batchDownloadCompleted = ref(0);
const batchDownloadTotal = ref(0);
const batchDownloadMessage = ref("");
const batchDownloadFailed = ref(0);
type CatalogListEntry = CatalogEntry & {
  catalogKey: string;
  hostLabel: string | null;
  existingProjectId: string | null;
  existingProjectFavorite: boolean;
  existingProjectCount: number;
};



const libraryProjectsBySource = computed(() => {
  const map = new Map<string, Project[]>();
  for (const project of projects.value) {
    for (const raw of [project.source_url, project.project_json_url]) {
      const key = normalizeLibrarySourceUrl(raw || "");
      if (!key) continue;
      const matches = map.get(key) || [];
      if (!matches.some(item => item.id === project.id)) matches.push(project);
      map.set(key, matches);
    }
  }
  return map;
});
function matchingLibraryProjects(entry: CatalogEntry): Project[] {
  const matches = new Map<string, Project>();
  for (const raw of [entry.website, ...(entry.alternateUrls || []), ...(entry.projectUrls || [])]) {
    for (const project of libraryProjectsBySource.value.get(normalizeLibrarySourceUrl(raw)) || []) matches.set(project.id, project);
  }
  return [...matches.values()];
}
function preferredLibraryProject(matches: Project[]): Project | undefined {
  const current = matches.filter(project => !project.restored_from_archive);
  return current.length === 1 ? current[0] : matches[0];
}

function canonicalCatalogAuthors(raw = ''): string[] {
  // Preserve exact aliases containing separators when explicitly configured.
  const resolved = resolveAuthor(raw);
  if(resolved !== raw) return [resolved];
  return raw.split(/[,;\n]+/).map(name => resolveAuthor(name.trim())).filter(Boolean);
}
const indexedList = computed(() => entries.value.map(entry => {
  const matches = matchingLibraryProjects(entry), existingProject = preferredLibraryProject(matches);
  const canonicalAuthors = canonicalCatalogAuthors(entry.author);
  const author = canonicalAuthors.join(", ");
  const aliases = authors.value.filter(item => canonicalAuthors.includes(item.name)).flatMap(item => [item.name, ...item.aliases]);
  return { ...entry, source_author: entry.author, author, catalogKey: buildCatalogKey(entry), hostLabel: extractHostLabel(entry.website), existingProjectId: existingProject?.id || null, existingProjectFavorite: matches.some(project => project.favorite), existingProjectCount: matches.length, searchIndex: indexCatalogEntry(entry,author,aliases) };
}));
const displayedList = computed(() => {
  let list = indexedList.value.slice();
  const tokens = parseCatalogSearch(search.value);

  list = list.filter(entry => (showDiscoveries.value || entry.sourceId !== "discoveries") && (!membershipFilter.value || (membershipFilter.value === "library" ? Boolean(entry.existingProjectId) : !entry.existingProjectId)) && (!statusFilter.value || entry.status === statusFilter.value) && (!sourceFilter.value || entry.sourceId === sourceFilter.value));
  if (tokens.length > 0) {
    list = list.filter((entry) => matchesCatalogSearch(entry.searchIndex, tokens));
  }

  if (authorFilter.value) {
    list = list.filter((entry) => canonicalCatalogAuthors(entry.author).includes(authorFilter.value));
  }

  if (universeFilter.value) {
    list = list.filter((entry) => entry.universe === universeFilter.value);
  }

  if (importerFilter.value) {
    list = list.filter((entry) => entry.importer === importerFilter.value);
  }

  if (typeFilter.value) {
    list = list.filter((entry) => entry.type === typeFilter.value);
  }

  if (povFilter.value) {
    list = list.filter((entry) => entry.pov === povFilter.value);
  }

  if (lengthFilter.value) {
    list = list.filter((entry) => entry.length === lengthFilter.value);
  }

  if (tagFilter.value) {
    list = list.filter((entry) => entry.tags?.includes(tagFilter.value));
  }

  if (favoriteFilter.value === "favorites") {
    list = list.filter((entry) => entry.existingProjectFavorite);
  }

  if (sort.value === "name") {
    list.sort((left, right) => left.name.localeCompare(right.name));
  } else {
    list.sort((left, right) => (Date.parse(right.date) || 0) - (Date.parse(left.date) || 0));
  }

  return list;
});

const authorOptions = computed(() => buildOptionList(entries.value.flatMap((entry) => canonicalCatalogAuthors(entry.author))));
const universeOptions = computed(() => buildOptionList(entries.value.map((entry) => entry.universe)));
const importerOptions = computed(() => buildOptionList(entries.value.map((entry) => entry.importer)));
const typeOptions = computed(() => buildOptionList(entries.value.map((entry) => entry.type)));
const povOptions = computed(() => buildOptionList(entries.value.map((entry) => entry.pov)));
const lengthOptions = computed(() => buildOptionList(entries.value.map((entry) => entry.length)));
const tagOptions = computed(() => buildOptionList(entries.value.flatMap((entry) => entry.tags || [])));
const statusOptions = computed(() => buildOptionList(entries.value.map(entry => entry.status)));
const pageCount = computed(() => Math.max(1, Math.ceil(displayedList.value.length / PAGE_SIZE)));
const pageEntries = computed(() => displayedList.value.slice((page.value-1)*PAGE_SIZE,page.value*PAGE_SIZE));
const selectionOptions = { search, sort, authorFilter, universeFilter, importerFilter, typeFilter, povFilter, lengthFilter, tagFilter, favoriteFilter, membershipFilter, statusFilter, sourceFilter, showDiscoveries };
try {
  const saved = JSON.parse(localStorage.getItem('catalog-discovery-filters-v1') || '{}');
  for(const [key,value] of Object.entries(selectionOptions)) {
    if(typeof saved[key] === typeof value.value) (value as {value: unknown}).value = saved[key];
  }
} catch { /* Storage may be disabled or malformed; filters remain usable. */ }
watch(Object.values(selectionOptions), () => {
  page.value = 1;
  try { localStorage.setItem('catalog-discovery-filters-v1', JSON.stringify(Object.fromEntries(Object.entries(selectionOptions).map(([key,value])=>[key,value.value])))); } catch { /* Optional persistence. */ }
});
watch(pageCount, count => { page.value = Math.min(page.value,count); });
function clearFilters() { for(const [key,value] of Object.entries(selectionOptions)) (value as {value: unknown}).value = key === 'sort' ? 'date' : key === 'favoriteFilter' ? 'all' : key === 'showDiscoveries' ? false : ''; }
function togglePageSelection() {
 const keys=pageEntries.value.map(entry=>entry.catalogKey), remove=keys.every(key=>selectedCatalogKeySet.value.has(key)), next=new Set(selectedCatalogKeys.value);
 for(const key of keys) { if(remove)next.delete(key);else next.add(key); } selectedCatalogKeys.value=[...next];
}
const selectedCatalogKeySet = computed(() => new Set(selectedCatalogKeys.value));
const filteredCatalogKeys = computed(() => displayedList.value.map((entry) => entry.catalogKey));
const filteredSelectedCount = computed(() => filteredCatalogKeys.value.filter((key) => selectedCatalogKeySet.value.has(key)).length);
const hasSelectedEntries = computed(() => selectedCatalogKeys.value.length > 0);
const allFilteredSelected = computed(() => filteredCatalogKeys.value.length > 0 && filteredSelectedCount.value === filteredCatalogKeys.value.length);
const batchStatusVisible = computed(() => batchDownloadInProgress.value || batchDownloadTotal.value > 0);
const batchStatusText = computed(() => {
  if (batchDownloadTotal.value === 0) {
    return "";
  }

  return `Downloaded ${batchDownloadCompleted.value} / ${batchDownloadTotal.value}${batchDownloadFailed.value > 0 ? ` • Failed ${batchDownloadFailed.value}` : ""}`;
});

onMounted(() => {
  void loadLibrary();
  void loadAuthorAliases();
  void loadCatalog();
});

onBeforeUnmount(() => { stopBatch.value = true; });

async function loadCatalog(force = false) {
  if(loading.value) return;
  loading.value = true; error.value = null;
  let backgroundRefresh = false;
  try {
    const zipFallbacks = await loadZipFallbackMap().catch(() => new Map<string,CatalogEntry>());
    let result: CatalogSnapshot;
    try { result = await invoke<CatalogSnapshot>('get_catalog_snapshot',{force}); }
    catch(e) {
      if(!/unknown command|command .*not found|unsupported command/i.test(String(e))) throw e;
      result = {sources:[{id:'beta',name:'Beta Index',rows:await invoke<string[][]>('get_catalog_rows'),fetchedAt:'',stale:false,error:null}],fetchedAt:'',stale:false,warnings:['Legacy catalog backend; persistent snapshots unavailable.']};
    }
    snapshot.value = result;
    entries.value = normalizeCatalogSnapshot(result,zipFallbacks);
    backgroundRefresh = result.stale && !force;
  } catch(e) { error.value = String(e); }
  finally { loading.value = false; }
  if(backgroundRefresh) void loadCatalog(true);
}
async function addEntry(entry: CatalogEntry) {
  if(batchDownloadInProgress.value || !canAddCatalogEntry(entry)) return;
  selectedDownloadEntry.value = entry;
}
async function catalogChanged() { await loadLibrary(true); }

async function toggleCatalogFavorite(entry: CatalogListEntry) {
  if (!entry.existingProjectId) {
    return;
  }

  await setProjectFavorite(entry.existingProjectId, !entry.existingProjectFavorite);
}

async function downloadSelectedEntries() {
  if (batchDownloadInProgress.value || selectedCatalogKeys.value.length === 0) {
    return;
  }

  const queue = indexedList.value.filter((entry) => selectedCatalogKeySet.value.has(entry.catalogKey) && canAddCatalogEntry(entry));
  if (queue.length === 0) {
    batchDownloadCompleted.value = 0;
    batchDownloadTotal.value = 0;
    batchDownloadFailed.value = 0;
    batchDownloadMessage.value = "Selected entries need a playable link before they can be downloaded.";
    return;
  }

  batchDownloadInProgress.value = true;
  batchDownloadCompleted.value = 0;
  batchDownloadTotal.value = queue.length;
  batchDownloadFailed.value = 0;
  batchDownloadMessage.value = `Queued ${queue.length} entr${queue.length === 1 ? "y" : "ies"}`;
  error.value = null;
  successMessage.value = null;

  let completed = 0;
  let failed = 0;

  try {
    stopBatch.value = false; failedKeys.value = [];
    for (const entry of queue) {
      if(stopBatch.value) { batchDownloadMessage.value = "Queue stopped; remaining entries stay selected."; break; }
      batchDownloadMessage.value = `Downloading ${entry.name}`;
      const success = await downloadCatalogEntry(entry, getExistingCatalogProjectId(entry));
      if (!success) {


        failedKeys.value.push(entry.catalogKey);
        failed += 1;
        batchDownloadFailed.value = failed;
        batchDownloadMessage.value = `Skipped ${entry.name}`;
        await yieldToUi();
        continue;
      }

      completed += 1;
      batchDownloadCompleted.value = completed;
      batchDownloadMessage.value = `Finished ${entry.name}`;
      removeSelectedCatalogKey(entry.catalogKey);
      await yieldToUi();
    }

    if (completed + failed === queue.length) {
      successMessage.value = failed === 0
        ? `Added ${completed} catalog entr${completed === 1 ? "y" : "ies"} to the library.`
        : `Added ${completed} catalog entr${completed === 1 ? "y" : "ies"}; skipped ${failed} failed download${failed === 1 ? "" : "s"}.`;
      batchDownloadMessage.value = failed === 0 ? "Batch complete" : "Batch complete with skipped failures";
    }
  } finally {
    batchDownloadInProgress.value = false;
      }
}

async function downloadCatalogEntry(entry: CatalogEntry, existingProjectId: string | null): Promise<boolean> {
  addingLink.value = buildCatalogKey(entry); error.value = null; addStatus.value = "Inspecting and downloading…";
  try {
    if (existingProjectId) throw new Error("This entry is already in your library. Open Check for updates to review its differences individually.");
    const plan = await invoke<{mode: string; reason: string}>("inspect_catalog_download", {websiteUrl:entry.website || entry.projectUrls?.[0] || "", zipUrl:null});
    if (plan.mode !== "project") throw new Error(`${entry.name}: ${plan.reason} Open Download options to choose a website copy, online play or recovery.`);
    const project = await invoke<Project>("import_catalog_download", {websiteUrl:entry.website || entry.projectUrls?.[0] || "", projectName:entry.name, maxSizeMb:settings.value.downloadSizeLimitMb, archiveUrl:null, allowArchive:false});
    const patch = catalogMetadataPatch(project, entry);
    await loadLibrary(true);
    if (Object.keys(patch).length) { try { await updateProject(project.id, patch); } catch (failure) { error.value = `Downloaded ${entry.name}; optional catalog metadata could not be filled: ${String(failure)}`; } }
    return true;
  } catch (failure) { error.value = String(failure); return false; }
  finally { addingLink.value = null; }
}

function isCatalogEntrySelected(entryKey: string): boolean {
  return selectedCatalogKeySet.value.has(entryKey);
}

function toggleCatalogEntrySelection(entryKey: string) {
  if (selectedCatalogKeySet.value.has(entryKey)) {
    removeSelectedCatalogKey(entryKey);
    return;
  }

  selectedCatalogKeys.value = [...selectedCatalogKeys.value, entryKey];
}

function removeSelectedCatalogKey(entryKey: string) {
  selectedCatalogKeys.value = selectedCatalogKeys.value.filter((key) => key !== entryKey);
}

function handleSelectAllFiltered(event: Event) {
  const checked = (event.target as HTMLInputElement).checked;
  if (!checked) {
    const filteredSet = new Set(filteredCatalogKeys.value);
    selectedCatalogKeys.value = selectedCatalogKeys.value.filter((key) => !filteredSet.has(key));
    return;
  }

  const next = new Set(selectedCatalogKeys.value);
  for (const key of filteredCatalogKeys.value) {
    next.add(key);
  }
  selectedCatalogKeys.value = [...next];
}

async function yieldToUi() {
  await nextTick();
  await new Promise((resolve) => window.setTimeout(resolve, 0));
}

async function fetchCatalogEntries(url: string): Promise<CatalogEntry[]> {
  const response = await fetch(url, { cache: "no-store" });
  if (!response.ok) {
    throw new Error(`HTTP ${response.status}`);
  }

  const source = await response.text();
  return parseCatalogData(source);
}

async function loadZipFallbackMap(): Promise<Map<string, CatalogEntry>> {
  const fallbackEntries = await fetchCatalogEntries(LOCAL_CATALOG_URL);
  const map = new Map<string, CatalogEntry>();

  for (const entry of fallbackEntries) {
    const key = normalizeCatalogWebsite(entry.website);
    const current = map.get(key);
    if (!current || compareCatalogDates(entry.date, current.date) > 0) {
      map.set(key, entry);
    }
  }

  return map;
}

function normalizeLibrarySourceUrl(raw: string): string {
  const trimmed = raw.trim();
  if (!trimmed) {
    return "";
  }

  try {
    const url = new URL(trimmed);
    if (url.protocol !== "http:" && url.protocol !== "https:") {
      return "";
    }


    const segments = url.pathname.split("/").filter(Boolean);
    const lastSegment = segments[segments.length - 1]?.toLowerCase() || "";

    if (lastSegment === "project.json") {
      segments.pop();
      url.pathname = segments.length > 0 ? `/${segments.join("/")}/` : "/";
    } else if (!url.pathname) {
      url.pathname = "/";
    }

    return url.toString();
  } catch {
    return trimmed;
  }
}

function compareCatalogDates(left:string,right:string):number{return left.localeCompare(right);}
function normalizeCatalogWebsite(website:string):string{return catalogUrl(website);}
function canAddCatalogEntry(entry: CatalogEntry): boolean {return Boolean(catalogUrl(entry.website) || entry.projectUrls?.length || entry.recoveryUrls?.length);}


function buildCatalogKey(entry: CatalogEntry): string {
  return catalogKey(entry);
}

function buildOptionList(values: Array<string | undefined>): string[] {
  return [...new Set(values.map((value) => value?.trim()).filter((value): value is string => Boolean(value)))].sort((left, right) => left.localeCompare(right));
}

function parseCatalogData(source: string): CatalogEntry[] {
  const marker = "window.CATALOG_DATA =";
  const start = source.indexOf(marker);
  if (start === -1) {
    throw new Error("CATALOG_DATA assignment was not found");
  }

  let payload = source.slice(start + marker.length).trim();
  if (payload.endsWith(";")) {
    payload = payload.slice(0, -1);
  }

  const parsed = JSON.parse(payload);
  if (!Array.isArray(parsed)) {
    throw new Error("CATALOG_DATA is not an array");
  }

  const entries = parsed.filter(isCatalogEntry);
  if (entries.length === 0) {
    throw new Error("Catalog did not contain any usable entries");
  }

  return entries;
}

function isCatalogEntry(value: unknown): value is CatalogEntry {
  if (!value || typeof value !== "object") {
    return false;
  }

  const candidate = value as Record<string, unknown>;
  return typeof candidate.name === "string"
    && typeof candidate.date === "string"
    && typeof candidate.website === "string"
    && typeof candidate.link === "string"
    && (candidate.engine === undefined || typeof candidate.engine === "string");
}

async function copyWebsiteLink(entry: CatalogEntry) {
  try {
    await navigator.clipboard.writeText(entry.website);
    successMessage.value = `Copied link for ${entry.name}.`;
    error.value = null;
  } catch (copyError) {
    error.value = copyError instanceof Error ? copyError.message : String(copyError);
  }
}

async function openCatalogWebsite(entry: CatalogEntry) {
  try {
    await openUrl(entry.website);
  } catch (openError) {
    error.value = openError instanceof Error ? openError.message : String(openError);
  }
}

function getExistingCatalogProjectId(entry: CatalogEntry): string | null {
  return preferredLibraryProject(matchingLibraryProjects(entry))?.id || null;
}

function extractHostLabel(website: string): string | null {
  try {
    return new URL(website).hostname.replace(/^www\./, "");
  } catch {
    return null;
  }
}

function getTypeBadgeClass(type: string | undefined): string {
  const normalized = (type || "").toLowerCase();

  if (normalized === "nsfw") {
    return "badge-type-nsfw";
  }

  if (normalized === "sfw") {
    return "badge-type-sfw";
  }

  return "badge-type-default";
}
</script>

<template>
  <div class="catalog-view">
    <div class="toolbar">
      <input
        v-model="search"
        class="search"
        type="text"
        placeholder="Search catalog"
        :title="'Search aliases, titles and metadata. Use &quot;quoted phrases&quot;, -word to exclude matches, author:, fandom:, tag: or title:.'"
        aria-label="Search catalog"
      />

      <select v-model="sort" class="filter-select" title="Sort">
        <option value="date">Catalog date: newest first</option>
        <option value="name">Name A-Z</option>
      </select>

      <span class="sync-status" role="status">{{ snapshot?.fetchedAt ? `Last sync: ${new Date(snapshot.fetchedAt).toLocaleString()}` : 'Catalog sources' }}{{ snapshot?.stale ? ' · Cached / stale' : '' }}</span>
      <div class="toolbar-spacer" />

      <div class="toolbar-actions">
        <label v-if="displayedList.length > 0" class="select-all-control">
          <input
            type="checkbox"
            :checked="allFilteredSelected"
            :disabled="batchDownloadInProgress"
            @click.stop
            @change="handleSelectAllFiltered"
          />
          <span>Select all {{ displayedList.length }} results</span>
        </label>

        <button class="btn-secondary" @click="togglePageSelection" :disabled="batchDownloadInProgress">Select / clear page</button>
        <button v-if="selectedCatalogKeys.length" class="btn-secondary" @click="selectedCatalogKeys = []" :disabled="batchDownloadInProgress">Clear selection</button>
        <div v-if="hasSelectedEntries || batchStatusVisible" class="batch-download-group">
          <button
            class="btn-primary"
            :disabled="loading || addingLink !== null || batchDownloadInProgress || !hasSelectedEntries"
            @click="downloadSelectedEntries"
          >
            {{ batchDownloadInProgress ? "Downloading…" : `Download Selected (${selectedCatalogKeys.length})` }}
          </button>
          <button v-if="batchDownloadInProgress" class="btn-secondary" @click="stopBatch = true">Stop after current</button>
          <button v-if="failedKeys.length && !batchDownloadInProgress" class="btn-secondary" @click="selectedCatalogKeys = [...failedKeys]; downloadSelectedEntries()">Retry failed</button>
          <div v-if="batchStatusVisible" class="batch-download-status">
            <div>{{ batchStatusText }}</div>
            <div v-if="batchDownloadMessage">{{ batchDownloadMessage }}</div>
          </div>
        </div>

        <button class="btn-secondary" :disabled="loading || batchDownloadInProgress" @click="loadCatalog(true)">
          {{ loading ? "Loading…" : "Reload Catalog" }}
        </button>
      </div>
    </div>

    <div class="filter-row">
      <select v-model="membershipFilter" class="filter-select" aria-label="Library membership"><option value="">All library states</option><option value="library">In library</option><option value="missing">Not in library</option></select>
      <select v-model="favoriteFilter" style="max-width: 100px;" class="filter-select" aria-label="Filter by favorites">
        <option value="all">All ♥♡ </option>
        <option value="favorites">Only Favorites ♥</option>
      </select>

      <select v-model="authorFilter" class="filter-select" aria-label="Filter by author">
        <option value="">All authors</option>
        <option v-for="option in authorOptions" :key="option" :value="option">{{ option }}</option>
      </select>

      <select v-model="universeFilter" class="filter-select" aria-label="Filter by universe">
        <option value="">All universes</option>
        <option v-for="option in universeOptions" :key="option" :value="option">{{ option }}</option>
      </select>

      <button class="btn-secondary" @click="advancedFilters = !advancedFilters" :aria-expanded="advancedFilters">More filters</button>
      <button class="btn-secondary" @click="clearFilters">Clear filters</button>
      <div v-if="advancedFilters" class="advanced-filters">
      <select v-model="importerFilter" class="filter-select" aria-label="Filter by importer">
        <option value="">All importers</option>
        <option v-for="option in importerOptions" :key="option" :value="option">{{ option }}</option>
      </select>

      <select v-model="typeFilter" class="filter-select" aria-label="Filter by type">
        <option value="">All types</option>
        <option v-for="option in typeOptions" :key="option" :value="option">{{ option }}</option>
      </select>

      <select v-model="povFilter" class="filter-select" aria-label="Filter by point of view">
        <option value="">All PoV</option>
        <option v-for="option in povOptions" :key="option" :value="option">{{ option }}</option>
      </select>

      <select v-model="lengthFilter" style="max-width: 100px;" class="filter-select" aria-label="Filter by length">
        <option value="">All lengths</option>
        <option v-for="option in lengthOptions" :key="option" :value="option">{{ option }}</option>
      </select>

      <select v-model="tagFilter" style="max-width: 100px;" class="filter-select" aria-label="Filter by tag">
        <option value="">All tags</option>
        <option v-for="option in tagOptions" :key="option" :value="option">{{ option }}</option>
      </select>

      <select v-model="statusFilter" class="filter-select" aria-label="Catalog status"><option value="">All catalog statuses</option><option v-for="option in statusOptions" :key="option">{{ option }}</option></select>
      <select v-model="sourceFilter" class="filter-select" aria-label="Catalog source"><option value="">All sources</option><option value="beta">Beta Index</option><option value="projects">Project links</option><option value="broken">Broken links / recovery</option><option value="discoveries">Unreviewed discoveries</option></select>
      <label><input v-model="showDiscoveries" type="checkbox" /> Include unreviewed discoveries</label>
      </div>
      <div class="status-bar">
        <span>{{ displayedList.length }} / {{ entries.length }} entries</span>
      </div>
    </div>



    <details v-if="snapshot" class="source-health"><summary>Source freshness</summary><div v-for="source in snapshot.sources" :key="source.id">{{ source.name }}: {{ source.stale ? "Cached / stale" : "Current snapshot" }} · {{ source.fetchedAt || "Never fetched" }}<span v-if="source.error"> · {{ source.error }}</span></div></details>
    <div v-if="snapshot?.warnings.length" class="banner">{{ snapshot.warnings.join(" · ") }}</div>
    <div v-if="successMessage" class="banner success">{{ successMessage }}</div>
    <div v-if="error" class="banner error">{{ error }}</div>

    <div v-if="loading && !entries.length" class="center-msg">Loading catalog…</div>
    <div v-else-if="displayedList.length === 0" class="center-msg">No catalog entries match your filter.</div>

    <div v-else class="grid">
      <article
        v-for="entry in pageEntries"
        :key="entry.catalogKey"
        class="card"
        :class="{ selected: isCatalogEntrySelected(entry.catalogKey) }"
        @click="toggleCatalogEntrySelection(entry.catalogKey)"
      >
        <div v-if="entry.type" class="badge badge-type" :class="getTypeBadgeClass(entry.type)">{{ entry.type }}</div>
        <div class="info">
          <div class="card-head">
              <input
                type="checkbox"
                :aria-label="`Select ${entry.name}`"
                @click.stop
                :checked="isCatalogEntrySelected(entry.catalogKey)"
                :disabled="batchDownloadInProgress"
                @change="toggleCatalogEntrySelection(entry.catalogKey)"
              />

            <div class="title-block">
              <h3 class="name" :title="entry.name">{{ entry.name }}</h3>
              <div v-if="entry.author" class="author-line">
                <span class="author-label">Author</span>
                <span class="author-name">{{ entry.author }}</span>
              </div>
              <div class="byline">
                <span v-if="entry.universe">{{ entry.universe }}</span>
                <span v-if="entry.hostLabel">{{ entry.hostLabel }}</span>
                <span v-if="entry.existingProjectCount > 1">{{ entry.existingProjectCount }} library editions — choose one in Download options</span>
                <span v-if="entry.importer">Imported by {{ entry.importer }}</span>
                <span v-if="entry.length">{{ entry.length }}</span>
              </div>
            </div>
            <div class="badge badge-date">{{ entry.date }}</div>
          </div>

          <div class="entry-status">{{ entry.sourceId === "discoveries" ? "Unreviewed discovery · " : "" }}{{ entry.availability }}<span v-if="entry.status"> · Catalog: {{ entry.status }}</span><span v-if="entry.version"> · {{ entry.version }}</span></div>
          <p v-if="entry.warnings" class="entry-warning">{{ entry.warnings }}</p>
          <details v-if="entry.aliases?.length || entry.alternateUrls?.length || entry.projectUrls?.length || entry.recoveryUrls?.length || entry.staticUrls?.length || entry.comments" @click.stop><summary>Source details and links</summary><div v-if="entry.aliases?.length">Also known as: {{ entry.aliases.join("; ") }}</div><div>Added: {{ entry.added || "Unknown" }} · Posted: {{ entry.posted || "Unknown" }} · Updated: {{ entry.updated || "Unknown" }}</div><div v-for="group in [{label: 'Alternate',urls:entry.alternateUrls},{label:'Project',urls:entry.projectUrls},{label:'Archived recovery (age unverified)',urls:entry.recoveryUrls},{label:'Static',urls:entry.staticUrls}]" :key="group.label"><button v-for="url in group.urls" :key="url" class="source-link source-detail-link" @click="openUrl(url)">{{ group.label }}: {{ url }}</button></div><p v-if="entry.comments">{{ entry.comments }}</p><div>Source: {{ entry.provenance?.join(" · ") }}</div></details>
          <p v-if="entry.description" class="description">{{ entry.description }}</p>

          <div v-if="entry.tags?.length" class="tag-list">
            <span v-for="tag in entry.tags" :key="tag" class="tag-chip">{{ tag }}</span>
            <span v-if="entry.pov?.trim()" class="tag-chip">PoV: {{ entry.pov }}</span>
          </div>

          <p v-if="addingLink === entry.catalogKey" role="status">{{ addStatus }}</p>

          <div class="actions">
            <button class="btn-secondary action-link" :disabled="!entry.website" @click.stop="openCatalogWebsite(entry)">
              Open Site
            </button>
            <button class="btn-secondary" :disabled="!entry.website" @click.stop="copyWebsiteLink(entry)">Copy Link</button>
            <button
              v-if="entry.existingProjectId"
              class="favorite-action-btn"
              :disabled="entry.existingProjectCount > 1"
              :class="{ active: entry.existingProjectFavorite }"
              :title="entry.existingProjectFavorite ? 'Remove favorite' : 'Add favorite'"
              @click.stop="toggleCatalogFavorite(entry)"
            >
              {{ entry.existingProjectFavorite ? "♥" : "♡" }}
            </button>
            <button
              :class="'btn-primary'"
              :disabled="addingLink !== null || batchDownloadInProgress || !canAddCatalogEntry(entry)"
              :title="canAddCatalogEntry(entry) ? undefined : 'A valid website link is required'"
              @click.stop="addEntry(entry)"
            >
              {{ addingLink === entry.catalogKey ? "Working…" : entry.existingProjectId ? "Check for updates" : canAddCatalogEntry(entry) ? "Download options" : "No download link" }}
            </button>
          </div>
        </div>
      </article>
    </div>

    <nav class="pagination" aria-label="Catalog pages"><button class="btn-secondary" :disabled="page <= 1" @click="page--">Previous</button><span>Page {{ page }} / {{ pageCount }} · {{ displayedList.length }} results · {{ selectedCatalogKeys.length }} selected across pages</span><button class="btn-secondary" :disabled="page >= pageCount" @click="page++">Next</button></nav>
    <CatalogDownloadDialog v-if="selectedDownloadEntry" :entry="selectedDownloadEntry" :existing-project="selectedDownloadProject" :existing-projects="selectedDownloadProjects" @close="selectedDownloadEntry = null" @changed="catalogChanged" />
  </div>
</template>

<style scoped>
.catalog-view {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-height: 0;
  overflow-y: auto;
  --catalog-card-top-tint: transparent;--catalog-card-side-tint: transparent;--catalog-card-shadow: none;
  --catalog-date-bg: var(--input-bg);--catalog-date-border: var(--border);--catalog-date-color: var(--text);
  --catalog-author-label: var(--muted);--catalog-author-name: var(--text);--catalog-byline: var(--muted);
  --catalog-meta-bg: var(--input-bg);--catalog-meta-border: var(--border);--catalog-meta-highlight-bg: var(--input-bg);--catalog-meta-highlight-border: var(--border);--catalog-meta-label: var(--muted);
  --catalog-description: var(--text);--catalog-tag-bg: var(--tag-bg);--catalog-tag-border: var(--border);--catalog-tag-color: var(--tag-color);--catalog-source-link: var(--accent);
}

.source-link {
  color: var(--catalog-source-link);
}

.source-link:hover {
  color: var(--accent-hover);
}

.confirm-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.6);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 210;
}

.confirm-dialog {
  width: 420px;
  max-width: calc(100vw - 32px);
  background: var(--dialog-bg);
  border: 1px solid var(--border);
  border-radius: 12px;
  padding: 20px;
  box-shadow: 0 10px 30px rgba(0, 0, 0, 0.35);
}

.oversize-remember-choice {
  margin-top: 12px;
  display: flex;
  align-items: center;
  gap: 8px;
  color: var(--muted);
  font-size: 0.85rem;
}

.oversize-remember-choice input {
  width: 15px;
  height: 15px;
  accent-color: var(--accent);
}

.toolbar {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 10px;
  padding: 14px 20px;
  border-bottom: 1px solid var(--border);
  flex-shrink: 0;
  position: sticky;
  top: 0;
  z-index: 2;
  background: var(--bg);
}

.search {
  flex: 1;
  min-width: 150px;
  padding: 7px 12px;
  background: var(--input-bg);
  border: 1px solid var(--border);
  border-radius: 8px;
  color: var(--text);
  font-size: 0.9rem;
  outline: none;
}

.search:focus,
.filter-select:focus {
  border-color: var(--accent);
}

.filter-select {
  max-width: 200px;
  padding: 7px 10px;
  background: var(--input-bg);
  border: 1px solid var(--border);
  border-radius: 8px;
  color: var(--text);
  font-size: 0.875rem;
  outline: none;
  cursor: pointer;
}

.toolbar-spacer {
  flex: 1;
}

.toolbar-actions {
  flex-wrap: wrap;
  display: flex;
  align-items: flex-start;
  gap: 10px;
}

.select-all-control {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  min-height: 34px;
  padding: 0 10px;
  border: 1px solid var(--border);
  border-radius: 8px;
  background: var(--panel-bg);
  color: var(--muted);
}

.batch-download-group {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.batch-download-status {
  font-size: 0.78rem;
  line-height: 1.35;
  color: var(--muted);
}

.filter-row {
  display: flex;
  flex-wrap: wrap;
  gap: 10px;
  padding: 12px 20px 0;
}

.status-bar {
  margin-left: auto;
  padding: 12px 20px 0;
  color: var(--muted);
  font-size: 0.8rem;
}

.banner {
  margin: 12px 20px 0;
  padding: 10px 12px;
  border-radius: 8px;
  font-size: 0.85rem;
}

.banner.success {
  background: rgba(95, 179, 107, 0.14);
  border: 1px solid rgba(95, 179, 107, 0.45);
  color: var(--success);
}

.banner.error {
  background: rgba(200, 50, 50, 0.15);
  border: 1px solid var(--danger);
  color: var(--danger);
}

.source-health { padding:8px 20px; font-size:.8rem; color:var(--muted); }
.advanced-filters { display:flex; flex-wrap:wrap; gap:10px; width:100%; align-items:center; }
.sync-status,.entry-status { font-size:.8rem; color:var(--muted); }
.source-detail-link { display:block; overflow-wrap:anywhere; text-align:left; background:none; border:0; padding:4px 0; }
.entry-warning { color:var(--danger); margin:0; }
.pagination { display:flex; flex-wrap:wrap; justify-content:center; align-items:center; gap:12px; padding:16px; }
.grid {
  padding: 20px;
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(min(100%, 330px), 1fr));
  gap: 16px;
  align-content: start;
}

.card {
  position: relative;
  background:
    linear-gradient(180deg, var(--catalog-card-top-tint), rgba(110, 160, 255, 0) 120px),
    linear-gradient(135deg, var(--catalog-card-side-tint), transparent 55%),
    var(--card-bg);
  border: 1px solid var(--border);
  box-shadow: var(--catalog-card-shadow);
  border-radius: 14px;
  padding: 16px;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  cursor: pointer;
}

.card.selected {
  border-color: var(--accent);
  box-shadow: 0 0 0 1px color-mix(in srgb, var(--accent) 55%, transparent), var(--catalog-card-shadow);
}

.badge {
  padding: 2px 8px;
  border-radius: 999px;
  background: rgba(0, 0, 0, 0.28);
  border: 1px solid transparent;
  color: #fff;
  font-size: 0.72rem;
  font-weight: 400;
  white-space: nowrap;
}

.badge-date {
  background: var(--catalog-date-bg);
  border-color: var(--catalog-date-border);
  color: var(--catalog-date-color);
}

.badge-type {
  position: absolute;
  top: 14px;
  right: 14px;
  z-index: 1;
  padding: 6px 11px;
  font-size: 0.7rem;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.badge-type-nsfw {
  background: linear-gradient(135deg, rgba(255, 100, 124, 0.95), rgba(255, 152, 84, 0.95));
  color: #fff6f1;
}

.badge-type-sfw {
  background: linear-gradient(135deg, rgba(70, 191, 166, 0.95), rgba(80, 134, 255, 0.95));
  color: #f3fffd;
}

.badge-type-default {
  background: linear-gradient(135deg, rgba(114, 123, 144, 0.95), rgba(83, 92, 110, 0.95));
}

.info {
  display: flex;
  flex-direction: column;
  gap: 10px;
  flex: 1;
}

.card-head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 18px;
  flex-wrap: wrap;
  padding-right: 88px;
}

.entry-select {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  font-size: 0.78rem;
  color: var(--muted);
  flex-shrink: 0;
}

.title-block {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.name {
  margin: 0;
  font-size: 1.06rem;
  color: var(--text);
}

.author-line {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 8px;
}

.author-label {
  color: var(--catalog-author-label);
  font-size: 0.72rem;
  letter-spacing: 0.1em;
  text-transform: uppercase;
}

.author-name {
  color: var(--catalog-author-name);
  font-size: 0.92rem;
  font-weight: 400;
}

.byline {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  color: var(--catalog-byline);
  font-size: 0.82rem;
}

.byline span::after {
  content: "•";
  margin-left: 8px;
  opacity: 0.5;
}

.byline span:last-child::after {
  content: "";
  margin: 0;
}

.meta-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(130px, 1fr));
  gap: 10px;
}

.meta-item {
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding: 8px 10px;
  border-radius: 8px;
  background: var(--catalog-meta-bg);
  border: 1px solid var(--catalog-meta-border);
  font-size: 0.82rem;
  color: var(--text);
}

.meta-item-highlight {
  background: var(--catalog-meta-highlight-bg);
  border-color: var(--catalog-meta-highlight-border);
}

.meta-label {
  color: var(--catalog-meta-label);
  font-size: 0.72rem;
  text-transform: uppercase;
  letter-spacing: 0.04em;
}

.description {
  margin: 0;
  color: var(--catalog-description);
  font-size: 0.88rem;
  line-height: 1.5;
}

.tag-list {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.tag-chip {
  padding: 4px 10px;
  border-radius: 999px;
  background: var(--catalog-tag-bg);
  border: 1px solid var(--catalog-tag-border);
  color: var(--catalog-tag-color);
  font-size: 0.76rem;
}

.actions {
  display: flex;
  flex-wrap: wrap;
  gap: 10px;
  margin-top: auto;
}

.actions > * {
  flex: 1;
}

.action-link {
  display: inline-flex;
  align-items: center;
  justify-content: center;
}

.favorite-action-btn {
  flex: 0 0 48px;
  min-width: 48px;
  padding: 0;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border-radius: 10px;
  border: 1px solid rgba(255, 255, 255, 0.12);
  background: rgba(255, 255, 255, 0.05);
  color: rgba(255, 255, 255, 0.86);
  font-size: 1rem;
  line-height: 1;
  transition: background 0.15s, border-color 0.15s, color 0.15s, transform 0.15s;
}

.favorite-action-btn:hover {
  background: rgba(255, 255, 255, 0.1);
  border-color: rgba(255, 255, 255, 0.2);
  transform: translateY(-1px);
}

.favorite-action-btn.active {
  background: rgba(255, 122, 143, 0.14);
  border-color: rgba(255, 122, 143, 0.35);
  color: var(--danger);
}

.btn-overwrite {
  background: linear-gradient(135deg, #d97a25, #b74219);
  border-radius: 10px;
  color: #fff7ef;
  border: 1px solid rgba(142, 54, 23, 0.5);
}

.btn-overwrite:hover:not(:disabled) {
  filter: brightness(1.06);
}

.btn-overwrite:disabled {
  opacity: 0.45;
  cursor: not-allowed;
}

.center-msg {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--muted);
  font-size: 0.95rem;
  padding: 30px;
}
.badge-type-nsfw{background:var(--danger);color:var(--accent-text)}.badge-type-sfw{background:var(--accent);color:var(--accent-text)}.badge-type-default{background:var(--input-bg);color:var(--text)}
</style>
