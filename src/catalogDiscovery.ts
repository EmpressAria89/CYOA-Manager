import type { CatalogEntry } from './types';
import { normalizeTags } from './catalogTags';
export type CatalogSourceId = 'beta' | 'projects' | 'broken' | 'discoveries';
export interface CatalogSource { id: CatalogSourceId; name: string; rows: string[][]; fetchedAt: string; stale: boolean; error: string | null }
export interface CatalogSnapshot { sources: CatalogSource[]; fetchedAt: string; stale: boolean; warnings: string[] }
export function catalogUrl(raw: string): string {
  try { const url = new URL(raw.trim()); return ['http:', 'https:'].includes(url.protocol) && !url.username && !url.password ? url.href : ''; } catch { return ''; }
}
function urls(raw: string): string[] { return [...new Set((raw.match(/https?:\/\/[^\s<>"\]]+/g) || []).map(catalogUrl).filter(Boolean))]; }
export function catalogKey(entry: CatalogEntry): string { return `${entry.sourceId || 'beta'}:${entry.sourceRow ?? entry.website}:${entry.name}`; }
export function normalizeCatalogSnapshot(snapshot: CatalogSnapshot, archives: Map<string, CatalogEntry> = new Map()): CatalogEntry[] {
  const entries: CatalogEntry[] = [];
  const exact = new Map<string, CatalogEntry | null>();
  for (const source of [...snapshot.sources].sort((a,b) => (a.id === 'beta' ? -1 : b.id === 'beta' ? 1 : 0))) {
    const header = source.rows[0] || [];
    const columns = new Map(header.map((value,index) => [value.trim().toLowerCase(), index]));
    source.rows.slice(1).forEach((row, index) => {
      const cell = (...names: string[]) => names.map(name => row[columns.get(name.toLowerCase()) ?? -1]?.trim() || '').find(Boolean) || '';
      const discovered = source.id === 'discoveries' ? urls(row.join('\n')) : [];
      const primary = catalogUrl(cell('Interactive'));
      const alternateUrls = urls(cell('Interactive Alt'));
      const projectUrls = source.id === 'projects' ? urls(cell('Link')) : [];
      const recoveryUrls = urls(cell('Latest Functional Archive (If Exists)'));
      const staticUrls = urls(cell('Static'));
      const website = primary || alternateUrls[0] || (source.id === 'discoveries' ? discovered[0] : '') || '';
      const name = cell('Title','Name') || (source.id === 'discoveries' ? website : '');
      if (!name) return;
      const candidates = [primary, ...alternateUrls, ...projectUrls].filter(Boolean);
      const matches = candidates.map(url => exact.get(url));
      const targets = [...new Set(matches.filter((entry): entry is CatalogEntry => Boolean(entry)))];
      const existing = !matches.includes(null) && targets.length === 1 ? targets[0] : undefined;
      if (existing && source.id !== 'beta') {
        existing.projectUrls = [...new Set([...(existing.projectUrls || []), ...projectUrls])];
        existing.recoveryUrls = [...new Set([...(existing.recoveryUrls || []), ...recoveryUrls])];
        existing.alternateUrls = [...new Set([...(existing.alternateUrls || []), ...alternateUrls])];
        existing.provenance = [...(existing.provenance || []), source.name];
        return;
      }
      const added = cell('Added','Date'), posted = cell('Posted'), updated = cell('Updated');
      const fallback = archives.get(website);
      const entry: CatalogEntry = { name, website, link: fallback?.link || '', date: updated || added || posted,
        added,posted,updated,aliases: cell('Alias').split(/[,\n;|]+/).map(s=>s.trim()).filter(Boolean), version:cell('Version'),
        engine:cell('Engine'), author:cell('Author'), importer:cell('Importer','Porter'), universe:cell('Universe'),
        type:cell('Type'),pov:cell('POV'),length:cell('Length'),description:cell('Description'),
        tags:normalizeTags([cell('Design Tags'),cell('Content Tags'),cell('Tags')]),status:cell('Status') || (source.id === 'broken' ? 'Broken Link' : ''),
        warnings:cell('Warnings'),comments:cell('Comments','Note'),alternateUrls,projectUrls,recoveryUrls,staticUrls,
        sourceId:source.id,sourceName:source.name,sourceRow:index+2,provenance:[source.name],
        availability: website ? primary ? 'Primary link' : 'Alternate link' : projectUrls.length ? 'Project link only' : recoveryUrls.length ? 'Archived recovery only' : staticUrls.length ? 'Static link only' : 'No playable link' };
      entries.push(entry);
      for (const url of candidates) {
        if(!exact.has(url)) exact.set(url,entry);
        else if(exact.get(url) !== entry) exact.set(url,null);
      }
    });
  }
  return entries;
}
export interface SearchToken { value: string; excluded: boolean; field: string }
export function parseCatalogSearch(raw: string): SearchToken[] {
  return (raw.match(/-?(?:[\w]+:)?(?:"[^"]*"|[^\s"]+)/g) || []).map(part => {
    const excluded = part.startsWith('-'); if(excluded) part=part.slice(1);
    const match=part.match(/^(author|fandom|tag|title):/i); const field=match ? match[1].toLowerCase() : '';
    if(match)part=part.slice(match[0].length);
    return {excluded,field,value:part.replace(/^"|"$/g,'').toLocaleLowerCase()};
  }).filter(token=>Boolean(token.value));
}
export interface CatalogSearchIndex { all: string; author: string; fandom: string; tag: string; title: string }
export function indexCatalogEntry(entry: CatalogEntry, canonicalAuthor = entry.author || '', authorAliases: string[] = []): CatalogSearchIndex {
 const title=[entry.name,...(entry.aliases || [])].join('\n');
 const author=[entry.author,entry.source_author,canonicalAuthor,...authorAliases].filter(Boolean).join('\n');
 const fandom=entry.universe || '', tag=(entry.tags || []).join('\n');
 return Object.fromEntries(Object.entries({title,author,fandom,tag,all:[title,author,fandom,tag,entry.importer,entry.description,entry.website,entry.status,entry.version,entry.comments,entry.warnings,...(entry.alternateUrls||[])].filter(Boolean).join('\n')}).map(([key,value])=>[key,value.toLocaleLowerCase()])) as unknown as CatalogSearchIndex;
}
export function matchesCatalogSearch(index: CatalogSearchIndex,tokens: SearchToken[]): boolean {
 return tokens.every(token=>{const match=index[(token.field || 'all') as keyof CatalogSearchIndex].includes(token.value); return token.excluded ? !match : match;});
}
