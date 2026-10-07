<script setup lang="ts">
import { onMounted, ref } from "vue";
import { useAuthorAliases, type AuthorAlias } from "../composables/useAuthorAliases";
const { authors, loadAuthorAliases, saveAuthorAliases } = useAuthorAliases();
const draft = ref<AuthorAlias[]>([]), newName = ref(""), inputs = ref<Record<string,string>>({});
const error = ref(""), status = ref(""), saving = ref(false);
function reset() { draft.value=authors.value.map(a=>({...a,aliases:[...a.aliases]})); inputs.value={}; status.value=""; }
onMounted(async()=>{try {await loadAuthorAliases();reset();}catch(e){error.value=String(e);}});
function addAuthor() {
  const name=newName.value.trim();if(!name)return;
  draft.value.push({id:crypto.randomUUID(),name,aliases:[]});newName.value="";status.value="";
}
function addAlias(author:AuthorAlias) {
  const name=(inputs.value[author.id]||"").trim();if(!name)return;
  if(!author.aliases.some(a=>a.toLocaleLowerCase()===name.toLocaleLowerCase())) author.aliases.push(name);
  inputs.value[author.id]="";status.value="";
}
async function save() {
  error.value="";status.value="";saving.value=true;
  try {if(newName.value.trim())addAuthor();draft.value.forEach(addAlias);await saveAuthorAliases(draft.value);reset();status.value="Aliases saved. Library, catalog and archive author names update automatically.";}
  catch(e){error.value=String(e);}finally{saving.value=false;}
}
</script>
<template>
  <div class="aliases-view">
    <h1>Author aliases</h1>
    <p>Choose the author name you want to see. Add source names as aliases when an author uses another name. Earlier aliases stay until you remove them.</p>
    <p>Original project credits are preserved. For example, Gaston1231 and Gaston1597 can both appear as Gaston.</p>
    <form class="add-author" @submit.prevent="addAuthor"><label>New author name<input v-model="newName" placeholder="Your preferred author name" /></label><button class="btn-secondary" type="submit">Add author</button></form>
    <section v-for="author in draft" :key="author.id" class="author-group" :aria-label="`Aliases for ${author.name}`">
      <div class="group-heading"><label>Author name<input v-model="author.name" /></label><button class="btn-ghost remove" type="button" @click="draft=draft.filter(a=>a.id!==author.id)">Remove author</button></div>
      <ul class="alias-list"><li v-for="alias in author.aliases" :key="alias"><span>{{alias}}</span><button type="button" :aria-label="`Remove alias ${alias}`" @click="author.aliases=author.aliases.filter(a=>a!==alias)">×</button></li></ul>
      <form class="add-alias" @submit.prevent="addAlias(author)"><label>Another source name<input v-model="inputs[author.id]" placeholder="e.g. Gaston1597" /></label><button class="btn-secondary" type="submit">Add alias</button></form>
    </section>
    <p v-if="!draft.length">No author aliases configured.</p>
    <p v-if="error" role="alert" class="error">{{error}}</p><p v-if="status" role="status">{{status}}</p>
    <div class="actions"><button class="btn-primary" :disabled="saving" @click="save">{{saving?'Saving…':'Save aliases'}}</button><button class="btn-secondary" :disabled="saving" @click="reset">Discard changes</button></div>
  </div>
</template>
<style scoped>
.aliases-view{padding:24px;overflow:auto;max-width:1000px;width:100%;height:100%;}h1{margin-top:0;}p{color:var(--muted);line-height:1.5;}label{display:flex;flex-direction:column;gap:6px;flex:1;}input{width:100%;min-width:0;padding:9px 12px;background:var(--input-bg);color:var(--text);border:1px solid var(--border);border-radius:8px;font:inherit;}.add-author,.add-alias,.group-heading,.actions{display:flex;align-items:flex-end;gap:12px;}.author-group{margin:20px 0;padding:18px;border:1px solid var(--border);border-radius:12px;background:var(--card-bg);}.remove,.error{color:var(--danger);}.alias-list{padding:0;display:flex;flex-wrap:wrap;gap:8px;list-style:none;}.alias-list li{display:flex;align-items:center;gap:10px;padding:5px 10px;border-radius:20px;background:var(--tag-bg);color:var(--tag-color);}.alias-list button{border:0;background:none;color:inherit;cursor:pointer;padding:0 3px;font:inherit;}.actions{margin-top:24px;}@media(max-width:550px){.add-author,.add-alias,.group-heading{align-items:stretch;flex-direction:column;}}
</style>
