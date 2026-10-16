# CYOA Manager: maintainer handoff

Audit date: 2026-10-16. This fork develops and tests **Linux x86_64**. The manager is Vue 3/TypeScript with Tauri 2/Rust and WebKitGTK. It manages local files; it is not a hosted service.

## Start here

1. Read [skill assurance](docs/skills-assurance.md) for the latest review and [the historical audit](docs/audit-2026-10-16.md) for its evidence and limitations.
2. Read [development](docs/development.md), [tests](tests/README.md) and [storage/recovery](docs/storage-and-recovery.md).
3. Choose one bounded item from [the roadmap](docs/roadmap.md), with its acceptance conditions.
4. Before publishing, follow [GitHub publication](docs/github-publication.md). Publication remains a separate, explicit action.

In the installed workspace, source is `app/` and the installed executable is `runtime/bin/cyoa-manager`. In a clone, this document and `package.json` are at the repository root. Launch the installed app using the outer `launch.sh`. A source checkout needs a build; a runtime archive does not contain the source build environment.

## Resume after interruption

The installed workspace's `backups/README.md` points to the private audit recovery directory. Read its `state.json`, `RESUME.md` and latest logs before repeating work. The recovery snapshot includes the pre-audit source and executable with a SHA-256 manifest. Verify its hash before extracting into a separate staging folder. Do not extract an old snapshot over live library data.

Record phase completion only after verification. Write checkpoints through a temporary file and rename. Never remove the last verified source/runtime snapshot, unique full-data backups, starred editions or preserved Git metadata. A local checkpoint helps recover interrupted work; it does not survive failure of the disk containing it. A second physical destination is needed for that.

## Compatibility boundaries

- Keep the application identifier `com.om1cr0n.cyoa-manager`: native browser saves depend on it.
- Keep project UUIDs, edition fingerprints, viewer IDs, original credits and associated builds stable.
- Never infer that a build belongs to a different CYOA from a title alone.
- Re-downloads archive changed editions before replacement. Restored copies retain their archived name and independent identity; normal re-download is blocked. Force update is explicit.
- Keep original project exports/artwork unchanged by reader presentation controls. Lossless archive compression differs from optional lossy artwork reduction.
- The reader image cache is session-scoped and disposable. Retain viewport image release, intrinsic-ratio placeholders, disabled eager preload and `DocumentViewer` cache mode.
- Manager typography must not leak into author-created pages. Reader overrides are independent by text section and CYOA; author-default mode and pinned overrides must remain available.
- Hidden-choice preview never opens every tab. Automatic child-section hiding changes rendering only, not selections or points. Complex dependencies remain authored; manual hiding provides the fallback.
- Only the exact main window may invoke manager commands; reader/website/recovery windows are rejected. Do not weaken website capabilities to make a site work.
- Library startup failures block manager commands instead of operating on an empty fallback.

## Module ownership

| Location | Responsibility |
|---|---|
| `src/views/`, `src/components/` | Manager pages, filters, project/archive dialogs |
| `src/composables/` | Shared library state, aliases, fonts/settings, bounded cover cache |
| `src-tauri/src/commands.rs` | Import/download routing, project mutations, viewer creation; legacy large module |
| `library.rs`, `models.rs` | SQLite schema, identities, serialized metadata and sessions |
| `history.rs`, `archive_storage.rs`, `assets.rs` | Verified edition copies, retention, compression and assets |
| `builds.rs` | Identified build persistence, legacy matching and edition association |
| `metadata.rs`, `catalog_metadata.rs`, `author_aliases.rs` | Source metadata, public catalog enrichment and personal display aliases |
| `protocol.rs` | Session-bound resources and version-specific viewer bridge patches |
| `reader_assets.rs`, `viewer_images.js` | Lossless externalization and image resource lifetime |
| `reader_settings.rs`, `viewer_presentation.js` | Independent presentation preferences and live display controls |
| `viewer_tools.js`, `viewer_builds.js`, `viewer_overlay.*` | Shared dock, native save/load adapter and cheats |
| `websites.rs`, `perk_index.rs` | Website snapshots and rebuildable full-text search |
| `public/viewers/` | Required third-party engines; preserve versions and attribution |

Bridge patches target exact minified bundle markers. If a viewer changes, revalidate every marker and native behavior; a compile or mocked DOM is insufficient. Native tests need valid exported schemas, including styling/addons/templates.

## Delivery gate

Run source/privacy/docs checks, TypeScript/Vite build, Rust tests, browser checks and the relevant native WebKit scenario. Verify packaged/executable hashes and embedded assets. Install atomically without stopping a user's active reader. Ask the user to save builds and restart afterward. Keep local data, private fixtures, screenshots and logs outside the published source. Do not auto-fetch upstream, add platform release jobs, push or reattach Git as a side effect of preparing an export.

## Local follow-up: scripted viewer compatibility

After publication `2de3c7e`, added `compatibility.rs`, a streaming, read-only preflight for Remix scripts and variable operations. Built-in reader opening (including archives) now warns with source-site/separate-website-copy/explicit-bypass actions. Import metadata no longer claims these exports are ICC2-compatible. No script converter is enabled; implement only exact, tested operation mappings in a derived copy with explicit build identity rules. The existing included-viewer downloader recognizes legacy `app.js`/`core.js` layouts, not every Remix site's loader; use the separate website-copy path for those sites. This follow-up is not published to GitHub.

Validation: parser regression tests, real Fire Emblem export inspection (1,402 expressions; 24 unsupported variable modifications), full Rust suite, 12 browser suites and source/export/embedded-assets gates. The local runtime was installed atomically; restart the manager to load it. The previous runtime package and validation logs are retained under the private audit backup's `viewer-compatibility/` directory. Personal exports and builds were not changed.

## Local catalog redesign

Catalog work follows [the specification](docs/specs/catalog-redesign.md), [domain decisions](docs/adr/0001-catalog-and-edition-identities.md) and [catalog guide](docs/catalog.md). Local tickets are under ignored `.scratch/catalog-redesign/issues/`. An organized pre-change source snapshot, implementation/review checkpoint and validation evidence are indexed by the outer workspace's backup index. Read that checkpoint before resuming interrupted work. This redesign is local and unpublished; the last GitHub publication remains `2de3c7e`. Git stays detached.

Catalog validation: 57 Rust tests passed (1 optional test ignored), 15 browser suites passed, and TypeScript/Vite and embedded-asset checks passed. Cached catalog reads remain available while a refresh runs. Delivery receipts and rollback packages are indexed by the outer `backups/catalog-redesign/state.json`; consult that file for the installed executable and package hashes. No source has been pushed; Git remains detached. Restart the manager after installation to load the new executable.

## Assurance follow-up

[Skill assurance](docs/skills-assurance.md) records requirement coverage and review corrections without reverting later changes. [Next reliability slices](docs/specs/assurance-followups.md) specify the remaining direct update-commit integration proof and measured efficiency work. Completed catalog tickets are marked done; future slices remain open. The installed workspace backup index identifies final validation, rollback and delivery hashes.
