# Linux user guide

This fork is developed for Linux x86_64. Manager and reader windows open normally, without forced maximization or fullscreen. You can maximize them yourself.

## Library and catalog

Import a project file or folder, download from a project/site URL, or add an entry from the Infaera catalog. ICC2 Plus is preferred after a structural format check. **Edit Project → Preferred viewer** can force ICC Original or another viewer when needed. Structural checks do not simulate every author's gameplay logic.

Library cards show title, author and useful tags. Filter separately by author, fandom, mod status, modder, player status and extra tags. Search accepts `-word` exclusions; hover over a search box for its full value and syntax. Card menus open above surrounding content.

Available structured metadata supplies author, fandom and description. Exact catalog source matches fill missing fields without replacing existing fields or manual tags. Unknown or conflicting credits remain unspecified. The original author and modder are separate fields; personal completion is `Completed(N)`, calculated from saved builds.

## Author aliases

Open **Author aliases** in the sidebar. Choose a preferred author name and add each source name as an alias, then **Save aliases**. For example, Gaston1231 and Gaston1597 can both appear as Gaston. Adding a name keeps earlier aliases. Names cannot belong to two authors.

Aliases persist across restarts and update library cards, author filters, catalog entries and archived author names. Source credits remain stored unchanged, so removing an alias restores the source display name. Editing another project field does not replace its source credit with the alias. The initial Gaston1231 → Gaston mapping is editable. Author aliases do not change the separate modder field.

## Themes, fonts and covers

**Settings → Appearance** selects Catppuccin Mocha, Macchiato, Frappé or Latte. Manager and CYOA fonts can be selected independently from installed font families. Manager text size (12–24 px) applies immediately across manager pages. When a CYOA font override is enabled, all reader text uses the selected regular face, including headings and authored containers; clearing the override restores author typography.

Covers fit inside a consistent frame without cropping. Missing covers can be recovered from later row or choice artwork; projects without artwork show a fallback. Large thumbnails are optimized automatically to at most 512 pixels and 60 KiB. This does not rewrite project artwork.

## Builds and reader controls

**Builds** and **Search choices** share a dock at the reader's bottom right. Save current selections through Builds or import a text/identified save. Identified builds carry CYOA identity, viewer and edition fingerprint. Other-CYOA identified saves are rejected; recovered or older saves require review before loading.

Legacy recovery reads original native storage without deleting it. Only uniquely matched choice IDs can be associated automatically. Empty legacy payloads cannot restore missing choices, and ambiguous saves remain available for manual association.

Enable cheats in Settings to open the reader's **Cheat Menu**. ICC2 Plus offers **Show hidden choices**, which previews hidden choices and addons inside rows/tabs you open normally. It does not open every tab or bypass row navigation, selection requirements or point costs. Toggle it off to restore normal visibility. Unsupported viewers do not offer this toggle.

## Archives and updates

Archiving moves an edition out of the library; open or restore it directly from **Archives**. An older library copy can be archived under its newer counterpart. Manager-owned duplicate files are retired only after a verified archive copy; external source files are preserved.

Re-download stages the replacement and compares choices, fields and local assets. Changes archive the previous edition and show a difference report. Identical downloads keep the current edition; failed downloads leave it intact. **Find duplicates** reports identical exports and likely matches without deleting them.

Keep 3, 4 or 5 ordinary editions per CYOA. Starred and currently open editions are protected. Archives use verified lossless ZIP compression. Opening or restoring extracts a verified copy; idle extraction caches are removed on close or subsequent maintenance.

## Download size and compression

**Large-project threshold** triggers the ICC download rule and caps saved website downloads. **Large-project action** offers:

| Action | Effect |
|---|---|
| Ask | Choose when a large project is downloaded |
| Keep original artwork | Preserve embedded artwork |
| Separate images (lossless) | Move image data into local files |
| Reduce artwork quality (lossy) | Reduce image quality/size; preserve a prior edition |

Thumbnail optimization and lossless archive compression happen automatically, independently of this rule. Existing settings are retained.

## Saved websites

Non-ICC entries can be saved as website snapshots with linked resources and literal JavaScript dependencies. Missing resources are reported. **Website (saved)** opens the copy internally. **Website (online)** handles live services, dynamic URLs and navigation beyond the snapshot. Website readers cannot invoke manager commands. Saved websites also have version history.
