<script setup lang="ts">
import { ref, watch, computed } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import type { Project, ProjectPatch, Viewer } from "../types";
import { useLibrary } from "../composables/useLibrary";
import { normalizeTags } from "../catalogTags";

const props = defineProps<{
  project: Project;
  viewers: Viewer[];
}>();

const emit = defineEmits<{
  (e: "save", patch: ProjectPatch): void;
  (e: "close"): void;
}>();

const name = ref(props.project.title || props.project.name);
const author = ref(props.project.author || "");
const fandom = ref(props.project.fandom || "");
const modder = ref(props.project.modder || "");
const playerStatus = computed(() => props.project.build_count ? `Completed(${props.project.build_count})` : "Not completed");
const isMod = ref(Boolean(props.project.is_mod));
const { projects } = useLibrary();
const tagInput = ref("");
const tags = ref(normalizeTags(props.project.tags));
const tagSuggestions = computed(() => normalizeTags(projects.value.flatMap(project => project.tags))
  .filter(tag => !tags.value.some(existing => existing.toLowerCase() === tag.toLowerCase())
    && tag.toLowerCase().includes(tagInput.value.trim().toLowerCase()))
  .sort((a, b) => a.localeCompare(b)).slice(0, 8));
const authors = computed(() => [...new Set(projects.value.map(project => project.author).filter((value): value is string => Boolean(value)))]);
const modders = computed(() => [...new Set(projects.value.map(project => project.modder).filter((value): value is string => Boolean(value)))]);
const fandoms = computed(() => [...new Set(projects.value.map(project => project.fandom).filter((value): value is string => Boolean(value)))]);
function addTag(value = tagInput.value) {
  for (const tag of normalizeTags(value.split(","))) {
    if (!tags.value.some(existing => existing.toLowerCase() === tag.toLowerCase())) tags.value.push(tag);
  }
  tagInput.value = "";
}
function tagKey(event: KeyboardEvent) {
  if (event.key === "Enter" || event.key === ",") { event.preventDefault(); addTag(); }
  if (event.key === "Backspace" && !tagInput.value) tags.value.pop();
}
const description = ref(props.project.description);
const cover = ref(props.project.cover_image ?? "");
const sourceUrl = ref(props.project.source_url ?? "");
const viewerPreference = ref(props.project.viewer_preference ?? "");
const excludeFromPerkIndex = ref(props.project.exclude_from_perk_index);
const coverPreviewError = ref(false);
const directJsonCopyStatus = ref("");

watch(() => cover.value, () => { coverPreviewError.value = false; });
watch(() => props.project, (project) => {
  name.value = project.title || project.name;
  author.value = project.author || "";
  fandom.value = project.fandom || "";
  modder.value = project.modder || "";
  isMod.value = Boolean(project.is_mod);
  description.value = project.description;
  tags.value = normalizeTags(project.tags);
  tagInput.value = "";
  cover.value = project.cover_image ?? "";
  sourceUrl.value = project.source_url ?? "";
  viewerPreference.value = project.viewer_preference ?? "";
  excludeFromPerkIndex.value = project.exclude_from_perk_index;
  directJsonCopyStatus.value = "";
}, { deep: true });

async function pickCover() {
  const selected = await open({
    title: "Select cover image",
    filters: [{ name: "Images", extensions: ["png", "jpg", "jpeg", "webp", "gif", "avif"] }],
  });
  if (selected) cover.value = selected as string;
}

async function copyDirectJsonUrl() {
  if (!props.project.project_json_url) {
    return;
  }

  try {
    await navigator.clipboard.writeText(props.project.project_json_url);
    directJsonCopyStatus.value = "Copied";
  } catch {
    directJsonCopyStatus.value = "Copy failed";
  }
}

function formatDateAdded(value: string): string {
  const parsed = new Date(value);
  if (Number.isNaN(parsed.getTime())) {
    return value;
  }

  return parsed.toLocaleString();
}

function save() {
  addTag();
  const patch: ProjectPatch = {
    name: name.value.trim() || props.project.name,
    title: name.value.trim() || props.project.name,
    author: author.value.trim() === props.project.author ? props.project.source_author ?? author.value.trim() : author.value.trim(),
    fandom: fandom.value.trim(),
    modder: isMod.value ? modder.value.trim() : "",
    is_mod: isMod.value,
    description: description.value,
    cover_image: cover.value,
    source_url: sourceUrl.value,
    viewer_preference: viewerPreference.value,
    exclude_from_perk_index: excludeFromPerkIndex.value,
    tags: normalizeTags(tags.value),
  };
  emit("save", patch);
}
</script>

<template>
  <div class="overlay" @click.self="emit('close')">
    <div class="dialog" role="dialog" aria-modal="true" aria-labelledby="edit-project-title" @keydown.esc="emit('close')">
      <h2 id="edit-project-title">Edit Project</h2>

      <label>Title
        <input v-model="name" type="text" placeholder="CYOA title" />
      </label>

      <label>Description
        <textarea v-model="description" rows="2" placeholder="Optional description" />
      </label>

      <div class="metadata-grid">
        <label>Author
          <input v-model="author" list="project-authors" placeholder="From metadata when available" />
          <datalist id="project-authors"><option v-for="value in authors" :key="value" :value="value" /></datalist>
        </label>
        <label>Fandom / universe
          <input v-model="fandom" list="project-fandoms" placeholder="e.g. Naruto" />
          <datalist id="project-fandoms"><option v-for="value in fandoms" :key="value" :value="value" /></datalist>
        </label>
        <label>My status
          <output class="player-status">{{ playerStatus }}</output>
        </label>
        <label class="checkbox-row"><input v-model="isMod" type="checkbox" /><span>Mod / adaptation</span></label>
        <label v-if="isMod" class="modder-field">Modder
          <input v-model="modder" list="project-modders" placeholder="Who made this adaptation" />
          <datalist id="project-modders"><option v-for="value in modders" :key="value" :value="value" /></datalist>
        </label>
      </div>
      <div class="tag-editor">
        <label for="manual-tag">Extra tags <span class="hint">Optional · Enter or comma to add</span></label>
        <div v-if="tags.length" class="tag-chips">
          <button v-for="tag in tags" :key="tag" type="button" class="tag-chip" :aria-label="`Remove tag ${tag}`" @click="tags = tags.filter(value => value !== tag)">{{ tag }} <span aria-hidden="true">×</span></button>
        </div>
        <input id="manual-tag" v-model="tagInput" placeholder="Add a tag" @keydown="tagKey" @blur="addTag()" />
        <div v-if="tagInput.trim() && tagSuggestions.length" class="tag-suggestions" aria-label="Suggested tags">
          <button v-for="tag in tagSuggestions" :key="tag" type="button" @mousedown.prevent @click="addTag(tag)">{{ tag }}</button>
        </div>
        <span v-if="project.build_count" class="hint">Completed({{ project.build_count }}) is updated automatically from saved builds.</span>
      </div>

      <label>Preferred viewer
        <select v-model="viewerPreference">
          <option value="">Automatic (ICC2 Plus)</option>
          <option v-for="viewer in viewers.filter(v => project.kind === 'website' ? v.id.startsWith('website') : !v.id.startsWith('website'))" :key="viewer.id" :value="viewer.id">
            {{ viewer.id === "icc-original" || viewer.name.toLowerCase() === "icc original" ? "Force ICC Original (legacy)" : viewer.name }}
          </option>
        </select>
      </label>

      <label>Cover image URL or path
        <div class="cover-row">
          <input v-model="cover" type="text" placeholder="https://... or leave empty" />
          <button class="btn-secondary" @click="pickCover">Browse</button>
        </div>
        <img
          v-if="cover && !coverPreviewError"
          :src="cover"
          class="cover-preview"
          @error="coverPreviewError = true"
          alt="cover preview"
        />
      </label>

      <label>Source URL
        <input v-model="sourceUrl" type="text" placeholder="https://... or leave empty" />
      </label>

      <div v-if="project.project_json_url" class="file-path">
        <strong>Direct JSON:</strong>
        <button class="direct-link" type="button" @click="copyDirectJsonUrl">
          {{ project.project_json_url }}
        </button>
        <span v-if="directJsonCopyStatus" class="copy-status">{{ directJsonCopyStatus }}</span>
      </div>

      <label class="checkbox-row">
        <input v-model="excludeFromPerkIndex" type="checkbox" />
        <span>Exclude from Perk Index</span>
      </label>

      <div class="file-path">
        <strong>Date added:</strong> {{ formatDateAdded(project.date_added) }}
      </div>

      <div class="file-path">
        <strong>File:</strong> {{ project.file_path }}
      </div>

      <div class="dialog-actions">
        <button class="btn-secondary" @click="emit('close')">Cancel</button>
        <button class="btn-primary" @click="save">Save</button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.6);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 100;
}
.dialog {
  background: var(--dialog-bg);
  border: 1px solid var(--border);
  border-radius: 12px;
  padding: 28px;
  width: 480px;
  max-width: 95vw;
  max-height: 95vh;
  display: flex;
  flex-direction: column;
  gap: 14px;
  overflow-y: auto;
}
h2 {
  margin: 0 0 4px;
  font-size: 1.2rem;
}
label {
  display: flex;
  flex-direction: column;
  gap: 5px;
  font-size: 0.875rem;
  color: var(--muted);
}
label input,
label textarea,
label select,
.tag-editor > input {
  background: var(--input-bg);
  border: 1px solid var(--border);
  border-radius: 6px;
  color: var(--text);
  padding: 7px 10px;
  font-size: 0.9rem;
  outline: none;
  transition: border-color 0.15s;
}
label input:focus,
label textarea:focus,
label select:focus {
  border-color: var(--accent);
}
.checkbox-row {
  flex-direction: row;
  align-items: center;
  gap: 10px;
  color: var(--text);
}
.checkbox-row input {
  width: 16px;
  height: 16px;
  margin: 0;
}
.player-status { border: 1px solid var(--border); border-radius: 6px; padding: 7px 10px; color: var(--text); background: var(--input-bg); }
.modder-field { grid-column: 1 / -1; }
label select option { background: var(--input-bg); color: var(--text); }
.metadata-grid { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 12px; }
.metadata-grid input { min-width: 0; }
.tag-editor { display: flex; flex-direction: column; gap: 8px; }
.tag-chips, .tag-suggestions { display: flex; flex-wrap: wrap; gap: 6px; }
.tag-chip, .tag-suggestions button { border: 1px solid var(--border); background: var(--tag-bg); color: var(--tag-color); border-radius: 999px; padding: 4px 10px; font: inherit; font-size: 0.8rem; cursor: pointer; }
.tag-chip:hover, .tag-suggestions button:hover { border-color: var(--accent); }
@media (max-width: 480px) { .metadata-grid { grid-template-columns: 1fr; } .dialog { padding: 18px; } }
.hint {
  font-size: 0.75rem;
  opacity: 0.6;
}
.cover-row {
  display: flex;
  gap: 8px;
}
.cover-row input {
  flex: 1;
}
.cover-preview {
  margin-top: 6px;
  height: 80px;
  width: 100%;
  object-fit: cover;
  border-radius: 6px;
}
.file-path {
  font-size: 0.75rem;
  color: var(--muted);
  word-break: break-all;
}
.direct-link {
  background: none;
  border: none;
  color: var(--accent);
  cursor: pointer;
  font: inherit;
  padding: 0;
  text-align: left;
  word-break: break-all;
}
.direct-link:hover {
  text-decoration: underline;
}
.copy-status {
  margin-left: 6px;
  color: var(--accent);
}
.dialog-actions {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
  margin-top: 4px;
}
</style>
