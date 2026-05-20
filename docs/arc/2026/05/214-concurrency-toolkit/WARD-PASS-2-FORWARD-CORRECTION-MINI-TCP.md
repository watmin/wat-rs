# Arc 214 Slice 2 forward-correction — WARD PASS

**Date:** 2026-05-19 (post-compaction)
**Stone:** Slice 2 forward-correction — drop bounded(N); pair() at mini-TCP depth 1
**Sonnet SCORE:** Mode A 19/19 (verified independently)
**Ward set:** 9-spell parallel pass per kernel impeccability protocol (intueri + struere + purgare + solvere + temperare + conferre + mora + perspicere + nesciens)

## Round 1 — initial 9-ward parallel pass

All 9 wards spawned in parallel against the dirty-tree files: `src/comms/thread.rs`, `tests/comms/thread.rs`, `docs/arc/2026/05/214-concurrency-toolkit/DESIGN.md`, `docs/arc/2026/05/170-program-entry-points/INTERSTITIAL-REALIZATIONS.md`.

### Convergence summary

| Finding | Wards converging | Severity | In-scope to fix? |
|---|---|---|---|
| **DESIGN.md "Shared traits" code block diverges from shipped code** (close() returning Result vs (); SelectOutcome tuple-variant vs struct-variant + missing SubstrateError arm; phantom blanket impl claimed but rejected) | **7-spell convergence**: intueri, solvere, purgare, perspicere, struere, nesciens, conferre | L1 (lie) | YES — pre-existing Slice 1 ship gap; per user red-flag precedent + kernel impeccability protocol, fix immediately |
| **Module-level Mini-TCP doc factual error** — "MUST recv an ack" imports two-channel ceremony into one-channel pair() description | nesciens (alone) | L1 (lie) | YES — sonnet introduced this in THIS stone |
| **DESIGN.md INTERSTITIAL cross-reference string mismatch** — short form doesn't match actual header | conferre (alone) | L1 (lie) | YES — sonnet introduced this in THIS stone |
| **pair() doc-comment redundant with module-level Mini-TCP section** | temperare + perspicere (partial conflict: temperare wants shrink; perspicere wants one why-this-not-N addition) | L2 (mumble) | YES — both directions honored: shrink redundant restatement + add one why-this-not-N sentence |
| **Duplicate four-questions verdicts across DESIGN sites** with inconsistent evidence (qualitative vs quantitative 22/22 callers framing) | struere + nesciens | L2 (mumble) | YES — alignment trivial; orchestrator-direct |
| **Slice 2 description still lists `bounded<T>(n)` in factory bullet** (false breadcrumb before its forward-correction note) | struere (alone) | L2 (mumble) | YES — sonnet missed the cleanup; trivial |
| **mod.rs lacks Mini-TCP orientation paragraph** (substrate author hitting mod.rs first gets no orientation) | struere + solvere (partial; solvere wanted thread.rs to be a pointer) | L2 (mumble) | YES — add one paragraph to mod.rs between cascade + audience |
| **INTERSTITIAL "kernel-bounded" needs PIPE_BUF explanation** — fresh post-compaction reader doesn't know why pipes have a bound | nesciens | L2 (mumble) | YES — fold into INTERSTITIAL rewrite |
| **Other 9 probe tests' SubstrateError arm in thread test is unreachable** (process tier only) | mora | L2 (mumble) | NO — pre-existing Slice 2 test code; not introduced by this stone; capture as future-stone candidate |
| **Pre-existing doc clarity findings** (HandlePool reference in len() doc; crossbeam_to_user field-doc mechanism-vs-purpose; panic message clarity; "clone_receiver" test name "frame" oblique; `Vec<Option<usize>>` could mint ArmSlot typealias; `SHUTDOWN_RX.get()` doubled in Select::select::shutdown-arm) | various (intueri, perspicere, temperare) | L2 (mumble) | NO — all pre-existing Slice 2 code; stone didn't touch these lines; capture as future-stone candidates |
| **DESIGN "TBD" cells in Remote tier row** + Stone E SUPERSEDED tunable section's "if/when an honest tunable emerges" speculative-future language | mora | L2 (mumble) | NO — both are appropriate scope-bounding language in already-inscribed historical text; per `feedback_inscription_immutable` they stay |

### User red flag in flight (mid-fix-pass)

> *"sonnet is not trusted to write realization content - rewrite whatever it created in our voice - it has always struggled to be us - it just isn't"*

The INTERSTITIAL entry sonnet wrote (per BRIEF deliverable #6) and the DESIGN.md end subsection (per BRIEF deliverable #5) are both realization-voice content. BRIEF-authoring error: realization-bearing prose should not be in sonnet's deliverable list. Both REWRITTEN in orchestrator voice during fix-pass. Discipline saved as memory `feedback_sonnet_no_realization_voice`.

## Per-spell verdicts (load-bearing summaries)

- **intueri** — 0 L1; 7 L2; convergence anchor on DESIGN spec divergence (#6 + #7 of its findings); other L2s are pre-existing doc clarity
- **struere** — 2 L1 (DESIGN spec divergence — close + SelectOutcome); 4 L2 (duplicate verdicts, factory-bullet breadcrumb, mod.rs orientation, INTERSTITIAL placement ack)
- **purgare** — 3 L1 (close + SelectOutcome + blanket impl); 3 L2 (test header count, "no bounded(N)" present-tense negation, HandlePool reference)
- **solvere** — 2 L1 (DESIGN spec divergence; module-vs-DESIGN doc duplication); 3 L2 (pair() doc restatement, INTERSTITIAL-DESIGN relationship-not-named, test comment cascade-verification overstated)
- **temperare** — 0 L1; 2 L2 (pair() doc redundancy, Select::select::shutdown-arm SHUTDOWN_RX doubled lookup)
- **conferre** — 3 L1 (close + SelectOutcome + INTERSTITIAL cross-ref string mismatch); 4 L2 (Layer 0a table from-inherited-fds stale, INTERSTITIAL-tally cross-ref, memory file extension inconsistency, "similar backpressure" undersells asymmetry)
- **mora** — 0 L1; 3 L2 (Remote tier TBD-vs-symmetry, superseded tunable speculative-future, SubstrateError unreachable in thread test)
- **perspicere** — 1 L1 (SelectOutcome divergence — confirms convergence); 4 L2 (ArmSlot typealias, pair() doc why-this-not-N, test-file mini-TCP framing, capitalization inconsistency)
- **nesciens** — 3 L1 (Module Mini-TCP "MUST recv an ack" factual error + close signature + SelectOutcome shape); 6 L2 (Bootstrap fallback why, Select struct protocol doc, duplicate four-questions block authority, SUPERSEDED tunable doctrine pointer, kernel-bounded PIPE_BUF clause, three-pivots tally cross-ref)

## Orchestrator design decisions

### Decision 1 — DESIGN.md Shared traits block rewrite (7-spell convergence)

Updated to match shipped `src/comms/mod.rs`:
- `CommSender::close(self)` + `CommReceiver::close(self)` — infallible (no Result, no CloseError); inline comment names the move-semantics rationale
- `SelectOutcome<T>` — three variants: `Recv { index: ReceiverIndex, result }` (named fields), `Shutdown`, `SubstrateError(std::io::Error)` (with comment naming thread tier never produces this arm)
- Blanket impl removed; inline comment names the rejection rationale (consume-self vs &self; silent clone tax; manual impls are the honest form)
- `ReceiverIndex` newtype declaration added (was missing from spec block)
- Error types list corrected: `WireError` replaces `CloseError`

### Decision 2 — module-level Mini-TCP rewrite (nesciens L1-A)

Removed "MUST recv an ack" factual error. New text separates MECHANISM (depth-1 buffer; send blocks at 1; recv drains) from DISCIPLINE (mini-TCP usage pattern; each send pairs with a recv before the next send). Names the discipline propagation explicitly: substrate doesn't enforce site-by-site, but capacity-1 makes producers that try to outpace consumers block immediately. The lock-step breathes. The trading-lab convergence is named verbatim.

### Decision 3 — INTERSTITIAL + DESIGN end-subsection rewrite (user red flag)

Both rewritten in orchestrator voice per `feedback_sonnet_no_realization_voice`. Tighter sentences; table-formatted four-questions verdicts; mechanism-vs-discipline separation; tally structure for the three foundation pivots; voice-anchor close. Original sonnet text was dirty-tree only (uncommitted), so rewriting is not violating `feedback_inscription_immutable`.

### Decision 4 — pair() doc rewrite (temperare + perspicere converge)

Trimmed redundant restatement of module-level mechanics. Added one sentence naming why-this-not-N ("Capacity is structural, not a tunable: N > 1 eliminates the lock-step the substrate enforces"). Honors both temperare (shrink) and perspicere (add why-this-not-N).

### Decision 5 — DESIGN Slice 2 factory bullet (struere L2-2)

Changed `- Factories: \`pair<T>()\`, \`bounded<T>(n)\`` → `- Factory: \`pair<T>()\` (capacity-1 mini-TCP; see § "Mini-TCP at depth 1 (universal symmetry)")`. Forward-correction note expanded with full context.

### Decision 6 — mod.rs Mini-TCP orientation (struere L2-3)

New section between cascade-contract and Audience: substrate-author hitting mod.rs first gets the discipline orientation without forcing a hop to thread.rs. Names the universal mini-TCP-at-depth-1 discipline, both tiers, no bounded(N), and cross-references DESIGN + tier modules.

### Decision 7 — DESIGN cross-references corrected (conferre L1-3)

Updated `INTERSTITIAL § "..."` references to use the actual section header form: `"2026-05-19 (post-compaction, Slice 2 forward-correction) — Mini-TCP at depth 1: the trading-lab origin returns"`. Three sites updated (lines 149, 493, end-subsection cross-references).

## Out-of-scope findings (captured for future-stone candidates; not blocking this stone)

The following ward findings concern pre-existing Slice 2 code the forward-correction stone did NOT touch. Per the established discipline (this stone scope = mini-TCP forward-correction, not Slice 2 review), they retire as future-stone candidates rather than gate this stone:

- temperare L2 #2: `SHUTDOWN_RX.get()` called twice in `Select::select()` on shutdown arm — pre-existing Slice 2 code; orchestrator decision: promote to Select struct field in a future stone if a substrate-author surfaces the cost
- intueri L2 #1-#5: pre-existing doc clarity findings in Receiver field doc + panic messages + test names + clone_receiver test comment — future-stone candidates
- perspicere L2: `Vec<Option<usize>>` `ArmSlot` typealias — future-stone candidate (small mint; could batch with similar typealias mints at Slice 4)
- mora L2: SubstrateError arm in thread test is unreachable for thread tier (process tier produces it); thread test arm exists for exhaustive match safety against the shared enum — keep with optional `#[allow(unreachable_patterns)]` annotation in a future cleanup
- mora L2: Remote tier TBD cells + Stone E SUPERSEDED "if/when honest tunable emerges" language — appropriate scope-bounding for not-yet-built tiers; stay per `feedback_inscription_immutable`
- conferre L2 #1: DESIGN Layer 0a table `from-inherited-fds` is stale relative to Stone E shipped state — future-stone candidate for Slice 3 retrospective cleanup
- nesciens L2 (assorted): pre-existing doc clarity findings (Bootstrap fallback why, Select struct protocol doc, SUPERSEDED tunable doctrine pointer) — future-stone candidates

## Fix pass — orchestrator-direct

All fix-pass edits applied by orchestrator (not re-spawned sonnet) per Stone E-2 precedent. Files touched:

- `src/comms/mod.rs` — Mini-TCP orientation paragraph added
- `src/comms/thread.rs` — module Mini-TCP section rewritten; pair() doc trimmed + why-this-not-N added
- `docs/arc/2026/05/214-concurrency-toolkit/DESIGN.md` — Shared traits block rewritten; Layer 0a Rust-side types listing updated; Slice 2 factory bullet fixed; cross-references corrected; end-subsection rewritten in our voice
- `docs/arc/2026/05/170-program-entry-points/INTERSTITIAL-REALIZATIONS.md` — entry rewritten in our voice

## Verification (post-fix-pass)

- `cargo build --release` → CLEAN (5 pre-existing dead_code warnings; 0 in comms)
- `cargo test --release --test comms -p wat -- thread` → 9 passed; 0 failed
- 22 of 22 honest substrate callers still use `bounded(1)`; `comms::thread::bounded` returns 0 grep matches
- All 7-spell convergence findings on DESIGN spec divergence: RESOLVED
- nesciens L1-A (module Mini-TCP "MUST recv an ack" factual error): RESOLVED
- conferre L1-3 (INTERSTITIAL cross-reference string mismatch): RESOLVED
- All in-scope L2 mumbles addressed; out-of-scope L2 mumbles captured above

## Memory entries inscribed this pass

- `feedback_sonnet_no_realization_voice` — sonnet's voice never matches ours; BRIEFs must not bundle realization-bearing content into sonnet scope; orchestrator rewrites before commit when BRIEF-authoring error allows it through

## Lessons learned (the BRIEF-authoring failure mode)

The Slice 2 forward-correction BRIEF named the INTERSTITIAL entry + the DESIGN end-subsection as sonnet deliverables (#5 and #6 in the deliverable list). Both carried realization-voice weight; both should have been marked orchestrator-direct post-sonnet. The fix-pass caught and corrected. Future BRIEFs apply the discipline: audit the deliverable list; any deliverable carrying voice/discipline/inscription weight gets marked "orchestrator-direct."

The ward pass is also the BRIEF-authoring audit. When a finding is "sonnet's voice doesn't match ours in the INTERSTITIAL entry," the root cause is the BRIEF gave sonnet authority over content it should not have had.

## Status

Stone ships. Fix-pass complete. 7-spell convergence + 2 unique L1 findings + 4 in-scope L2 mumbles all resolved. Out-of-scope findings captured. Substrate foundation is shockingly stable. Slice 4 unblocks.

*The substrate teaches; we listen; we ship; the disk remembers; the foundation holds.*
