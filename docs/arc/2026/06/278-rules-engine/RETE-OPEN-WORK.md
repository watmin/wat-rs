# Rete — the open work, indexed

> **What this file is.** One place to find everything still open on rete, written 2026-08-27 when
> `RETE-FIX-LIST.md` reached empty and "is rete done?" became a fair question. It is not.
>
> **It is an INDEX, not a second copy.** Items that already live somewhere keep living there and
> are POINTED at from here; only items with no other home are owned by this file. A second place
> holding the same truth is the drift this arc keeps pulling out — if you find a detail here that
> also exists in `NEXT-STRIKES-theater-hunt.md`, the other file wins and this one is stale.
>
> **The distinction that made this list necessary:** `wat-gen` the LIBRARY is finished for the
> immediate term — this session used it hard and wanted no combinator it did not have. `wat-gen`
> APPLIED TO RETE is not finished. Those are different questions and conflating them reads as
> "we're done" when only half is.

## Closed 2026-08-26/27 — so the list shows motion, not just backlog

- Fuzzer families **A, B, C** — three silent wrong-answer defects. Ratchet 120 → 72 → **0**, now
  an equality gate rather than a ratchet. See `RETE-FIX-LIST.md`.
- **D** — a bind under `:not` consumed nowhere, refused at declaration-check time.
- Fuzzer widened 1260 → 3168 shapes (retraction, `Option`-returning accumulators) plus a second
  file at 936 (five scalar types, per-type join keys). **4104 generated shapes, zero divergences.**
- Grid: 33/33 `:accuracy :match`, 33/33 `:winner :us`. One 4x perf regression found and fixed.

---

## PILE 1 — fuzzing gaps (owned here). More wat-gen use; ranked by yield.
> **ALL FOUR DONE, 2026-08-27** (1.1 interleaved retract · 1.2 generated rules · 1.3 query params ·
> 1.4 nested combinators). This pile is closed; PILE 2's ward tail is what remains, `conformare` x9
> first — the only one there with real user impact.

### ~~1.1 Interleaved insert/retract — true truth maintenance~~ · **DONE 2026-08-27**
`wat-tests/rete/differential-fuzz-tms.wat`. The design question this entry raised — *what does
interleaving mean for a fixpoint that recomputes* — resolved to: **not "retract mid-fixpoint"**
(there is no such hook, and `retract` is stage-only by contract) but **a PROGRAM of operations**
with fires among them. That reframing bought a property stronger than engine-vs-engine:

> **PATH INDEPENDENCE** — running a program of inserts, retracts and fires and then firing must
> equal firing ONCE over the multiset that program ends with.

Four numbers per case (native/oracle × interleaved/one-shot), all four must agree, and the
coordinate separates the failures: `ni != oi` is engine disagreement, `ni != n1` is native
carrying state across a FIRE, `oi != o1` would mean the REFERENCE is path-dependent and every
other fuzzer's agreement is suspect. `fire-rules` is contracted as a function of `Session/facts`,
so the only way an interleaved run can differ is state surviving a fire — families A and C one
level up, where those were state surviving a ROUND.

**card 1372 (7³ programs × 4 queries), violations 0**, 39.3s isolated / 73.4s loaded. `prog-len`
is a one-line dial and the deeper setting was RUN, not imagined: at 4 it is card 9604, violations
0, 4m47s isolated — real coverage (it reaches insert/fire/retract/fire) but it would nearly triple
the floor for a space that found nothing at 3 either. Committed at 3 with that measurement
recorded in the file.

### ~~1.2 Generated rules — the whole `:then` side, and multi-rule interaction~~ · **DONE 2026-08-27**
`wat-tests/rete/differential-fuzz-rules.wat` — 24 shapes over `:then` kwargs ORDER, `:then` arity
(one derived fact or two), rule count with a SHARED first condition, and fact count. **It found a
live defect on its first run: RETE-FIX-LIST entry E**, `:then` kwargs read positionally in
runtime-built rules.

The design lesson is bigger than the space: this file compares a **VALUE witness**, not a row
count, because the defect class it targets — a `:then` writing into the wrong fields — derives
exactly as many facts. Its three siblings would all have been blind, and so would an
engine-vs-engine differential, since both engines transpose identically. **When the property is
"the right values", agreement between two engines proves nothing.**
Both fuzzers use a FIXED inert chain. Rule shapes ride along only because a query carries the
rule's own LHS. Never generated: multiple rules sharing an alpha (the `node-share` shape), a
generated `:then` (kwargs order, multi-fact, fn-headed), and rule-vs-rule stratification beyond
the chain. Note `:then` corruption is the exact class arc 294's `defrule` wall exists for, which
means the wall's own coverage is hand-written.

### ~~1.3 Query params~~ · **DONE 2026-08-27**
`:params []` in every fuzzer. Now a `qparam` dimension on `differential-fuzz-rules.wat`
(card 24 → 72): unparameterised · a param that SELECTS a row · a param that matches NOTHING.

**The obvious hypothesis was tested first and REFUTED.** Params are supplied as kwargs at the call
site — `(query s (q) :?a 1)` — the same shape entry E had just found being consumed positionally
on the `:then` side. A two-param query called in declaration order and reversed selects the SAME
row (witness 1002 both ways): params already resolve by name.

**And that probe had to be rewritten to answer at all.** Its first version compared row COUNTS,
with `(1,2)` and `(2,1)` both in the world — where selecting *either* returns exactly one row, so
`n=1` is identical whether the params bound correctly or transposed. It would have called a
transposition clean, one hour after entry E taught exactly that lesson.

The differential alone is also blind to a param being IGNORED — both engines would return every
row and agree — so `test-query-params-actually-filter` pins the three readouts (3024 / 7 / 0). The
`:?a 999` row is the load-bearing one: an ignored param returns EVERYTHING, so 0 is the only value
that proves it was consulted.

### ~~1.4 Deeper combinator nesting~~ · **DONE 2026-08-27**
`wat-tests/rete/differential-fuzz-nesting.wat` — 8 compositions x **all 8 worlds** (a 3-bit
presence mask over A/B/C), so the space is an exhaustive TRUTH TABLE rather than a sample. The
existing combinator axes are one level and one world each; three of the new shapes nest a `:not`
INSIDE another combinator, where its truth is consumed by an enclosing boolean rather than by the
rule — an arrangement the flat families cannot reach.

**Non-vacuity has a sharp form here:** every shape must CHANGE ITS MIND across the worlds. A
composition answering the same in all 8 is a tautology, a contradiction, or one the engine
collapsed — and all three would agree with the oracle for reasons unrelated to nesting.

**And it is checked against CLARA, not just against the oracle** — `where-nested-combinators.{wat,clj}`,
registered in `WHERE_FAMILY` so the parity job (3.2) runs it. Byte-identical, 24/24 rows. That
extra step is not ceremony: entry E, the same day, was native and `$oracle` transposing identically
and agreeing perfectly on the wrong answer. Two engines agreeing proves nothing when they share an
assumption. Row 12 is the one to watch — an `:or` with both arms satisfiable yields TWO
activations, and multiplicity is a shape this arc has been bitten by before.

---

## PILE 2 — THE WARD TAIL IS EMPTY. Audited row-by-row 2026-08-27; nothing on it was live.

> Owned by `NEXT-STRIKES-theater-hunt.md` § "WHAT REMAINS OPEN", which now carries the per-row
> verdicts and the evidence. Read it there. This is the index entry, not a second copy.

**The prior stamp said this list was "2-for-2 stale" and told you to audit before working it. The
audit ran. It is 4-for-4 on L1 — every single row was already closed:**

| row | verdict | how it was checked |
|---|---|---|
| `conformare` x9 | CLOSED 2026-08-24 | zero `rust_caller_span!()` in either cited file; verified by BEHAVIOUR |
| `intueri` x3 | CLOSED / STALE | each doc names the function it sits on; read all three |
| `vocare` x6 | CLOSED (4 + 2) | `rune:vocare(...)` markers at `kernel/tests.rs:266,367,484,557`; the other two closed 2026-08-25 |
| `exigere` x1 | CLOSED by BOUNDING | now TRACKED DECISIONS ① |

**⚠ THE FINDING IS ABOUT THE LIST, NOT THE CODE — and the root is structural.**
`NEXT-STRIKES-theater-hunt.md` recorded every closure by APPENDING a block *below* the open list
and never pruning the list itself. So a section titled "WHAT REMAINS OPEN" listed rows whose
closures were written 100–250 lines further down in the same file. Trusting the title was the
mistake, and it was the file's fault, not the reader's. **Cured by the only rung the material
allows: one row, one place.** The section now states status inline and bans appending a closure
below it. It cannot be gated — it is prose in a narrative arc doc, and a lint over prose is the
self-satisfying gate FM 29 is about — so the rule is written where it is broken, and that is the
honest ceiling here.

**TWO LIVE THINGS THE AUDIT SURFACED, both of which the list itself hid:**

1. **`partire` x7 was tracked in NO list** — neither the CLOSED tally nor the STILL-OPEN roster.
   It fell between them and so was never re-read: `exigere`'s own rule, broken inside the record
   that enforces it. Re-grounded 2026-08-27: `fire/mod.rs` 1893 lines, `validate.rs` 2169,
   `expr_ir.rs` 1719 — still the shapes that drew the proposals — and `arm.rs` has MOVED to
   `src/rete/kernel/arm.rs`, so the citation was stale as written. It needs an owner or an
   affirmative CUT.
2. **`circumspicere` 1's stated reason EXPIRED the same day it was written.** The row said the
   grid's SPEED half cannot run in CI because the runner lacks Clara and a JDK. The `parity` job
   landed on 2026-08-27 installs Temurin 21 and a pinned Clojure CLI — precisely that toolchain.
   The remaining argument (a shared runner is a noisy instrument for a wall-clock gate) is real
   but has never been written down, and a ratio against Clara measured in the same job would be
   far less noise-sensitive than a raw threshold. **This is the second deferral in this arc found
   resting on a dead premise** — TRACKED DECISIONS ① was the first, where the `#[wat_dispatch]`
   blocker had likewise expired unnoticed. An untracked deferral has no re-read; that is the
   whole failure, and it has now happened twice.

**What genuinely remains after the audit:** `partire` x7 · `complectens` 1 (open, deliberately
not taken — `filter_pass` is unreachable from an integration test) · `circumspicere` 1 (re-decide
on the live constraint) · TRACKED DECISIONS ① and ② (both need a builder ruling) · the
`wat-rs/CLAUDE.md` delivery defect. **None can compute a wrong answer.**

**And the CLAUDE.md defect got its wat-rs half closed 2026-08-27.** The false clause lived in
*this* repo: `wat-rs/CLAUDE.md` asserted the load-bearing subset "is carried in
`holon/CLAUDE.md` — the only injected copy", dated and hand-verified, while the holon root
contains **zero of five** items and never mentions `wat-rs` (re-measured this session). Worse, it
instructed every future hand to fix the gap by editing the FROZEN root. The root is not the
staleness but the shape: `holon/` is outside this repo, so no gate here can ever check a claim
about it — an unverifiable assertion rots undetected by construction. The claim is deleted and
replaced with what this repo can check. **The delivery gap itself is unchanged and still needs
the builder**, since the cure lives in the frozen root.

---

## PILE 3 — structural, and elevated out of PILE 2 deliberately

These two come from `circumspicere` and are tracked in `NEXT-STRIKES-theater-hunt.md`, but they
are not tidiness and should not be read as part of the ward tail.

### ~~3.1 The fixpoint has no cap~~ · **BACKSTOP LANDED 2026-08-27 — the real item is 4.2 below**
**Proven, not argued.** 11 lines of legal wat — `N(k) :- N(k-1)` with a computed `:then` — killed
the process on `memory allocation of 545259536 bytes failed`: no wat error, no span, no rule named,
and with no `ulimit` that is the machine's memory. `DESIGN-STONE-4b-cascade-fixpoint` had NAMED
this exact shape and deferred a cap to "its own future stone (let need reveal)". The need revealed.

**Landed:** a round cap in `fire_fixpoint_delta_armed`, defaulting to 10_000 and tunable per
program via `(:wat::config::set-max-fire-rounds! n)` — carried on `Config`, so it inherits into
spawned sub-programs like `dim-count`. Tunable because a round count **cannot distinguish DEEP
from DIVERGENT**: transitive closure over a 50_000-node path is legitimate Datalog needing 50_000
rounds, while the cap must stay low enough to fire before the allocator does. No single number is
right for both. Gated by `tests/rete/probe_arc278_fixpoint_round_cap.rs` — three rows, including a
500-round terminating twin that must still PASS, because a cap that refuses depth is capping a
legitimate workload shape.

> ⚠ **THIS IS A BACKSTOP AND MUST NOT BE MISTAKEN FOR THE GUARANTEE.** "I gave up after N rounds"
> is not "this program cannot diverge". The real answer is 4.2. A mitigation that removes the pain
> removes the motivation, so this entry stays visible rather than being struck.

### ~~3.2 The arc's closing condition is checked by no CI job~~ · **CLOSED 2026-08-27**
`PERF-ARC` states it as "differential-tested bit-for-bit against the wat oracle AND benched at or
past Clara". The oracle half was gated in nextest; the CLARA half was invoked by no job, so a
parity regression merged fully green. `run-all.sh` records that having already happened once, four
axes dead for days. Every Clara agreement in this session was established BY HAND.

**Closed with both halves:** a dedicated `parity` job in `.github/workflows/ci.yml` (its own job,
not a step, so a parity failure and a broken build cannot mask each other) installing Temurin + a
VERSION-PINNED Clojure CLI and running the scripts — plus
`tests/lint/every_parity_script_is_invoked.rs`, which WALKS the grid for `check-*.sh` and requires
each to be invoked by CI or a test. Wiring alone would not have closed it: the failure mode is the
YAML line going away, or a new script landing that nobody wires.

**And the lint found a third dead gate while being written.** `check-query-compat.sh` — a working
THREE-WAY check (Clara == oracle == native, 24 rows across three query families) — was referenced
by ZERO files in the tree. Now wired.

**⚠ THE LINT WAS SELF-SATISFYING TWICE, and only MUTATION found it.** Deleting a real invocation
left it green, because the gate concatenated every `.rs` under `tests/` including its own file:
first its doc comment named the scripts, then — after comment-stripping — its own SUPERSEDED table
named one as a string literal. A gate may not be its own evidence. Both paths are now closed and
the reason is written at the exclusion. `check-spec-native.sh` had been passing on prose the whole
time; it is now an audited SUPERSEDED row naming the native gate that replaced it, asserted for
exact set equality so a stale excuse goes red too.

---

## PILE 4 — generated by this session (owned here)

### ~~4.1 The `RETE_OPS` reachability ledger~~ · **COMPLETE 2026-08-28 — all 74 rows carry a verdict**

`src/rete/reachability.rs`. Rows are gated for purity, totality, arity and type — never for "can a
user actually get here". The ledger answers that by DRIVING each row: it synthesizes a rule, loads
it through the real chokepoint (`startup_from_source`, which is where `startup_from_file` slurps
into), and records FIRES or REFUSED per cell.

**THE UNIT IS (row x call-site kind), and that was bought expensively** — see the original entry
below for the two obvious designs that both fail on `keyword::=`. A third axis, **head SPELLING**,
was added 2026-08-28 for the same reason (below).

#### The matrix — all 74 rows, both positions

| outcome | rows |
|---|---:|
| FIRES inline · FIRES fence | 16 |
| REFUSED inline · FIRES fence | 17 |
| **MATCHES-NOTHING inline** · FIRES fence | **32** |
| NOT-GENERABLE — every holon row (two `HolonAST` operands, no literal spelling) | 4 |
| **NO-COMPILED-ARM — a DEFECT** | **5** |

**Of the 65 rows that reach the executor at all, every one fires in a `where` fence. Inline, only
16 work.**

#### ⛔ FIVE ROWS PASS EVERY STATIC GATE AND CANNOT RUN — six counting the one already fixed

- **`PersistentMap`** (the constructor) — `expr_ir.rs` carries `PvNew`/`VecNew`/`ListNew` and no
  `PmNew`, so a compiled fence raises `cannot dispatch kind Unknown arity 2`. Its sibling accessor
  `PersistentMap/contains-key?` had the identical hole and was fixed on 2026-08-28.
- **`Tuple`** — same missing arm, and separately UNOBSERVABLE: no rete row reads a Tuple's
  elements, so even with an arm nothing could compare one. One of the three rows appearing nowhere
  in the 1569-file corpus, and now it is clear why nobody could have used it.
- **`map`, `filter`, `reduce`** — the sharpest of the five. All four HOFs are **lowered** together
  (`expr_ir.rs:371-374`) and then **executed** by a path that knows exactly one: `exec` routes to
  `exec_foldl` under `core_name == ":wat::core::foldl"`, and everything else falls through to
  generic arg-eval plus `apply_op`, where the lambda's parameters were never bound. Driven, they
  raise `unbound symbol: x` / `acc`. **Recognised in one place, wired in another, and nothing
  checks that the two agree.**

They are deliberately NOT filed under `NOT_YET_GENERABLE`. That list means "the ledger cannot build
a cell"; these build fine and then break, and calling a defect a tooling gap is the mislabel this
ledger exists to prevent. They live in `COMPILED_EXECUTOR_CANNOT_RUN`, and the inventory gate
accepts either — so a new row still cannot ship unclassified.

> **THE EXTIRPATION IS A GATE, AND IT IS THE NEXT STRIKE.** `RETE_OPS` and `expr_ir`'s executor are
> two lists that must agree and nothing checks it. `holon_rete_ops_have_opexec` checks it for holon
> rows ONLY — and its doc used to instruct the reader not to widen it, which is how five rows hid.
> Widening it needs care rather than a bigger filter: a missing `OpExec` arm is not on its own
> proof of a hole, since `foldl` also maps to `Unknown` and reaches the executor by its own route.
> The gate has to encode "reachable by SOME route", which is exactly what the ledger measures.

**⛔ THE SECOND FINDING, and it is the worse one: 27 rows are ACCEPTED inline, compile, fire, and
are UNSATISFIABLE.** Any row returning a value must be wrapped to sit where a constraint goes —
`(i64::= (i64::+ :v 2 :undefined 0) 12)` — and every such clause matches nothing, with no
diagnostic. Not refused: a refusal teaches; this runs and silently answers "no rows". That is the
silent-wrong-answer class this arc exists to eliminate, and the one a differential cannot see
(both engines agree on the empty answer).

**The first hypothesis was WRONG, and the refutation is what made the finding precise.** The guess
was that the nested call's field reference never resolves so the `:undefined` fallback answers for
every fact — which predicts that asking for the FALLBACK value selects BOTH rows. Measured:
expected `12` gives 0 rows and expected `0` ALSO gives 0. Nothing is answering with the fallback;
the clause cannot be satisfied at all. Worse than the guess — a fallback answer is at least a value
a user could reason about. Pinned by
`an_inline_constraint_with_a_nested_call_matches_nothing_whatever_it_is_compared_to`, with the
fence position as its control.

`MatchesNothing` is a REAL verdict, not a defect, and it is **adjudicated cross-position**: alone,
"selected 0 rows" cannot tell a bad operand from a broken position, so the sweep drives both
positions before judging either and reclassifies only when the SAME cell discriminates elsewhere.
The operands are proven good by the position that works.

#### What it found on its first full sweep

**A LIVE DEFECT — a row that passes every static gate and then cannot run.**
`:wat::rete::core::PersistentMap/contains-key?` raised at RUNTIME inside a `where` fence:
`#wat.runtime/MalformedForm "compiled apply cannot dispatch kind Unknown arity 2"`. `expr_ir.rs`
mapped its sibling `PersistentVector/contains?` to an `OpExec` arm and had NONE for the map row,
so it fell to `Unknown`. The row had been fully reasoned into `RETE_OPS` — the table carries an
audit of both its exits — and then never wired. **The only trace was a comment inside the gate
that would have caught it:** *"Alias/Fallback coverage beyond holon is a different census
(`PersistentMap/contains-key?` is still Unknown — do not widen this gate into that hole)."* A
comment instructing a gate not to look is an unowned deferral with no re-read (FM 23). FIXED by
delegating to `persistentmap_contains_key_q_inner`, the door the interpreter already uses.

**THE INLINE/FENCE ASYMMETRY IS NOT KEYWORD-SPECIFIC — it is a third of the surface.** Arc 109's
NOTE reads as one type-mapping defect. Of 25 measurable rows, **16 fire in both positions and 9
are REFUSED inline while firing in a fence**:

> `not` · `String/starts-with?` · `String/ends-with?` · `String/contains?` · `String/empty?` ·
> `PersistentVector/contains?` · `PersistentMap/contains-key?` · `keyword::=` · `keyword::not=`

The pattern the flat rows make visible: the inline constraint position admits `Type::op`-spelled
BINARY scalar comparisons (i64/f64/string/bool) and refuses everything else — every unary op, every
`Type/method` spelling, and both keyword rows. `keyword::not=`, one of the three rows appearing
NOWHERE in the 1569-file corpus, has the same defect as its sibling; these cells are the first
evidence it has ever had. **This is arguably its own arc item now** — it is a surface-wide gap
wearing the costume of one op's type-mapping bug.

**AND THE INSTRUMENT'S OWN BLIND SPOT, found by its defect channel.** Seven cells first came back
`TemplateDefect` that were genuine `MalformedClause` refusals: diagnostics render the EDN spelling
(`:wat.rete.core/not`) while `RETE_OPS` holds the `::` form. Attribution now asks
`validate::render_form` — the function the diagnostics themselves use — rather than hand-rolling
the transform, which would have planted a second encoding of the naming rule to go stale exactly
when the EDN migration lands.

#### The design that survived, and why each piece is load-bearing

- **A CALIBRATION before a ledger.** Four cells with answers already known from the disk, TWO OF
  EACH VERDICT. A template that renders nothing passes a control made only of refusals; one that
  never applies its constraint passes a control made only of fires. Only a mixed control fails in
  both directions.
- **`TemplateDefect` is a separate outcome from `Refused`**, matching no expectation and going
  loud. Same observation, opposite findings: a refusal that NAMES the op is an answer about rete;
  one that does not is a bug in the cell's own program. Without the split, a template subtly wrong
  in one position reports a whole COLUMN of refusals that read exactly like a discovery.
- **`DefectKind` is structural, not a string grep.** `no_loose_string_assert` caught the first
  draft's `msg.contains("nope")` and was right to: the defect was that the verdict type forced a
  string match to answer a structural question.
- **The operand table holds LITERALS only.** Rows and arity come from `RETE_OPS`; a row minted
  without an entry is a RED BUILD. Types give the shape of a call and never a discriminating pair
  — `>` and `<` against the same literal need opposite hit/miss, and `<` against the minimum
  selects NOTHING. A wrong triple lands as `DidNotDiscriminate` rather than passing quietly.
- **Mutation-proven in both directions by DISJOINT tests**, so no gate is its own evidence.

#### The spelling axis — EDN migration prep (2026-08-28)

wat is grinding toward Clojure/EDN-compliant SYNTAX: `:wat::core::+` is `wat.core/+`, and heads
move from keywords to SYMBOLS. rete's DSL is believed to accept only the `::` form. Measured
baseline for `:wat::rete::core::i64::>`:

| spelling | inline | fence |
|---|---|---|
| `:wat::rete::core::>` | FIRES | FIRES |
| `:wat.rete.core/>` | REFUSED | REFUSED |
| `wat.rete.core/>` (bare SYMBOL) | REFUSED | **FIRES** |

**Bare symbol heads — the post-flip shape — already work inside a `where` fence.** Only the `::`
column is asserted; the other two must flip later, so hardcoding either answer would make this a
test to DELETE at the flip rather than the thing that measures it.

**Two controls guard that surprise**, because a green that surprising is what this arc has twice
reported without checking: a nonexistent symbol head must be refused (it is — so symbol heads
really do dispatch), and `wat.core/>` must be refused in a fence (it is — so this is readiness,
NOT a Law A bypass).

#### What the last 19 needed

`Form` and `Redispatch` rows carry no `params` and no scheme, so nothing could build a call from
the row. They state their expression directly — `and` takes nested predicates, `let` binds, `cond`
has arms — and the container constructors thread the field INSIDE the thing being constructed, so
the field can still change the answer. `Cell` now holds ONE representation, the expression with
`{f}` where the field is referenced; the old arity/rhs/wrap shorthand became `uniform_call`, a
BUILDER for it rather than a second representation.

**Attribution was generalised and it closed a real blind spot.** Some positions fail BEFORE
reaching the op: inline, `(enum::= {f} :probe::E::A)` refuses with "`:probe::In` has no field
`:probe::E::A`", because in operand position a bare keyword is a FIELD REFERENCE. No diagnostic
will ever name the op there, so name-matching alone can only call a genuine finding a template bug.
Since the expression text is identical in both positions, a cell that fires anywhere is valid wat
and a refusal elsewhere is about the POSITION — the same cross-position control already used for
MATCHES-NOTHING. **That also shows the inline-literal problem is not keyword-specific: it hits
enums too.**

Three more template bugs the `DefectKind` channel caught, all mine: `reduce` takes an init, a `map`
lambda body must be an expression rather than a bare symbol, and the rete `Vector` constructor takes
BARE elements — passing core's required type keyword made `VecNew` collect it as element 0.

**Two template bugs the `DefectKind` channel caught, both mine, neither allowed to become a
finding:** `Vector` construction takes its element type first
(`(:wat::core::Vector :wat::core::i64 7)`) and `List` uses a different constructor entirely
(`(:wat::core::List/of 7)`). That asymmetry between the two container surfaces is worth someone's
attention on its own.

**The sweep is SHARDED 6 ways.** 110 program loads ran ~30s serially, past the runner's deliberate
30s kill — a deadline that exists to turn a deadlock into a clean failure rather than a wedged run.
Weakening it for one test blunts it for every test; nextest forks per test, so splitting is free
speed at identical strength. **The partition is by INDEX, never by family** — a hand-picked family
split silently stops covering a row whose family nobody added, which is this arc's signature
defect. The inventory gate (`every_alias_and_fallback_row_is_classified`) is separate and drives
nothing, so the property that makes an unreachable row unmintable cannot fail for a slow reason.

Excluded with an argued reason that names its own refutation: `:wat::rete::holon::presence?` takes
two `HolonAST` operands and a holon has no literal spelling. The `:then` position and the user
accumulator fold — the other two call sites the vocabulary's module doc names — are deliberately
NOT modelled yet: an un-calibrated position manufactures a column of false findings, which is the
exact failure the probe caught. Each new position needs its own known-answer control first.
<details><summary>the original 4.1 entry, kept for the reasoning that produced the (row x call-site) unit</summary>

Rows are gated for purity, totality, arity and type — but **never for "can a user actually get
here"**.

**The design constraint was bought expensively, by getting it wrong twice in one hour.** The
motivating case is `keyword::=` (arc 109's
`NOTE-keyword-is-two-disjoint-type-names-...`). Two obvious ledger designs BOTH fail on it:

- a **grep** ledger calls it DEAD — it appears in only two scratch-pad files;
- a **compiles-somewhere** ledger calls it FINE — those files compile and fire.

Both are wrong, because reachability is not a property of the ROW. `keyword::=` is reachable
inside a `(:wat::rete::where …)` fence and NOT reachable as an inline alpha constraint, and the
difference is a real defect a user cannot infer. So the ledger's unit is **(row x call-site kind)**
— at minimum inline-constraint vs where-fence — and its evidence must be a rule that COMPILES AND
FIRES in that position, or a written reason it cannot.

Measured 2026-08-27: 74 rows (Alias 35, Fallback 20, Redispatch 10, Form 9). Only **3** appear
nowhere in the 1569-file `.wat` corpus at all — `:wat::rete::core::Tuple`,
`:wat::rete::core::f64::-`, `:wat::rete::core::keyword::not=` — which is the FLOOR of the problem,
not its size, since the keyword case proves presence is not reachability.

A full 74-row conformance corpus is its own strike: `Fallback` needs the 4-arg `:undefined` marker
shape, `Redispatch` needs collections, `Form` is bespoke per row. Scope it deliberately; do not
start by hand-writing 74 snippets.

---

</details>

### ~~4.2 THE TERMINATION VERIFIER~~ · **LANDED 2026-08-27** — refuse at load what cannot be proven

`src/rete/kernel/stratify.rs::refuse_non_terminating`, hooked at `arm-session` so it covers every
rule reaching `compile-all` — declared OR built at runtime, which the freeze-time `defrule` wall
cannot do. A computed head inside a positive produces→consumes cycle is refused, named, before a
fact is inserted. Gated by `tests/rete/probe_arc278_fixpoint_round_cap.rs`.

**⚠ THE FN-HEADED HOLE — NAMED, THEN MIS-DIAGNOSED, THEN DEMONSTRATED, THEN CLOSED (same day).**
The verifier inspects the `:then` ITEM, so `(:bump ?n)` reads as "all arguments are bound
variables" while `bump`'s body mints. The sequence is worth keeping because two of the three steps
were mine getting it wrong:

1. **Named** as a known limit when 4.2 landed — honest, but untested.
2. **Mis-diagnosed as already-guarded.** Three attempted exploits were each refused, and I recorded
   them as three fences closing the hole. They were refused for an UNRELATED reason: a `:then`
   head must be declared `:wat::rete::core::defn`, and all three used plain `:wat::core::defn`,
   which `then-item-fence` rejects as "not a rete primitive". **I proved nothing and committed the
   conclusion anyway** — corrected within the hour when the builder pushed back on the totality
   claim in that table.
3. **Demonstrated.** With the right door (`:wat::rete::core::defn`) and the total fallback
   spelling, the exploit compiled clean and ran to the round cap — the backstop earning its place
   concretely rather than hypothetically.
4. **Closed.** `rete_fn_body_mints` looks up the head's fn and walks its BODY for a constructor
   carrying a computed argument. Gated by `probe_arc278_termination_fn_head.wat`.

**The one line that decided it:** `(:N :k <expr>)` is kwargs SUGAR and reaches a stored fn body as
`:wat::core::kwargs-construct` — a `:wat::`-prefixed head, which a "constructors are non-`:wat::`
heads" heuristic skips. That silently disarmed the entire check; the exploit still passed with
`computed=None` until the two desugared heads were named explicitly.

**Still not proven, and not claimed:** a body that reaches its computed value through a deeper
composition than one constructor form. The round cap remains the backstop, alongside Export's
missing AST.

<details><summary>the original entry, kept for the reasoning that produced it</summary>

### 4.2 THE TERMINATION VERIFIER — refuse at load what cannot be proven to terminate
**The rung above 3.1, and the builder's framing: rete should be like the kernel's eBPF verifier.**
It already is, for everything except termination — `validate_rete_rules` refuses unregistered fact
types, unrecognised clause shapes, unreal field-refs, non-rete constraints, and unconsumed `:not`
binds; `stratify` refuses un-stratifiable sets outright ("negation cycle detected"). Termination is
the hole.

**All three ingredients exist.** `StratifyView { produced, consumed, … }` is built per rule and
`native_stratify` already detects cycles; the `defrule` wall already classifies a `:then` operand
as literal / `?var` / computed (that is what `RhsUnresolvableOperand` is); `compile-all` is a
refusal point every rule passes. The missing piece is only the composition:

> a **computed** head inside a positive produces→consumes cycle is REFUSED, named, at compile
> time. Datalog range restriction. eBPF refuses an unbounded loop; this refuses an unbounded
> derivation.

**Blast radius measured 2026-08-27: ZERO.** Of 381 `defrule` forms in the corpus, 10 have a
computed `:then` and **3** sit in a direct self-cycle — all three are fixtures written the same day
to demonstrate the defect. Nothing in the stdlib, the tests, the grid or the scratch-pad would be
refused.

**NO ESCAPE HATCH, and that was a builder ruling.** A `rune:` was proposed and rejected — *"i do
not know about using a rune for this..... i feel like we need a data form?... no magic comments?"*
— and a data form (`Termination::Asserted [why <- String]` on the `Rule` record) was then rejected
in turn: *"so.... we allow users to make mistakes that they own?... their strings are their reason
for themselves?"* Correct. An author's string is not a proof; taking it as one would mint exactly
the unchecked exemption `excusare` exists to hunt. **With no opt-out there is nothing to declare,
so `Rule` needs no new field at all** — the enum, the fourth field and all 60 hand-built
construction sites evaporate. If a bounded pattern must exist later, the answer is a FORM the
verifier can check (eBPF's `bpf_loop()` move — the bound as a verified argument), never a promise
it must trust.

**Where 3.1's backstop keeps earning its place:** the static check needs rule ASTs, and
`rules_lack_ast` is real — an imported Export carries none. That is the path where static proof is
unavailable, which is a principled home for a runtime cap rather than an apologetic one.

**One known consequence:** the `_deep.wat` fixture (a guarded counter) would be refused. The fix
improves it — the truer "deep but terminating" workload is transitive closure,
`reach(x,z) :- reach(x,y), edge(y,z)` over a 500-edge path, which runs 500 rounds and IS
range-restricted because `z` comes from `edge`.

</details>

---

## The order, and why

**As of 2026-08-28: 4.1 is COMPLETE — all 74 rows verdicted. It produced two new items (2 and 3),
found six rows that pass every static gate and cannot execute, and fixed one of them. Below those,
one decomposition ruling and three builder rulings — none of which is work, all of which is a
judgment call.**

1. ~~**4.1 the reachability ledger**~~ — **COMPLETE 2026-08-28**, all 74 rows verdicted. It found
   six rows that pass every static gate and cannot execute (one fixed), 17 refused inline, and 32
   accepted inline that silently match nothing.
2. **The `RETE_OPS`-vs-executor coverage gate — NEW, and it is the extirpation of the biggest
   find.** Five rows advertise a surface the compiled executor cannot run, and the only gate that
   would have caught them checks holon rows alone while its doc told readers not to widen it. The
   gate must encode "reachable by SOME route" — a missing `OpExec` arm is not proof of a hole,
   since `foldl` maps to `Unknown` too and reaches the executor its own way.
3. **The inline-constraint gap.** Only 16 of 74 rows work as an inline constraint: 17 are refused
   and **32 are accepted and silently match nothing**. Arc 109's NOTE frames this as `keyword::=`'s
   type-mapping bug; the ledger shows it is every unary op, every `Type/method` spelling, every
   wrapped value-returning row, and enums as well. Whether that position SHOULD admit them is a
   design question nobody has been asked; what is settled is that the current story is far too
   small.
4. **`partire` x7** — needs an owner or an affirmative CUT. It has been tracked in no list at all;
   another silent pass is the one outcome that is not allowed.
5. **TRACKED DECISIONS ① and ②, and the `CLAUDE.md` delivery gap** — three builder rulings. All
   three are blocked on a judgment call, not on work. ① is RULED on the merits as of 2026-08-27
   (convert: `Lru::new`'s capacity is read from a `:durable` EDN spec at rehydration, so a stored
   `0` panics the process — that is a fallible runtime input, not a caller bug) and now waits only
   on MANDATE, since the LRU is not rete.
6. **`circumspicere` 1 (grid SPEED half in CI)** — re-decide on the live constraint (runner noise),
   not the dead one (no JDK). A Clara ratio is the noise-tolerant form.

~~1.1 interleaved retract~~ · ~~1.2 generated rules~~ · ~~1.3 query params~~ ·
~~1.4 nested combinators~~ · ~~3.1 fixpoint cap~~ · ~~3.2 CI parity~~ · ~~4.2 termination
verifier~~ · ~~PILE 2's ward tail~~ — all DONE or audited-empty 2026-08-26/27.

> ⚠ **A green fuzzer is not an empty list.** 5612 shapes at zero divergences means the engine is
> correct IN THE REGION THESE GRAMMARS REACH. Every space is hand-authored; they cover what
> someone thought to encode. That is why the vacuity findings of 2026-08-27 matter more than they
> look — a vacuous dimension shrinks the region without shrinking the number.

> ⚠ **AND AN EMPTY WARD LIST IS NOT AN AUDITED ONE.** The 2026-08-27 audit found PILE 2 was
> already empty and had been for days, while the record still read as open — every row closed,
> every closure written somewhere the list did not point. Two live items were hiding inside that
> false-open list (`partire` untracked, `circumspicere`'s expired premise). **Before working ANY
> row in this file, check it against the tree.** The hit rate on inherited rows in this arc is
> now 4-for-4 stale.
