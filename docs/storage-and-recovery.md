# Storage and recovery

Release data defaults to `$XDG_DATA_HOME/cyoa-manager/`, or `~/.local/share/cyoa-manager/` when XDG_DATA_HOME is unset. An absolute `CYOA_MANAGER_DATA_DIR` overrides the manager data location. Preserve native browser storage separately when backing up or testing.

| Relative path | Contents |
|---|---|
| `cyoas/` | Manager-owned project files and local assets |
| `save/library.sqlite3` | Library identities, source credits, tags and metadata |
| `save/preferences.json` | Theme, fonts and archive retention |
| `save/author-aliases.json` | Preferred author names and all retained aliases |
| `save/builds/<project-id>/` | Identified builds and recovery records |
| `save/versions/<project-id>/<version-id>/` | Edition metadata, checksums and lossless archive ZIP |
| `save/perk-index.sqlite3`, `save/perk-images/` | Rebuildable perk search index and optional images |
| `backups/` | Previous alias settings and desktop shortcuts saved before replacement |

Native browser storage uses the preserved application identifier `com.om1cr0n.cyoa-manager`; on the tested Linux installation it lives below `~/.local/share/com.om1cr0n.cyoa-manager/`. Dev/test browser profiles should use separate XDG directories as well as an isolated manager data directory.

## Back up

Close manager and reader windows before copying the manager data folder and native browser storage. Keep external imported source files too: the manager's data folder cannot back up files it does not own. Do not publish these snapshots to GitHub.

Edition history is not a substitute for a full backup. Archives protect against project updates, while a full snapshot also preserves library metadata, aliases, builds and native browser saves.

## Recover archives

`files.zip` is lossless and paired with SHA-256 checksums. Opening/restoring verifies extraction. Corruption stops extraction/retirement; altered cache contents are preserved. Starred editions and currently open editions are excluded from automatic retention cleanup.

Before using an older executable that expects loose archive files, close the app and preview extraction:

```bash
python3 scripts/extract-archives.py --data-dir /absolute/path/to/cyoa-manager-data
```

Add `--apply` to materialize the older layout. ZIP files remain intact. Preserve new work separately before restoring an older full library snapshot.

Alias settings are written atomically. A successful change keeps the previous mapping in `backups/author-aliases.previous.json`. Invalid/conflicting aliases are rejected; an unreadable or newer-format alias file is preserved rather than reset. Aliases do not rewrite original project credits, so removing a mapping reverses its display effect.

## Local workspace snapshots

Historical snapshots and private test reports must stay outside the entire development workspace. The local recovery directory is `$XDG_DATA_HOME/cyoa-manager/backups/workspace-history-2026-10-07/` (normally under `~/.local/share/`). Its `checkpoints/README.md` indexes the preserved snapshots. The workspace `backups/README.md` is only a location pointer. Never stage recovery material, runtime data or private reports.
