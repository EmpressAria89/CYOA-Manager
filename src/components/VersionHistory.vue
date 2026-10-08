<script setup lang="ts">
import {ref,computed,onMounted} from 'vue';
import {invoke} from '@tauri-apps/api/core';
import type {Project} from '../types';
import {useSettings} from '../composables/useSettings';
import {useLibrary} from '../composables/useLibrary';
import {resolveAuthor} from '../composables/useAuthorAliases';
const props=defineProps<{project:Project;starredOnly?:boolean}>();
const emit=defineEmits<{(e:'close'):void;(e:'restored'):void}>();
const {settings}=useSettings();
const {loadLibrary}=useLibrary();
interface Version{id:string;created_at:string;reason:string;starred:boolean;project:Project}
const versions=ref<Version[]>([]),copies=ref<Project[]>([]),groups=ref<Project[]>([]),library=ref<Project[]>([]);
const selectedCopy=ref(''),busy=ref(false),error=ref(''),message=ref(''),active=ref(false),groupName=ref(props.project.title||props.project.name);
const renameGroup=ref(false),groupDraft=ref(''),editing=ref(''),mode=ref<'rename'|'move'>('rename'),editionName=ref(''),destination=ref(''),newGroupName=ref('');
const destinations=computed(()=>{const result=new Map(groups.value.map(p=>[p.id,p]));for(const p of library.value)if(!result.has(p.id))result.set(p.id,p);return [...result.values()].filter(p=>p.id!==props.project.id);});
function label(project:Project){return [project.title||project.name,resolveAuthor(project.author),project.modder?`Modder: ${project.modder}`:'',`Group ${project.id.slice(0,8)}`].filter(Boolean).join(' · ');}
async function load(){
 const [editions,projects,archiveGroups]=await Promise.all([invoke<Version[]>('list_versions',{id:props.project.id}),invoke<Project[]>('get_library'),invoke<Project[]>('list_archive_groups')]);
 versions.value=props.starredOnly?editions.filter(v=>v.starred):editions;library.value=projects;groups.value=archiveGroups||[];active.value=projects.some(p=>p.id===props.project.id);copies.value=projects.filter(p=>p.id!==props.project.id);groupName.value=groups.value.find(p=>p.id===props.project.id)?.title||props.project.title||props.project.name;
}
async function run(action:()=>Promise<unknown>){busy.value=true;error.value='';message.value='';try{await action();await loadLibrary(true);emit('restored');await load();}catch(e){error.value=String(e);}finally{busy.value=false;}}
function open(version:Version){return run(()=>invoke('open_viewer_window',{projectId:props.project.id,versionId:version.id,projectName:version.project.title||version.project.name,viewerId:version.project.viewer_preference||'icc2-plus',cheatsEnabled:settings.value.cheatsEnabled}));}
function restore(version:Version){return run(async()=>{const project=await invoke<Project>('restore_version',{id:props.project.id,versionId:version.id});const existed=library.value.some(p=>p.id===project.id);message.value=existed?`“${project.title||project.name}” is already in your library.`:`Added “${project.title||project.name}” as a separate library card. Other cards and this archive are unchanged.`;});}
function edit(version:Version,next:'rename'|'move'){editing.value=version.id;mode.value=next;editionName.value=version.project.title||version.project.name;destination.value='';newGroupName.value=editionName.value;}
function saveEdit(version:Version){return run(async()=>{
 if(mode.value==='rename'){await invoke('rename_archive_version',{id:props.project.id,versionId:version.id,name:editionName.value});message.value='Edition renamed.';}
 else {await invoke('move_archive_version',{id:props.project.id,versionId:version.id,targetId:destination.value==='new'?null:destination.value,newGroupName:destination.value==='new'?newGroupName.value:null});message.value='Edition moved. Its label, contents and star are preserved.';}
 editing.value='';
});}
onMounted(()=>load().catch(e=>error.value=String(e)));
</script>
<template>
<div class="history-backdrop" @click.self="!busy&&emit('close')"><section class="history-panel" role="dialog" aria-modal="true" aria-labelledby="history-title">
<h2 id="history-title">{{groupName}} — Archives</h2>
<p class="group-id">Archive group {{project.id.slice(0,8)}} · Group names do not change edition names.</p>
<p>Open an edition here, or add an independent copy to the library. Keep {{settings.archiveLimit}} recent unstarred editions; stars protect editions from automatic cleanup.</p>
<p v-if="error" role="alert">{{error}}</p><p v-if="message" role="status">{{message}}</p>
<div class="history-actions"><button v-if="active" class="btn-primary" :disabled="busy" @click="run(()=>invoke('archive_version',{id:project.id}))">Move current edition to archive</button><button class="btn-secondary" :disabled="busy" @click="groupDraft=groupName;renameGroup=!renameGroup">Rename archive group</button><button class="btn-secondary" :disabled="busy" @click="emit('close')">Close</button></div>
<form v-if="renameGroup" @submit.prevent="run(async()=>{await invoke('rename_archive_group',{id:project.id,name:groupDraft});renameGroup=false;message='Archive group renamed.';})"><label>Archive group name<input v-model="groupDraft" maxlength="200" required :disabled="busy" /></label><button class="btn-primary" :disabled="busy">Save group name</button><button type="button" class="btn-secondary" :disabled="busy" @click="renameGroup=false">Cancel</button></form>
<template v-if="active"><label>Move another library card into this archive group:<select v-model="selectedCopy" :disabled="busy"><option value="">Choose a library copy</option><option v-for="copy in copies" :key="copy.id" :value="copy.id">{{label(copy)}}</option></select></label>
<button class="btn-secondary" :disabled="busy||!selectedCopy" @click="run(()=>invoke('archive_existing_copy',{id:project.id,sourceId:selectedCopy}))">Move selected copy to archive</button></template>
<p v-if="!versions.length">{{starredOnly?'No starred editions remain in this group.':'No archived editions remain in this group.'}}</p>
<ul><li v-for="version in versions" :key="version.id"><div><strong>{{version.project.title||version.project.name}}</strong><p>{{new Date(version.created_at).toLocaleString()}} · {{version.reason}}</p></div><div class="version-actions">
<button class="btn-secondary" :disabled="busy" :aria-pressed="version.starred" @click="run(()=>invoke('star_version',{id:project.id,versionId:version.id,starred:!version.starred}))">{{version.starred?'★ Starred':'☆ Keep forever'}}</button>
<button class="btn-primary" :disabled="busy" @click="open(version)">Open archive</button><button class="btn-secondary" :disabled="busy" @click="restore(version)">Add copy to library</button>
<button class="btn-secondary" :disabled="busy" @click="edit(version,'rename')">Rename edition</button><button class="btn-secondary" :disabled="busy" @click="edit(version,'move')">Move edition</button>
</div>
<form v-if="editing===version.id" @submit.prevent="saveEdit(version)">
<label v-if="mode==='rename'">Edition name<input v-model="editionName" maxlength="200" required :disabled="busy" /></label>
<template v-else><label>Destination archive group<select aria-label="Destination archive group" v-model="destination" required :disabled="busy"><option value="" disabled>Choose a destination</option><option value="new">New archive group…</option><option v-for="target in destinations" :key="target.id" :value="target.id">{{label(target)}}</option></select></label><label v-if="destination==='new'">New archive group name<input v-model="newGroupName" maxlength="200" required :disabled="busy" /></label><p>Moving changes the group only. The edition stays archived; nothing is added to or replaced in the library.</p></template>
<div class="history-actions"><button class="btn-primary" :disabled="busy|| (mode==='move'&&!destination)">{{mode==='rename'?'Save edition name':'Move to group'}}</button><button type="button" class="btn-secondary" :disabled="busy" @click="editing=''">Cancel</button></div>
</form></li></ul></section></div>
</template>
<style scoped>
.history-backdrop{position:fixed;inset:0;background:#0008;display:grid;place-items:center;z-index:1200;padding:20px}.history-panel{background:var(--dialog-bg);color:var(--text);border:1px solid var(--border);border-radius:12px;padding:24px;width:min(880px,95vw);max-height:90vh;overflow:auto}h2{margin-top:0}p{color:var(--muted);line-height:1.5;overflow-wrap:anywhere}.history-actions,.version-actions{display:flex;flex-wrap:wrap;gap:8px;align-items:center;margin:12px 0}label{display:block;margin:20px 0 8px}select,input{display:block;width:100%;padding:8px;margin:8px 0;border:1px solid var(--border);border-radius:6px;background:var(--input-bg);color:var(--text);font:inherit}ul{list-style:none;padding:0}li{padding:16px 0;border-bottom:1px solid var(--border)}form{border:1px solid var(--border);border-radius:8px;padding:0 12px 12px}.group-id{font-size:.8rem}
</style>
