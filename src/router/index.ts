import { createRouter, createWebHashHistory } from "vue-router";
import AuthorAliasesView from "../views/AuthorAliasesView.vue";
import ArchiveView from "../views/ArchiveView.vue";
import LibraryView from "../views/LibraryView.vue";
import SettingsView from "../views/SettingsView.vue";
import CatalogView from "../views/CatalogView.vue";
import PerkSearchView from "../views/PerkSearchView.vue";

const router = createRouter({
  history: createWebHashHistory(),
  routes: [
    { path: "/", component: LibraryView },
    { path: "/authors", component: AuthorAliasesView },
    { path: "/archives", component: ArchiveView },
    { path: "/perks", component: PerkSearchView },
    { path: "/catalog", component: CatalogView },
    { path: "/settings", component: SettingsView },
  ],
});

export default router;
