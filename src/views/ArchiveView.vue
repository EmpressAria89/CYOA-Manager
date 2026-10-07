<script setup lang="ts">
import { resolveAuthor } from "../composables/useAuthorAliases";
import {ref,computed,onMounted} from 'vue';
import {invoke} from '@tauri-apps/api/core';
import type {Project} from '../types';
import VersionHistory from '../components/VersionHistory.vue';
interface Version{id:string;project:Project;starred:boolean}
const versions=ref<Version[]>([]),selected=ref<Project|null>(null),search=ref(''),error=ref('');
const groups=computed(()=>{const result=new Map<string,{project:Project;count:number;stars:number}>();for(const version of versions.value){const g=result.get(version.project.id)||{project:{...version.project,author:resolveAuthor(version.project.author)},count:0,stars:0};g.count++;g.stars+=Number(version.starred);result.set(version.project.id,g);}return [...result.values()].filter(g=>JSON.stringify([g.project.name,g.project.author,g.project.fandom]).toLowerCase().includes(search.value.toLowerCase()));});
async function load(){try{versions.value=await invoke('list_archives');}catch(e){error.value=String(e);}}
onMounted(load);
</script>
<template><div class="archive-view"><h1>Archives</h1><input v-model="search" aria-label="Search archives" placeholder="Search archived CYOAs…"/><p v-if="error" role="alert">{{error}}</p><p v-if="!groups.length">No matching archives.</p><section v-for="group in groups" :key="group.project.id"><div><h2>{{group.project.title||group.project.name}}</h2><p>{{group.project.author}} · {{group.count}} editions · {{group.stars}} starred</p></div><button class="btn-primary" @click="selected=group.project">Browse / open editions</button></section><VersionHistory v-if="selected" :project="selected" @close="selected=null;load()" @restored="load"/></div></template>
<style scoped>.archive-view{padding:28px;overflow:auto}input{background:var(--input-bg);color:var(--text);border:1px solid var(--border);border-radius:8px;padding:10px;width:min(500px,100%)}section{display:flex;justify-content:space-between;align-items:center;gap:16px;padding:16px 0;border-bottom:1px solid var(--border)}h2{font-size:1.1rem}p{color:var(--muted)}@media(max-width:600px){section{align-items:stretch;flex-direction:column}}</style>
