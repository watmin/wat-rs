# SCORE — B1, weighed against the orchestrator's own re-run

> Every row below was re-run here, at `7319c1ea4`. The rider's report is cited only where it
> reports something I cannot reconstruct.

## The scorecard, re-run

| # | expected | actual |
|---|---|---|
| 1 | control green before | ✅ green (it is row 1082 of the floor, still green after) |
| 2 | both probes RED before | ✅ driven by me at `85043bbab`: `:414` and `:442`, `table grew 0 -> 1` each |
| 3 | both probes GREEN after | ✅ |
| 4 | `with-overlay` inherits | ✅ **and I drove the mutation myself** — see below |
| 5 | control still green | ✅ |
| 6 | no release call in the wat form | ✅ `grep release-session wat/rete/syntax.wat` → 2 hits, **both prose**, zero calls |
| 7 | exactly one adopt site | ✅ one call (`:326`), one comment (`:291`) |
| 8 | guard adopts, never acquires | ✅ **by reading**: `ArmLease::adopt` is a bare struct literal, no `rete_arm_intern` |
| 9 | blast radius | ✅ exactly the six files DESIGN names |
| 10 | rete surface green | ✅ (subsumed by the floor) |
| 11 | floor 5,191 | ✅ `Summary [ 370.662s] 5191 tests run: 5191 passed, 21 skipped`, **zero FAIL rows** |
| 12 | clippy silent | ✅ rc=0, zero warnings |

**Row 4, driven here, not credited from the report.** Reverting only `wat/rete/syntax.wat` to the
pre-fix form and rebuilding (`wat/**` is `include_str!`-embedded, so the rebuild is mandatory):

```
Summary  4 tests run: 1 passed, 3 failed
  PASS  scoped_work_with_network_releases_the_lease_it_takes        ← the normal-return control
  FAIL  scoped_work_with_overlay_releases_the_lease_when_the_body_raises
  FAIL  scoped_work_with_network_releases_the_lease_when_the_body_raises
  FAIL  scoped_work_with_network_releases_the_lease_when_the_body_panics
```

Three unwind probes red, the control green. **The `with-overlay` row is a gate that can fail**,
which is the only thing that makes it worth having.

## What the rider returned that I could not have written

**Two holes the DESIGN did not anticipate, both found by DRIVING rather than reading.**

1. **`try_with` prevents an ABORT, not a panic.** The mutation (revert to `.with`) changed nothing
   observable — 438/438 still green — which is a **coverage** finding: nothing in the suite lets a
   guard reach thread teardown. The rider did not stop at "documented behaviour"; it reproduced the
   shape standalone, and `.with()` during TLS destruction gives
   `fatal runtime error: thread local panicked on drop, aborting` — SIGABRT, exit 134. **Not
   catchable.** The defensive change is worth strictly more than the word "defensive" implied.

2. **The purity wall would have called a live resource handle PURE.** `make_rust_opaque` takes a
   `&'static str` with no registration, so `is_registered_rust_opaque` cannot see a hand-minted
   path, and `is_pure_type`'s bottom arm is `None => true` ("unknown path ⇒ a formal type parameter
   ⇒ portable by convention"). `:rust::rete::ArmLease` would have been admissible as a `Record`
   field and onto the wire. **Driven with a positive control** — `:rust::rete::NotAThing`, an
   unregistered sibling, comes back PURE — so the arm is real and not a reading. I confirmed the
   `None => true` arm on the disk. Closed with one impure-path row; the general shape (hand-minted
   opaques invisible to the self-enrolment) stays open and is named as `BRIEF-opaque-purity-self-
   enrolls` STOP-1 territory.

## ⛔ Where MY brief was thin — six, and two of them matter

- **A. I prescribed the step the gate forbids.** DESIGN and BRIEF both called `purity.rs` "the op
  list — it already lists `release-session`". Line 2380 is inside `KNOWN_UNREVIEWED`, a **ratchet**
  whose own doc reads *"Never add a line to make a red gate green … CLASSIFY the verb in
  `intrinsic_meta`, or give its namespace a disposition in `RULES`."* **Verified on the disk.** And
  my implied alternative was worse: for a `:wat::rete::` head, `intrinsic_meta` classifies via
  `rete_op_for` → `RETE_OPS`, whose **ORDER is hashed into the export ABI** (`abi_of`,
  `export.rs:479-497` — verified). Classifying there would have silently moved the ABI hash, far
  outside the stated blast radius. The park is legitimate for a reason neither of my documents
  gave: the `:wat::rete::` namespace **already** carries `Disp::Unreviewed` in `RULES` (`:2146`),
  so the verb inherits its two siblings' open question rather than laundering a fresh red.
- **B. "No new `Value` variant, use `make_rust_opaque`" was right and had a hole I did not mention**
  — finding 2 above. My blast radius listed `check.rs` as "(TypeScheme)"; it carries two
  load-bearing edits.
- **C. Trap 2 did not fire.** The checker accepted the unused `lease` binding; the `(do lease
  result)` accommodation was never needed. Costless, but the prediction was wrong.
- **D. Trap 5's grep measures the wrong population.** `grep -rn … wat/` is the stdlib only; it
  would still say 1 if `wat-scripts/` grew a caller. The real fence is `#[restricted_to]`.
- **E. ★ A COUNT IN A SCORECARD IS A CEILING ON THE EXECUTOR.** Row 11 pinned the floor at
  5,188 + **exactly three** new tests. So writing the second `with-overlay` arm (host-panic) or a
  `restricted_to`-fence probe **would have falsified my own row 11 before I ran it.** The rider
  wrote exactly three and named the two arms that fell outside. This is the sharpest correction of
  the nine-plus riders so far: *a scorecard that fixes a test COUNT does not merely predict the
  work, it bounds it* — and it bounds it in the direction of less coverage, silently, while
  looking like rigour.
- **F.** Two prose claims inside `syntax.wat` had gone false with the old shape (*"releases it
  after the body runs"*; `with-overlay`'s *"one release site, not two"* — now zero). Corrected in
  place. A doc that describes the **shape** instead of the **guarantee** is exactly how the false
  `with-open-file` parity claim survived this long.

## Arms not driven, named

- **host-panic unwind through `with-overlay`** — reachable, not driven; same inner-closure layer as
  the wat-error arm above it, and row E is why it was not written.
- **the `#[restricted_to]` fence** — reachable, not driven. STOP-3 did not fire; the attribute
  applied cleanly.
- **`eval_adopt_session_lease`'s two `TypeMismatch` arms + the `ArityMismatch` arm** — not reachable
  from wat: one call site, one argument, always `compile-all`'s `Compiled` session, and the mouth
  admits only `:wat::rete::` callers.
