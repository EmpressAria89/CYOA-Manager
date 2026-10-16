# Linux user guide

This fork is developed for Linux x86_64. Manager and reader windows open normally, without forced maximization or fullscreen. You can maximize them yourself.

## Library and catalog

Import a project file or folder, download from a project/site URL, or add an entry from the Infaera catalog. ICC2 Plus is preferred after a structural format check. **Edit Project → Preferred viewer** can force ICC Original or another viewer when needed. Structural checks do not simulate every author's gameplay logic.

Library cards show title, author and useful tags. Filter separately by author, fandom, mod status, modder, player status and extra tags. Search accepts `-word` exclusions; hover over a search box for its full value and syntax. Card menus open above surrounding content. Use **Clear filters** to reset searches and metadata filters together.

Available structured metadata supplies author, fandom and description. Exact catalog source matches fill missing fields without replacing existing fields or manual tags. Unknown or conflicting credits remain unspecified. The original author and modder are separate fields; personal completion is `Completed(N)`, calculated from saved builds.

## Author aliases

Open **Author aliases** in the sidebar. Choose a preferred author name and add each source name as an alias, then **Save aliases**. For example, Gaston1231 and Gaston1597 can both appear as Gaston. Adding a name keeps earlier aliases. Names cannot belong to two authors.

Aliases persist across restarts and update library cards, author filters, catalog entries and archived author names. Source credits remain stored unchanged, so removing an alias restores the source display name. Editing another project field does not replace its source credit with the alias. The initial Gaston1231 → Gaston mapping is editable. Author aliases do not change the separate modder field.

## Themes, fonts and covers

**Settings → Appearance** selects Catppuccin Mocha, Macchiato, Frappé or Latte. Manager and CYOA fonts can be selected independently from installed font families. Manager text size (12–24 px) applies immediately across manager pages. **Default CYOA typography** chooses **My CYOA font** or **Author default** for all projects. Your chosen font stays saved when using author defaults. **Edit Project → CYOA typography** sets an exception for that CYOA: **Use global setting**, **Author default**, or **My CYOA font**. Global font changes also update open ICC2 Plus readers; per-card Edit Project exceptions apply when opening a reader. Custom fonts use the selected regular face, including headings and authored containers; author mode leaves original typography intact.

Covers fit inside a consistent frame without cropping. Missing covers can be recovered from later row or choice artwork; projects without artwork show a fallback. Large thumbnails are optimized automatically to at most 512 pixels and 60 KiB. This does not rewrite project artwork.

## Builds and reader controls

**Builds** and **Search choices** share a dock at the reader's bottom right. Save current selections through Builds or import a text/identified save. Identified builds carry CYOA identity, viewer and edition fingerprint. Other-CYOA identified saves are rejected; recovered or older saves require review before loading.

Legacy recovery reads original native storage without deleting it. Only uniquely matched choice IDs can be associated automatically. Empty legacy payloads cannot restore missing choices, and ambiguous saves remain available for manual association.

ICC2 Plus keeps large embedded image bytes out of its parsed browser model by making a disposable, lossless resource cache. Artwork loads near the viewport and releases its image source when scrolled far away; intrinsic SVG placeholders preserve the layout without adding HTML width/height attributes or changing author CSS. Image preloading is disabled in this reader. The library export, image quality, choices and saved-build fingerprints stay unchanged. Temporary reader resources are removed when the window closes; caches left by a crash are cleaned at the next startup. The first load may take longer while resources are prepared, and needs temporary disk space for the extracted artwork.

A reader that is still open retains its project. Close finished CYOA windows to release their document; saving a build alone does not close the window. WebKit uses a document-oriented cache policy to reduce retained pages after closing readers. Save builds regularly: these changes reduce memory pressure but do not guarantee recovery from every system or WebKit crash.

Enable cheats in Settings to open the reader's **Cheat Menu**. ICC2 Plus offers **Show hidden choices**, which previews hidden choices and addons inside rows/tabs you open normally. It does not open every tab or bypass row navigation, selection requirements or point costs. Toggle it off to restore normal visibility. Unsupported viewers do not offer this toggle.

## Archives and updates

Archiving moves an edition out of the library; open or restore it directly from **Archives**. An older library copy can be archived under its newer counterpart. Manager-owned duplicate files are retired only after a verified archive copy; external source files are preserved.

**Add copy to library** restores a separate card with the archived edition's own name, leaving the main card and archive intact. Restored cards show **Archived edition** and cannot be re-downloaded. Rename individual editions or archive groups, and use **Move edition** to correct a misplaced archive. Group labels do not rename library cards.

The **Archives** tab lists starred editions only. Unstarred automatic backups stay on disk and remain accessible through a library card's **Version history** or the separate **Manage automatic backups** control, including groups whose library card was removed. Unstarring an edition hides it from Archives; it does not immediately delete it.

**Edit → Force update** explicitly allows replacing an older restored card from its source. Confirming saves the form edits and archives the current edition before replacement. Normal re-download stays blocked for restored cards. Empty downloaded projects are rejected even when forcing an update.

Re-download fills missing title, author, modder, fandom and description from the source, including when the project file is otherwise unchanged. Nonempty edited fields are preserved. Missing fandoms also use known title fallbacks, including Dragon Age; absent source metadata cannot reliably identify every universe.

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

## Live reader display controls

ICC2 Plus readers have one consistent dock with **Top**, **Bottom**, **Display**, **Search choices**, **Builds**, and **Cheats** when enabled. It sits above the viewer’s point bar. Cheats use a larger panel with larger inputs and buttons.

**Display** applies changes immediately without reloading the CYOA. Choose **Whole CYOA**, **Choice titles**, **Descriptions**, or **Costs and gains**. Fonts and text-size percentages use the viewer’s authored styles, including private row/choice styles: 100% keeps each original size. An empty font uses that section’s author font. Your installed fonts are offered as suggestions.

Choose **This CYOA** to pin a section’s font and size for that project. **Global defaults** affects inherited sections across open readers. A saved project section is protected from later global edits until **Reset section** is used. Display preferences are separate from builds and project exports, stored in `save/reader-settings.json`; they persist across restarts and ordinary re-downloads of the same library card. A separately restored card has a separate identity.

Under **Layout and colors**, **Choice scale** shrinks/enlarges the CYOA and adjusts choice widths proportionally so more/fewer cards fit where the authored layout permits. Headers, order, image proportions and game rules are preserved. Text size can be adjusted independently. Cost badges use red, gains use green, and assigned values use blue; classification follows the viewer’s score/discount sign. Turn off **Distinct cost / gain colors** to restore author score styling.

**Display → Hide sections** automatically hides dependent sub-sections when their parent tab is closed (ICC2 Plus). It follows unambiguous positive choice dependencies; complex OR/group rules stay author-controlled. Disable automatic hiding globally or for one CYOA. Under **This CYOA**, uncheck individual sections to hide them, then **Apply and save**. Recheck to show them again when their authored requirements allow, or **Reset section** to clear hiding preferences. Hidden sections retain every selected choice and point effect.
