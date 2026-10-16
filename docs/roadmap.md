# Project trajectory

These are proposed future changes, not claims of implemented features. Prioritize correctness and long-session stability before expansion. Deliver one reviewed slice at a time; preserve existing library/build identities.

## Phase 1: reliability and recovery

| Feature | Reason | Acceptance condition |
|---|---|---|
| Reproducible native fixture suite | Chromium mocks miss WebKit/engine failures | A valid, redistributable small export exercises opening, closing, build capture, display overrides and parent tabs on Linux CI |
| Interrupted-operation journal | Downloads/archives span files and SQLite | Kill the app at each staged transition; restart preserves the previous edition or completes the new one without losing assets |
| Per-project mutation coordination | Separate background tasks can target the same project | Concurrent update/archive/delete has deterministic outcomes, rejects stale inputs and preserves builds |
| Crash recovery checkpoint | Unsaved long sessions are costly | Opt-in bounded local checkpoints restore the right UUID/edition; explicit saved builds remain independent |
| Reader diagnostics | Intermittent crashes need evidence | Export a user-reviewed report with versions, crash reason and memory samples; no CYOA text, choices or personal paths by default |
| Backup verification wizard | Archives alone are not full backups | Checks SQLite integrity, manifest hashes and a staged restore; clearly distinguishes a local copy from a second-disk backup |

Do not promise that every WebKit crash is eliminated. Measure repeated scrolling, reader close/reopen, idle time and multiple readers with representative large CYOAs. Compare both memory peaks and settled memory, with unchanged choice codes and document layout.

## Phase 2: maintainability and performance

- Split `commands.rs` by download routing, artwork operations and reader windows. Preserve command names/payloads; move one boundary at a time with the same regression suite before/after.
- Convert minified viewer replacements into a versioned bridge adapter with explicit marker counts and compatibility reporting. Unsupported engine updates must fail safely rather than partially patching gameplay.
- Add native dependency advisory checks and a controlled upgrade cadence. The 2026-10-16 scan has zero vulnerability findings but retains seven unmaintained and two unsoundness warnings in upstream dependencies. Review upstream GTK3 binding retirement before a Tauri/WebKit stack migration; do not swap binding generations inside the current runtime.
- Add catalog refresh feedback and a persisted last-known-good public catalog. Label stale data; do not confuse refresh failures with an empty catalog.
- Consider library virtualization only after measuring thousands of cards. Preserve keyboard navigation, open menus, filters and cover frames. Avoid adding a virtualization dependency for the current library without evidence.
- Add bounded download cancellation/retry and progress recovery. Cleanup may remove only uncommitted staging; committed files must survive later errors.
- Strengthen power-loss durability with file/directory sync at important commit boundaries. Atomic rename protects against partial files but alone is not a guarantee against sudden disk/power failure.

## Phase 3: useful product additions

| Feature | User value | Boundary |
|---|---|---|
| Saved filter presets and collections | Quickly return to favorite authors/fandoms or unfinished stories | Separate personal organization from imported source metadata |
| Edition comparison view | See changed choices, costs and requirements before updating | Match stable IDs; show unknown matches rather than guessing from names |
| Build comparison and notes | Compare character variants and remember goals | Preserve original native code and edition fingerprint |
| Per-CYOA reading profiles | Convenient typography, density and section visibility presets | Explicit reset; global edits cannot overwrite pinned exceptions |
| Manual dependency associations | Correct ambiguous child-section relationships | Presentation-only relationships; no requirement/point changes |
| Keyboard shortcuts and command palette | Faster open/search/build/navigation workflows | Discoverable shortcut list, focus-safe dialogs, configurable conflicts |
| Bulk metadata review | Review missing author/fandom/modder fields together | Preview changes and preserve manually supplied values and aliases |
| Reader session overview | Close completed readers and identify heavy sessions | No silent closing or unloading of unsaved work |

## UI direction

Keep Catppuccin palettes, regular control text, consistent cover frames and a compact desktop workflow. Improve one page at a time with realistic long titles, empty/loading/error states, keyboard focus and large manager font sizes. Use [UI guidelines](ui-guidelines.md) rather than applying promotional website styles to a library manager or rewriting an author's CYOA.

## Stop conditions

A phase is complete when its acceptance conditions, relevant native checks and documentation pass. Defer broad rewrites, cloud accounts/sync, cross-platform releases, AI-generated metadata and new UI frameworks until there is a specific user need and a migration plan. Linux x86_64 remains the supported delivery target.
