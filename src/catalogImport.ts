import type { CatalogEntry, Project, ProjectPatch } from './types';

/** Only fill absent source fields on a newly imported card. Never use importer as modder. */
export function catalogMetadataPatch(project: Project, entry: CatalogEntry): ProjectPatch {
  const patch: ProjectPatch = {};
  if (!project.author?.trim() && (entry.source_author || entry.author)) patch.author = entry.source_author || entry.author;
  if (!project.fandom?.trim() && entry.universe) patch.fandom = entry.universe;
  if (!project.description?.trim() && entry.description) patch.description = entry.description;
  if (!project.is_mod && /(?:^|[,/\s])mod(?:$|[,/\s])/i.test(entry.status || '')) patch.is_mod = true;
  return patch;
}
