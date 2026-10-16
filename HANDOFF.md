# CYOA Manager: maintainer handoff

Audit date: 2026-10-16. This fork develops and tests **Linux x86_64**. The manager is Vue 3/TypeScript with Tauri 2/Rust and WebKitGTK. It manages local files; it is not a hosted service.

## Start here

1. Read [the audit](docs/audit-2026-10-16.md) for current evidence and limitations.
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
