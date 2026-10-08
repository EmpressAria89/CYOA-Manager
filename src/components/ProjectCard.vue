<script setup lang="ts">
import { ref, computed, watch, nextTick, onMounted, onBeforeUnmount } from "vue";
import { openUrl } from "@tauri-apps/plugin-opener";
import type { Project, Viewer } from "../types";
import { resolveCover } from "../composables/useCover";
import { resolveViewerId } from "../viewers";

const props = defineProps<{
  project: Project;
  viewers: Viewer[];
  defaultViewer: string | null;
  redownloadBusy?: boolean;
  redownloadLabel?: string | null;
}>();

const emit = defineEmits<{
  (e: "open", viewerId: string): void;
  (e: "remove"): void;
  (e: "remove-disk"): void;
  (e: "edit"): void;
  (e: "history"): void;
  (e: "relink"): void;
  (e: "redownload"): Promise<void> | void;
  (e: "toggle-favorite"): void;
}>();

const menuOpen = ref(false);
const menuButton = ref<HTMLButtonElement | null>(null);
const menuElement = ref<HTMLElement | null>(null);
const menuPosition = ref({ left: "0px", top: "0px" });
const cardTitle = computed(() => props.project.title?.trim() || props.project.name);
const displayTags = computed(() => [...new Set([
  props.project.fandom,
  props.project.modder ? `Modder: ${props.project.modder}` : "",
  props.project.is_mod ? "MOD" : null,
  ...(props.project.build_count ? [`Completed(${props.project.build_count})`] : []),
  ...props.project.tags,
].filter((tag): tag is string => Boolean(tag)))]);
function isLegacy(viewer: Viewer) {
  return viewer.id === "icc-original" || viewer.name.toLowerCase() === "icc original";
}
const availableViewers = computed(() => props.viewers.filter(viewer =>
  props.project.kind==='website' ? viewer.id.startsWith('website') : !viewer.id.startsWith('website') && (!isLegacy(viewer) || viewer.id === props.project.viewer_preference)));
const imageFailed = ref(false);
const coverImageSrc = ref<string | null>(null);
const openingSource = ref(false);
const redownloading = ref(false);
const selectedViewerId = ref<string | null>(null);

const initials = computed(() => {
  const words = props.project.name.trim().split(/\s+/);
  if (words.length >= 2) return (words[0][0] + words[1][0]).toUpperCase();
  return props.project.name.slice(0, 2).toUpperCase();
});

const coverColor = computed(() => "var(--cover-placeholder)");

const selectedViewer = computed(() => {
  return props.viewers.find((viewer) => viewer.id === selectedViewerId.value) ?? null;
});

const sourceUrl = computed(() => {
  const raw = props.project.source_url;
  return raw && raw.trim() ? raw : null;
});

const redownloadUrl = computed(() => {
  if (props.project.restored_from_archive) return null;
  const raw = props.project.project_json_url || props.project.source_url;
  return raw && raw.trim() ? raw : null;
});

const isRedownloading = computed(() => redownloading.value || Boolean(props.redownloadBusy));
const redownloadText = computed(() => {
  if (!isRedownloading.value) {
    return "Re-download";
  }

  return props.redownloadLabel?.trim() || "Re-downloading...";
});

watch(
  () => [props.project.file_path, props.project.cover_image],
  async () => {
    imageFailed.value = false;

    if(props.project.cover_image?.startsWith("data:") || props.project.cover_image?.startsWith("http")){coverImageSrc.value=props.project.cover_image;return;}
    try {
      coverImageSrc.value = await resolveCover(props.project.file_path,props.project.cover_image);
    } catch {
      coverImageSrc.value = null;
    }
  },
  { immediate: true }
);

watch(
  () => [
    props.project.id,
    props.project.viewer_preference,
    props.defaultViewer,
    props.viewers.map((viewer) => viewer.id).join("|"),
  ],
  () => {
    selectedViewerId.value = resolveViewerId(
      availableViewers.value,
      props.project.viewer_preference,
      props.defaultViewer,
    );
  },
  { immediate: true }
);

function positionMenu() {
  const anchor = menuButton.value?.getBoundingClientRect();
  if (!anchor || !menuElement.value) return;
  const width = menuElement.value.offsetWidth;
  const height = menuElement.value.offsetHeight;
  menuPosition.value = {
    left: `${Math.max(8, Math.min(anchor.right - width, window.innerWidth - width - 8))}px`,
    top: `${Math.max(8, Math.min(anchor.bottom + 6, window.innerHeight - height - 8))}px`,
  };
}
async function openMenu() {
  if (menuOpen.value) return closeMenu();
  menuOpen.value = true;
  await nextTick();
  positionMenu();
  menuElement.value?.querySelector<HTMLButtonElement>("button")?.focus();
}
function closeMenu(restoreFocus = false) {
  menuOpen.value = false;
  if (restoreFocus) menuButton.value?.focus();
}
function dismissMenu(event: PointerEvent) {
  const target = event.target as Node;
  if (!menuElement.value?.contains(target) && !menuButton.value?.contains(target)) closeMenu();
}
function menuKey(event: KeyboardEvent) {
  if (event.key === "Escape") { event.preventDefault(); closeMenu(true); return; }
  if (event.key === "Tab") { closeMenu(); return; }
  if (!["ArrowDown", "ArrowUp", "Home", "End"].includes(event.key)) return;
  event.preventDefault();
  const buttons = Array.from(menuElement.value?.querySelectorAll<HTMLButtonElement>("button") || []);
  const index = buttons.indexOf(document.activeElement as HTMLButtonElement);
  const next = event.key === "Home" ? 0 : event.key === "End" ? buttons.length - 1
    : (index + (event.key === "ArrowUp" ? -1 : 1) + buttons.length) % buttons.length;
  buttons[next]?.focus();
}
onMounted(() => {
  document.addEventListener("pointerdown", dismissMenu);
  window.addEventListener("resize", positionMenu);
  window.addEventListener("scroll", positionMenu, true);
});
onBeforeUnmount(() => {
  document.removeEventListener("pointerdown", dismissMenu);
  window.removeEventListener("resize", positionMenu);
  window.removeEventListener("scroll", positionMenu, true);
});

function onOpen(viewerId: string) {
  closeMenu();
  emit("open", viewerId);
}

function onToggleFavorite() {
  emit("toggle-favorite");
}

function onImageError() {
  imageFailed.value = true;
}

async function onOpenSource() {
  if (!sourceUrl.value || openingSource.value) {
    return;
  }

  openingSource.value = true;
  try {
    await openUrl(sourceUrl.value);
  } catch (error) {
    console.error("Failed to open source URL:", error);
  } finally {
    openingSource.value = false;
  }
}

async function onRedownload() {
  if (!redownloadUrl.value || isRedownloading.value) {
    return;
  }

  redownloading.value = true;
  try {
    await emit("redownload");
  } finally {
    redownloading.value = false;
  }
}
</script>

<template>
  <div
    class="card"
    :class="{ missing: project.file_missing }"
    @click.self="closeMenu()"
  >
    <!-- Cover -->
    <div class="cover" :style="!coverImageSrc || imageFailed ? { background: coverColor } : {}">
      <img
        v-if="coverImageSrc && !imageFailed"
        :src="coverImageSrc"
        :alt="project.name"
        loading="lazy"
        @error="onImageError"
      />
      <span v-else class="initials" :title="`No cover supplied for ${cardTitle}`">{{ initials }}<small>No cover</small></span>

      <div v-if="sourceUrl || redownloadUrl" class="source-actions">
        <button
          v-if="sourceUrl"
          class="source-btn"
          :disabled="openingSource || isRedownloading"
          @click.stop="onOpenSource"
        >
          Open Source
        </button>
        <button
          v-if="redownloadUrl"
          class="source-btn secondary"
          :class="{ busy: isRedownloading }"
          :disabled="isRedownloading || openingSource"
          @click.stop="onRedownload"
        >
          {{ redownloadText }}
        </button>
      </div>

      <div v-if="project.file_missing" class="badge missing-badge">File missing</div>

      <!-- Menu button -->
      <button ref="menuButton" class="menu-btn" @click.stop="openMenu" @keydown.esc="closeMenu(true)"
        :aria-expanded="menuOpen" aria-haspopup="menu" aria-label="Project options" title="Options">⋮</button>
      <button
        class="favorite-btn"
        :class="{ active: project.favorite }"
        :title="project.favorite ? 'Remove favorite' : 'Add favorite'"
        @click.stop="onToggleFavorite"
      >
        {{ project.favorite ? "♥" : "♡" }}
      </button>


    </div>

    <!-- Info -->
    <div class="info">
      <h3 class="name" :title="cardTitle">{{ cardTitle }}</h3>
      <span v-if="project.author" class="author" :title="project.author">{{ project.author }}</span>

      <span v-if="project.restored_from_archive" class="tag" title="Restored archive edition; re-download is disabled to preserve this version">Archived edition</span>
      <div v-if="displayTags.length" class="tags">
        <span v-for="tag in displayTags" :key="tag" class="tag">{{ tag }}</span>
      </div>

      <!-- Open action -->
      <div class="actions">
        <template v-if="availableViewers.length === 0">
          <span class="no-viewers">No viewers found</span>
        </template>
        <template v-else>
          <select v-model="selectedViewerId" class="viewer-select">
            <option v-for="v in availableViewers" :key="v.id" :value="v.id">{{ v.name }}</option>
          </select>
          <button
            class="btn-open"
            :disabled="project.file_missing || !selectedViewer"
            @click="selectedViewer && onOpen(selectedViewer.id)"
          >
            Open
          </button>
        </template>
      </div>
    </div>
  </div>
  <Teleport to="body">
    <div v-if="menuOpen" ref="menuElement" class="menu" :style="menuPosition"
      role="menu" aria-label="Project options" @keydown="menuKey">
      <button role="menuitem" @click="emit('history'); closeMenu()">Version history</button>
      <button role="menuitem" @click="emit('edit'); closeMenu()">Edit</button>
      <button v-if="project.file_missing" role="menuitem" @click="emit('relink'); closeMenu()">Re-link file</button>
      <button role="menuitem" class="danger" @click="emit('remove'); closeMenu()">Remove from library</button>
      <button role="menuitem" class="danger" @click="emit('remove-disk'); closeMenu()">Remove from disk</button>
    </div>
  </Teleport>
</template>

<style scoped>
.card {
  background: var(--card-bg);
  border: 1px solid var(--border);
  border-radius: 10px;
  overflow: hidden;
  display: flex;
  flex-direction: column;
  transition: border-color 0.15s;
  position: relative;
}
.card:hover {
  border-color: var(--accent);
}
.card.missing {
  opacity: 0.7;
}

.cover {
  position: relative;
  height: 160px;
  overflow: hidden;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--cover-placeholder);
}
.cover img {
  width: 100%;
  height: 100%;
  object-fit: contain;
  padding: 8px;
  display: block;
}
.initials small { display:block; font-size:0.7rem; color:var(--muted); margin-top:8px; }
.initials {
  text-align:center;
  font-size: 2.5rem;
  font-weight: 400;
  color: var(--muted);
  user-select: none;
}

.badge {
  position: absolute;
  top: 8px;
  left: 8px;
  padding: 2px 8px;
  border-radius: 4px;
  font-size: 0.7rem;
  font-weight: 400;
}
.missing-badge {
  top: 42px;
  background: var(--danger);
  color: var(--text);
}

.source-actions {
  position: absolute;
  top: 6px;
  left: 6px;
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.source-btn {
  padding: 6px 10px;
  background: var(--menu-bg);
  border: none;
  border-radius: 6px;
  color: var(--text);
  font-size: 0.76rem;
  font-weight: 400;
  cursor: pointer;
  opacity: 0;
  transition: opacity 0.15s, background 0.15s;
}

.card:hover .source-btn,
.card:focus-within .source-btn {
  opacity: 1;
}

.source-btn.secondary {
  background: var(--menu-bg);
}

.source-btn.busy {
  background: var(--accent);
  color: var(--accent-text);
}

.source-btn:hover:not(:disabled) {
  background: var(--hover);
}

.source-btn:disabled {
  cursor: wait;
}

.menu-btn {
  position: absolute;
  top: 6px;
  right: 6px;
  background: var(--menu-bg);
  border: none;
  color: var(--text);
  border-radius: 4px;
  font-size: 1.2rem;
  line-height: 1;
  padding: 2px 6px;
  cursor: pointer;
  opacity: 0;
  transition: opacity 0.15s;
}
.card:hover .menu-btn,
.card:focus-within .menu-btn {
  opacity: 1;
}

.favorite-btn {
  position: absolute;
  right: 6px;
  bottom: 6px;
  background: var(--menu-bg);
  border: none;
  color: var(--text);
  border-radius: 999px;
  width: 32px;
  height: 32px;
  display: grid;
  place-items: center;
  font-size: 1rem;
  line-height: 1;
  cursor: pointer;
  opacity: 0;
  transition: opacity 0.15s, background 0.15s, color 0.15s, transform 0.15s;
}

.card:hover .favorite-btn,
.favorite-btn.active {
  opacity: 1;
}

.favorite-btn:hover {
  background: var(--hover);
  transform: scale(1.04);
}

.favorite-btn.active {
  color: var(--danger);
}

.menu {
  position: fixed;
  max-width: calc(100vw - 16px);
  max-height: calc(100vh - 16px);
  background: var(--menu-bg);
  border: 1px solid var(--border);
  border-radius: 8px;
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.4);
  z-index: 1000;
  min-width: 190px;
  overflow-y: auto;
}
.menu button {
  display: block;
  width: 100%;
  padding: 8px 14px;
  background: none;
  border: none;
  color: var(--text);
  text-align: left;
  cursor: pointer;
  font-size: 0.875rem;
}
.menu button:hover,
.menu button:focus-visible {
  background: var(--hover);
}
.menu button.danger {
  color: var(--danger);
}

.info {
  padding: 10px 12px 12px;
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.name {
  margin: 0;
  font-size: 0.95rem;
  font-weight: 400;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.tags {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
}
.tag {
  background: var(--tag-bg);
  color: var(--tag-color);
  border-radius: 999px;
  padding: 3px 8px;
  font-size: 0.72rem;
}
.actions {
  margin-top: auto;
  display: flex;
  align-items: center;
  gap: 6px;
}
.viewer-select {
  flex: 1 1 auto;
  min-width: 0;
  padding: 5px 10px;
  background: var(--input-bg);
  border: 1px solid var(--border);
  border-radius: 6px;
  color: var(--text);
  font-size: 0.8rem;
  outline: none;
}
.viewer-select option { background: var(--input-bg); color: var(--text); }
.author { color: var(--muted); font-size: 0.8rem; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.viewer-select:focus {
  border-color: var(--accent);
}
.btn-open {
  flex: 0 0 auto;
  min-width: 72px;
  padding: 5px 10px;
  background: var(--accent);
  color: var(--accent-text);
  border: none;
  border-radius: 6px;
  cursor: pointer;
  font-size: 0.8rem;
  font-weight: 400;
  transition: background 0.15s;
}
.btn-open:hover:not(:disabled) {
  background: var(--accent-hover);
}
.btn-open:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}
.no-viewers {
  font-size: 0.75rem;
  color: var(--muted);
}
</style>
