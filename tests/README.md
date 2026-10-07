# Tests

All retained checks are here. Run commands from the repository root.

```bash
pnpm test:rust
pnpm build
pnpm exec playwright install chromium
pnpm test:ui
```

| Check | Coverage |
|---|---|
| `rust/` | Archive integrity/retention, asset preservation, build identity, update diffs, metadata, aliases, website snapshots and viewer gates |
| `smoke-ui.cjs` | Palettes, menus and archive open/star/move/restore |
| `check-ui-revision.cjs` | Fonts, cover fitting, metadata editor, structured filters, tags and website import warnings |
| `check-author-aliases.cjs` | Persistent additive aliases, catalog/library filters and preserved source credits on edit |
| `check-viewer-dock.cjs` | Narrow/wide viewport placement and isolation from authored CSS |
| `check-viewer-cheats.cjs` | Four palettes, font weights, existing cheats and reveal toggle binding |
| `check-release-assets.py` | Current frontend assets embedded in the packaged executable |

Rust modules are included under `cfg(test)` by their owning source modules, allowing focused checks of private storage functions without exposing production APIs. Browser checks mock backend calls and use Chromium; they do not prove native WebKit rendering or every authored CYOA.

`run-ui.cjs` starts/stops a built-frontend preview server on port 5797. Individual checks can use `CYOA_TEST_URL` with an existing server. Screenshots and historical native reports live in ignored `results/`; disposable native fixture data belongs in ignored `fixtures/` and can be removed after verification.

For native regression checking, use a separate absolute `CYOA_MANAGER_DATA_DIR` and isolated XDG data/config/cache directories. Verify that enabling hidden-choice previews leaves row/tab visibility unchanged, preserves choice codes, points and requirements, and reveals only choices/addons in a section opened normally. Test alias persistence across a restart and verify that source-author metadata remains unchanged. Manager and reader windows must report neither fullscreen nor maximized at startup.
