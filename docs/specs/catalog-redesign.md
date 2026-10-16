# Catalog discovery and safe downloads

## Problem statement

The catalog omits useful source fields, cannot reliably search offline, renders every result at once, and chooses download behavior from broad engine labels. An archive fallback can silently substitute a different edition. Users need reliable discovery and clear choices without risking library/build identity.

## Solution

One normalized catalog sourced primarily from Beta Index, augmented conservatively by Project.json, broken-link recovery and optional unreviewed discoveries. Persist validated source snapshots, expose freshness, preserve the last good snapshot during failure, and support explicit refresh. Search useful source fields, canonical author aliases and quoted/excluded/field-qualified terms. Keep the UI compact and render bounded result pages. Inspect actual source content for download compatibility; provide project/native-website/online choices with evidence. Updates show actual differences before applying, preserve the prior edition and reject changed or mismatched preview inputs. Archive recovery never silently replaces a current card.

## User stories

1. Search the last successful catalog offline and see when it was fetched.
2. Refresh explicitly without losing good data during network failure.
3. Find a CYOA by title, alias, author, fandom, description or normalized tag.
4. Use quoted phrases, exclusions and author/fandom/tag qualifiers.
5. Filter library membership, safety, catalog status and source category.
6. Distinguish catalog author/importer/status from personal author/modder/completion.
7. See alternate playable links, direct project files and archived recovery links.
8. Keep main entries missing a primary playable URL discoverable with availability labels.
9. Exclude unreviewed discoveries by default and inspect them explicitly.
10. Avoid duplicate listings from the two Infaera website interfaces.
11. Download according to actual content rather than a broad Engine label.
12. Keep Remix/native-engine projects intact through original-viewer or online fallback.
13. Inspect update differences before applying; keep the prior edition and builds.
14. Recover archived content as a separate card, labelled with provenance and uncertain age.
15. Queue downloads serially or with bounded concurrency; stop remaining queued items and retry failures without repeating successful imports.
16. Preserve selections, filters and readable controls at narrow widths and large fonts.

## Implementation decisions

- Public sources need no account/API key. Cache data belongs outside published source.
- Primary source is gid 843963812; supplementary gids 208890884, 574767915 and 1285606730 have different roles. Introduction, KeyMap and Other Lists are references, not playable-entry feeds.
- Stored snapshots include source identity, source rows, fetched timestamps and stale/error state. Partial failures retain each source's last good data.
- Do not deduplicate by title/domain, strip meaningful query/fragment routes, infer modders from importer, or infer edition freshness from catalog dates.
- Keep independent source/UI/download changes parallel behind explicit interfaces, then integrate and verify together.
- Git remains detached, local tickets only, no automatic commit/push. Preserve the pre-change source export as the review baseline.

## Testing decisions

Use the existing external behavior boundaries: catalog snapshot loading with fixture fetches and isolated cache directories; source-to-entry/search outputs; browser-visible filtering/download/preview actions with mocked Tauri; resolver/update operations on temporary valid project fixtures. No tests write the live library. Validate source export, Linux package assets and full Rust/browser suites. Native tests are required when reader/WebKit behavior changes; this work changes catalog/manager behavior, not the reader engine.

## Out of scope

General-purpose script conversion, crawling all linked catalogs, automatic downloads during searching, cloud accounts, Windows/macOS releases, mass live-library migrations, guesses about authoritative modder credits, guaranteed offline operation for arbitrary dynamic websites.

## Acceptance

All implemented paths must remain resumable from local tickets and the organized baseline/checkpoint. Unavailable/uncertain capabilities must be labelled and handled explicitly rather than represented as verified.
