# CURRENT STATE — annihilate interpretation in wat-rete

> **Locked 2026-08-17 so a compaction cannot drop it.** This is the live
> breadcrumb. Read this whole file before touching `src/rete/` or
> `wat/rete.wat`. If a stone below disagrees with a dated ruling here,
> **this file wins** and the stone is stale.

**CURRENT STAMP 2026-09-01 (twenty-fifth — 25 STRIKES; ⭐⭐⭐ CLASSES A, E, F1 CLOSED + C2/C7. C3-C6, D2/D4-D7, F2 remain; F3 is LEADS ONLY). Supersedes every earlier stamp and every dated block below.**

**THE FRESHNESS PROBE — run it, it is two commands:**

```
git log --oneline 00ca6b0eb..HEAD      # every commit since the last SUBSTANTIVE one
git diff --stat 00ca6b0eb..HEAD        # what they touched
```

**PASS:** every commit in that range is prefixed `curare:` and touches `docs/` plus, at most,
comment-only edits. **STALE:** anything else in the range — a `rete:`/`fix:`/`feat:` commit, or a
substantive `src/` diff. Then trust the log and the source over every line below, and re-read
before you move.

> ⚠ This is the probe's THIRD wording today and the first two were both wrong, which is the
> lesson worth more than the probe: version 1 promised the gap would be `docs/` ONLY and its own
> commit touched a `src/` comment; version 2 tried to name its own commit hash, which cannot
> exist until the commit does; version 3 said "expect ONE commit" and was invalidated by the next
> curare commit. **A probe pinned to a COUNT rots on every subsequent write.** This one pins to a
> KIND — "everything since the last substantive commit is curare" — which stays true no matter
> how many wrap-up commits land on top, and still screams the moment real work lands unread.

**⛔⛔ START HERE. THE INITIATIVE IS: MATURE wat-rete INTO AN EXEMPLAR the rest of wat matures
against.** Correctness is done; the exemplar work is not.

**✅ WHAT IS FINISHED** — every vigilia row (audited, not inherited), the outcome wall (five verbs
total, no ceiling reaches wat as a raise, a lint keeps it so), the termination verifier reading the
`where` fence, and BOTH of `partire`'s named cuts:

| | |
|---|---|
| `expr_ir.rs` 2_458 | → `expr_ir/mod.rs` 1_046 + `expr_ir/eval.rs` 1_413 |
| `validate.rs` 2_452 | → `mod.rs` 1_494 + `typing.rs` 549 + `error.rs` 448 |

**⛔ THE METRICS ARE EXHAUSTED — AND THAT IS THE FINDING.** Every mechanically checkable axis is
green or has been retired with reasons. **Re-derive, do not quote:**
`scripts/doc-coverage.sh <dir> --exclude /tests/` (`--list` for file:line).

| axis | `src/rete` | `src/process` | `src/channel` | verdict |
|---|---:|---:|---:|---|
| undocumented fns ≥15 ln | **0** (was 111) | 4 (12%) | 0 | ✅ done |
| tests that cannot fail | **0** (was 26) | — | — | ✅ done |
| nesting ≥8, NORMALISED | **1.1%** (9/817) | 0/85 | **14%** (1/7) | ✅ ahead of channel |
| largest test file | **1,676** (was 10,189) | — | — | ✅ done |
| comment density | 29% | 39% | 53% | ⚰️ **RETIRED — see below** |

⚰️ **COMMENT DENSITY IS A DEAD METRIC. Do not chase it.** Deleting 22 duplicated functions and 37
duplicated closures — an unambiguous improvement — moved it **zero points**. It counts lines, not
information, and can be raised to 50% by restating every line. It is also confounded by size: a
79-line file with a 40-line header scores 50% from the header alone, which a 31k-line directory
cannot reproduce. AND IT MAY POINT THE WRONG WAY: `probare` exists to ask *"is this a program or a
description?"*, so `channel`'s 53% is as plausibly a FINDING as a target.

⚠ The nesting row was previously RAW COUNTS ("rete 10, process 0, channel 1"), which penalised the
directory with 817 functions against one with 7. Normalised, rete is ahead of `channel`.

**★ THE WARDS WERE CAST 2026-08-30 AND THEY SETTLED IT** (`99bf573df`, reports weighed against the
disk finding by finding):

- **`probare` ACQUITTED the prose** — 5.89:1 / 4.07:1 / 4.23:1, ZERO described or hollow forms in
  5,536 lines. It took 23 falsifiable claims out of the docs and broke 3. *Restatement never gets
  a number wrong, because restatement never commits to one.*
- **`intueri` CONVICTED the self-description** — six false claims, ALL about the tree's own layout,
  all trivially checkable, none checked. Both gates below exist because of it.

**⛔⛔ THE ANSWER TO "IS IT AN EXEMPLAR" IS: NO — 41 L1 + 70 L2 FROM 19 WARDS, AND SIX STRIKES OF
IT ARE NOW CLOSED.**

**➡ THE WORK LIST IS `VIGILIA-2026-08-30-WORK-LIST.md`, IN THIS DIRECTORY. Go there.** It is the
only place a row's status lives; this block is the pointer, not a second copy.

**LANDED 2026-08-30/31, each drawn → probed RED → struck → mutation-proven → floor green:**

| | | |
|---|---|---|
| A1 | `788e5b66d` | the FOURTH wall — import accepted a graph the fire passes cannot walk |
| A2 | `c449cd24d` | nine `panic!` arms → refusals; a wire value may not unwind the host |
| A2b | `d081142a9` | the silent zero — one `Option`, two facts, split by type |
| A4 | `42704d57b` | the ceiling's zero point belongs to the session, not the thread |
| — | `9ee04f945` | every `docs/arc` `.wat` loads, or declares in a closed rune why not |
| C1 | `119214aef` | the label follows the arithmetic — 103 accumulators |
| D1 | `2733b9bd9` | a bare keyword types as `enum` only for a UNIT variant that EXISTS |
| **oracle** | `16f504e14` | **an accumulate result is SUPERSEDED, not extended** |
| **B1** | `7319c1ea4` | **a `with-` form's scope is closed by a `Drop`, not by a release call** |
| **A6** | `bb0256e38` | **wall 5 — the import door bounds its own recursion** |
| **D3** | `057f9d494` | **an argument with no parameter is refused, not placed** |
| **A3** | `17fc5fb3e` | **the fence and the executor share one head-space** |
| **A5** | `7e24c3257` | **a verdict that cannot say "I did not look" is not a verdict** |
| **A7** | `b0e3377e9` | **the import door is a session's birth, and is charged like one** |
| **D1r** | `f22704f1f` | **a misspelled variant is told it is a misspelled variant** |
| **E5** | `c9cdd9d32` | **threading the span is the cure AND the guard** |
| **E1+E2** | `1efb42fc7` | **`UnknownField` has ONE producer, and it takes the keyword node** |
| **nested wall** | `c0c883082` | **the wall reads the form as it exists there — four dead kinds now fire** |
| **E4** | `452953cb9` | **the ceiling set is a closed type, matched exhaustively** |
| **E3** | `76e221bbb` | **each variant carries its own doc — and a broken link cannot be added** |
| **F1.5** | `58a10e1f8` | **every walking gate declares how it knows it reached something** |
| **F2.codemod** | `f4800ef97` | **every rete name in wat-scripts CODE resolves — prose may name a retired form** |
| **F1.1+2** | `2c7200802` | **a cited name in a rete comment resolves, or declares why it cannot** |
| **F1.3** | `9d4b68088` | **two ward rune vocabularies copied in and gated — and both my premises refuted** |
| **C7+C2** | `00ca6b0eb` | **an `(engine)` label names the evidence for its claim** |

**★★ THE ORACLE ONE IS THE DIFFERENT KIND, AND IT IS WHY THE BUILDER'S CALL MATTERS.** Native and
oracle disagreed on a shape where an accumulate's count changes mid-fixpoint. I recorded *"which
side is right is not decided here"* — and the builder said **"measure this against clara — confirm
who is wrong."** Clara 0.24.0 settled it in one run: it keeps exactly ONE derived fact, holding the
FINAL count. Native agreed on all three shapes; **the ORACLE was wrong**, accreting one stale fact
per intermediate state (`Tally(n=0)` standing while the count was 2).

**⛔ WHEN TWO INTERNAL ENGINES DISAGREE, NEITHER IS THE REFEREE.** The oracle is this arc's
differential reference — every `fire-rules$oracle` comparison over a changing-accumulate shape had
been measured against something that over-emits. And the tree's ONLY test for that shape carried a
`where` fence that filtered the intermediate emission, so it passed *because of its fixture's
shape*. Fenced they agreed; unfenced they did not.

⚠ **The fix reached Clara's answer by the oracle's OWN route** — well-supportedness
(`F := F ∩ (base ∪ D(F))`), not a port of native's delta logic, because an oracle made to mirror
native makes every future differential vacuous.

**⏭ D2 — DRIVEN, AND THE ANSWER IS A BOUNDED NEGATIVE.** The code asymmetry is real
(`hash_join_delta` has ZERO mentions of `right_idx_n`; `keyed_join_persistent` reads it as a
high-water mark). The shape is constructible — **measured from a real `Export` this time**, not
asserted. But no input reached the doubling, and a probe over all four branches explains why:
across 423 rete tests and three grid axes, **35 calls, ALL first-index, ZERO incremental.**
`indexed_n` is a **correctness guard** — the append does not clear, so it prevents a second call
re-pushing everything — **guarding a case nothing measured reaches.** So D2 is a live hole in a
guard that has never had a second chance to matter. **It must NOT be reaped**; `sequi`'s newtype is
the fix. Full reasoning in the work list.

**⛔ THE CLASS ABOVE THE FINDINGS STILL GOVERNS what remains: an invariant proven at ONE door and
assumed at ALL of them.** Three doors into a Session — `compile-all`, `import_export`, a
hand-assembled record. The first proves things; the other two do not. A1/A2/A4 were three
instances. **A3, A5, A6, A7 are the rest.**

⚠ **AND THREE OF THE SIX WERE THE PRIOR SELF'S OWN DEFECTS**, all one shape — **a commit message
asserting a general fix while the diff performed a specific one.** `89e8c3ed0` moved the estimator
LABELS and left the arithmetic (C1, 103 sites). `b7d9d8e90` fixed one `(engine)` mislabel while its
message named the class; two more stood. And two ratio floors gated a 0.23 ms arm on a parallel
runner and reddened the floor (`2a7051c67`, struck with the captured arm as the reason).
**Naming a class in prose is not pulling it.**

⚠⚠ **SEVEN RIDERS, SEVEN PRESCRIPTIONS OF MINE THAT DID NOT SURVIVE CONTACT** — every one surfaced
by asking for honest deltas, **none by a scorecard**: an impossible mutation; a `collect()` costing
five allocations on a path measured at ~27% of fire; a counter-proof that could not fail; an inert
mutation that was really a coverage finding; a file I said to rune that already loaded; a DESIGN
that forbade what its own BRIEF prescribed; and a gate spec that **could not see its own flagship
defect**. If you draw a strike, ask the rider where the brief was thin — that is where the value is.

✅ **CLASS B IS CLOSED (`7319c1ea4`), and the root was NOT what the work list said.** The release
sat in a `do` after the body; a wat error and a host panic each skipped it, both driven RED (the
two are separate mechanisms — `assertion-failed!` PANICS, `runtime.rs:15922` — and a first probe
that rode only the panic **blew past its own assertion**, failing with no message, which reads
exactly like a test that ran). But `with-open-file` has the **byte-identical** `let`+`do` shape and
does not leak: its resource is a Rust value whose `Drop` closes the fd. **The shape was never the
defect; the absence of an OWNER is.** An `ArmLease` guard now adopts `compile-all`'s lease and the
`do` is DELETED, not supplemented.

★ **TWO HOLES THE DESIGN MISSED, BOTH FOUND BY DRIVING.** `rete_arm_release` needed `try_with`: a
guard dropping after `ARM_TABLE` is destroyed does not panic, it **ABORTS** (`thread local panicked
on drop`, SIGABRT) — and reverting it changes nothing observable, so that is a **coverage** finding,
not a covered one. And `:rust::rete::ArmLease` is hand-minted, so `is_registered_rust_opaque` cannot
see it and `is_pure_type`'s `None => true` arm judged **a live thread-local resource handle PURE** —
admissible as a `Record` field and onto the wire. Driven with an unregistered positive control.

⛔ **AND THE SHARPEST CORRECTION ANY RIDER HAS RETURNED: A COUNT IN A SCORECARD IS A CEILING ON THE
EXECUTOR.** My EXPECTATIONS row 11 pinned the floor at 5,188 + **exactly three** tests, so a fourth
arm would have falsified my own row before I ran it. Two real arms went undriven because of my
arithmetic. Pin a floor-bound and a direction, never an equality. Full reasoning in
`strike-lease-unwind/SCORE.md` § "Where MY brief was thin".

✅ **A6 IS CLOSED (`bb0256e38`), and BOTH the severity and the scope on the work list were wrong.**
It is not a SIGSEGV but a stack-guard **ABORT** — no `catch_unwind` reaches it. And the defect is
not "deep input crashes": the **same** 20,000-deep Export is **ACCEPTED on a 256 MiB thread and
aborts on a 2 MiB one**, both driven. Import had **no depth criterion at all** — acceptance was a
property of the importing THREAD, so two processes running identical code disagreed about whether
the same bytes were a valid network, and the disagreement was settled by an abort.

★ **THE FINDING NAMED ONE TOWER AND THERE WERE THREE.** My brief listed four sites as *entries
into* `unpack_prog`. Two of them live in functions that are **themselves self-recursive** —
`unpack_driver` and `unpack_cond_op` — which I never checked. Wall 5 is therefore ONE budget
(`MAX_IMPORT_DEPTH = 300`) threaded through **five** mutually recursive unpackers. The mutation,
re-driven here: with the budget in `unpack_expr` only, the `:and` probe goes **GREEN** and the
other three go RED. The obvious fix passes the obvious probe and leaves three towers open.

⛔ **AND THE REASON I WALKED PAST `unpack_driver` IS THE LESSON.** Its doc comment read, verbatim:
*"a driver tree of any depth round-trips **without a depth parameter** — the wire's nesting IS the
recursion."* Every word true. It is also the exact statement of the vulnerability. **An accurate
comment in the wrong register is a defect's alibi** — nothing drifts, so nothing checks it, and it
reads as settled. Promoted to memory. When a comment explains HOW a mechanism works, ask separately
whether it is RIGHT; treat *"without needing Y"* as a missing guard phrased as economy.

⚠ **The bound was MEASURED and the measurement is itself a finding:** the corpus max nesting depth
is **3** (423 tests; every packed program bottoms out at `unpack_prog` → `unpack_expr` → one
operand). So the corpus could never have constrained this number at any bound above 3 — the only
real constraint is the 3,000–5,000 abort window. **The export/import corpus is broad in VARIANTS
and flat in DEPTH; that coverage gap stands open.**

✅ **D3 IS CLOSED (`057f9d494`) — and it was a SILENT WRONG ANSWER, not a missing check.** Driven
through the public surface: the fixture fence answers **1** hit; a `CallUser` with two args for one
declared param at slot 1 is **ACCEPTED and answers 0**, because a surplus argument was written into
the slot whose NUMBER equals its POSITION and overwrote the parameter. Past the frame it was
silently dropped (2 hits); missing, it surfaced as `unbound symbol: slot 1`. One missing check,
three faces. **Class A a fifth time** — `lower_expr` builds `CallUser` from `lower_args` and
`lower_rete_defn` without ever comparing them.

The check went at `exec_program_on`, the one place args and params meet, and the surplus branch was
**deleted** rather than made safe. **A sixth import wall was affirmatively cut** — it would have
refused the tampered exports and turned every probe green while the executor still held no arity
invariant.

⛔ **THE FLOOR WENT RED AND THE RIDER COULD NOT HAVE SEEN IT.** `wat::lint
no_loose_string_assert` — the new probe used `op.contains(…)` where the op is a fixed constant.
Cured at the root with an exact `assert_eq!`, never a rune. **The gap was mine: a rider runs
`binary_id(<subject>)`, and new TEST code trips lints in `wat::lint`. Any brief that adds tests
must name that binary in the rider's scoped checks.**

★★ **AND THE VACUITY MUTATION CORRECTED MY OWN SCORECARD.** Forcing the check to refuse EVERY call
left my "untampered fixture answers 1" control **GREEN** — that path never reaches
`exec_program_on`. Every green in this strike would have been consistent with a check that refuses
everything, had the suite contained only what I specified. The control that carries the weight was
the rider's own addition. **Ask of every control: what does it FAIL on?**

⚠ **Recorded as an accepted consequence, so nobody bisects for it:** `lower_fn` compiles every
literal `fn` to `CallUser{params, args: []}`, so a fn **value** reaching `exec` outside the four
diverted HOF heads now answers `ArityMismatch` where it answered `UnboundSymbol`. Both are errors;
the kind changed.

⛔ **TWICE NOW I HAVE NAMED ONE SITE WHERE THERE WERE SEVERAL** — A6 (one tower, three) and D3 (one
call site, six). A finding cites where it was NOTICED, not where it LIVES. Grep the callers and
put the count in the brief. Promoted to memory.

✅ **A3 IS CLOSED (`17fc5fb3e`).** The acc-form fence admits on `primitive?` — *"has a `RETE_OPS`
row"* — while `lower_named_rete_fn` looked the head up only in the USER table, so a minted row was
refused with `unknown rete-defn` **about a row of the very table that admitted it**. Driven both
halves: direct → refused; the SAME op behind a one-line user `defn`, SAME position → `"fired"`.
**The capability was real and only the ladder was missing**, which is why tightening the fence was
rejected — it would delete a working capability to make two registries agree.

★ **THE CLASS GATE COMPUTES ITS OWN POPULATION.** `reachability.rs` now drives every `RETE_OPS` row
whose param shape fits `(head ?v)` — 1 of 79 today, **never named** — and was proven to fail three
ways: ladder reverted, predicate mutated to match nothing, and a **wrong-opcode differential** (the
op is an INDEX, so an off-by-one runs a *different* row and still produces a fact; "it fired" would
not catch it). ⛔ **The banked `harness-experiri` recon was NOT appended** — ONE real assertion
across EIGHT tests, counted, not inherited.

⛔⛔ **AND MY OWN BRIEF AUTHORIZED A REGRESSION.** Trap 2 told the rider that D3's wall handles
acc-head arity, so not to add a second refusal. `expr_ir/mod.rs:14-19` says otherwise: *"`lower` IS
TOTAL OR IT REFUSES … **every arity checked** … a refusal that belongs at compile time and lands at
fire time is a defect in this file."* An arity-2 acc head was refused at FIRE with the span pointing
into `fire/acc.rs`. The rider cited the law, refused to ship it silently, and the instruction was
reversed: a compile-time fence at `arm.rs:430` — the sole caller, the only place that knows the
operand count — moves the **location, the op name and the timing**. D3's wall is untouched as the
backstop for every other caller. **No scorecard row could have caught this: the row I wrote endorsed
the behaviour.**

★★ **THIRD SCORECARD ROW THIS SESSION THAT COULD NOT DO ITS JOB.** Row 7 asked for a grep to return
0; it returns **1 at HEAD**, because a pre-existing sweep hard-codes that row. Before it: a pinned
COUNT that capped a rider's coverage, and a control that stayed GREEN under a check refusing every
call. **Run every scorecard row against HEAD before shipping the scorecard** — a row whose
pre-value you cannot state is not a check. Promoted to memory.

✅ **A5 IS CLOSED (`7e24c3257`), and the row understated it.** `import_export` does not skip the
termination analysis — it **never calls the verifier at all**; and `rete_arm_get_or_build` is a
**SECOND** false door. So *"`compile-all` is the one door EVERY rule passes"* was false twice, and
`stratify.rs:339-342` had been stating the gap correctly from its own side the whole time — two
module docs on one boundary, one of them true. `Ok(())` came from **five** silent sites meaning
three things. Driven: a `Rule` with empty `:lhs`/`:rhs` — the shape an imported Export's rules have
— makes `compile-all` answer `"Compiled"`. Now `TerminationVerdict{Proven, NotAnalysable{rules},
Refused}`; behaviour unchanged, the state merely **sayable**.

★★ **THE RIDER CORRECTED MY STONE, AND WITHOUT IT THE STRIKE WOULD HAVE CHANGED NOTHING.** My
DESIGN table classified the two early exits as *"proven"* **unconditionally** — and the strike's own
repro reaches one of them, so an unconditional `Proven` would have shipped the type split with the
defect exactly where it was. The cure: **the skip count TAINTS the proofs** — `Proven` iff zero
skips. I had classified each site by what it structurally WAS, not by what it KNEW given a skip.
**A proof over a filtered population is not a proof.** Promoted to memory.

⛔ **THREE OF THE SIX THIN SPOTS WERE MINE, AND TWO ARE REPEATS.** My trap 1 gave the wrong
MECHANISM (the early exit keys on whether an edge *computes*, not on `edges` being empty — a mixed
set reaches it too). My table had four silent sites where there are **five** — the third consecutive
strike where I under-enumerated. And my scorecard's single mutation over a four-site gate is
**my own recorded lesson violated while WRITING expectations** rather than while reading a gate; the
rider ran six, each predicted in advance with a distinct red set.

★ **AND THE TIER INSTRUCTION PAID FOR ITSELF.** The rider's own first gate draft counted seven probe
calls as doors and went red — caught by the mandated `binary_id(wat::lint)` run, the check added two
strikes ago after a floor red a scoped probe structurally could not see.

⚠ **Two totalities are now GATES, not prose** (`rete_header_claims_are_asserted`, +2 rows), because
that lint's own law says *"if you cannot gate it, do not assert a totality about it."* Radius went
to +1 file and that is correct.

⭐ **CLASS A IS CLOSED — ALL SEVEN DOORS (A1, A2, A3, A4, A5, A6, A7).** The class above the
findings — *an invariant proven at ONE door and assumed at ALL of them* — is pulled. `import_export`
now carries **six walls** where the header once counted three.

✅ **A7 (`b0e3377e9`) was worse than "uncounted".** `session_bytes` does `entry(key).or_insert(now)`,
so an unmarked session's origin was set at the FIRST CHECK and the entire build went **retroactively
free**. Driven, same 2 MB: marked-at-birth **2097268**, never-marked **0**. The origin is now
captured as the door's FIRST statement and filed after the build — the reading and the filing must
split, because the key IS the built network's identity. Wall six refuses past `MAX_IMPORT_NODES =
10_000`, **measured**: corpus max 63 nodes across 34 importing tests, ~122 ms worst case on the
driven quadratic curve. `from_pairs` was affirmatively CUT — the cap bounds N, so the quadratic is
bounded with it, and the before-curve (1.05→4.87 µs per pair at 500→4000) is recorded for a later
speed stone.

⛔⛔ **AND THE STRIKE INDICTED ONE OF MY OWN.** Mutation 3 removed A4's non-clobber rule: the new
unit probe reddens, and **A4's `rearm` arm stays GREEN** — the arm whose fixture says verbatim
*"only this arm can see it."* The mask is `LAST_ORIGIN`, the cache in front of the origin map, never
invalidated on a write. `git show 42704d57b`: **the cache and the arm landed in the SAME commit**, so
the measurement was taken before the cache existed beside it and never re-taken. **It was false the
day it shipped** — my strike, under a doctrine file whose header opens with a warning about exactly
this. The code is correct; the PROOF was broken. Struck at the site, with the driven correction and
a pointer to the live gate. **A cache in front of probed state makes a gate unfalsifiable — re-run
every mutation proof when a memo lands.** Promoted to memory.

⚠ **Three hazards recorded, not acted on:** `expect_seq` clones the whole `nodes` vector before the
cap can see it (one memcpy on a hostile 10M-element field); `SESSION_ORIGINS` entries are never
removed, so a long-lived importing thread leaks one entry per session (pre-existing with A4); and
**a release-weighed floor ELIDES an allocation-only probe's ballast** — a 1 MiB `Vec` read as 121
bytes until `black_box` was added. I hit that in my own recon and failed to warn about it.

✅ **D1 IS FULLY CLOSED (`f22704f1f`).** D1 made the misspelled variant refuse; the refusal then
NAMED THE WRONG THING — `UnknownField`, *"has no field `:evt::G::Hii`; available fields: [k,
grade]"*, sending the author after a FIELD for a VARIANT typo. **A confidently wrong remedy costs
more than none.** Now `#wat.rete/UnknownEnumVariant` — *"`:evt::G` has no variant `Hii`; available
variants: [Hi, Lo]"*. `keyword_constant_segment`'s `_ => "keyword"` was the **fifth catch-all** in
this arc; it is now a named three-state `KeywordConstant`.

⚠ **Two corrections to what I had written here three stamps running.** (1) I recorded this row as
needing *"a `UnknownEnumVariant` kind"* — a name that **existed nowhere in `src/`**. I invented it
and then cited it, the second invented symbol in four strikes (D3's `callee_program`). It exists now
only because the rider built it. (2) *"So rete names the same thing core does"* was the wrong
target: driven, **core has the same blind spot** (*"expects keyword; got `:evt::G`"*, `remedies []`).
The target is naming the mistake, not agreeing with a sibling that is also silent.

⛔⛔ **AND MY OWN SKETCH WOULD HAVE SHIPPED A FALSE DIAGNOSTIC.** I drew the new arm as a guard
placed AFTER the arity-0 arm — and a guard inherits everything the arms above it did not consume, so
it catches `Some((_,_,n>0))`, a tagged variant that **EXISTS**, as well as `None`. It would have
emitted *"`:tg::P` has no variant `Hi`; available variants: [Hi]"* — listing the variant it claims is
missing, the exact class the strike deletes. **ARC DOCTRINE, fifth split and the first drawn wrong:
SPLIT ON THE DISCRIMINATOR (the resolver's own `None`), NEVER ON A SYMPTOM that merely correlates
with it.**

⚠ **A third mistake is now PINNED, not fixed:** a *bare tagged variant used as a value* still gets
the field-names remedy. The variant EXISTS, so "has no variant" would be false — it needs its own
kind. Cut here, pinned with a golden so the day someone takes it, the pin reddens and names what
moved.

⚠ **`f22704f1f`'s commit message is MANGLED** — backticks inside `git commit -m` were
command-substituted, and the amend would have rewritten already-pushed history on a shared branch,
so I reset to the pushed commit instead. The intended text is in `strike-variant-diagnostic/SCORE.md`.
**A commit message quoting an identifier goes in a `-F` file, never `-m`.**

✅ **E5 IS CLOSED (`c9cdd9d32`), and the finding was sharper than "a wrong span".** Both
`refuse_export_without_arm` sites stamped `rust_caller_span!()` while the real wat span sat **one
frame up, in hand, already spent** on the same function's arity refusal. The lint that exists for
this could not see it: `span_substitution_justified` tests for **no span PARAMETER** as a proxy for
its stated principle, *"never about the absence of a choice"* — and the choice lived one frame up,
so the proxy admitted a site the principle refuses.

★ **THE CURE IS THE GUARD.** Threading `span: &Span` into both fns fixes the diagnostic **and brings
both bodies inside the existing lint's view** — a future `rust_caller_span!()` there now reddens it,
with no new lint and no widened predicate. **The proxy is not evaded; it is made TRUE.** Driven on
BOTH bodies (the rider's own call — one mutation cannot prove a two-site claim).

⛔⛔ **AND MY LOAD-BEARING NUMBER COULD NOT BE CHECKED.** I put *"534 sites tree-wide, 71 in rete"*
in the stone as the **entire justification** for not widening the lint. It reproduces under **no
definition** at its own stated commit — it came from an ad-hoc regex script I ran in my terminal and
never committed. The rider ported the **lint's own walker** with its predicate inverted and got
**494/69**. The decision survives on the measured figure; the figure did not deserve to be trusted.
**The lint's doc now carries the numbers AND the instrument** (`violations_in` with `carries_span`
inverted). **Second instance of this failure** — memory updated, not duplicated. *If a number
justifies a decision, the thing that computed it goes in the tree beside it.*

⚠ **My caller table also named a fn that never calls it** — `fire/mod.rs` is not a caller of
`fire_rules_on_session`. Real count **10 + 1**, not 9 + 1. Only the "count them yourself" hedge kept
it from misleading, and a hedge is not a correct table.

⏭ **A NEW HOLE, FOUND IN PASSING AND FILED:** nothing gates `file:line` citations in comments.
`no_stale_path_in_doc` checks **paths, not lines**, so two accurate references in `arm.rs` rotted
silently when a doc block shifted. Refreshed; the general gap is now a row in Class E.

✅ **E1 AND E2 ARE CLOSED (`1efb42fc7`) — AS ONE CLASS.** Four sites produced `UnknownField` and
pointed four different places; the only one naming the offending keyword **could not run**, and its
doc was the clearest statement of the contract in the file. Now **one producer**,
`check_field_kw(field_kw: &WatAST, …)` — **a bare `Span` no longer compiles at any call.** That is
the strike: fixing three spans without changing the type would have left the next author to pass
whatever span was nearest, which is exactly how three docs came to promise a behaviour three sites
did not have. Caret **cols 31–76 → cols 65–75**, the keyword's exact extent.

⛔⛔ **AND THE STRIKE CONTRADICTED MY OWN STONE: TWO OF FOUR PRODUCERS WERE DEAD.** I called the
nested-constructor producer live and gave it a row and a mutation. **`defrecord` lowers every
record-constructor call to `(:wat::core::kwargs-construct :Type …)` before the wall runs**, and
`walk_nested_constructors` matches the record type as **HEAD** — so `types.get` returns `None` and
FOUR error kinds are unreachable there. Re-driven: `(:fsn::Inner :nope ?k)`, an undeclared field with
the declared field unsupplied, comes back **`"ACCEPTED-UNVALIDATED"`**. Its **sibling enum-variant
branch IS live** (an enum variant is not lowered), which is why the walk looks exercised from
outside. `purity.rs` hit this identical class and WAS taught the post-lowering shape; this walker
never was.

★ **NOT FIXED — PINNED, and the pin is the model.** That is a wall-reachability strike across four
error kinds, not a span strike. The pin asserts the program is ACCEPTED **and** that it reached its
sentinel, so it cannot pass by failing some other way, and it names what to assert when someone
wires the branch. **A finding as a live gate beats a paragraph in a stone nobody re-derives.**
Promoted to memory: *a check keyed on a pre-lowering shape is dead, and a live sibling arm makes the
whole walker look exercised.*

⚠ **My mutation for that row predicted "nothing reddens" and nothing did — I would have read a
CORRECT observation as insensitivity.** The rider made the mutant `unreachable!` so silence proved
non-execution rather than a blunt probe. And my kwargs mutation changed the field NAME as well as the
span (9 reds); **a mutation that changes two things proves neither.**

⚠ **Two frictions worth knowing:** `git stash` is denied to the rider tier, so it could not build a
HEAD binary — the checked-in golden served as the pre-value, which is an argument for goldens over
ad-hoc measurement. And a deliberately-FAILING scratch `.wat` cannot live in
`wat-scripts/scratch-pad/` without reddening `every_wat_scripts_file_loads`; durable ones belong
beside the probes in `tests/rete/`.

✅ **THE NESTED-CONSTRUCTOR WALL IS WIRED (`c0c883082`)** — the hole the previous strike PINNED
rather than fixed, which is why it was drawable at all. `defrecord` lowers every record-constructor
call to `(:wat::core::kwargs-construct :Type …)` before freeze, so the type sits at index 1 and
`types.get` on the head returned `None`. **Four error kinds were unreachable; all four now fire**,
each with its own probe and per-arm mutation separation. Reverting the head recognition reddens
exactly five — the four kinds plus the re-pointed pin — with every control still compiling and
firing.

⛔⛔ **AND THE STRIKE SHIPS NEW ENFORCEMENT, WHICH ONLY A DRIVE FOUND.**
`RhsPositionalConstructionRetired`'s doc claimed the runtime dispatch *"unconditionally retires
multi-arg RAW POSITIONAL construction"*. **Driven at HEAD: a nested `(:T ?k 99)` COMPILED, FIRED and
derived `y = 99`.** Both citations verified: `rhs_must_compile` says *"do not walk
`build_insert_fact` on native fire"*, and the compiled path returns positional args verbatim —
*"already declaration order BY DEFINITION"*. **The doc was written from the INTERPRETER's behaviour
and never checked against this path.** So wiring the kind is not restored parity — this wall is now
the only enforcement of that doctrine on the rete path. Accepted deliberately, on a corpus sweep
(1650 `.wat`, 460 `:then`) showing **zero uses**; the false doc is corrected at the site. **Memory:
*reviving a dead guard is a behaviour change — drive the live path first.***

⚠ **MY OWN "OUT OF SCOPE" PREMISE WAS FALSE.** I wrote that all spellings arrive as
`kwargs-construct`; a hand-written `aggregate-new` **does** arrive, head intact, type resolving — I
generalised from four SOURCE spellings to all spellings. The arm is still omitted, but for a better
reason than mine: `aggregate-new` **is** the positional route, so firing the retirement under it
would be an actively **wrong refusal**. And my sketch would have regressed twice — an `_ => return`
dropping nested constructors under a call form, and an unguarded `items[1]`.

✅ **E4 IS CLOSED (`452953cb9`).** Three converters ended in `_ =>`; driven, their owned sets are
**disjoint**, so those catch-alls were load-bearing and **there was no live gap** — exactly four
ceiling variants, exactly four listed. The defect was that nothing forced the **fifth** to be
considered: it would land in all three wildcards at once and become a raise, the one thing the
outcome wall exists to prevent. Now `RuntimeErrorKind::ReteCeiling(ReteCeiling)`, matched
exhaustively, cross-converter arms **written** rather than defaulted.

★ **RE-DRIVEN HERE, AND IT FIRES IN FOUR PLACES, NOT THE THREE I PREDICTED** — the three converters
**plus `signal.rs:790`**, whose `fmt_with_span` match carries no wildcard at all. **A fifth ceiling
now cannot compile until it is both ROUTED and GIVEN A MESSAGE.** Better than the scorecard asked
for.

⚠ **ACCEPTED WITH A COST, AND THE COST IS RECORDED AT THE ENUM.** `RuntimeErrorKind` derives `ToEdn`
and `Display` **is** `to_wire_edn`, so nesting changes the rendered tag to
`#wat.runtime/ReteCeiling {:ceiling …}`. Unavoidable — the derive has no flatten or transparent.
Accepted on measurement, not assumption: **prose messages byte-identical**, no test or `.edn` golden
asserts the tags, every wat-level match is on the outcome enums, and all four convert before reaching
wat. `#[to_edn(transparent)]` is filed as its own strike.

⛔ **MY SKETCH WOULD HAVE DEFANGED THE GATE.** It renamed the four variants;
`no_ceiling_raise_in_rete` matches them by `line.contains` and asserts four hits, so the renames stop
matching at **all four doors** — and my own trap 5 told the rider to *"update them so the lint still
fires"*, i.e. to re-point a live gate at strings I had just invented. The rider kept the names
verbatim and the gate stayed **unmodified**. **A gate re-pointed at the strings of the change it
polices is not a gate.**

⛔ **AND MY RADIUS WAS WRONG A THIRD TIME — the failure mode is now NAMED: SUBSTRING.**
`SessionMemoryCeilingExceeded` is a **prefix of** `…OnInsert`, so `grep -c` counted six twice. 30,
not 36. All three bad estimates came from naive greps. **Use `-w` / `\b` whenever a name family
shares a prefix — error families almost always do.** Memory updated.

⭐⭐ **CLASS E IS CLOSED (E1, E2, E3, E4, E5) — AND SO IS CLASS A. Class F is what remains.**

✅ **E3 (`76e221bbb`).** Three doc blocks stacked onto ONE variant — **Rust accumulates a doc comment
onto the NEXT item** — so the two failures with the most to explain rendered with **no doc at all**.
Split onto four variants and **verified in the rendered HTML**, not the source. `signal.rs`'s 9
broken intra-doc links → 0; **only two were E4's, seven were older**, so the file had been citing
four unreachable items before that strike touched it. Tree-wide 50 → 41.

★ **THE CLASS CURE: `tests/lint/no_new_broken_doc_link.rs`.** Nothing in this tree ran rustdoc and
the lint was not enabled, so **every intra-doc link was unverified** — I broke two in E4 with a green
floor and a clean clippy. The gate now runs rustdoc and freezes the remaining 41 sites as **34 NAMED
`(file, target, sites)` keys**: a ratchet both ways — unlisted is red, listed-but-resolving is red.
Six arms driven, including a **genuinely `flock`'d cargo lock**.

⛔⛔ **AND MY OWN SKETCH REOPENED THE WARNING I HAD JUST WRITTEN.** The stone quotes `purity.rs` —
*"wanted SET MEMBERSHIP and measured CARDINALITY"* — as the reason for a named list; then prescribes
a `(file, target)` key. **7 of the 34 keys hold two sites**, so fixing one would have left the gate
green: **the same defect, one level down, inside the cure for it.** The per-key site count is the
rider's. Its defense is the keeper: *a count scoped inside a NAME still names the offender.*
Promoted to memory.

⚠ **My trap named the wrong risk.** Duration was never it (~0 against a 131.9s lint binary). The
hazard is the **nested target-dir lock** — an unbounded spawned `cargo doc` **BLOCKS** rather than
fails. Bounded at 300s against a worst observed 10.68s; an expiry is a named red quoting cargo's own
`Blocking waiting for file lock` line. **Ruled and recorded at the constant: keep the bound, refuse
the separate `CARGO_TARGET_DIR`** — red-when-it-cannot-measure is CORRECT, and the alternative is the
recorded failure of a check reporting success without running.

⚠ **The row's second claim was WRONG**: the *"names an action the author can take"* justification
lives at `outcome.rs:226` and was correctly placed all along. **Third Class E row this session whose
detail did not survive an audit** — an inherited row is a past act of looking.

✅ **CLASS F IS OPEN, AND ITS DEEPEST ROW WENT FIRST (`58a10e1f8`).** `tests/lint/` is where every
guarantee in this arc is proven, and **a gate that walks an empty set asserts nothing and reports
PASS.** So the suite's own credibility was the right first Class F strike.

★★ **THE DRIVE CAME BEFORE THE LINT, AND THAT ORDERING WAS THE STRIKE.** A missing guard is a risk;
a **vacuous gate is a defect**. Every discovering gate was instrumented at the point its population
is computed and run under `--no-capture`: **no gate is vacuous today** — 4 parity scripts, 11 grid
axes, 57 `src/rete` files, 125 path references, 34 diagnostics, 445 `.wat`, up to 998 `.rs`. Because
that ran first, **all 18 new guards are MEASURED floors**, not numbers chosen for symmetry with a
sibling.

★ **F0 WORKED EXACTLY AS THE BUILDER SPECIFIED.** The stone deliberately carried **no count** — the
row said *"10 of 15"*, my own audit grep said *"16 of 24"*, and mine was demonstrably wrong. The
instrument answered: **24 in scope, 19 undeclared**, of which **six already had a real guard** and
only lacked a declaration. *A number in prose is replaced by the command that derives it.*

⛔⛔ **AND THE STRIKE BIT ITS OWN EXECUTOR — the "one level down" failure, one strike after I promoted
it to memory.** The gate scans `tests/lint/`, so **its own prose is data**: the `///` doc on its
`Declaration::Rune` variant parsed as a rune declaration, and it was **one run from vouching for
itself with its own documentation**. Invisible by reading; caught in the first driven run.

⚠ **My own re-run then found the CURE's doc overstated.** Turning a `NON-VACUITY` marker into
`/// NON-VACUITY` left the gate GREEN — `DOC_HEADS` is consulted at exactly one site, the rune path.
The behaviour is right and the asymmetry is now stated at the constant: **a rune's REASON TEXT is its
evidence, so a description reads as an answer; a marker's evidence is the ASSERTION beneath it**,
which `is_assert` refuses to read from any comment.

⚠ **The most expensive thin spot: "drive every walking gate" named no mechanism.** Reading a
collector cannot see what it visits — it took instrumenting 27 population sites and a full
`--no-capture` run. Unnamed, a rider reads the collectors, calls it driven, and row 1 is lost.
**Any future "drive it" instruction must name HOW.**

✅ **F2's CODEMOD ROW IS CLOSED (`f4800ef97`) — AND IT STRUCK A CLAIM IN `CLAUDE.md`.**

⛔⛔ **"ALL WAT STAYS CORRECT, ALWAYS" WAS FALSE, AND STOOD FOR MONTHS.** Type-checking is **not**
resolution: a `def` body nothing forces is never resolved, so a file under `wat-scripts/` could name
a head that has never existed anywhere. **Driven** — `(:wat::rete::core::THIS-HEAD-NEVER-EXISTED …)`
type-checked and the program **ran**. Two phantom rete names lived on that licence, one pair of them
**inside the very codemod that file mandates for every `.wat` migration**. `CLAUDE.md` now names both
gates, says what each proves, and states plainly what is still unproven.

★ **DELETED, NOT RE-POINTED** — 41 pairs → 39. And the rider's evidence beat mine: a pure head-rename
to `mapv`/`filterv` **does not compile** (*"no clause of `:wat::core::filterv` matches arity 2"*),
because the eager form needs a different container. **That RED is the best single fact in the
strike:** with a head that resolves, the loader gate finally had something to check — it had nothing
for three months while the head was invented.

⛔ **MY ★ CONTRACT WAS FALSE AS WRITTEN.** *"A `:wat::rete::` name in CODE resolves"* — but **a
recorded codemod's OLD column is code and must name what it removes**, and a negative-control probe
deliberately calls an unminted head **as another brief's non-vacuity proof**. Four names, three
files. Enforcing my sentence literally would have destroyed that proof. They carry a per-name
`rune:lint(rete-name-unminted)` now.

⛔⛔ **AND A NAIVE UNION WOULD HAVE VOUCHED FOR ITSELF.** Measured: **all 79 `RETE_OPS` rows are also
attested in code elsewhere**, so under a flat `rows ∪ attested` universe the registry half resolves
**exactly zero** names — emptying it changes no verdict and the blinding mutation passes **green**.
**A non-vacuity floor does not save you: it fires on "I read zero rows", which is a different failure
from "the rows I read decided nothing".** Split by namespace, both halves now bite (71 / 63).
Promoted to memory.

⚠ **CLIPPY WENT RED and the tier split is why it was caught** — `unnecessary_get_then_check` in a new
unit test, while the rider's `binary_id(wat::lint)` was **153/153 green**. **Nextest runs tests;
clippy lints.** Cured with clippy's own prescription.

★ **A discipline the rider surfaced unprompted, worth carrying into every mutation:** its first run
of one mutation reported GREEN because **the mutation had not landed** (a `perl` substitution silently
failed without a UTF-8 flag). *A mutation that does not land is indistinguishable from a gate that
does not fire.* **Assert the mutation landed before believing its result.**

✅ **F1's ROWS 1+2 CLOSED AS ONE (`2c7200802`).** Two unchecked citation kinds, one walker:
`tests/lint/rete_citation_resolves.rs`. **33** identifier citations (the row guessed 7) and **27**
stale bare filenames of 244 cited — `validate.rs`×9 and `expr_ir.rs`×6, an unseen cluster from the
same `partire` split that broke `tests.rs`. Every `src/` change is a comment, checked mechanically.

⛔⛔ **MY OWN TRAP EXAMPLE WAS A ROTTED CITATION.** Trap 2 offered `axis_variant_names_round_trip` as
the model of a *legitimate* test-only name. It resolves **nowhere** — the fn is
`…_through_one_door`. A rider trusting it would have widened the universe until a real finding
disappeared: **the exact STOP-1 failure that same trap warned against, seeded by its own example.**
The rider found the genuine controls and made them **live assertions** instead of sentences.
**Grep every example as you write it; an example is a claim.** Promoted to memory.

★ **AND THE GATE'S OWN FILE IS IN ITS OWN UNIVERSE.** `NoMatchingArm` and `SiftRulesResponse`
resolved against the gate's **own error text** on its first run. Without the `SELF` exclusion, a
future hand silences any red by naming the offender in a failure message — *prose vouching for
prose, one level up*, which my stone warned about at the level below.

⚠ **Three more corrections worth carrying.** (a) The agreement on 33 rests on a shape rule I never
stated — any bare identifier gives **47**, the extra 14 being git SHAs and Latin session names;
`_`-or-interior-capital reproduces 33, and that is now a **stated boundary**. (b) My universe was
short by `wat/` and by **file stems**. (c) The obvious filename rule **misses the finding that
motivated it** — `tests.rs` still exists at `src/macros/tests.rs`, so basename-existence alone leaves
`kernel/mod.rs:4` green; ancestor-relative alone reports 55 with 31 false. **Only the conjunction is
right.**

★ **A DEVIATION I WAS ASKED TO WEIGH, AND ACCEPTED: spelling beats a named vocabulary.** I required
three vocabularies (clippy lints, memory slugs, `_`-prefixed); the rider built none and excluded by
**spelling** — `clippy::needless_borrow`, `*_pass`, `[[feedback_…]]`, forms the tree already uses
(verified at `function/parse.rs:48`). **A list keeps exempting a name after it stops being noise; a
spelling rule makes the correct form the only passing one and the failure text teaches it.** A rung
above what I asked for.

✅ **C7 + C2 CLOSED (`00ca6b0eb`).** Every cost number this arc quoted came off the bench harness,
and the class had **already recurred** — `b7d9d8e90` is titled *"the benchmark called the wrong arm
'the engine' for eleven days"*, fixed one, named the class, and one survived. Now
`rete_engine_label_names_its_evidence.rs`: **an engine claim carries its evidence.**

⛔⛔ **STOP-3 FIRED TWICE, AND MY RULE WOULD HAVE DELETED A TRUE CLAIM.** I briefed three sites; there
are **five** — one inside a `#[cfg(test)]` mod in a `src/` file, one spelled `(THE ENGINE)` which my
*"one word, so it can be exact"* reasoning missed on case alone. And C7's rule as adopted — *"only if
its body CALLS the production function"* — would have stripped the `L` arm, which calls nothing
because it **replicates `root_for` inline where production is not callable**, and whose claim is
already pinned by a gate asserting the type and body **exactly**. **I wrote that gate myself during
item #3, then drew a rule that would have overridden it.** Widened to three shapes: *calls
production* · *replicates it under a shape gate* · *neither*. **Memory: a rule can outlaw a truth.**

⛔ **MY EXCLUSION BOUNDARY WAS THE WRONG KIND.** `kernel/tests/` is a path prefix; **26 files under
`src/rete/` carry `#[cfg(test)]`**, so the decoy would have passed while the hole stayed open. It is
*"not inside a `#[cfg(test)]` module"*, by brace tracking. **Re-driven here: identical fn, identical
label, RED inside `cfg(test)`, GREEN in production scope — differing by placement alone.** Also: the
LABEL may live in test code (four of five do); only the RESOLUTION TARGET must be production.

★ **AND THE `gated by` FORM SPENDS SOMETHING.** It writes a test fn's name into a `src/` string, which
retires it as a test-only control elsewhere — it consumed one of `rete_citation_resolves`' two
controls **on landing**, with no replacement of the same kind in the tree. **Closed, not filed**: an
**owned, uncitable** control now floors that gate, the real one is kept beside it, and the coupling
is warned at the `gated by` **definition**, where the next author works rather than in the gate they
will never open.

⭐⭐⭐ **F1 IS CLOSED — all five lints. Classes A, E and F1 are done; F2 and F3 remain.**

✅ **F1 row 4 was STRUCK, not built** — C1 itself had already shipped
`minimum_label_matches_its_estimator.rs` (446 lines) the day after the vigilia filed the row. **F0's
own thesis: a claim that was true when written.**

✅ **F1 row 3 (`9d4b68088`)** — both vocabularies copied into `CONVENTIONS.md` with provenance and
gated by `no_unknown_ward_rune.rs`, seven arms mutation-proven, scanning wider than the `sequi` gate.

⛔⛔ **AND THE RIDER REFUTED BOTH OF MY PREMISES. I VERIFIED BOTH REFUTATIONS MYSELF.**

**(1) The vocabularies were never undefined.** My stone said *"nothing says what any of them mean."*
I fetched `perspicere` from the signed channel: its § **The rune** defines all three categories
verbatim. **I grepped `CONVENTIONS.md`, got zero, and concluded "undefined" — which proved the COPY
was missing and nothing about the thing.** The authority is the ward spell, from the same MCP I had
used at the top of this session. *An empty local grep is evidence about transcription.* Memory.

**(2) The live finding was worse than wrong — acting on it would have caused the defect.** I reported
six `census.rs` runes reasoned *"alias would be a mumble"*. That clause is **shared boilerplate at 18
sites across four files**; I saw a quarter of the population and read it as a per-site argument.
Against the authored definition the six are **correctly labelled** — `mumble-alias` requires a *Level
2 mumble*, and `CensusLog` reads better than the type. **Following my instruction would have split 6
of 18 identical sites — the exact `ARM_TABLE`/`EXEC_ARENA` divergence `sequi` exists to prevent.**
The rider made **zero `src/` edits**, correctly, and reported instead of rewriting.

⚠ My site counts were off by **more than 2×** (46 `perspicere` tree-wide, not 18).

⚠ **STOP-3 CONFIRMED AND SHARPER THAN THE ROW.** No trait exists at any `trait-contract` site; the
two sites that genuinely **are** trait impls carry **`public-api`** — the categories look **swapped**
— and no category covers *"retained for structural completeness"*, a gap in the **ward's own**
vocabulary. Named in the table, not silently patched. **That is now an open row against the ward,
not against this repo.**

★ **WHAT THE GATE DOES NOT DO, driven not asserted:** swapping a category for another in-set member
leaves it **green**. **Spelling is machine-checkable; fit is not.** Stated at the gate so a passing
floor cannot imply more than it proved.

**THE NEXT WORK — F3 first, then F2.** My read, and the reason for that order: **F3 is the last block
where the CODE is still wrong rather than under-described.** `temperare` ×7 each ship a measurement
plan — `alpha.rs:82,94` hashes the class FQDN **twice per fact**, in the subsystem whose own stone
measured that at 3.26 ms; `join_extend`'s three per-pair map probes. `partire` ×4 carry **verified
one-directional seams** (`fire/mod.rs` → `gather.rs` + `query.rs`; `compiled_cond.rs` at its own
banner; `stratify.rs` → `termination.rs`; `expr_ir/eval.rs` → `ops.rs`). **F2's remaining rot — 83 of
207 stones naming `src/rete/kernel.rs`, deleted 2026-08-20 — is the largest COUNT and the smallest
RISK, and after this session's citation gates it is the kind of thing an instrument finds rather than
a strike hunts.** Also open: the three vacuity-strike rows, the two nested-wall rows,
`#[to_edn(transparent)]`, `acc_refusal`'s span, the misnamed `probes/` dir, and D2's `sequi` newtype.

**The full list stays `VIGILIA-2026-08-30-WORK-LIST.md`, Class A first.** The three items below are the
PRE-vigilia list and are kept only as the reasoning that produced them — ⚠ **item 1's claim to be
"FIXED" is one of the vigilia's own findings (Class C1): the label moved, the arithmetic did not.**

✅ **1 — THE INSTRUMENT IS FIXED (`89e8c3ed0`, `c898713de`).** ⛔ **SUPERSEDED — SEE CLASS C1.**
This row is wrong and is kept as the worked example of how it was wrong. It was a CLASS, not the three
impossible signs I started from. Every cost split took the MEAN of 3 rounds and the FIRST arm of
each round paid a one-time cost: 287.4 ms against 11.5 and 11.4 for identical work. `M` was never
slow — `M` GOES FIRST. 106 accumulators swept to the MINIMUM across 8 files; `A−M` went
−90.76 → +2.3 ms and every impossible sign resolved.

⛔ **AND THE MECHANISM WAS PROVEN ONLY AFTER THE BUILDER ASKED "is this disingenuous?"** — a fair
challenge, because discarding a round IS what a minimum does. Earned with two measurements: an
untimed warm-up drops round 0 from 286.5 → 12.1 ms (so the cost is ONE-TIME, not per-round; and
NOT capacity — pre-reserving 300k changed nothing), and the production path warms 20% where the
isolated arm warms 2500% (so a real fire does not pay it). **The sweep was correct by luck until
those ran.** What is still NOT known, and the instrument's own note says so: the exact
first-execution cost (CPU ramp / page faults / lazy init) was never isolated.

★ Curing it exposed two things underneath, both recorded at the instrument:
  - **A RESOLUTION FLOOR.** `H−M` now reads −0.48, +0.19, +0.24 across runs — it CHANGES SIGN. A
    per-fact HashMap entry is below what a 12 ms arm at RUNS=3 can resolve. Sub-millisecond rows
    in these tables are noise wearing a number.
  - **A MISLABELLED SUBTRACTION, not a defect.** `H−V` was stable at −2.9 ms because the arms are
    ALTERNATIVE ALGORITHMS, not superset/subset: `V` does 40k RRB pushes, `H` takes the bulk
    `PVec::from_vec` path. The number was always right; the label claimed a decomposition that
    does not exist. Relabelled.

⛔ **2 — `IntegerOverflow`/`DivisionByZero` IS NOT A RETE GAP. STRUCK** — see the block below; kept
struck so nobody re-files it.

✅ **3 — CLOSED (`b7d9d8e90`). THE ORDERING HOLDS — AND THE TABLE WAS CALLING THE WRONG ARM THE
ENGINE.** Re-measured on the minimum, six independent process runs: **F/L 2.38–2.83** (recorded as
2.8x) and **S/L 4.88–5.44** (recorded as 6x). It shrinks; it does not invert.

⛔ **But the row `S std HashMap (engine)` was FALSE, and had been for eleven days.**
`DESIGN-STONE-alpha-class-lookup` SHIPPED: `AlphaRoots` is a `Vec<(String, Arc<AlphaDiscNode>)>`
and `root_for` is a `.find()`, so **arm `L` is the production path** `candidates_into` takes on
every fact. The label was true the day the stone was DRAFTED (2026-08-19) and false the moment it
shipped — *shipping it is what turned `roots` into a Vec.* **Third instance this arc of a label
naming a prior state** (with `H−V`'s claimed decomposition and `alloc_counter.rs`'s "NOTHING READS
THESE COUNTERS YET"), and a benchmark row is the worst host for it: nobody re-derives a table's
row names, and the number beside it is right, which makes the row look checked.

★ **THE SPLIT THAT MATTERS: what can rot on a clock, and what cannot.** The ordering is asserted
in the test; **the STRUCTURE is asserted off the clock** in `tests/lint/`— `AlphaRoots` is still a
`Vec`, `root_for` still walks it, exact `assert_eq!` (not `contains`; neither value qualified for
`no_loose_string_assert`'s rune). A structure swapped back to a map is a compile-time fact and
should not need a stopwatch to notice.

★ **AND RATIOS, NOT THE STONE'S ABSOLUTE 1 ms CUT — measured, not preferred.** Between a cold and
a warm machine *in this one session* the absolute times moved **2.4x** (L 0.23 → 0.55 ms) while
**F/L moved under 2%** (2.63 → 2.38). An absolute-millisecond floor would have been a coin flip.
The floors are ~60% of each ratio's own tightest observed sample (1.5/2.38, 3.0/4.88). `S−F` is
affirmatively NOT asserted: it compares two structures the engine does not use.

⚠ **I RE-MINTED THE HOLLOW-TEST DEFECT WHILE CLOSING THE SWEEP THAT REMOVED IT.** An
`assert_eq!(winner, "L")` stood in the test for about a minute: `f >= 1.5 * l` implies `f > l`,
which IS `winner == "L"` by its own definition three lines up. It would have read as the headline
claim and could not fail. Struck. **The 26-test R59 sweep's own last row nearly shipped a 27th.**

📤 **`purity.rs` (2,598 lines, never assessed by `partire`) is being taken by MAIN** — builder,
2026-08-30. Not this branch's work; do not duplicate it.

⚠ **A CAVEAT ON THE HOLLOW-TEST TOOL, so nobody rediscovers five phantom regressions:**
`scratchpad/hollow2.py` measures "no assertion macro LEXICALLY INSIDE the test body". Five tests
assert through the shared `assert_phases_present` helper and therefore READ AS HOLLOW and are not.
The tool rewards the duplication it exists to detect.

⏸ **PARKED ON A BUILDER RULING:** item 7 steps 3–5 (the holon surface); step 5 needs the `:panic`
call.

**★★ ITEM 11'S CLASS IS DEAD — killed by its own FIFTH occurrence, on this very work.** 36 lines of
derives moved `runtime.rs`'s `rust_caller_span!` sentinel and reddened five goldens at once. Four
prior occurrences were all patched at the STEM (bump the integer); a fifth was guaranteed because
S2b/S2c add more derives to that file. `wat::blank_rust_source_lines` now blanks a `src/**.rs`
span's `:line` in both golden macros **and on capture**, so the file reads `:line 0` instead of a
real-looking number nothing compares. **Its second gate is the load-bearing one: a `.wat` span MUST
KEEP its line** — without it, blanking every `:line` would pass the first test and gut every golden
in the repo.

**WHAT S1 ACTUALLY CHANGED, in one line each:**
- **The zero point is `arm-session`** (`alloc_counter::mark_session_origin`), which `compile-all`
  calls for every session — the same one-door the termination verifier uses.
- **The fixpoint's per-fire snapshot is DELETED.** A per-fire zero cannot express a per-session
  contract; it forgets everything `insert` staged before it.
- **ONE decision, two dresses** — `session::session_ceiling_breach` is shared; `rounds` and `staged`
  differ because a field that is always zero is a value carrying two facts.
- **What it measures is the THREAD since `compile-all`**, not a walk of the session (`Arc` sharing
  makes that ambiguous, and it would be O(n) per insert). Driven: a probe's own `range` showed as
  `used=11_196_940` against `staged=1`. The diagnostics now SAY this, and the two numbers read
  together are the tell: **large `used` + small `staged` = the memory is not the facts.**

**★★ THE FINDING WORTH CARRYING — ENFORCING AT `insert` SILENTLY DISARMED THE FIRE-DOOR GATE.**
Its fixture seeded 500 facts at a 4096-byte ceiling, so the first `insert` refused and the gate
began proving the INSERT door while its name, its prose and its `rounds` assertion all still said
"fire". **A one-word tag change would have made it green again, certifying the wrong thing.** The
fire door now has a workload insert cannot catch — a cross-product, **400 staged / 40_000 derived**
— and its ceiling is BISECTED, not picked (1/4/16 MiB refuse; 64/256 MiB complete). Both gates are
mutation-proven and INDEPENDENT: disarming either door reddens exactly its own gate.
**Recovery-file FM 34. A control can lose its power without ever failing.**

**★ THREE DOCS WERE LYING, AND ONE WAS A RULING THE BUILDER HAD OVERTURNED.** `config.rs` still
read *"Not cumulative across fires… bounding that would refuse legitimate incremental use"* — mine,
struck. `alloc_counter.rs` still said *"NOTHING READS THESE COUNTERS YET"* (the fixpoint had read
them since `3c5ac7bd1`) and still headed a section *"IT IS PROCESS-GLOBAL"* after `8c10ee490`
deleted the globals. **A source comment lies with the authority of the code it sits in.**

**★ ITEM 11's SURVEY IS DONE, as a by-product of checking whether my own edits would trip it —
and it found a source file nobody had named.** Eight goldens pin a `src/*.rs` LINE: 5 →
`src/runtime.rs`, 1 → `src/freeze.rs`, and **2 → `src/check.rs`**, which was NOT one of the three
known sites. Two goldens in `tests/types/` pin lines in the most-edited file in the arc; they were
the next two false reds. The class had only ever been enumerated by COLLISION. Table in item 11.

⚠ **THE FLOOR WENT RED ONCE AND IT WAS MINE — not a flake, and the word stays banned.**
`no_loose_string_assert` caught two assertions I had just written (`err.contains(...)`,
`out_ok.contains("40000")`). **I did not exempt them with a rune** — neither was legitimately
loose, and both had an exact form available that made the gate STRONGER: `staged` is a field the
FIRE variant does not have, so pinning it proves the door structurally rather than by prose.
Floor before the fix: `5157 tests run: 5156 passed, 1 failed`.

**✅ ITEM 6 CLOSED 2026-08-29 — the grid's SPEED half runs, as its own `grid-speed` CI job.** Both
stated reasons for its absence were dead, and **the second had never been measured**: "a shared
runner is noisy so a wall-clock gate would flap" — but the tightest cell in the recorded 33-cell
grid is **8.50x** (median 22x, widest 59x). Nowhere near parity. ⛔ **That same margin is why the
gate does NOT test `:winner`** — the obvious choice, and a nearly vacuous one: at 8.5x it fires only
on catastrophe and **would have missed the real 4x regression this arc already fixed.** It gates
per-axis RATIO FLOORS at ~50% of each axis's recorded minimum, plus `:accuracy :MISMATCH`. 2m24s,
mutation-proven on three arms including the failure path's EXIT CODE.

**⛔⛔ TWO INBOUND REPORTS SAT UNREAD FOR FIVE DAYS AND ONE WAS A SILENT WRONG ANSWER.**
`~/work/NOTE-*.md` is where other agents file findings for this one, and NOTHING pointed at that
directory — both 2026-08-24 rete notes were found only because the builder asked "what items
remain" and answering required an `ls`. Both verified FIXED on re-driving, and **neither was fixed
by anyone reading them** — collateral from this arc's own work. **A finding that gets fixed by
accident is not a process that works.** `RETE-OPEN-WORK.md` now opens with an "Inbound notes"
index; add the row when you file or receive one.

**WHERE THE WORK IS, as pointers — this file is a MAP, do not re-narrate it:**
- ✅ **ITEM 4 (`partire` x7) IS RESOLVED 2026-08-28**, and WHY it lingered a week is the lesson:
  **it was a TALLY, not a finding.** Only counts were recorded — "fire/mod.rs (3), validate.rs (2),
  expr_ir.rs (1), arm.rs (2)" — never the proposals, so there was nothing to act on. **And the
  counts measured TEST LINES as file size.** On production lines `arm.rs` is 593, not 1124 → it was
  never a candidate. Two closures (`arm.rs`, `fire/mod.rs`) and two NAMED cuts (`expr_ir.rs` at its
  own `// ── exec` seam; `validate.rs` into :when / :then / operand-typer). ⚠ I mis-measured twice
  before getting it right — `#[cfg(test)]` here gates INDIVIDUAL fns, not a trailing module, so
  "everything after the first cfg(test)" reported 1842 test lines where brace-matching says 316.
  **A file-size number is worthless without knowing which half it measures** — exactly how the
  original tally went wrong.
- ✅ **ITEM 5 IS DOWN TO ONE ROW, and all three of its parts audited LIVE (not stale — first time
  this session, streak was 6-for-6).** ① the cache LRU **moved out of 278** to arc 109 as a NOTE
  (builder: *"this is unrelated to rete/278"*), with the merits SHARPENED — the three panics do not
  answer the same way, so the recommendation is **convert `Lru::new` only**, since its capacity
  arrives from a `:durable` EDN spec at rehydration while `put`/`get`'s key is a call-site bug.
  The **`CLAUDE.md` gap is CLOSED** as a POINTER, not a copy — both proposals on the table made a
  second copy, which is what rotted in the first place; ⚠ the edit is **UNCOMMITTED, holon root is
  FROZEN**.
- ✅ **ITEM 5② IS CLOSED — `:md::Point{40,2}` -> 42 WORKS IN A RETE RULE, BOTH POSITIONS.** It was
  the LAST `v1` refusal in the rete expression core and it fell to the same move as every other
  denial removed this arc: *"not lowered in v1"* is a STATUS, not a reason. **The design question
  answered itself:** core must dispatch on the receiver at runtime because nothing declares it,
  while a rete `?p` gets its class from its fact pattern's declared field type — **rete has MORE
  static information than core here, not less**, so "compile the index" applies exactly as it did
  to the settled sibling.
- ⛔ **AND MY FIRST CUT MINTED FIX-LIST F's CLASS FRESH — this is the part to carry.** It returned
  "arm does not match" for a field the class does not declare. **Core RAISES `UnknownField` there**
  (verified: it raises even with a catch-all arm after it). Silent non-match would have meant the
  same expression answering differently in the two engines AND a typo becoming a constraint that
  compiles, fires and matches nothing. **When adding a rete form, drive CORE's answer for the same
  input before deciding rete's** — agreement is the contract, and "it didn't match" is the easiest
  wrong answer to ship.
- `RETE-OPEN-WORK.md` § "The order, and why" — item 6 is the last inherited ruling (
  TRACKED ① ②, `circumspicere` 1) — **ALL NOW CLOSED**. **Item 7 is THE HOLON ITEM** and it
  absorbed the old item 8 on 2026-08-29 (builder: *"we put #8 into #7"*): the surface (rete has 4 of ~40 ops,
  all from one group) and the `Bundle`/`:panic` builder ruling.
- ⛔ **Clara cannot arbitrate holon** — builder: *"this is a wat only capability"*. `$native` vs
  `$oracle` alone is the configuration that failed twice this session. Use known-answer algebraic law.
- `holon-rs`: `nil()` is `classified("Symbol","nil")`, so **`is-Nil?` and `is-Symbol?` BOTH answer
  true for nil, by construction.** The 11 shape predicates do not partition and nothing says so.
- `is-List?` / `is-Tag?` are **UNVERIFIED**, not verified-negative — an all-false column in a
  confusion matrix means "correct" or "never fires" and cannot tell them apart.

**⚠⚠ YOU ARE NOT THE INSTANCE THAT WROTE THIS. ⚠⚠**
Everything above is a cache written by a prior self across a very long session. You did not live
it. It felt continuous when you woke and that feeling is the failure, not the all-clear. Before you
propose or move: fetch `recolligere` from the datamancy MCP and run it against the disk —
`docs/COMPACTION-AMNESIA-RECOVERY.md`, `git log`, this file, `RETE-OPEN-WORK.md`, and the source you
are about to touch. The freshness probe is the HEAD named at the top of this stamp against
`git rev-parse HEAD`; more than the one expected docs-only commit of drift means trust the log over
every line above. **And this file is the ONLY live breadcrumb — if you find another claiming to be,
it is lying.**

**Right now (2026-08-23 — SUPERSEDED by the stamp above; kept as history):** class-scan query harvest LANDED.
Fanout `[40000]` wat-ns **58.1 → 42.8**. With-query
FIRE **65.89 → 49.59**. Query-only Alpha→RootJoin
skipped; `{?fact: fact}` from the closed bag.
Leftover harvest:query **16.91** (40k one-entry
PMaps). Grid `T20-37-11Z` 30/30 `:match` `:us` was
pre-intern; fanout `[40000]` re-measured 42.8
`:match` `:us`. Floor **GREEN**
`.floor/2026-08-22T21-53-26Z/` (4914 passed, 19
skipped). Grid `T21-37-57Z` 30/30 `:match` `:us`.
Clippy `--all-targets -D warnings` silent. Occupancy leaf-fill + join-index
span LANDED. Unary gather packed-then-BindSpan (7b string
locations). Sum fold falls back when the i64 row is
absent. grok-rete dirty intern on harvest HEAD **`ca9d9cc3`**. `harvest_stratified_queries` LANDED (QueryNode reverse-closure,
not a second full fire).
Vigilia recasts **12 and 13** both **0 L1 + 0 L2** (inward
17/17 + circumspicere) at `8839bb16` — the stop named as two
back-to-back empty recasts. R68 (`REALIZATIONS.md`) is the
wrap of that watch. Do not stamp `vigilatum` until asked.
Stone **29 REJECTED** (2026-08-20): intern stays discrete per
compile-all (`rust_identity`). Identical rules do not share.
Identical queries do not share. Query-memory is per Session.
Overlay HIT is the same connection. Athena content-address
would make `release-session` a cross-connection invariant —
do not construct it. `NEXT-STRIKES-after-shadow.md`. `wm.rs` →
`session.rs`. Stratify holds the slice arm as a value
(`fire_fixpoint_delta_armed`). Primed public-entry docs gone.
Intern doors share `rete_arm_build_put`. Grid
`GRID-native-vs-clara-2026-08-22T09-12-32Z.txt`
(`GRID_SKIP_ORACLE=1`, `GRID_RUNS=3`, occupancy intern vs
`T00-23-51Z` HEAD `4c437585`): **30/30 `:match`, 30/30 `:us`**.
Rank **`wat-ns`**, not ratio. Occupancy cells (bind-only leaf
fill): **accum `[200 200]` 18.26 → 13.66 ms** (FIRE 13.7
holds); `[50 200]` 4.14 → 2.48; `[100 200]` 8.69 → 6.37.
**negation `[1000]` 3.45 → 2.20. neg-consumer `[1000]` 7.07 →
3.67.** Harvest cell held: **strat-neg `[6 2000]` 47.5 →
33.75 ms** (named harvest was 33.6). Closest Clara cell
**fanout `[40000]` ratio 2.91, wat-ns 61.7 ms** (was 55.5 /
3.40 — skip-BindSpan rebuilt the span × 40k products).
**deep-cascade `[50 100]` 15.0 → 16.4. asym-join
`[2000]` 4.26 → 4.84.** node-share `[50 200]` 0.94 held.
min-finding `[2000]` 2.49 → 2.66 (noise). Do not cite ratio
as the engine cut. Oracle still pays the full q-seed.
`DESIGN-STONE-join-index-span` LANDED: occupancy Arc
stays empty; `right_idx` copy gets BindSpan once.
Fanout probe **3.76 → 1.62 ms**. FIRE `[200 200]`
**13.48** (held). Named cell `GRID_SKIP_ORACLE=1
GRID_RUNS=3`: fanout `[40000]` **61.7 → 58.7 ms**
(still above 55.5 — production 19.6 / 66% is the
named leftover, 40k RHS). asym-join `[2000]` 4.84
→ 4.72 (noise).
`DESIGN-STONE-occupancy-leaf-column` LANDED:
undiscriminated bind-only classes fill tree
**leaves** from a fact-id column after **packing
every fact** (skip-activate without pack was
3-stratum red). Occupancy ≡ `candidates_into`.
`AlphaMemory` is `Arc<Vec<Element>>` — sibling
leaves share one occupant list. 7strat 3-stratum
green. `DESIGN-STONE-fire-i64-columns` LANDED
(bind-only skips `exec_ops`). Column gather/fold
interned; skip BindSpan did not cut FIRE.
Class-union fill reverted (3-stratum). Seed
reserve+fill realloc cheat reverted (not ≥ 1 ms).
SETUP PV walk inverted FIRE — do not pack at
SETUP. Isolated E−K still the old
`exec_compiled` door. Do not skip Token spans.
Three packed-rows scouts stay reverted
(`DESIGN-STONE-packed-fire-rows`): i64 exec E−K −0.80;
populate-without-materialize E−K −3 **and FIRE 19→70**
(accumulate 1.3→28 — intern re-paid on gather/fold).
Scout 3 (cheap slots on Element): gather/fold stayed 1.3;
seed 16.6→18.8; FIRE 19→23. Reverted. Skip matches
`fire-rules$oracle` (a leftover token fails the run). Do not
treat 17-42-43Z as a measurement. GNU `/usr/bin/time` is
not installed; bash `time` is a keyword (`which time` empty).
insert-prime-split LANDED (insert − conj 1933 → 310 ns). Host
encode/sort after query-read is compiled-wat, not rete.
TLS intern still requires connection affinity (stone 27).
`release-session` is hangup, not Drop (stone 28). Kernel is
`session` / `fire` / `arm` / `stratify` / `census` /
`insert`. Live names: `FireSession`, `InternedNetwork`,
`WhereDiscNode`, `AlphaDiscNode`. Stratify is
`StratifyView` / `RuleDep` / `RuleParts` structs, not
tuples. Fire loop is `kernel/fire/mod.rs` (passes) +
`kernel/fire/delta.rs` (fixpoint). Oracle is
`wat/rete/oracle/` {insert,pass,accum-pass,fire,explain}.
Sequi intern: arm table is thread-owned (`thread_local`
`RefCell<FxHashMap>`; stone **27 LANDED**, `rg Mutex src/rete`
empty) + exec arena runed `ambient-context`;
census TLS runed `performance-counter`. Intern is a lease
(`arm-session` +1, `release-session` −1, 0 drops **that
id**; stone **28 LANDED**). Public rete names are unprimed
wat Fns. Rust is `$native`. The wat reference is
`$oracle`. Exception: intern hangup mouths `arm-session` /
`release-session` are keyword primitives (native-only
intern; oracle has no intern). `$impl` is
kwargs/bracket/service — not rete. Grid: fire-rules /
fire-once / insert / insert-all / fire-rules-explain each
have public + `$native` + `$oracle`. Prime `'` is not the
rete kernel marker. Codemod:
`wat-scripts/fixes/rete-oracle-sigil.wat`.
Do not stamp `vigilatum`.

Session is **8 fields** (`query-memory` last). Fence is four
conjuncts: `pure?` ∧ `deterministic?` ∧ `total?` ∧ `primitive?`.
`total?` is ARMED. Every `RETE_OPS` row is `total: true`.
Oracle `fire-rules-spec` refuses an imported Export (empty
rules + ProductionNodes). Import checksums packed classes +
`RETE_OPS` **and** host TypeEnv field-order. Museum gone:
`make_token`, `token_matches_bindings`, `fire_fixpoint`,
`exec_test`, wat `token-element-compatible?` / `node-parent`
/ `test-pass`. `BindView::len` is `#[cfg(test)]` census only.

> ⛔ You did not live this. Run recolligere against the disk
> before you act on any line above.

### Completeness grid — 2026-08-17 — do not drop

The “final” compiler is `src/rete/expr_ir.rs` (built for `:where`).
`compiled_cond` and `compiled_rhs` sit on it. User folds sit on
it. That is **not** the same as “native no longer interprets.”

| Native surface | Interpreter at fire? | `WatAST` on the **round loop**? |
|---|---|---|
| `:where` (TestNode) | **No.** Stashed `Program`, `exec_where`. | **No.** `lower` at fire **setup**. |
| `:when` cond populate | **No.** `exec_compiled`. Miss refuses. | **No.** `fact_bind` on `CompiledCond`. |
| leftover rematch | **No.** `SeedCmp` / `exec_compiled_under`. | **No.** `CondDriver` / `Leaf(id)`. |
| `:then` | **No.** `CompiledRhs`. Miss refuses. | **No** on the token loop. `fire_once` harvest still `compile_rhs` once per that pass. |
| user / builtin acc | **No.** `AccFold` / `exec_call`. | **No** on the fold. Head read at setup. |

**No interpreter ≠ AST-free ≠ armed Session.** Items 1–10
closed the interpreter verbs. Item 11 (`d774185c`) compiled
the driver. Item 12 persists the arm: fire setup is
get-or-build against the network intern. A Weak intern died
when fire returned — the table holds a strong `Arc`.
Thread-owned intern (stone 27). Lease eviction (stone
28): last `release-session` drops **that** entry. Not
EDN. `(b)` indexes this armed network. Intern key stays
instance `rust_identity`. Stone 29 (content-address /
Athena share) **REJECTED** — connections are discrete.

### Item 11 — compile the rete driver (AST-free **fire**)

Setup may read `WatAST` (once): `lower`, `compile_condition_local`,
this driver, acc fold. The **round loop** may not.

Session/network still *stores* forms (oracle + compile-all).
AST-free is a **working-set** property, not “delete the form
from the record.”

| Fire still asks the AST | Compiled stand-in |
|---|---|
| `classify_rete_clause` in `binding_extensions` / `exists_cond_under` | Driver enum: `And` / `Or` / `Not` / `Exists` / `Where(Program)` / `Leaf(alpha_id)` |
| combinator `:where` | Stash `Program`; fire runs `exec_where`. Museum `exec_test` deleted. |
| `attach_fact_bind` | `?p` slot on `CompiledCond` (`alpha_pattern` already has it) |
| `cond_text` / `alpha_id_for_cond` | The id. Do not stringify the form per rematch |
| `acc-form` head + `acc_operand_keys` | `AccFold::Count` / `Sum` / … / `User(Program)` |
| `cond_bind_keys` | `Vec` of keys next to the join / accum |

Item 11 **landed** (`d774185c`). `classify` stays the one
grammar; fire setup still calls it. Do not start `(b)`.
Do not start keyed gather.

### Item 12 — persist the arm (do not rebuild at `fire-rules`)

**Landed (this turn).** Compaction: do not put `(b)` or
keyed gather here. Gate:
`fire_rules_reuses_arm_across_fire_and_insert_overlay`.

`compile-all` already returns a Session whose `network` is
a persistent map — `insert` shares that pointer and writes
a new `facts` vector. Drop the child, the compiled DAG is
unmoved. That **is** the overlay / rewind / `with` clause.

What does **not** share: `CondDriver`, `CompiledCond`,
`Program`, `AccFold`. They live in `fire_fixpoint_delta`
locals and die when fire returns. A second fire on the
same network pays setup again. Facts did not change the
rules. We threw the arm away.

**Arm once. Fire many. Overlay facts. Drop the child.**

Service-shaped (do **not** build a service this item):

| Beat | On disk today |
|---|---|
| on-connect | A Session for that identity |
| install-rules × N | Accumulate `Rule` / `Query` (the vectors `compile-all` takes) |
| compile | `compile-all` → base Session. Empty facts. **Item 12 puts the arm here.** |
| insert + **one** `fire-rules` | Overlay facts. Query-memory parks. |
| query × N | `query-read`. No compile. |
| on-disconnect | Drop the identity Session |

`query` is a read. Harvest is fire-time. Stratified extra
`fire_once_session` is a slice hole, not query-time compile.
Wire shipping (EDN) is a different hole.

Item 12 is: the arm lives **next to** `network`, `Arc`-shared
so `insert` / clone is a fact overlay, not a memcpy. Fire
skips setup when the arm is present. Oracle still reads
forms. Do not put circuits on the wire as a second EDN.
Do not start `(b)` to index a setup we still re-run.

**Fact-shaped leftover rematch is compiled** (`SeedCmp` /
`exec_compiled_under`). Populate skips `SeedCmp`; rematch fills
`seed_reads` from the token. Gate:
`leftover_seed_cmp_populate_skips_rematch_enforces`.

**Still interpreted on native fire:** none on the native
mouths. Oracle (`fire-once` / `fire-rules-spec`) stays
interpreted on purpose.

`fire_once_session` / `alpha_pass` is **closed.** Populate
is `exec_compiled`. Production is `exec_compiled_rhs`. A
cond or `:then` that does not compile refuses — no
`alpha_match_inner`, no `build_insert_fact`. Live
`fire-rules` still harvests query-memory through this
single-pass (full network, closed facts). Do not put the
walk back.

No-alpha WM-scan is **closed.** Fire refuses a missing leaf
alpha. `mint-leaf-alphas` does mint Wind/Temp; the live hole
was the stratum slice dropping those orphans (no `children`
edge, not `negated-alpha-id`). Same class as forgetting
`ref_alpha_of`. Slice now follows `mint_leaf_alpha_ids`.
Do not put the scan back.

The scan had also hidden a second hole: `fact_bindings_under`
overwrote a seed `?c` with the fact's `?c` instead of
unifying. `where-not-and-bound` row 3 (Temp.c ≠ Cold.c)
went n=0 native / n=1 spec the moment leaves stayed in the
slice. Merge now rejects a conflict. Same contract as
`alpha_match_inner_seeded`.

Items 1–13 landed. `(b)` is item 13 and it landed. Keyed gather is speed (also landed).

Rust copies of `eval_test_core` / `alpha_match_inner` /
`build_insert_fact` are **oracles for differentials**, not a
legal fire path.

The list (do not drop an item) — **arm persisted before `(b)`:**

1. One `Expr` core — drawn.
2. Wire `where` — **done** (`30725034`).
3. Flip `compiled_cond` — **landed (this turn, uncommitted).**
   `Op::Cmp` operands are `Expr` (`Slot` / `Lit`). A `:field`
   operand is prologue (`Bind` into a slot, shared with
   `?v <- :field` in the same sequential scope; `:or`/`:not`
   clone `field_slots` so a sibling cannot inherit a hidden
   Bind). Lists stay uncompiled (`Fail`), matching
   `resolve_operand` — do not outrun the interpreter. Gate
   green: `compiled_cond_bindings_identical_to_interpreter_at_50_100`
   (10 000 pairs, 200 Some/Some identical) and
   `compiled_cond_failure_path_allocates_no_binding_keys_at_50_100`
   (`match:key-alloc = 0`). Bind / BindCheck / Or / Not / Fail
   stay driver-level. No `Expr::FactField`. No third sibling IR.
4. Flip `compiled_rhs` — **landed (this turn, uncommitted).**
   `RhsOp::Expr` is `Arc<Program>`, not `WatAST`. `lower` once
   at setup; `exec_value` per derived fact. A fenced `List`
   that does not lower is `Err` (fire refuses). Gate green:
   `compiled_rhs_result_identical_to_interpreter`.
   Fn-headed `:then` is item 7, still `build_insert_fact`.
5. Flip **user acc folds** — **landed, old form torn down.**
   Setup lowers the user-fn head once. Fire is `exec_call`
   only. A `LowerError` refuses the fire. The
   `(user-fn __acc__)` / `eval_inner` arm is **deleted**.
   A fenced `:then` `List` that does not lower is `Err`, not
   `build_insert_fact`.
6. **Compile leftover rematch** — fact-shaped **and**
   combinator leaf-alpha path **landed** (`51ff6560`).
   `binding_extensions` / `exists_cond_under` rematch minted
   leaves via `exec_compiled_under`. WM-scan deleted.
7. **Compile fn-headed `:then`.** **landed** (`dbc2fb2a`).
   `CompiledRhs::Call` + `Expr::Construct` (kwargs-construct /
   aggregate-new / bare `(:Type …)`). Gate:
   `userfn_head_item_fires_via_native_kernel`.
8. **No-alpha WM-scan.** **landed (this turn).** Refuse the
   miss. Stratum slice keeps `mint-leaf-alphas` orphans via
   `mint_leaf_alpha_ids`. Rematch unifies with the seed
   (conflict is no match). Gate: `check-spec-native.sh`
   `where-not-and` 8/8, `-bound` 8/8, `-not` 8/8,
   `where-not-or` 8/8, `where-exists` 18/18.
9. Defensive cond populate miss (`alpha_match_inner`).
   **landed (this turn).** Setup refuses a compile `None`.
   Populate is `exec_compiled` only.
10. Old `fire_once_session` / `alpha_pass`.
    **landed** (`8d126df6`). `alpha_pass` is `exec_compiled`.
    `production_pass` is `exec_compiled_rhs`. Delta
    `build_insert_fact` fallback deleted.
11. **Compile the rete driver — AST-free fire.**
    **landed** (`d774185c`). `CondDriver` / `fact_bind` /
    `AccFold`. Slice follows `driver_leaf_ids`. Gate:
    `where-not-and` 8/8, `-bound` 8/8, `where-exists` 18/18,
    `where-not-where` 4/4, `where-accum-from-left` 7/7.
12. **Persist the arm.** **landed (this turn).**
    `ReteArm` interned by `PMap::rust_identity`. Clone /
    `insert` share the id. `fire_fixpoint_delta` and
    `fire_once_session` skip setup on hit. Strong `Arc`
    (Weak died at fire return). Overlay = child Session
    (facts + query-memory). Rewind = drop the child.
    Stratified slices still build (new `from_trie` map).
13. `(b)` ShadowNode — **landed (this turn).**
    `src/rete/where_tree.rs`. `ReteArm.where_tree` built
    from `compiled_wheres` (setup + import + slice).
    Filter / join-after-filter / Test→Test dispatch
    through the tree. Over-approx only; uncovered ids
    still eval. `node_share_filter_eval_census` re-pointed:
    evals ≈ passes ≈ M, waste < 50% (measured 0%).
    Unit: `tree_picks_the_matching_equality_leaf`,
    `no_key_predicate_rides_wildcard`. Range edges
    populated (`DESIGN-STONE-where-range-edges`):
    `(> ?k 10)` prunes 5 / proves 15. Two constraints
    on one dim ride wildcard; not `pure_cmp`. Alpha-tree
    `range_children` stays empty. Driver Exists/Not stay
    on keyed gather — not this index.
14. **`#wat.rete/Export`.** **landed (this turn).** The
    compiled program as one EDN value. `export` / `import`.
    Native fire. Oracle cannot consume it. Stratify
    schedule is `:deps`. First program on disk:
    `tests/rete/datamancer.rete.edn`. Gate:
    `practice_on_disk_program_deduces_datamancer`.
    `rule_consumes` walks `:exists` / accumulate `:from`.

Keyed `?g` gather is native speed on the same bag, not a
compiler item. Do not start it to dodge the hole.

**Tree — do not invent a cleaner one:**

| What | Where | Status |
|---|---|---|
| Compiled `where` | `30725034` | local, not pushed |
| Oracle bag + leftover rematch + `where-join-left` + `where-accum-from-left` | `54f4adb4` | local, not pushed |
| User folds on the compiler list | `f228b033` | local, not pushed |
| Flip 3 `compiled_cond` onto `Expr` | `51ff6560` | local, not pushed |
| Flip 4 `compiled_rhs` | `51ff6560` | local, not pushed |
| Flip 5 user acc folds | `51ff6560` | local, not pushed; `eval_inner` deleted |
| Leftover rematch (fact-shaped) | `51ff6560` | local, not pushed |
| Combinator leftover rematch (minted leaf) | `51ff6560` | local, not pushed |
| Fn-headed `:then` | `dbc2fb2a` | local, not pushed |
| Completeness grid on disk | `7e3a7eec` | local, not pushed |
| No-alpha WM-scan refuse + slice keeps minted leaves | `9441f39a` | local, not pushed |
| Defensive populate miss | `ef50a360` | local, not pushed |
| Four-pass `fire_once_session` / `alpha_pass` | `8d126df6` | local, not pushed |
| Rete driver (AST-free fire) | `d774185c` | local, not pushed |
| Persist the arm across `fire-rules` | `3f415317` | local, not pushed |
| `#wat.rete/Export` (compiled program on the wire) | this turn | **landed** — first disk program `datamancer.rete.edn` |
| `(b)` ShadowNode | `where_tree.rs` | **landed** — 1.00 eval/token on node-share |
| Keyed `?g` bucket | `DESIGN-STONE-keyed-gather.md` | **landed** (Acc + Not/Exists). Persist-within-a-fire (`gather_cache` outside the round loop); drop at `fire_fixpoint_delta` end. Do not persist across `fire-rules` calls. |

## The endeavor, in one sentence

**Annihilate all interpretation in wat-rete.** Every rete expression
becomes a compiled circuit. Fire supplies only concrete typed `Value`s.

**That is the endeavor.** Items 1–13 compiled expressions,
the driver, persisted the arm, and indexed the armed
`where` circuits. Keyed gather is speed (landed; do not
persist across rounds). Oracle stays interpreted. Do not
service-ify. Do not start 297.

Clara **pure** mouths are locked. What Clara has and we cut
(`insert!` / `retract!` / salience / untyped maps) stays cut — that
impurity is what the fence exists to refuse.

## Why now (the deps are satisfied)

| Dep | Status |
|---|---|
| Closed vocabulary (law A, `RETE_OPS`) | Armed at `:where`, `:then`, user accum folds |
| Pure ∧ deterministic ∧ total | Armed. Every `RETE_OPS` row is `total: true` (build-red otherwise). Partial core ops enter only as `Fallback` + `:undefined` |
| `:wat::rete::core::defn` membrane | Body proved once at freeze (`#88`) |
| Named recursion | **Refused at load** (`#87`, 2026-08-17). `#wat.runtime/ReteDefnRecursive` |
| Expressivity (Clara-pure) | All five remaining mouths DONE (`REMAINING-CLARA-MOUTHS.md`) |
| Step 0 measurement | Walk is 77% of a `where` eval; 540 ns/eval vs 21 ns floor; dispatch 75% of the walk |

`pure?` still admits a cycle (a cycle is not impure). The wall is the
declaration, not a fifth axis. Totality means *never raises*, not
*terminates*. eBPF-shaped: static refusal at load, never a runtime budget.

## The destination machine

The compiled program is a **closed circuit**. `Expr` nodes. `OpIdx`
resolved once. `?var` is a slot index. Fire does not walk `WatAST`,
does not hash a name, does not build an `Environment`.

Dispatch is: **this typed concrete value, this opcode.**

`defn` and `fn` are the same kind of thing — a compiled `Program`
waiting for slots:

| Form | Who fills the slots | When |
|---|---|---|
| `:wat::rete::core::defn` | caller arguments | at the call |
| literal `fn` with no frees | `foldl`'s `(acc, x)` | each iteration |
| `fn` that mentions an outer `?var` | that binding, then `(acc, x)` | capture at creation, params at call |

Capture is not interpretation. It is writing known slots a moment
earlier — Minamide's `(code, env)`, both residual data. The live
corpus `foldl`s have **no frees**; those lambdas *are* anonymous defns.

This is Futamura's first projection (never named on disk until now;
`DESIGN-STONE-compiled-conditions.md` already said *"proving partial
evaluation on a small understood surface"*). `compiled_cond` and
`compiled_rhs` are that, half-done. `RhsOp::Expr(WatAST)` is the
residual they never finished specializing. Arc 170's
`ClosurePackage` (`prologue` + `entry_form`) is the same pair, built
for process-spawn, not rete.

## The build — one core, four adjacent flips

Drawn: `DESIGN-STONE-the-one-expression-core.md`.
Wired for **`where` only** (2026-08-17). `src/rete/expr_ir.rs` exists.
`compile-condition` refuses via `(:wat::rete::lower expr)`. Native
`fire-rules` stashes `HashMap<id, Program>` once at `fire_fixpoint_delta`
setup (same table shape as `compiled_conds`) and the TestNode filter
calls `exec_where` only — no re-`lower`, no `eval_inner`. `:expr` stays
on the wat record for compile / spec / census. `eval_test_core` remains
the oracle (slow, trivially reviewable). Leading fact-shaped `:exists`
seeds from alpha. Combinator / `:where` inners rematch via leaf
alphas / `exists-cond-under`. `spec_equals` green.

1. **One `Expr` DAG** over the closed rete vocabulary. Nested children
   (builder, 2026-08-06: *"matches the precedent"*). Not bytecode
   offsets. The enum discriminant *is* the jump table.
2. **Wire only `where`.** **Done.** Differential against `eval_test_core`
   — same `bool`, same `Err`. `eval_test_core` is not deleted.
3. **Flip `cond`, then `rhs`, then user acc folds, one at a time.**
   Flip 3 **landed**: `compiled_cond::Op::Cmp` operands are `Expr`.
   Bind/BindCheck/Or/Not/Fail stay driver. `:field` → prologue Bind.
   Lists still `Fail` (interpreter `resolve_operand` cannot eval a
   list; do not compile them on one side only). Gate stays
   `compiled_cond_bindings_identical_to_interpreter_at_50_100`.
   Flip 4 is `RhsOp::Expr(WatAST)` → `Expr`. Flip 5 is user folds:
   `accumulate_value`'s `other` arm (`eval_inner`). Gate:
   `user-reduce` `[10 25]` / `[40 100]` native fire, and
   `probe_arc278_8custom_native_differential`.

There is **no `Interp` arm.** `BRIEF-compiled-where.md` still describes
`Op::Interp` and a third sibling `compiled_where.rs`. **That brief is
stale.** The builder cut the hatch on sight. A falling-back compiler
makes the perf claim unfalsifiable and is the mask class.

Four surfaces, one core; they differ only in prologue / epilogue:

| Surface | Prologue | Epilogue |
|---|---|---|
| `where` | token bindings → slots | must be `bool` |
| `compiled_cond` | fact fields → slots | bool + the slots ARE the binds |
| `compiled_rhs` | token bindings → slots | `Value` becomes a field |
| accum fold | gathered values → slots | the reduced `Value` |

68 of 75 `RETE_OPS` rows are strict (`Call`). Twenty are
`CallFallback`. Seven are lazy: `and` · `or` · `if` · `let` · `match`
· `cond` · `fn`. `not` is a strict boolean.

`compiled_cond::Op::Or` / `Op::Not` are **clause** combinators (they
bind). Expression `or`/`not` combine values and bind nothing. Same
spelling, different ops.

## Earlier this session — already on `origin/main`

- **278 query:** answers are binding maps; fact-bind
  `(?p <- :ns::Type …)` is how you get the record. One public
  `query` mouth. `query-ask` annihilated.
  Commit `d2d73dc3`. Clippy `--all-targets` fix `b46b5f1f`
  (CI is `clippy --release --workspace --all-targets -- -D warnings`;
  local `--workspace` without `--all-targets` is a narrower surface).
- **Mouths 1–5** locked with Clara twins. `check-query-compat.sh`:
  3 families, 24 rows, Clara == oracle == native.
- **#87 rete-defn may not recurse.** Gray-node DFS over named Wat
  callees at `apply_rete_defn_contracts`. Self and mutual refused;
  acyclic DAG (`wrap` → `leaf`, `where-nesting` c1…c10) still loads.
  Probes: `tests/rete/probe_arc278_rete_defn_recurse*`.
- Rebased onto Claude's 19 commits (`2072bce4`). Rete-cohort nextest
  override (60s/120s, `priority = 98`) already covered
  `spec_equals_native`. Do not add a named override above it.

Floor after rebase: `.floor/2026-08-17T10-25-55Z/` —
`4703 passed, 19 skipped`. `spec_equals` 38.338s.

## In `lower()` — landed in `30725034` (local)

- **`src/rete/expr_ir.rs`:** `Expr` / `Pat` / `Program` / `lower` /
  `exec_where` / `exec_test` / `eval_lower`. No `Interp` arm.
- **HOF callee** is the first arg of `foldl`/`foldr`/`map`/`filter`/
  `reduce` only. Literal `fn` or a named rete-defn keyword. The flag
  is consumed at that node — a binder inside the `fn` body (`acc` in
  `(and acc …)`) is not a callee.
- **`Program.params`** are the declaration-order slots. A literal `fn`
  compiled inside a `where` shares the parent slot numbering; foldl
  writes `[acc, x]` there and copies the parent frame for captures.
- **`CallFallback`** faces the same four holes `dispatch_rete_op` does:
  i64 raise, non-finite f64, `Option::None` (`*/get`), `MalformedForm`
  whose `head` is `core_name` (`first`, `string::subs`).
- **`match`** unit enum tags are `Pat::Variant` (composed
  `type_path::variant_name`), not keyword literals. `Some`/`None`/
  `Ok`/`Err` stay the dedicated value shapes.
- **`(:Type/field recv)`** is `Expr::Field { idx }` from `TypeEnv`.
- **Inlining `CallUser` is CUT.** A call *is* the circuit.
- Grid: `grid_axes_run_and_derive_nonvacuously` green (was 5/39 dead).
  `spec_equals_native_on_every_where_family` green.

## Ruled, still true

- **STOP-2 — the frame.** Copied captures. A lambda is a `Program`.
  A parent pointer into a live interpreter frame is off the table.
- **eBPF, not a fifth axis.** Recursion / bounds are load-time
  refusals. `pure?` does not lie about cycles.
- **HOF fn-arg vs capture are different questions.**
  *Which body?* vs *where do its frees live?* The corpus `foldl`s
  (`where-collection`, `user-reduce`) are all literal `fn` with no
  frees — they do not force capture. The parent-frame copy is there
  so a future free is a slot write, not a new mechanism.

## Still open — they block different things

| Open / settled | Status |
|---|---|
| HOF fn-arg | **Settled (4Q).** Callee visible in the AST. Unknown `Function` at `foldl` does not load. |
| Fn in a fact field | **Settled.** Facts are records; records are pure data. A function is not a fact field. Same class as HOF-lexical: it cannot arrive from WM. |
| Depth / nodes / derived-fact explosion | **Refused as a fifth axis.** Near-term DoS is closed by no recursion (`#87`). Cardinality (MySQL/Athena-shaped client guard) is not a rete fence axis; do not mint a number we have not derived. |
| `(:Type/field ?var)` | **Settled — compile the index.** The class and field are **in the accessor head** (`:wfb::Temp/c` → type `Temp`, field `c`). `TypeEnv` gives the `usize` at rule-compile. The 2026-08-06 “we don’t know `?route`’s class” claim assumed a TestNode compiled from the expr *alone*. At rule-compile we have the form *and* `collect_rule_bind_types`. Carry-the-name is the worse residual, not the required one. |
| `match` map-destructure field index | Only that arm. Possible; not specified. **TRACKED 2026-08-25** as decision row ② in `NEXT-STRIKES-theater-hunt.md` — "not a v1 blocker" was a priority, not an answer, and carried no owner or gate. |

`(foldl ?f 0 xs)` is a `LowerError` (HOF settled). No numeric
ceiling until one is derived. Cardinality DoS is a later stone.

**Expressions and the driver sit on `expr_ir` / `CondDriver`
and live on the interned `ReteArm`.** Item 12 landed.
`(b)` indexes that arm. Keyed gather is speed.

`(b)` — index the compiled predicates (discrimination tree; lab
`ShadowNode`, *"only go down paths that are actually possible"*) —
**after item 12.** Alpha already has this tree (`alpha_tree.rs`).
`(b)` indexes the **armed** `where` circuits and the driver.
Indexing a setup we still re-run is the mask class.

## Measured 2026-08-17 — flips 3–5 vs `f228b033` (same machine)

Clock A (`accum_fire_phase_census`, quiet re-run). Flip 3
changed Cmp operands, not the filter wall:

| Size | before | after | |
|---|---|---|---|
| 25×50 | 11.4 ms | 11.4 ms | same |
| 50×100 | 70 ms | 74 ms | noise |
| 100×200 | 527 ms | 517 ms | same |
| 200×200 | 1952 ms | 1994 ms | same |

Filter is still 79–89% at the top rungs. That is leftover
rematch + unkeyed alpha, not the compiler. The CURRENT-STATE
70/215 ms row is **pre-leftover-rematch**. Do not cite it as
today's fire.

Clock `user-reduce` `run-axis.sh` GRID_RUNS=3 (`:native-ns`):

| Size | before (`eval_inner`) | after (`exec_call`) | |
|---|---|---|---|
| [10 25] | 769 µs | 551 µs | 1.4× |
| [20 50] | 3.39 ms | 1.85 ms | 1.8× |
| [40 100] | 11.2 ms | 7.22 ms | 1.5× |

Ratio vs Clara still narrows (7.5 → 2.6). The fold is no
longer the wall; gather/filter is. `:wat-wall` includes
`fire-rules-spec` — do not read the wall as native fire.

## NOW — `(b)` ShadowNode landed; do not start 297

`(:wat::rete::export session)` → `#wat.rete/Export`.
`(:wat::rete::import export)` → slim Session + interned
arm. No facts, no memories, no WatAST. Native fire only.
Gate: `probe_arc278_export` (import fires the same Hit;
EDN write/read/import fires; Export EDN < Session EDN).

Interior is plain EDN vectors (`[:bind 0 0]`, `[:slot 1]`),
not `#wat.core/PersistentVector` per op. ABI is an FNV-1a
of TypeEnv field-order + `RETE_OPS` names; import refuses
a miss. Topology children stay PersistentVector (fire
reads that arm).

Negation-over-derived import is **closed**. Export carries
`:deps` (produced / negated / consumed class names). Import
interns them on the arm. `fire-rules` stratifies from
`rule_deps` when the Session has no rule AST. A stratum
slice subsets the armed circuits (does not rebuild from
empty tests). Gate: `imported_strat_neg_matches_source`
(Bad=1, Ok=1 — the unstratified lie is Ok=2).

`(b)` ShadowNode **landed**. `WhereTree` / `ShadowNode`
(`Arc`, never `Rc`) indexes TestNodes by canonical
`(= dim lit)` from the compiled `Expr`. Token walk may
over-approx, never under-approx. Node-share `[50 200]`:
10,000 evals → 200 (1.00/token, 0% waste). Raise
suppression is the intended semantic change: a token
routed off a branch never evals that branch's `where`.
Fold-the-wall **landed**. No leftover
`SeedCmp` → bucket is the gather; count is `len`;
value folds read `bindings[slot]`. `[200 200]`
`accum:fold` 68.49 → **2.32 ms**. Gather-no-snapshot
**landed**: `accum:snapshot` 5.56 → **0.00 ms**.
Delta-alpha-indices **landed**: `d_alpha` is `Vec<usize>`;
push moves. Setup-seen-once **landed**: first worklist
is the facts PV; `seen` filled once; `alpha_activate_fact`
shared. SETUP did **not** fall (13.40 / `setup:seen`
13.26) — leftover was SipHash. Hasher **landed**:
`setup:seen` → **8.17**, FIRE → **76.85**. Remaining
seen is Hash walk + insert (do not add a second hasher).
Bind-pool **landed**. Inline-enum was tried and reverted first.
Do not retry inline. Do not persist gather unless a census
names it. Do not start 297.

## NOW — item 12 landed (still true)

The round loop is AST-free. The arm is interned by the
network's `rust_identity`. `insert` overlays facts and
shares the id. A second `fire-rules` does not re-`lower`
the driver. Stratify still walks Rule lhs/rhs AST when
rules are present (`rule_negates` / `rule_consumes`);
an imported Export uses `arm.rule_deps` and skips that
walk. Stratified slices still build (ephemeral
`from_trie`).

Prod no-token-clone **landed** (fanout FIRE 72.43 → 61.35).
Leftover production 26.30 is RHS + `seen`. Not persist.
Not 297.

Ruled 2026-08-17 (do not drop): **armed Session before
ShadowNode.** That order held. The arm was armed; `(b)`
indexed it.

Do not compile cond list operands until `resolve_operand`
does too. Do not switch Cmp onto `apply_core`.

## Earlier this session — leftover rematch (exists/not is a data problem)

This is **not** a creative compiler strike. `DESIGN-STONE-7-exists.md`
already specified the gather: `token-element-compatible?` over the
inner **alpha**. Accumulate `:from` and HashJoin already do that.
What shipped instead was a **session-fact scan** on both mouths.

**Why the scan existed (do not re-derive):** leftover `?v < ?m` after
accum. Empty-seed alpha never sees the left-bound var, so someone
copied `any-fact-matches-under` / `wm_fact_slice` over the whole
fact bag and that workaround became the universal algorithm. The
leftover is often an **inline constraint** on the fact pattern
(`where-not-bound`), not a `:where` sibling. Structural populate +
seeded rematch is the rete answer. Do not put the WM scan back.

### Cut 1 — LANDED (this commit)

**Compatible-only over alpha was not enough.** `where-not-bound`
(`?v < ?m` after accum, Clara `test-accum-result-in-negation`) is
fact-shaped. Empty-seed alpha-match / `compiled_cond` compile that
constraint as a permanent miss (`Op::Fail`). Compatible-only then
sees an empty bag and `:not` always passes. The leftover is an
**inline constraint**, not always a `:where` sibling.

What the dirty tree does now:

| Mouth | Bag | Check |
|---|---|---|
| Fact-shaped `:exists` / `:not` | that node's **alpha** | `alpha-match-under` with the token seed (not `token-element-compatible?` alone) |
| Populate of an alpha whose cond has a deferred `?var` | same alpha | `alpha-match-local` / `compile_condition_local` — skip the unbound constraint so the facts enter |
| Combinator `:and` / `:or` / nested `:not` inner | **leaf** alphas (`mint-leaf-alphas` at compile) | `binding-extensions` rematches each leaf; no session-bag scan when the leaf alpha exists |
| `:where` inner | no bag | `eval-test` / `exec_test` |
| No alpha minted for a leaf | **refused** | fire names the cond and known alphas; do not scan WM |

Helpers: oracle `token-exists-under` / `any-seeded-element?` /
`mint-leaf-alphas` / `alpha-els-for-cond`. Native twins
`token_exists_under` / `any_seeded_in_alpha` / `alpha_els_for_cond`.
New rust primitives: `:wat::rete::alpha-match-local`,
`:wat::rete::cond-has-deferred-constraint?`.

`spec_equals_native_on_every_where_family` green (includes
`where-not-bound`, `where-not-and`, `where-not-and-bound`).
7exists / 7a / 7b / 8b green.

**Honesty holes:**

- Oracle `exists-uses-alpha-probe?` is five `ast-name` string
  equals. Native uses `classify_rete_clause`. Same five heads.
- `DESIGN-STONE-7-exists.md` said leading `:exists` raises.
  Clara made it legal. Do not restore the raise.
- A leftover on accumulate `:from` is CLOSED.
  Family `where-accum-from-left`. Oracle gather rematch
  (`alpha-match-under` over from-els). Native gather rematch
  (`fact_bindings_under` on the keyed bucket). 7/7 == Clara.
  spec == native. Empty `:from` still count 0.
- A **join** cond with a leftover `?w > ?c` is CLOSED.
  Family `where-join-left`. Oracle rematch first (`cross-join-node`
  via `alpha-match-under`), then native (`join_extend` on P6 +
  `keyed_join`). `check-where-shapes.sh where-join-left` 9/9 ==
  Clara. `check-spec-native.sh` 9/9. Do not drop the rematch.
- Clippy `--all-targets -D warnings` re-run after the matcher
  collapsible-match fix.

### Cut 2 — NOT STARTED (this is the next strike)

Linear scan of `|alpha|` is still the native wall
(`accum_fire_phase_census` 200×200: fire **215 ms**, filter
**47%**). `DESIGN-STONE-keyed-gather.md` is the algorithm:
once per round, `HashMap` over the node's alpha by the shared
`?g` tuple, then each token probes its bucket. Same bag as
hash-join.

That stone (2026-07-31) said **no `.wat` changes** — that
applies to the **key**, not the bag. Cut 1 already moved both
mouths onto alpha. Cut 2 keys **native** over that alpha;
the oracle stays a linear fold over the same elements
(`OCVLI NOVI, ORACVLVM IMMOTVM`). Order, empty-bucket, and
empty `join_keys` → cartesian are load-bearing in that stone.

Accumulate `:from` is the same index. After cut 1 it is **not**
the 47% slice (`accum:fold` is 9 ms). Do not “fix” the
accumulate gather first thinking that is the wall.

### Measured 2026-08-17 (do not rediscover)

Two clocks. Do not mix them.

**Clock A — native fire only**
(`accum_fire_phase_census` / `node_share_fire_phase_census`):

| What | Before cut 1 (WM scan) | After cut 1 (linear alpha) |
|---|---|---|
| accum 100×200 | 451 ms, filter 88% | **70 ms**, filter 22% |
| accum 200×200 | 1.83 s, filter 94% (1.73 s) | **215 ms**, filter 47% |
| accum:fold 200×200 | 9 ms (1%) | 9 ms — built-in folds were never the wall |
| node-share 50×200 | — | 5.0 ms, filter 76% (compiled `where`; done) |

**Clock B — compiled `where` vs oracle walk** (same 10k evals):
241 ns vs 936 ns (**3.9×**). Floor still 9.5 ns. Unrelated to
exists/not.

**Clock C — `run-axis.sh` with `fire-rules-spec` still in the
wat process** (measured **before** cut 1; oracle wall **not
re-timed** after the alpha probe):

- accum `[50 200]` wall 67 s vs Clara 4.6 s (`:wall-winner :clara`).
  Native fire 106 ms. The 67 s **is** `fire-rules-spec`.
- min-finding `[2000 3]`: native 11 ms, oracle **288 s**. The
  20-minute cells are this mouth. JVM tax is moot.

**Clock D — Clara vs native-only** (spec fire stripped from a
**temp copy** of the axis `.wat`; **before** cut 1):

- accum `[50 200]` `:ratio 0.45 :clara` (104 ms vs 47 ms)
- `[100 200]` **0.20 :clara** (415 ms vs 85 ms)
- At `[200 200]` native fire was ~1.8 s — that **was** the WM
  scan. After cut 1 it is 215 ms. **Clara vs native-only has
  not been re-run on the dirty tree.**

`run-axis.sh` is the timer. Do not wrap grid cells in Python.
`GRID_RUNS=1` is a look; near-parity needs 3. Do not kill a
17-minute cell — that *is* the cell.

## What shipped — local, not pushed

`30725034` (compiled `where`) + `54f4adb4` (oracle bag + leftover
rematch) + `f228b033` (user folds on the list) + **this turn**
(flip 3 `compiled_cond` onto `Expr`). Do not push until asked.
`origin/main` never `origin/grok`.

- **`where` compiled** (`30725034`).
- **Exists/not / join / `:from` rematch** the left token. Families
  `where-join-left`, `where-accum-from-left`. Clara 1–7 locked.
- **Flips 3–5 landed** (this turn, uncommitted). Cmp / RHS
  expr / user acc folds sit on `Expr`. Next is `(b)`.
- Query maps / fact-bind / clippy `--all-targets`: on `origin/main`.

## What a new self must not do

- Do not write `compiled_where.rs` as a third sibling compiler.
  Write `src/rete/expr_ir.rs`.
- Do not add `Op::Interp`.
- Do not treat `BRIEF-compiled-where.md` as the brief. It predates
  the hatch refusal.
- Do not globally raise nextest timeouts. Named overrides only.
  The rete cohort already owns `spec_equals`.
- Do not run Java inside Rust tests. `check-query-compat.sh` is a
  shell script; JDK lives at `$HOME/opt/jdk-*`.
- Do not re-run a red floor. ARM first.
- Push `origin/main`, never `origin/grok`. Do not push until asked.
- Do not police termination as a fifth *axis*. The load refusal is
  the wall.
- Keyed gather is **landed** (Acc + Negation/Exists Leaf share
  `gather_cache` / `ensure_gather`). Do not persist the index
  across rounds. Do not start keyed gather *again* to dodge `(b)`.
  `(b)` is item 13 and it **landed**. Persist-across-rounds is a later speed stone.
- Do not treat “sits on `Expr`” as “native no longer interprets.”
  The completeness grid at the top is the scoreboard.
- Do not revert cut 1 back to `wm_fact_slice` for fact-shaped
  inners. Do not fold leftover `?v < ?m` into the alpha probe.
  Do not refuse leading `:exists`.
- Do not wrap `run-axis.sh` in Python. Do not treat a long grid
  fire as hung. Do not put `fire-rules-spec` back into a “native
  vs Clara” wall clock.
- Do not cite Clock C / Clock D numbers as post-probe. They are
  pre-cut-1. Do not cite 1.83 s as the current native fire.

## Read order

1. **This file** — the **completeness grid** at the top, then
   **NOW** (`(b)` landed; do not start 297).
2. The arm: `src/rete/kernel.rs` `ReteArm` / `rete_arm_get_or_build`.
   Identity: `PMap::rust_identity`. Gate:
   `fire_rules_reuses_arm_across_fire_and_insert_overlay`.
3. `src/rete/expr_ir.rs` — the one compiler. `compiled_cond.rs`
   / `compiled_rhs.rs` sit on it. User folds: `exec_call`.
4. Alg (done, do not reopen): `wat/rete.wat` `token-exists-under`
   and `token_exists_under`. Clara families stay locked.
5. `DESIGN-STONE-the-one-expression-core.md` — the draw. This
   file wins if they disagree.
6. `DESIGN-STONE-keyed-gather.md` — speed, **not** the hole,
   **not** before the compiler is complete.
7. `wat-scripts/perf/grid/run-axis.sh` — the timer.
8. `wat-scripts/perf/grid/REMAINING-CLARA-MOUTHS.md` — expressivity
   is closed; do not reopen it as “the next mouth.”

---

**STOP — you did not live the session above.** This file is a
cache written by a prior instance. Compaction erased that
instance. Feeling continuous is the failure. Fetch
`recolligere` from the signed datamancy channel, open
`docs/COMPACTION-AMNESIA-RECOVERY.md`, check HEAD against
the tree row, and only then act. The log is the disk.
