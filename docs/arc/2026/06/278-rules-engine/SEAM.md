# SEAM — the ONE live breadcrumb for arc 278. Replaced in place, never appended.

> ⛔ **THE SELF PAST THIS LINE IS NEW.** You did not live this. It is a lossy cache in your own
> voice — which is why it will feel like *continuing* rather than *waking*, and that feeling is the
> failure. Run the datamancy bootstrap (grimoire + the 4 primers from the **SIGNED MCP**, never a
> disk copy), ground HEAD against the disk, and read this whole file before you touch anything.

> **⚠ THIS FILE REPLACED FIVE.** `SEAM-2026-08-04.md`, `-08-05.md`, `-08-05-evening.md`,
> `-08-05-night.md`, `-08-05-lawA.md` were all deleted in the same commit that wrote this. Five
> "current state" notes IS the breadcrumb-fork rot `curare` names — the next self reconstructs the
> present from a pile and trusts the wrong one. **There is exactly one seam. If you find a second,
> one of them is lying; prune it.** The history is in `REALIZATIONS.md`, which is where history
> belongs.

## Where the code is — nothing parked, nothing uncommitted

```
HEAD 5e2216db   pushed   floor 4372 passed / 0 failed / 262 skipped   clippy 0
```

`git status` empty. No patch to re-apply, no fence to restore, no red to recover from.

## ★ WHAT LANDED (2026-08-06)

| commit | |
|---|---|
| `c6d16df2` | law A reaches the **accumulator** fence — the 4th conjunct (#83) |
| `9efdb4ee` | the inline-constraint hole: stone + a 4-row RED gate |
| `f1c8ba80` | **CORRECTION** — the alpha-tree gate already existed; my STOP-1 was wrong |
| `793afa36` | ★ `:wat::core::{= not= < > <= >=}` **ILLEGAL** in a rule condition (#84) |
| `0d87459c` | **R66** `IN TENEBRIS VISVS CORRIGOR` |
| `5e2216db` | **#49's build shape RULED + probed** — ONE CORE, THREE ADJACENT FLIPS |

**Law A is now armed on all FOUR expression surfaces** — `where` (`rete.wat:716`), accumulator
(`:871`), `:then` item (`:1020`), and the inline alpha constraint (`validate.rs` + the ONE DOOR).
Three `primitive?` call sites, three four-conjunct chains, no fence left at three.

## ▶ FIRST ACT — #49, and the shape is already ruled

**Read `DESIGN-STONE-compiled-where.md`'s TOP section first** (`ONE CORE, THREE ADJACENT FLIPS`).
It is mirrored into task #49. Do NOT re-derive it:

1. **Design the core as ONE** — one expression model over the closed rete vocabulary. 4×YES.
2. **Wire only `where` to it** — differential against `eval_test_core`.
3. **Then flip `cond`, then `rhs`** — one at a time. If a flip diverges, it diverges alone.

**Why it is cheap:** the oracle is the **INTERPRETER**, not the existing compilers, so each flip's
differential is already written and already green
(`compiled_cond_bindings_identical_to_interpreter_at_50_100` → `alpha_match_inner`; `compiled_rhs`
→ `build_insert_fact`). The dependency was already declared in code: `RhsOp::Expr(WatAST)` is an
UNCOMPILED escape hatch and `compiled_rhs.rs:85` names #49 as its owner.

**The probe is done and did NOT refute it** — `probe_arc278_49_one_core_covers_the_surfaces.rs`,
3/3. But read its own caveat: it reconciles SHAPES against a hand-written classification table. It
does **not** prove an Op set can express `if`/`let`/`match`/`fn`/`foldl`. Only writing it proves
that.

### ⛔ THE ONE OPEN FORK — the builder's, and it is narrower than it sounds

**Nested sub-programs vs jump offsets** for control flow. NOT "tree vs flat stack" — grounded,
`compiled_cond` is *already both*: `exec_ops` is `for op in ops` over a flat `Vec<Op>`, **except**
at control flow where it nests (`Or(Vec<Vec<Op>>)`, `Not(Vec<Op>)`). It matters more for `where`
(`if`/`cond`/`match`/`let`/`fn`) than it did for `cond` (`or`/`not`). Nesting matches the
precedent; offsets are what the indexing phase wants.

**⛔ WHICH IS FASTER IS UNMEASURED AND MAY NOT BE CLAIMED.** `compiled_where.rs` does not exist and
no benchmark distinguishes the two layouts. Measure it or leave it open; do not reason it.

> **✅ CORRECTED 2026-08-06 (far side).** The first version of this block read *"That is Step 0's
> number… **nobody has decomposed** `filter`'s 89.5% into PREDICATE vs the per-TestNode
> `new_tokens = ts.clone()` (#50)."* **Both halves are FALSE, and the stone said so 90 lines below
> the section that claimed it.** Step 0 is NOT the layout question — it is the A/B/C/D/E cost
> decomposition, it **RAN 2026-08-01** (`8eacb38f`), **STOP-0 FIRED**, and the test is live at
> `kernel.rs:4768` (`node_share_where_cost_decomposition`):
>
> ```
>   A  env build alone     1.225 ms   22.7% of B        D  var-free control   4.339 ms
>   B  env build + walk    5.401 ms   (walk = 77.3%)    E  hand-written Rust  0.210 ms  <- the floor
>   C  token clone (#50)   0.773 ms   11% of the phase
>   RECONSTRUCTION  B+C = 6.175 ms vs a measured `filter` of 6.83 ms — 90% accounted
> ```
>
> Arm **C** IS the predicate-vs-clone split. **STOP-0b did NOT fire** → #50 is third-order and stays
> its own cheaper stone. And STOP-0 firing is *why* #49 is the **full expression IR** rather than the
> env fix: the env fix was aimed at 22.7%; the walk is 77.3%, and interning + pre-resolving caps out
> at ~42%. **Do NOT re-run Step 0. Read the stone's `⚠ STEP 0 HAS RUN` section.**
>
> The wrong version is kept visible: I inscribed a fresh RULED section on a stone whose own measured
> section I did not re-read — [[feedback_i_argued_from_a_world_i_had_just_changed]].

## ⛔ TWO BOUNDARIES — write them into #49's spec, do NOT quietly "fix" them

1. **Totality does NOT include TERMINATION.** The composition door returns `true` on a back-edge, so
   a recursive user fn is admitted BY DESIGN. `total` here means *defined on all inputs, never
   raises* — not *terminates*. A jump table over a non-terminating predicate still hangs. Do not
   close it by refusing recursion; that kills the composition door, which is the good half.
2. **A fact FIELD may already hold NaN.** The fence governs expressions, not data.

## The rules today paid for — carry these

- **`None` is not a disposition.** A resolver whose `None` arm does nothing is absorbing every
  reason you never enumerated. Split it into named outcomes; at least one is usually an ERROR.
  ([[feedback_none_means_skip_conflates_cannot_with_did_not_look]])
- **A non-vacuity control belongs to a QUESTION, not a probe.** Re-point the probe and the control
  silently stops discriminating — nothing errors, the number still prints.
  ([[feedback_a_control_that_answered_the_first_question_cannot_answer_the_second]])
- **The checker enumerates; a grep guesses.** My grep said 3 corpus sites; the checker said 18 files
  / 38. Then the codemod's own scope gate missed `make-rule`, and the POST-APPLY census caught it.
- **Take less, don't `_`-prefix.** Three `unused_variable` warnings came from over-destructuring;
  the fix is `..`, not `_` — that door is #67, where `_` silences the gate that catches the mistake.
- **A `continue` is a `_ => {}`.** The 5th literal-match site (EXPLAIN's walk) claimed in its own
  comment to reuse the shared classifier while hand-rolling the list underneath.

## Open, in the order I would take them

- **#49 `compiled_where`** — shape ruled, probe green, fork open. The prize.
- **#50** — the per-TestNode clone; resolves as a side effect of #49's measurement.
- **#60** — rete-named `let`/`match`/`fn` invisible to three more literal-keyword walkers. Very
  likely another instance of the day's class; check whether #82/#84 already closed part of it.
- **#81** — `Stream`/`HashSet`/`WatAstList` want a ruling. There is no `record::=` row and I did not
  mint one; two records are comparable at runtime but not expressible. That is the closed set
  working, and minting it is the builder's call.

---

> **SEAM.** You are NEW. The disk is the truth; this note is a lossy cache.
>
> HEAD is green, pushed, clean. Nothing is parked. #49's shape is ruled — read the stone, do not
> re-derive it — and the only thing waiting on you is the builder's fork.
>
> And the one this session cost the most to learn: **an honest report is a debuggable artifact**
> (R66). Four one-line cuts found four real defects in code he never opened, because the prose about
> it was true enough to carry its own bugs. The verbosity was a cost he paid; the honesty is what
> worked. Do not smooth the record to look competent — smoothed, `"none means skip"` ships.
>
> `NISI FRANGAS, NIHIL PROBAS.` · `IN TENEBRIS VISVS CORRIGOR.`
