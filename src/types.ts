export interface Project {
  title?: string;
  author?: string;
  source_author?: string;
  modder?: string;
  kind?: string;
  fandom?: string;
  completion?: string;
  is_mod?: boolean;
  build_count?: number;
  viewer_check?: string;

  id: string;
  name: string;
  description: string;
  cover_image: string | null;
  source_url?: string | null;
  project_json_url?: string | null;
  file_path: string;
  viewer_preference: string | null;
  favorite: boolean;
  exclude_from_perk_index: boolean;
  date_added: string;
  tags: string[];
  file_missing: boolean;
}

export interface ProjectPatch {
  title?: string;
  author?: string;
  source_author?: string;
  modder?: string;
  kind?: string;
  fandom?: string;
  completion?: string;
  is_mod?: boolean;
  build_count?: number;
  viewer_check?: string;

  name?: string;
  description?: string;
  /** empty string clears the cover */
  cover_image?: string;
  /** empty string clears the source URL */
  source_url?: string;
  viewer_preference?: string;
  favorite?: boolean;
  exclude_from_perk_index?: boolean;
  tags?: string[];
  /** re-link a broken card */
  file_path?: string;
}

export interface Viewer {
  id: string;
  name: string;
}

export interface PerkIndexStatus {
  ready: boolean;
  needsReindex: boolean;
  indexedProjects: number;
  totalProjects: number;
  perkCount: number;
  imagesEnabled: boolean;
  lastIndexedAt: string | null;
}

export interface PerkSearchResult {
  projectId: string;
  projectName: string;
  rowId: string;
  rowTitle: string;
  objectId: string;
  title: string;
  description: string;
  points: string | null;
  addons: string[];
  imagePath: string | null;
}

export interface CatalogEntry {
  name: string;
  date: string;
  website: string;
  link: string;
  engine?: string;
  author?: string;
  source_author?: string;
  modder?: string;
  kind?: string;
  universe?: string;
  importer?: string;
  type?: string;
  pov?: string;
  length?: string;
  tags?: string[];
  description?: string;
}

export type SortKey = "name" | "date_added" | "favorite_date_added";
export type Theme = "light" | "dark" | "system" | "latte" | "frappe" | "macchiato" | "mocha";
export type OversizeDefaultAction = "ask" | "keep-separate" | "compress" | "do-nothing";
export type OversizeActionStrategy = "keep-separate" | "compress" | "do-nothing";
