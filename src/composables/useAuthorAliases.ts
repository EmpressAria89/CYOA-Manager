import { ref, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";

export interface AuthorAlias { id: string; name: string; aliases: string[] }
const authors = ref<AuthorAlias[]>([]);
let loaded = false;
let pending: Promise<void> | null = null;
const key = (name: string) => name.trim().split(/\s+/).join(" ").toLocaleLowerCase();
const names = computed(() => {
  const map = new Map<string, string>();
  for (const author of authors.value) for (const alias of [author.name, ...author.aliases]) map.set(key(alias), author.name);
  return map;
});
export function resolveAuthor(name = ""): string { return names.value.get(key(name)) || name; }
export function useAuthorAliases() {
  async function loadAuthorAliases() {
    if (loaded) return;
    if (pending) return pending;
    pending = (async () => {
      try { authors.value = await invoke<AuthorAlias[]>("get_author_aliases") || []; loaded = true; }
      finally { pending = null; }
    })();
    return pending;
  }
  async function saveAuthorAliases(value: AuthorAlias[]) {
    authors.value = await invoke<AuthorAlias[]>("save_author_aliases", { authors: value });
    loaded = true;
  }
  return { authors, loadAuthorAliases, saveAuthorAliases, resolveAuthor };
}
