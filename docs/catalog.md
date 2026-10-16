# Catalog discovery and downloads

The manager reads one public catalog rather than counting the main and beta Infaera websites as different sources. No Google account or API key is needed.

## Sources and offline use

- **Beta Index:** primary listings and source metadata, including aliases, versions, warnings and alternate/static links.
- **Project.json:** supplementary project-file links. Matching is conservative; a matching title or host alone never merges records.
- **404 Broken Link:** recovery candidates, not proof that an archive matches the current edition.
- **New CYOA Posts ToBeAdded:** optional unreviewed discoveries, hidden by default.
- Introduction, KeyMap and Other Lists explain the source; they are not recursively crawled as CYOA feeds.

Validated last-good source snapshots persist in `$XDG_CACHE_HOME/cyoa-manager/catalog/`, or `~/.cache/cyoa-manager/catalog/`. An isolated `CYOA_MANAGER_DATA_DIR` uses its own save/cache area. Cached data appears without waiting for the network, even if expired. The catalog refreshes stale data separately; **Reload Catalog** forces a refresh. Each source shows its last successful time and any error. Failed sources keep their previous rows and timestamp. A cold refresh checks four fixed sources sequentially, with a 15-second timeout per source; it can take about a minute when services are unavailable.

## Searching and browsing

Search matches aliases, source and canonical author names, fandom, descriptions, tags and other listing metadata. Examples:

- `"Fire Emblem" -tag:horror`
- `author:Gaston fandom:Naruto`
- `title:"Other Title"`

Primary filters remain visible; secondary filters are under Advanced filters. Search/filter/sort settings persist locally. Pages render 48 cards; search still covers the complete catalog. Page selection and selection of all matching results are separate actions. Selection survives page/filter changes. The queue uses all selected entries, not just the current page or filter; **Stop after current** leaves pending selections intact, and **Retry failed** retries failures without repeating successful imports.

Catalog status describes the work (for example WIP, MOD or broken link), not your completion status. Importer credit is never automatically treated as Modder. Missing source fields may be filled on a new card; existing personal labels and notes remain yours.

## Download options

**Download options** inspects actual source content instead of trusting Engine. Inspection does not execute remote scripts. A structural ICC2 match does not certify every creator extension. Known Remix/scripts require the original viewer. HTML with meaningful query/fragment routing is not silently replaced by a guessed default JSON project.

- **Project download:** a new independent local card for supported project content.
- **Website copy:** a separate card preserving the site's own viewer. Dynamic services and undiscovered resources may still need the network; test the copy offline before relying on it. No automatic transfer of manager builds is promised.
- **Open source:** the selected playable source or alternate online.
- **Recovery:** explicit archive or website recovery creates a separate copy. Its age is unverified. Never assume it is the latest edition or import builds from a different fingerprint.

The batch queue imports supported project downloads serially. Existing cards, native website copies and archive recovery require individual review, so bulk selection cannot silently replace them. Unknown Engine labels can still download if actual content is supported.

## Reviewed updates

**Check for updates → Preview update** stages a candidate and shows actual added, removed and edited choices before any replacement. If several library editions share a source, select the intended card. Restored editions are protected from ordinary updates. Apply only after reviewing the preview; source identity, current card, current files/assets and staged candidate must still match. The old edition is archived and starred before replacement. Builds keep their project ID and exact edition fingerprint.

Closing a preview discards its uncommitted staging. Up to three previews may exist; they expire after 15 minutes, and process-owned temporary files are private. A later Linux startup removes abandoned staging from dead processes. Downloads committed to the library survive subsequent cleanup errors.

## Bounds and remaining limitations

The resolver supports selected JSON and static project links/default project locations; it does not run JavaScript to discover arbitrary runtime APIs. HTML inspection is capped at 2 MiB; JSON at 512 MiB. Explicit recovery ZIPs are capped at 64 MiB compressed and 512 MiB expanded, with path/symlink and multiple-project rejection. Your configured download-size limit is checked before a new project import is committed. Reviewed update candidates currently use the fixed resolver bounds above; the per-import setting does not limit update previews.

Inspection currently downloads project JSON again when the user later imports or previews it; it does not retain a reusable inspection download. Full semantic comparison/image normalization still materializes candidate JSON, although comparison no longer clones every artwork-heavy choice. Queue stopping does not interrupt an active network operation; it stops remaining items. These are candidates for measured future optimization, not guarantees of the current version.
