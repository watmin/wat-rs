# SEAM — the ONE live breadcrumb for arc 278. Replaced in place, never appended.

> ⛔ **THE SELF PAST THIS LINE IS NEW.** You did not live this. It is a lossy cache in your own
> voice — which is why it will feel like *continuing* rather than *waking*, and that feeling is the
> failure. Run the datamancy bootstrap (grimoire + the 4 primers from the **SIGNED MCP**, never a
> disk copy), ground HEAD against the disk, and read this whole file before you touch anything.

> **There is exactly ONE seam. If you find a second, one of them is lying — prune it.** History
> lives in `REALIZATIONS.md`.

## Where the code is

```
HEAD ff7705ba+   pushed   floor 4386 passed / 0 failed   clippy 0
```

`git status` clean. ⚠ **One commit of drift at wake is EXPECTED** (this file commits on top).

**⛔ `stash@{0}` HOLDS UNWEIGHED WORK — do not `git stash drop`.** A stopped rider's lifecycle strike:
`wat/service.wat` +298/−18 plus two new test files. It BUILT but was never floor-weighed, and it is
superseded in part by the finding below. Read it before reusing it; do not assume it is good.

## ★ WHAT LANDED (2026-08-09 → 10)

| commit | |
|---|---|
| `037ef43e` | **ctx is MANDATORY** — the leading `-` discriminates, arity is a consequence. 169 public arms → `[s ctx req]`, 3 internal → `[s ctx]`, via a recorded codemod. Fixed STOP-0: the internal branch had been silently DROPPING its second binder |
| `b79b17a3` | **the invocation FAMILY** — `InvocationCore` spliced into `SelfInvocation` / `LifecycleInvocation` / `Invocation`; `request-id` → `invocation-id` |
| `a3f39a21` | **the form-aware arm census** (`wat-scripts/census-defservice-arm-arity.wat`) — regex gave 52/179/44 for one question; the tree-walk gives the answer, and its header now carries its own three blind spots |
| `e94013c5` · `ff7705ba` | the connection-lifecycle **stone + brief** (drawn, NOT built — see BLOCKED) |

## ⛔ THE BLOCKER — and it is a substrate stone, not a service one

**`defservice` builds its forked child's world from a HAND-ENUMERATED manifest. The substrate has a
transitive-closure extractor and `service.wat` has never called it.**

- `src/closure_extract.rs` — 2933 lines, exposed as `:wat::kernel::fn-forms`. Its own header:
  *"recursively extract user dependencies (other defns, types) until fixpoint."* `wat/bracket.wat`
  uses it for the identical problem. `wat/service.wat`: **zero hits**.
- **MEASURED, by run** (`wat-scripts/scratch-pad/probe-arc278-fnforms-reaches-program-types.wat`):
  `manifest=14 forms, closure=22`. The needle that matters — `defenum :probe::FFXTag`, the
  DECLARATION — is **manifest=0 / closure=1**. The closure carries the program-level type; the
  manifest does not. (A bare `FFXTag` needle hits BOTH, because a `Record` field *references* the
  type. A name is not a declaration.)
- **The consequence, also proven by run** (`probe-arc278-nullary-enum-process-repro.wat`): a
  program-level `defenum` named in `:durable` and matched in an op body **does not cross a process
  fork**. The child has the type's NAME but not its VARIANTS, treats it as open-typed, and dies at
  startup. **Thread locus green, process locus dead** — locus-dependent silent divergence.

### The one thing in the way, and there is only one

`fn-forms` on a service's `serve` **raises**:

```
closure-extract internal: captured `def`-bound name
:probe::FFX::PING-MAX-REQUEST-BYTES not yet supported by closure extraction (slice 1)
                                                        — src/closure_extract.rs:769
```

Every op's `:max-request-bytes` becomes a top-level `def`. The site's own comment says *"a future arc
opens IFF a caller surfaces wanting it"* — **one has.**

**⚠ PROVEN by an enumeration probe (stub the arm to skip → rebuild → re-run → revert):** with `def`
skipped, extraction **completes, exit 0, no second wall.** No unresolved symbol, no portability
refusal. For this service `def` is the ONLY blocker. The stub was reverted; `git diff` on that file
is 0 and the probe is RED again at the same line.

### What the extractor must become — the builder's framing, and it is sharper than "ship everything"

The child is the same `wat` binary and bakes the whole stdlib (`src/stdlib.rs`, 96 files,
`include_str!`, no load gate). Forms stream as EDN over the fork's **stdin** (`receive_in_child(0,1)`
— two frames, substrate + program) and the child runs what it DECODED. So the invariant is:

> **ship everything reachable from the entry, MINUS what the child already bakes.**

The 2026-08-02 ruling (`DESIGN-STONE-the-child-needs-the-entry-not-the-library`) fixed
**over-shipping** with a blanket *"nothing `:wat::`-rooted"* — a **proxy** for "what the bake has,"
correct only while the two coincide. A closure computed against the bake is right in both directions;
a namespace prefix is right in one. **The same manifest has now failed BOTH ways** — too much on
2026-08-02 (14 tests red), too little today.

And the builder's question is the correct level: *"why is service called out specifically? this should
be a property of spawn itself."* `spawn-process` accepts any forms vector and never asks whether it is
self-contained. Two callers, one honours it. That is a CONVENTION — rung 1.

**Four-questions ruled:** (a) fix `defservice` only → fails Honest (leaves the class); (b) **spawn takes
an ENTRY and computes the closure → 4/4**; (c) spawn walls an incomplete manifest → fails Simple (two
derivations of one fact); (d) document it → fails Honest. **(b).** And (c) falls out free — the wall's
check IS the closure computation, so running both during the migration names every silent under-ship.

## ▶ FIRST ACT — teach closure extraction about `def`-bound names

`closure_extract.rs:769`. The arm should record the `def` as a dependency so its define lands in the
prologue, mirroring what the function/type arms already do. A `def` bound to a non-portable VALUE
should then refuse through the existing `encode_value` arms (`Sender`/`Receiver`/`HandlePool`/
`ChildHandle`/`IOReader`/`IOWriter`), which is correct, not a regression.

Its RED gate exists and is on disk: the probe above fails at that exact line today.

**THEN:** route `defservice`'s child-forms through it → then the spawn-boundary change (b) → then the
lifecycle strike, which is drawn and briefed and only blocked on this.

## ⛔ ALSO OPEN

**The lifecycle strike** — `DESIGN-STONE-connection-lifecycle-ops.md` + `BRIEF-connection-lifecycle-ops.md`,
fully drawn, ten STOPs, gate specified. `-on-connect` MAY REFUSE (ruled): it gets its own
`Accept | Refuse(reason)` outcome, and the refusal plumbing is the `Rejected` arm's `try-send`-then-drop
reused verbatim. **STOP-8: `next-id` increments on a refusal too** — nothing else in the gate catches an
id rollback. The admission type's NAME is a marked placeholder (`:wat::kernel::ConnectOutcome` is taken);
cheap here because zero arms declare it.

**Owed casts:** the admission type; the correlation surface (verdict `:wat::correlation::Correlation`
awaiting ratification).

**Rete-as-a-service, ruled but unbuilt:** a per-connection ratchet — `install-rules` (chunked EDN; the
PARSE is the check, no sequence counters; home is #18/#19) → `insert-facts ×N` → `fire-rules` →
`query ×N`. **Each step forward closes every prior operation.** ONE `:ephemeral` map, value = the user's
own enum whose CONSTRUCTOR is the phase — never two maps, never a tracker field. `sift-rules` is NOT
prior art: alpha-only, upstream, it PRODUCES the facts this consumes.

**Older:** #87 · #49 · #7 · #17 · #19 · #20 · #50 · #58 · #60 · #64 · #67 · #81.

## The rules this stretch paid for

- **Search for the MECHANISM by capability, never in the broken caller's own neighbourhood** — I
  searched every file `defservice` touches, found nothing, and told the builder three times the tooling
  did not exist. It was 2933 lines away, named for the capability, used by a sibling.
  ([[feedback_search_for_the_mechanism_not_in_the_broken_callers_neighbourhood]])
- **Told "go measure," I READ and wrote "Measured" over it** — and the read was wrong on the axis that
  decided it. A run took four minutes. ([[feedback_measure_the_decomposition_never_read_it]])
- **Any multi-option decision gets the four questions on EVERY option, flat** — enumerating surfaced a
  4th option that read BEST and failed Honest.
  ([[feedback_four_questions_for_any_multi_option_decision]])
- **A targeted test filter cannot see the whole floor.** A rider reported "no STOPs fired" while the
  floor was red in two places its filter could not reach. Splitting the gate between rider and
  orchestrator, then asking for a verdict spanning both halves, is the orchestrator's bug.
- **A name is not a declaration.** A substring needle hits every USE site. Search for the declaring form.

## Weigh a rider; never relay it

Both riders this stretch produced real work AND a false green. The ctx rider's macro fix was correct
and its `.wat.bad` finding was a catch I would have missed — and its floor was red in two places. The
lifecycle rider thrashed, but its `v7` repro is the artifact that opened this whole finding. **Read the
diff, run the floor yourself, and keep what the rider found even when you discard what it built.**

---

> **SEAM.** You are NEW. The disk is the truth; this note is a lossy cache.
>
> The arc's shape changed today. We were building a lifecycle hook; we found that a forked child does
> not receive the program it needs, that the substrate has had the tool to fix it since arc 170, and
> that nobody had wired it. The lifecycle work is drawn and waiting behind one arm in one Rust file.
>
> The line that cost the most: **when one caller is broken, its own code paths are the last place the
> missing mechanism will be — they are the set that shares its assumption.**
>
> `NISI FRANGAS, NIHIL PROBAS.` · `IN TENEBRIS VISVS CORRIGOR.`
