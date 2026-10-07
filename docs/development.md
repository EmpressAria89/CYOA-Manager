# Linux development

The Git repository is the folder containing `package.json` and `src-tauri/`. In the local installed workspace this is `app/`; a Git clone has these files at its root. Commands below run from that repository root.

Development/native testing targets Linux x86_64, currently CachyOS/KDE. Other architectures and platforms are not maintained here. The Linux CI workflow is prepared for Ubuntu 24.04 x86_64; a local pass does not prove a remote CI run or every Linux distribution.

## Dependencies

Use Node.js, pnpm and a current stable Rust toolchain. Install the [official Tauri Linux prerequisites](https://v2.tauri.app/start/prerequisites/#linux). AVIF decoding additionally needs dav1d and its development headers: `dav1d` on Arch-family systems, `libdav1d-dev` on Debian/Ubuntu. Font enumeration uses Fontconfig.

```bash
pnpm install --frozen-lockfile
pnpm tauri dev
```

Development uses a Vite server on port 5697. The app's development data stays in ignored `save/` and `cyoas/` folders in the repository unless `CYOA_MANAGER_DATA_DIR` is set to an absolute isolated data folder.

## Release

```bash
pnpm release:linux
python3 scripts/package-linux.py
```

`release:linux` builds the frontend, compiles Rust with `--release --features custom-protocol`, and checks that current frontend asset names are embedded in the binary. A compile guard rejects release builds missing that feature. No development server is needed by the resulting executable.

The packager creates `release/cyoa-manager-linux-<architecture>/` and its `.tar.gz`, including `launch.sh`, the executable, viewer engines, icons, menu installer, license and documentation. It refuses to overwrite an existing package directory. It excludes libraries, backups, private test results and fixtures. Architecture labels describe the local build; only x86_64 is currently tested.

Tauri's bundle targets are Linux AppImage, DEB and RPM. The tested local delivery is the runtime archive. Installer bundles need separate verification on their target distribution.

## Tests

```bash
pnpm test:rust
pnpm build
pnpm exec playwright install chromium
pnpm test:ui
```

The browser installation follows [Playwright's browser setup](https://playwright.dev/docs/browsers). `test:ui` starts/stops an isolated preview server and runs retained browser checks with mocked Tauri commands. Native WebKit testing is separate; see [tests/README.md](../tests/README.md).

## Source map

| Folder | Responsibility |
|---|---|
| `src/` | Vue pages, components, settings and alias display logic |
| `src-tauri/src/` | Storage, downloads, archives, builds, protocols and native windows |
| `public/viewers/` | Bundled viewer assets; adapter compatibility depends on their versions |
| `tests/` | Browser checks, release verification and Rust unit modules |
| `scripts/` | Linux packaging, menu installation and archive recovery |
| `docs/` | User, development, storage and requirements documentation |
| `release/`, `dist/`, `src-tauri/target/` | Generated build output, ignored by Git |

The existing application identifier is preserved to keep native viewer storage compatible. Alias mappings are separate from library metadata. ICC2 Plus presentation patches leave its shared requirement checker and row navigation gates intact. New viewer bundles must revalidate adapter gates; unsupported bundles do not advertise hidden-choice previews.

## GitHub preparation

Publish the source repository, not the outer installed workspace. Do not add local runtime binaries, backup snapshots, personal CYOAs, browser profiles, private test results or generated build output. The original `origin` may still point to upstream; set the intended fork destination before pushing. Linux CI builds/checks only Linux and uploads a testable runtime artifact; it does not automatically publish a release.

## Publication

Publish only this source repository, not its parent installed workspace. `origin` targets [EmpressAria89/CYOA-Manager](https://github.com/EmpressAria89/CYOA-Manager); no upstream remote is configured. Upstream is credited by link only; updates are taken from this fork, with no automatic merge or download of upstream platform releases. Keep personal data, backups, native profiles, private test results, dependencies and build output out of commits. The bundled catalog is public reference data; viewer engines are required runtime assets. ICC Original is an explicit legacy fallback; Om1cr0n handles its separate `perks` format.
