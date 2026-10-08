<script setup lang="ts">
import { resolveAuthor } from "../composables/useAuthorAliases";
import {ref,computed,onMounted} from 'vue';
import {invoke} from '@tauri-apps/api/core';
import type {Project} from '../types';
import VersionHistory from '../components/VersionHistory.vue';
interface Version{id:string;project:Project;starred:boolean}
const archiveGroups=ref<Project[]>([]);
const versions=ref<Version[]>([]),selected=ref<Project|null>(null),search=ref(''),error=ref('');
const starred=computed(()=>versions.value.filter(v=>v.starred).filter(v=>JSON.stringify([v.project.name,v.project.title,v.project.author,v.project.fandom,v.project.modder,v.project.id]).toLowerCase().includes(search.value.toLowerCase())));
function groupLabel(version:Version){const group=archiveGroups.value.find(p=>p.id===version.project.id);return group?.title||group?.name||version.project.name;}
async function load(){try{const [editions,labels]=await Promise.all([invoke<Version[]>('list_archives'),invoke<Project[]>('list_archive_groups')]);versions.value=editions;archiveGroups.value=labels||[];}catch(e){error.value=String(e);}}
onMounted(load);
</script>
<template><div class="archive-view"><h1>Archives</h1><p>Starred editions only. Automatic backups remain available through each library card’s Version history.</p><input v-model="search" aria-label="Search archives" placeholder="Search starred editions…"/><p v-if="error" role="alert">{{error}}</p><p v-if="!starred.length">No matching starred editions.</p><section v-for="version in starred" :key="version.id"><div><h2>{{version.project.title||version.project.name}}</h2><p>{{resolveAuthor(version.project.author)}}<template v-if="version.project.modder"> · Modder: {{version.project.modder}}</template></p><p class="group-id">{{groupLabel(version)}} · Edition {{version.id.slice(0,8)}}</p></div><button class="btn-primary" @click="selected=version.project">Browse / open editions</button></section><VersionHistory v-if="selected" :project="selected" :starred-only="true" @close="selected=null;load()" @restored="load"/></div></template>
<style scoped>.archive-view{padding:28px;overflow:auto}input{background:var(--input-bg);color:var(--text);border:1px solid var(--border);border-radius:8px;padding:10px;width:min(500px,100%)}section{display:flex;justify-content:space-between;align-items:center;gap:16px;padding:16px 0;border-bottom:1px solid var(--border)}h2{font-size:1.1rem}p{color:var(--muted)}@media(max-width:600px){section{align-items:stretch;flex-direction:column}}</style>
