# Tests

All retained checks are here. Run commands from the repository root.

```bash
python3 tests/check-project-layout.py
pnpm build
pnpm test:rust
pnpm exec playwright install chromium
pnpm test:ui
```

| Check | Coverage |
|---|---|
| `rust/` | Archive integrity/retention, asset preservation, build identity, update diffs, metadata, aliases, website snapshots and viewer gates |
| `check-composables.cjs` | Actual composable refresh coalescing and retry after a failed cover lookup |
| `check-manager-lifecycle.cjs` | 100-card listener counts, menu focus/unmount, delayed-cover protection, serialized refresh/favorites/preferences, stale perk search, filter reset and large-text Settings scrolling |
| `check-project-layout.py` | Source/export privacy, readback, local doc links, Python/shell syntax, versions and Linux-only configuration |
| `rust/command_access.rs` | Main-window-only manager commands; failed startup blocks every window |
| `smoke-ui.cjs` | Palettes, menus and archive open/star/move/restore |
| `check-archive-management.cjs` | Starred-only Archives, group/edition naming, reassignment, separate restored cards, refreshed library, re-download protection and confirmed Force update |
| `check-ui-revision.cjs` | Fonts, cover fitting, metadata editor, structured filters, tags and website import warnings |
| `check-author-aliases.cjs` | Persistent additive aliases, catalog/library filters and preserved source credits on edit |
| `check-viewer-dock.cjs` | Narrow/wide viewport placement and isolation from authored CSS |
| `check-viewer-cheats.cjs` | Four palettes, font weights, existing cheats and reveal toggle binding |
| `check-viewer-presentation.cjs` | Live section fonts/sizes, protected project overrides/reset, cost/gain colors, navigation, density, consistent buttons, larger cheats and UI preview screenshots |
| `check-viewer-sections.cjs` | Parent dependency hiding, nested dependencies, cycles and conservative handling of OR/negative/unknown/multiple-parent rules |
| `check-viewer-images.cjs` | Viewport image release, repeated scrolling, stable dimensions, responsive percentage widths without height:auto, authored sizing and removed/reinserted elements |
| `rust/preferences.rs` | Global/per-project author typography, retained custom family and legacy metadata defaults |
| `rust/reader_assets.rs` | Lossless image extraction, escaped JSON, resource path validation, closed-session refusal and cache retirement |
| `check-release-assets.py` | Current frontend assets embedded in the packaged executable |

Rust modules are included under `cfg(test)` by their owning source modules, allowing focused checks of private storage functions without exposing production APIs. Browser checks mock backend calls and use Chromium; they do not prove native WebKit rendering or every authored CYOA.

`run-ui.cjs` starts/stops a built-frontend preview server on port 5797. Individual checks can use `CYOA_TEST_URL` with an existing server. Screenshots and historical native reports live in ignored `results/`; disposable native fixture data belongs in ignored `fixtures/` and can be removed after verification.

For native regression checking, use a separate absolute `CYOA_MANAGER_DATA_DIR` and isolated XDG data/config/cache directories. Verify that enabling hidden-choice previews leaves row/tab visibility unchanged, preserves choice codes, points and requirements, and reveals only choices/addons in a section opened normally. Test alias persistence across a restart and verify that source-author metadata remains unchanged. Manager and reader windows must report neither fullscreen nor maximized at startup.

For an optional read-only preparation check on a large export, set `CYOA_READER_TEST_PROJECT` to its absolute path and run `cargo test --release --features custom-protocol --manifest-path src-tauri/Cargo.toml large_export_preparation -- --ignored --nocapture`. It checks the original digest, reports the reduced browser payload and retires its temporary resources. Native scrolling checks should record WebKit process memory over repeated passes, confirm layout and choice codes, and close only the isolated test windows.

Reader preference persistence and inheritance are covered by `rust/reader_settings.rs`. Native checks must also exercise the patched ICC2 Plus bundle: a mock DOM cannot catch minified-renderer variable scope errors. Validate cost/gain styles, live typography, choice widths and identical captured build codes before/after changes.

For section hiding, derive a native fixture from a valid export. Select a race and its sub-choice, close/reopen the parent navigation tab, and verify that the child disappears/reappears while those selections remain. Manual hide/show must leave the full captured build code identical. The reveal cheat must not open the closed parent; disabling automatic hiding must restore the authored visibility behavior.


## Repeatable native checks

`native-reader.py` uses a valid private DBZ Requiem export, strips fixture artwork and retains its actual navigation/race/sub-choice schema. It runs the Rust-patched engine in an ephemeral WebKit context; only settings endpoints are mocked. Requires Python GObject, GTK3 and WebKit2 4.1 on a desktop session. No private export is bundled in the repository.

```bash
CYOA_NATIVE_BUNDLE=/tmp/cyoa-native.js cargo test --release --features custom-protocol --manifest-path src-tauri/Cargo.toml reveal_changes_only_presentation_gates
python3 tests/native-reader.py --project /absolute/path/to/private-project.json --bundle /tmp/cyoa-native.js --report /tmp/native-reader-result.json
python3 tests/check-startup-failure.py --binary src-tauri/target/release/cyoa-manager --report /tmp/native-startup-result.json
```

The startup check launches and terminates only its own test process, uses isolated manager/XDG directories, and verifies that an invalid SQLite fixture remains unchanged while the app stays open to report the error. Neither helper operates on the running user's app or live browser profile.

The compatibility suite checks Remix/script detection without executing expressions, ordinary ICC2 variable defaults, actionable source/website-copy warnings, separate card creation and explicit bundled-viewer bypass. The streaming inspector skips artwork. A recognised structure is not a certification of all creator-specific extensions.

Catalog fixtures exercise restart/offline cache reads, forced/partial refresh, routed source identities, metadata and alias search, ambiguous links, bounded pages, explicit recovery, preview/discard/apply, selection across filters, serial queue stopping and retry. Browser fixtures mock Tauri; Rust fixtures use isolated temporary data and local HTTP responses. They do not migrate or download into the live library.
