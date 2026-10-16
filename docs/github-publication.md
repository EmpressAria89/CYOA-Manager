# Prepare a one-time GitHub update

Destination: [EmpressAria89/CYOA-Manager](https://github.com/EmpressAria89/CYOA-Manager). Publish the source tree, not the installed workspace. Preparation never pushes, fetches upstream or attaches Git to the working source.

## Validate and export

From the folder containing `package.json`:

```bash
python3 scripts/prepare-github.py --check
python3 tests/check-project-layout.py
python3 scripts/prepare-github.py
```

The export and adjacent SHA-256 manifest are written under ignored `release/`. The exporter uses an explicit source allowlist, excludes generated/personal data, rejects symlinks and common credential/private-path patterns, then reads back every archive member. It is a publication gate, not a proof that every possible secret format has been detected. Review the manifest and resulting diff.

The runtime archive is separate: `python3 scripts/package-linux.py`. Do not upload the source archive as a Linux executable package.

## Review in a separate checkout

This approach leaves the working source detached throughout. Replace the two absolute paths below with your exported archive and a new empty review directory:

```bash
git clone --no-checkout --branch main https://github.com/EmpressAria89/CYOA-Manager.git /absolute/path/to/new-publish-review
tar -xzf /absolute/path/to/cyoa-manager-source.tar.gz -C /absolute/path/to/new-publish-review
cd /absolute/path/to/new-publish-review
git add --all
git diff --cached --stat
git diff --cached --check
git diff --cached
```

`--no-checkout` prevents old branch files from being mixed into the exported working tree. The clone accesses this fork only; it does not fetch upstream releases or download Windows/macOS packages. Git history may still contain upstream's historical source files. The new working tree and CI remain Linux-only.

After reviewing and running checks in that checkout, commit and push explicitly:

```bash
git commit -m "Audit Linux manager reliability, UI and maintenance workflow"
git push origin main
```

Use a current appropriate commit message for later work. If the push is rejected because the branch moved, preserve the review checkout and reconcile the changes; do not force-push. Remove the separate review checkout only after verifying the remote commit and preserving any remaining local changes. The original source/runtime folders remain detached and runnable.

## Preserved local history

The installed workspace's `backups/README.md` points to preserved external Git metadata and its `REATTACH-GIT.md`. It remains an alternative for recovering local history; there is no need to move it into `app/.git` merely to produce an export. Its recorded publication commit is historical and does not imply that newer local edits are uploaded.

The local catalog redesign uses Markdown tickets under ignored `.scratch/`; its canonical specification, glossary, ADR and agent configuration are included in source exports. Review runtime cache files and local task/evidence folders only outside the export. Source preparation does not attach Git.
