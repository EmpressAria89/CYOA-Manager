# Next reliability and efficiency slices

Status: planned, not implemented. Preserve the current glossary, ADR-0001 and all later user choices. This plan adds to the roadmap; it does not authorize a broad rewrite or live-library migration.

## Problem and solution

The current project has repeatable unit/browser checks, but the complete reviewed-update commit lacks direct storage integration proof. Catalog inspection also downloads content again for the chosen action. Address the proof gap first, then measure and remove repeated work while preserving edition/build identity and explicit recovery decisions.

## User stories

1. As a player, I want an update test to prove my previous Edition and builds survive a committed replacement.
2. As a player, I want failed or stale updates to leave the existing Library card usable and offer a fresh comparison.
3. As a player, I want cancellation and crashes to remove only uncommitted staging.
4. As a player, I want inspection and the selected action to use the inspected content without redundant network transfer.
5. As a maintainer, I want bounded disk/memory use and measured improvements rather than an assumed leak fix.
6. As a maintainer, I want the same completion/warning rules in dialog and queue without duplicating transaction knowledge.
7. As a maintainer, I want each slice independently testable and recoverable before the next begins.

## Implementation decisions

Prefer the existing catalog command/storage interfaces and isolated application data. No database/schema/build-code format changes are planned. If the native command cannot be exercised in a headless test, first use an isolated native harness; introduce a smaller commit seam only if it is needed and both production and tests traverse it. Keep reader-session pinning in the native proof rather than silently replacing it with a no-op.

Use a bounded, private, single-consumer disk lifetime for retained inspected content. Bind it to selected Source, digest, owner and expiry; consume or discard it on action/cancel/source change. Existing script/source/size/relatedness/current-edition guards continue to apply. Budget disk capacity explicitly before implementation. No artwork-heavy global memory cache.

Edition scan consolidation is conditional on elapsed-time/bytes-read/peak-RSS evidence. Preserve exact semantic comparison and image bytes; fewer lines are not the acceptance measure. New-card completion sharing must preserve successful commit when optional metadata enrichment fails.

## Testing decisions

Observe the highest practical interface: command request through isolated files/SQLite and back to the resulting Library card, not privileged calls into a test-only state. Use loopback HTTP, temporary real files and a minimal valid export. Browser tests remain useful for dialog/queue presentation but are insufficient transaction proof. Keep captured build code, UUID and fingerprints before/after; restore the preserved Edition into separate staging and verify its assets.

## Slices and blocking edges

| Ticket | Delivers | Blocked by | Acceptance |
|---|---|---|---|
| 01: Update commit preservation proof | Actual reviewed update on an isolated Library card | None | UUID/notes/aliases/build files unchanged; prior Edition starred/restorable with correct asset hashes; current content updated; stale/tampered candidate and injected commit failure retain old usable data; independent test storage and no live profile access |
| 02: Reuse inspected content | One fetch through inspection and confirmed import/preview | 01 | Loopback request counter proves reuse; changed Source/expiry/tamper rejected; single-consumer ownership; cancel/quit/crash cleanup and disk budget tested; explicit Recovery candidate selection unchanged |
| 03: Measured edition inspection | Profiling report, followed by consolidation only if justified | None | Large embedded and asset-backed exports; repeated baseline/candidate elapsed/bytes/RSS; identical diff categories, original hashes, script detection, IDs and build codes; no regression on small files |
| 04: Shared new-card completion | One verified success/enrichment-warning rule in dialog and queue | 01 | Exactly one card even if optional metadata fails; no duplicate retry; blank-only metadata, alias/importer/modder rules, refresh, warnings, focus and stop/retry remain correct |

Local tracker copies use these numbers and blockers. Tickets 02 and 04 can run concurrently after 01; ticket 03 is independent. A failed experiment ends with its measurements and retained baseline rather than forcing a refactor.

## Out of scope and delivery

No converter, cloud sync, cross-platform release, live mass update, Git attachment or upload. Each future slice closes only after focused failure-first proof, adjacent regressions, Standards/Spec review, source/export gates and a resumable delivery receipt. Native checks are required for changed reader/session behavior. Keep dependency migration, crash checkpoints and other product proposals in the main roadmap.
