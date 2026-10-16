# Skill-based assurance: 2026-10-16

This follow-up checks the earlier full audit against the current project. Later reader, archive, scripted-viewer compatibility and catalog decisions remain authoritative. It does not reset the project to the older audit snapshot. Git stays detached; publication is separate.

## Method and evidence

Ask Matt routed this work to the existing setup, independent Standards and Spec reviews, architecture survey, and retrospective. The local tracker, glossary and ADR already existed and were retained. Reviews use a frozen pre-catalog source snapshot and the current exported source in a temporary checkout, never Git attached to the application folder. The original full audit predates that diff; its requirements are checked separately against the current documentation, code and retained evidence.

At entry, all 178 exported files exactly matched the completed catalog delivery manifest, and the installed executable matched its receipt. That ties the retained 57-passing-Rust-test/1-optional-ignored result and 15 browser suites to the starting tree. This run changes manager UI, browser checks, documentation and the source gate; Rust source and dependency locks remain unchanged. Final validation/install/export outcomes are recorded in the installed workspace's backup index.

## Requirement coverage

| Requirement | Current implementation or evidence | Limit |
|---|---|---|
| Full handoff and trajectory | [Handoff](../HANDOFF.md), [roadmap](roadmap.md), [next-slice plan](specs/assurance-followups.md) | Proposed features remain unimplemented |
| Clear source/runtime/test/backup layout | Source/privacy/layout gate, allowlisted exporter, backup index | Managed environment directories remain owned by their tools |
| Cleanup and older backups | Prior deletion journal records 178 MiB removed; this follow-up removes its generated staging after readback | Keep unique data snapshots, Git history and verified rollback; age alone is insufficient |
| Resumable work | Phase state, pre-change source/runtime, manifests, review reports and logs | Same-disk recovery cannot survive disk failure |
| Easy future upload | [Publication guide](github-publication.md), detached source exporter, separate checkout | No push or remote CI claim in this task |
| Linux only, windowed startup | Linux workflow and configuration gates | Native target is Linux x86_64; other distributions need verification |
| Efficient manager lifecycle | Listener/cache/refresh/preference regressions and prior native evidence | No claim every path is optimal or every crash eliminated |
| Reader memory and authored layout | Session resources, viewport image release, ratio/presentation tests | Chromium mocks do not replace WebKit long-session measurements |
| Typography and pinned sections | Author default, global/per-project section settings and reset checks | Preserve existing chosen fonts; author defaults are optional |
| Cost/gain, dock, cheats, navigation | Existing reader suites and saved synthetic screenshots | Presentation changes do not modify game rules |
| Hidden choices and child sections | Conservative dependency hiding, manual hiding, retained-selection checks | Ambiguous rules remain authored; reveal never opens every tab |
| Metadata and additive aliases | Metadata/alias tests, original credits retained, blank-only source enrichment | Missing source facts stay unknown |
| Archives and independent restores | Storage/restore/retention tests, named separate cards and protected updates | Full reviewed-update commit path still needs direct integration proof |
| Catalog discovery/downloads | [Catalog guide](catalog.md), source/cache/resolver/queue tests | Scripts are not converted; recovery candidates remain separate |
| Live data integrity | This run: SQLite quick_check, 95 cards with existing files; 30 archives/11 starred and 33 member hashes passed | Snapshot counts differ legitimately from historical 97/29; no migration performed |
| Documentation accuracy | Local links, syntax, version/config/export checks; fresh-checkout build order corrected | Dependency advisory results in the earlier audit are dated, not newly rescanned |
| UI consistency and usability | Named controls, focus, fonts, palettes, states and screenshot review | Complete accessibility certification is not claimed |

## Review outcomes

**Standards:** missing accessible names on seven catalog filters and a failed-Apply preview retry defect were corrected with failure-first browser regressions. No inspected identity/privacy/Linux constraint violation was found. Possible duplicate URL validation and string-based mode values are judgement calls, not demonstrated defects.

**Spec:** clean-checkout test instructions incorrectly ran custom-protocol Rust tests before building frontendDist. The source gate now rejects that ordering, and both guides build first. A separate test gap remains: browser Apply is mocked, and Rust guard/resolver tests do not execute the complete reviewed-update transaction. The next-slice plan makes this the first reliability item; it must not be described as proven by existing mocks.

**Architecture:** retain inspected content through a confirmed action to avoid downloading the same JSON twice; profile repeated edition scans before consolidating them; share duplicated new-card completion handling only when its next behavior change justifies it. These are proposals, not evidence of a leak. Each has implementation steps and acceptance criteria in the next-slice plan and private visual report.

## Retrospective and guardrails

- Mechanical failure: clean-checkout command ordering drifted from CI. Keep the deterministic documentation-order assertion in the existing source gate; it runs in Linux CI. Avoid duplicating this rule in always-loaded agent instructions.
- Evidence failure: aggregate test counts hide which storage transaction was exercised. Record exact covered interfaces and mock/native limits, with the update-commit integration ticket open.
- Recovery: retain immutable source/runtime digests and atomic phase checkpoints before mutations. Close local tickets when their recorded gates actually pass; completed prose must agree with their status/checklists.
- Tool economy: frozen manifest matching justifies reusing unchanged Rust proof; rerun the build/browser gates for changed UI. Broad refactors require a measured benefit and an independent before/after comparison.
- Maintain one canonical handoff, roadmap and glossary. This assurance record supplements the historical audit instead of rewriting its evidence as a new run.
