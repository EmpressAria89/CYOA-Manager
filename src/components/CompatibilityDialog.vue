<script setup lang="ts">
import { computed, nextTick, ref, watch } from "vue";
import { openUrl } from "@tauri-apps/plugin-opener";
import { invoke } from "@tauri-apps/api/core";
import { compatibilityWarning, openReader } from "../composables/useCompatibility";
import { useLibrary } from "../composables/useLibrary";
import { useSettings } from "../composables/useSettings";
const { loadLibrary } = useLibrary();
const { settings } = useSettings();
const panel = ref<HTMLElement | null>(null);
let previousFocus: HTMLElement | null = null;
watch(compatibilityWarning, async (warning, previous) => {
  if (warning) {
    if (!previous) previousFocus = document.activeElement as HTMLElement | null;
    await nextTick(); panel.value?.querySelector<HTMLButtonElement>("button:not(:disabled)")?.focus();
  } else { await nextTick(); previousFocus?.focus(); }
});
function trapFocus(event: KeyboardEvent) {
  if (event.key !== "Tab") return;
  const buttons = [...(panel.value?.querySelectorAll<HTMLButtonElement>("button:not(:disabled)") || [])];
  const first = buttons[0], last = buttons[buttons.length - 1];
  if (event.shiftKey && document.activeElement === first) { event.preventDefault(); last?.focus(); }
  else if (!event.shiftKey && document.activeElement === last) { event.preventDefault(); first?.focus(); }
}
const busy = ref(false);
const error = ref("");
const notice = ref("");
const source = computed(() => {
  try {
    const url = new URL(compatibilityWarning.value?.sourceUrl || "");
    return ["http:", "https:"].includes(url.protocol) && !url.username && !url.password ? url.href : "";
  } catch { return ""; }
});
function close() { if (!busy.value) { compatibilityWarning.value = null; error.value = ""; notice.value = ""; } }
async function act(action: "online" | "copy" | "anyway") {
  const warning = compatibilityWarning.value;
  if (!warning || busy.value) return;
  busy.value = true; error.value = ""; notice.value = "";
  try {
    if (action === "online") { await openUrl(source.value); }
    else if (action === "anyway") { await openReader(warning.request, true); }
    else {
      const result = await invoke<{unavailable: string[]}>("download_website", {
        url: source.value, title: `${warning.request.projectName} — website copy`,
        author: "", fandom: "", description: "Separate website copy using its original viewer. Builds stay separate from the JSON edition.",
        maxSizeMb: settings.value.downloadSizeLimitMb, existingProjectId: null, forceUpdate: false,
      });
      await loadLibrary(true);
      notice.value = result.unavailable.length
        ? `Added a separate website card. ${result.unavailable.length} resources could not be saved; use the online source if it fails.`
        : "Added a separate website card. Test it offline before relying on it; dynamic resources may still require the network.";
      return;
    }
    compatibilityWarning.value = null;
  } catch (e) { error.value = String(e); }
  finally { busy.value = false; }
}
</script>
<template>
  <div v-if="compatibilityWarning" class="compat-overlay" @keydown.esc="close" @click.self="close">
    <section ref="panel" class="compat-panel" @keydown="trapFocus" role="dialog" aria-modal="true" aria-labelledby="compat-title" tabindex="-1">
      <h2 id="compat-title">Use this CYOA’s original viewer</h2>
      <p>{{ compatibilityWarning.request.projectName }} uses features the bundled viewer cannot preserve.</p>
      <p>Editor: {{ compatibilityWarning.report.editor || 'custom / unknown' }} ·
        {{ compatibilityWarning.report.scripts }} script expressions ·
        {{ compatibilityWarning.report.variable_operations }} variable operations.</p>
      <p>Automatic ICC2 conversion is unavailable for these features. Scripts were inspected, never executed or changed.</p>
      <p>The website copy keeps its own viewer in a separate library card. Existing editions and builds stay intact; builds do not transfer automatically.</p>
      <p v-if="!source">No website URL is recorded. Add the playable website as this card’s Source URL in Edit.</p>
      <p v-if="error" role="alert">{{ error }}</p>
      <p v-if="notice" role="status">{{ notice }}</p>
      <p v-if="busy" role="status">Working… Website downloads can take several minutes.</p>
      <div class="compat-actions">
        <button class="btn-primary" :disabled="busy || !source" @click="act('online')">Open source website</button>
        <button class="btn-secondary" :disabled="busy || !source || !!notice" @click="act('copy')">Save website copy</button>
        <button class="btn-secondary" :disabled="busy" @click="act('anyway')">Try bundled viewer anyway</button>
        <button class="btn-secondary" :disabled="busy" @click="close">Close</button>
      </div>
    </section>
  </div>
</template>
<style scoped>
.compat-overlay {position:fixed;inset:0;z-index:1500;background:#0009;display:grid;place-items:center;padding:20px;}
.compat-panel {background:var(--dialog-bg);color:var(--text);border:1px solid var(--border);border-radius:12px;padding:24px;width:min(650px,100%);max-height:90vh;overflow:auto;}
h2 {margin-top:0;}
p {line-height:1.5;}
.compat-actions {display:flex;gap:10px;flex-wrap:wrap;}
</style>
