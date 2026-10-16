<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { openUrl } from "@tauri-apps/plugin-opener";
import type { CatalogEntry, Project } from "../types";
import { catalogMetadataPatch } from "../catalogImport";
import { useSettings } from "../composables/useSettings";
const props = defineProps<{ entry: CatalogEntry; existingProject: Project | null; existingProjects?: Project[] }>();
const emit = defineEmits<{(e: "close"): void; (e: "changed", project: Project): void}>();
const { settings } = useSettings();
interface Plan { mode: "project" | "website" | "online"; sourceUrl: string; reason: string; projectUrl?: string; archiveUrl?: string; compatibility?: {editor: string; scripts: number; variable_operations: number} | null }
interface Preview { token: string; projectName: string; sourceUrl: string; diff: {changed: boolean; added: string[]; removed: string[]; edited: string[]; other_changes: string[]} }
const panel = ref<HTMLDialogElement | null>(null);
const plan = ref<Plan | null>(null), preview = ref<Preview | null>(null);
let mounted = true;
const busy = ref(false), error = ref(""), notice = ref(""), completed = ref(false);
function webUrl(value: string | undefined): string {
  try { const url = new URL(value || ""); return ["http:", "https:"].includes(url.protocol) && !url.username && !url.password ? url.href : ""; } catch { return ""; }
}
const candidates = computed(() => [...new Set([props.entry.website, ...(props.entry.alternateUrls || []), ...(props.entry.projectUrls || [])].map(webUrl).filter(Boolean))]);
const recovery = computed(() => [...new Set([props.entry.link, ...(props.entry.recoveryUrls || [])].map(webUrl).filter(Boolean))]);
const selected = ref("");
const websiteSource = computed(() => {
  for (const candidate of [selected.value, webUrl(props.entry.website)]) {
    if (!candidate) continue;
    if (new URL(candidate).pathname.toLowerCase().endsWith(".json") || (props.entry.projectUrls || []).includes(candidate)) continue;
    return candidate;
  }
  return "";
});
const libraryCandidates = computed(() => props.existingProjects || (props.existingProject ? [props.existingProject] : []));
const selectedProjectId = ref("");
const targetProject = computed(() => libraryCandidates.value.find(project => project.id === selectedProjectId.value) || null);
let previousFocus: HTMLElement | null = null;

const archive = ref("");
const canUpdate = computed(() => !!targetProject.value && !targetProject.value.restored_from_archive && targetProject.value.kind !== "website");
async function discard() {
  if (!preview.value) return;
  const token = preview.value.token; preview.value = null;
  await invoke("discard_catalog_update", {token});
}
async function work(action: () => Promise<void>) {
  if (busy.value) return;
  busy.value = true; error.value = "";
  try { await action(); } catch (e) { error.value = String(e); } finally { busy.value = false; }
}
async function inspect() {
  await work(async () => {
    await discard(); plan.value = null; notice.value = ""; completed.value = false;
    if (!selected.value) return;
    plan.value = await invoke<Plan>("inspect_catalog_download", {websiteUrl:selected.value, zipUrl:archive.value || null});
  });
}
async function close() {
  if (busy.value) return;
  await work(discard); panel.value?.close(); emit("close");
  await nextTick(); previousFocus?.focus();
}
async function showPreview() {
  await work(async () => {
    await discard();
    const result = await invoke<Preview>("preview_catalog_update", {projectId:targetProject.value!.id, websiteUrl:selected.value});
    if (!mounted) { await invoke("discard_catalog_update", {token:result.token}); return; }
    preview.value = result;
  });
}
async function applyPreview() {
  await work(async () => {
    if (!preview.value) return;
    // Applying consumes the backend token even if validation rejects the update.
    const token = preview.value.token;
    preview.value = null;
    const project = await invoke<Project>("apply_catalog_update", {token});
    completed.value = true; notice.value = "Reviewed update applied. The previous edition is preserved."; emit("changed", project);
  });
}
async function importProject(allowArchive = false) {
  await work(async () => {
    const project = await invoke<Project>("import_catalog_download", {websiteUrl:selected.value, projectName:props.entry.name, maxSizeMb:settings.value.downloadSizeLimitMb, archiveUrl:allowArchive ? archive.value : null, allowArchive});
    let metadataWarning = "";
    const patch = catalogMetadataPatch(project, props.entry);
    if (Object.keys(patch).length) {
      try { await invoke("update_project", {id:project.id, patch}); }
      catch (failure) { metadataWarning = ` Optional source metadata could not be filled: ${String(failure)}`; }
    }
    completed.value = true; notice.value = allowArchive ? "Added a separate recovery copy. Its age and compatibility may differ from the current source." : "Added a separate library card."; notice.value += metadataWarning; emit("changed", project);
  });
}
async function saveWebsite(url = websiteSource.value, recovered = false) {
  await work(async () => {
    const result = await invoke<{project:Project;unavailable:string[]}>("download_website", {
      url, title:`${props.entry.name} — ${recovered ? "recovery website" : "website copy"}`,
      author:props.entry.source_author || props.entry.author || "", fandom:props.entry.universe || "",
      description:`Separate ${recovered ? "recovery" : "original-viewer"} website copy. Verify offline availability; manager builds do not transfer automatically.`,
      maxSizeMb:settings.value.downloadSizeLimitMb, existingProjectId:null, forceUpdate:false,
    });
    completed.value = true; notice.value = `Added a separate website card. ${result.unavailable.length ? `${result.unavailable.length} resources could not be saved. ` : ""}Test offline before relying on it; dynamic services may still need the network.`; emit("changed", result.project);
  });
}
async function openWebsite(url = selected.value) { await work(async () => { await openUrl(url); }); }
onMounted(async () => { previousFocus = document.activeElement as HTMLElement | null; selectedProjectId.value = props.existingProject?.id || (libraryCandidates.value.length === 1 ? libraryCandidates.value[0].id : ""); selected.value = candidates.value[0] || ""; archive.value = recovery.value[0] || ""; panel.value?.showModal(); await inspect(); });
onBeforeUnmount(() => { mounted = false; panel.value?.close(); const token = preview.value?.token; if (token) void invoke("discard_catalog_update",{token}).catch(console.error); });
</script>
<template>
  <dialog ref="panel" class="download-dialog" :aria-label="`${entry.name} — Download options`" @cancel.prevent="close" @click="event => { if (event.target === panel) close(); }">
    <h2>{{ entry.name }} — Download options</h2>
    <p>Catalog engine: {{ entry.engine || 'unknown' }}. The download method comes from inspecting the selected source.</p>
    <p v-if="entry.warnings" class="warning">Source warnings: {{ entry.warnings }}</p>
    <label v-if="candidates.length">Source / alternate / project file
      <select v-model="selected" :disabled="busy || completed" @change="inspect"><option v-for="url in candidates" :key="url" :value="url">{{ url }}</option></select>
    </label>
    <p v-else>No direct playable source is recorded. Use an explicitly chosen recovery link below.</p>
    <p v-if="plan" role="status">{{ plan.reason }}</p>
    <p v-if="plan?.mode === 'website' && !websiteSource">No playable website URL is recorded for this project file. Add its original playable page before saving a website copy.</p>
    <p v-if="plan?.compatibility?.scripts">{{ plan.compatibility.scripts }} scripted expressions were detected. No scripts were executed or converted.</p>
    <label v-if="libraryCandidates.length > 1">Library edition to update
      <select v-model="selectedProjectId" :disabled="busy || !!preview || completed" @change="work(discard)"><option value="">Choose an edition to preview</option><option v-for="project in libraryCandidates" :key="project.id" :value="project.id">{{ project.title || project.name }} · {{ project.id.slice(0,8) }}{{ project.restored_from_archive ? ' · restored copy' : '' }}</option></select>
    </label>
    <p v-if="targetProject?.restored_from_archive">This is a restored edition. Updating it could replace the older content you intended to keep. Import a separate card instead.</p>
    <section v-if="preview" class="preview" aria-label="Update preview">
      <h3>Review actual content changes</h3>
      <p v-if="!preview.diff.changed">No content changes found. The current edition can be kept.</p>
      <p v-else>{{ preview.diff.added.length }} added · {{ preview.diff.removed.length }} removed · {{ preview.diff.edited.length }} edited choices.</p>
      <template v-for="(items, category) in {Added:preview.diff.added, Removed:preview.diff.removed, Edited:preview.diff.edited, Other:preview.diff.other_changes}" :key="category"><h4 v-if="items.length">{{ category }}</h4><ul v-if="items.length"><li v-for="item in items" :key="item">{{ item }}</li></ul></template>
      <button class="btn-primary" :disabled="busy || completed" @click="applyPreview">Apply reviewed update</button>
    </section>
    <div v-if="!completed" class="actions">
      <button v-if="plan?.mode === 'project' && canUpdate && !preview" class="btn-primary" :disabled="busy" @click="showPreview">Preview update</button>
      <button v-if="plan?.mode === 'project' && !canUpdate" class="btn-primary" :disabled="busy" @click="importProject()">Download project as separate card</button>
      <button v-if="websiteSource" class="btn-secondary" :disabled="busy" @click="saveWebsite()">Save separate website copy</button>
      <button v-if="selected" class="btn-secondary" :disabled="busy" @click="openWebsite()">Open source website</button>
    </div>
    <details v-if="recovery.length && !completed" class="recovery">
      <summary>Recovery links — may be older or incomplete</summary>
      <p>Recovery never replaces your current edition. Confirm the source and version before reusing any build.</p>
      <label>Recovery source<select v-model="archive" :disabled="busy"><option v-for="url in recovery" :key="url" :value="url">{{ url }}</option></select></label>
      <div class="actions"><button class="btn-secondary" :disabled="busy || !archive" @click="importProject(true)">Download archive as separate copy</button><button class="btn-secondary" :disabled="busy || !archive" @click="saveWebsite(archive,true)">Save recovery website copy</button><button class="btn-secondary" :disabled="busy || !archive" @click="openWebsite(archive)">Open recovery link</button></div>
    </details>
    <p v-if="error" role="alert" class="warning">{{ error }}</p><p v-if="notice" role="status">{{ notice }}</p>
    <p v-if="busy" role="status">Working… Downloads may take several minutes.</p>
    <button class="btn-secondary close" :disabled="busy" @click="close">Close</button>
  </dialog>
</template>
<style scoped>
.download-dialog {color:var(--text);background:var(--dialog-bg);border:1px solid var(--border);border-radius:12px;width:min(720px,calc(100vw - 32px));max-height:90vh;overflow:auto;padding:24px;box-sizing:border-box;}
.download-dialog::backdrop {background:#0009;}
h2 {margin-top:0;} p {line-height:1.5;} label {display:grid;gap:8px;}
select {max-width:100%;width:100%;background:var(--input-bg);color:var(--text);border:1px solid var(--border);border-radius:6px;padding:8px;font:inherit;}
.actions {display:flex;gap:10px;flex-wrap:wrap;margin:16px 0;}.preview,.recovery {border:1px solid var(--border);padding:12px;border-radius:8px;margin:16px 0;}.preview ul {max-height:200px;overflow:auto;}.warning {color:var(--danger);} .close {margin-top:16px;}
</style>
