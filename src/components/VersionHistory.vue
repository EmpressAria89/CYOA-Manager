<script setup lang="ts">
import {ref,onMounted} from 'vue';
import {invoke} from '@tauri-apps/api/core';
import type {Project} from '../types';
import {useSettings} from '../composables/useSettings';
const props=defineProps<{project:Project}>();
const emit=defineEmits<{(e:'close'):void;(e:'restored'):void}>();
const {settings}=useSettings();
interface Version{id:string;created_at:string;reason:string;starred:boolean;project:Project}
const versions=ref<Version[]>([]),copies=ref<Project[]>([]),selectedCopy=ref(''),busy=ref(false),error=ref(''),active=ref(false);
async function load(){versions.value=await invoke('list_versions',{id:props.project.id});const projects=await invoke<Project[]>('get_library');active.value=projects.some(p=>p.id===props.project.id);copies.value=projects.filter(p=>p.id!==props.project.id);}
async function run(action:()=>Promise<unknown>){busy.value=true;error.value='';try{await action();emit('restored');await load();}catch(e){error.value=String(e);}finally{busy.value=false;}}
function open(version:Version){return run(()=>invoke('open_viewer_window',{projectId:props.project.id,versionId:version.id,projectName:version.project.title||version.project.name,viewerId:version.project.viewer_preference||'icc2-plus',cheatsEnabled:settings.value.cheatsEnabled}));}
onMounted(()=>load().catch(e=>error.value=String(e)));
</script>
<template>
<div class="history-backdrop" @click.self="!busy&&emit('close')"><section class="history-panel" role="dialog" aria-modal="true" aria-labelledby="history-title">
<h2 id="history-title">{{project.title||project.name}} — Archives</h2>
<p>Archived editions open here without being added to the library. Keep {{settings.archiveLimit}} recent unstarred copies; stars protect editions from automatic cleanup.</p>
<p v-if="error" role="alert">{{error}}</p>
<div class="history-actions"><button v-if="active" class="btn-primary" :disabled="busy" @click="run(()=>invoke('archive_version',{id:project.id}))">Move current edition to archive</button><button class="btn-secondary" :disabled="busy" @click="emit('close')">Close</button></div>
<template v-if="active"><label>Move an older library copy into this archive:<select v-model="selectedCopy" :disabled="busy"><option value="">Choose a copy of the same CYOA</option><option v-for="copy in copies" :key="copy.id" :value="copy.id">{{copy.name}}</option></select></label>
<button class="btn-secondary" :disabled="busy||!selectedCopy" @click="run(()=>invoke('archive_existing_copy',{id:project.id,sourceId:selectedCopy}))">Move selected copy to archive</button></template>
<p v-if="!versions.length">No archived versions yet.</p>
<ul><li v-for="version in versions" :key="version.id"><div><strong>{{version.project.title||version.project.name}}</strong><p>{{new Date(version.created_at).toLocaleString()}} · {{version.reason}}</p></div><div class="version-actions">
<button class="btn-secondary" :disabled="busy" :aria-pressed="version.starred" @click="run(()=>invoke('star_version',{id:project.id,versionId:version.id,starred:!version.starred}))">{{version.starred?'★ Starred':'☆ Keep forever'}}</button>
<button class="btn-primary" :disabled="busy" @click="open(version)">Open archive</button><button class="btn-secondary" :disabled="busy" @click="run(()=>invoke('restore_version',{id:project.id,versionId:version.id}))">Restore to library</button>
</div></li></ul></section></div>
</template>
<style scoped>
.history-backdrop{position:fixed;inset:0;background:#0008;display:grid;place-items:center;z-index:1200;padding:20px}.history-panel{background:var(--dialog-bg);color:var(--text);border:1px solid var(--border);border-radius:12px;padding:24px;width:min(880px,95vw);max-height:90vh;overflow:auto}h2{margin-top:0}p{color:var(--muted);line-height:1.5;overflow-wrap:anywhere}.history-actions,.version-actions{display:flex;flex-wrap:wrap;gap:8px;align-items:center;margin:12px 0}label{display:block;margin:20px 0 8px}select{display:block;width:100%;padding:8px;margin:8px 0;border:1px solid var(--border);border-radius:6px}ul{list-style:none;padding:0}li{padding:16px 0;border-bottom:1px solid var(--border)}
</style>
