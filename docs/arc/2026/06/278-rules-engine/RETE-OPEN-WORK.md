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
| FIRES inline · FIRES fence | **55** |
| REFUSED inline · FIRES fence — with a DIAGNOSTIC | 18 |
| ~~MATCHES-NOTHING inline~~ | **0** (was 39; fix-list F, closed 2026-08-28) |
| NOT-GENERABLE — every holon row (two `HolonAST` operands, no literal spelling) | 4 |
| **CANNOT RUN — a DEFECT** | **1** (was 6; five fixed 2026-08-28) |

**Of the 69 rows that reach the executor at all, every one fires in a `where` fence. Inline, only
16 work.**

#### ⛔ SIX ROWS PASSED EVERY STATIC GATE AND COULD NOT RUN — FIVE FIXED, ONE LEFT

**FIXED 2026-08-28:**
- **`PersistentMap/contains-key?`** — no `OpExec` arm; delegated to
  `persistentmap_contains_key_q_inner`, the door the interpreter already uses.
- **`PersistentMap`** (the constructor) — `PmNew`, mirroring `eval_persistentmap_ctor`: even
  arity, alternating pairs, `value_is_key_hashable` per key, `PMap::from_pairs`. The semantic
  primitives are CALLED, not re-derived.
- **`reduce`** — the builder's correction, and the disk agrees: **FOLDL IS REDUCE.**
  `wat/seq.wat:317-329` states both clauses outright — 3-arity is literally
  `(:wat::core::foldl f init coll)`; 2-arity seeds from the first element and raises by name on
  empty. `exec_reduce` mirrors exactly that. It needs an arm at all only because `reduce` is a
  wat-level `defclause` with no Rust dispatch to re-enter and a fence has no defclause machinery.

> ⚠ **AN INHERITED CONTRADICTION, SURFACED ONLY BY BEING ABLE TO RUN THE ROW.** `reduce`'s 2-arity
> form RAISES on an empty collection, while `RETE_OPS` declares the row `total: true` — a wall
> every row must pass (`every_rete_row_is_total`). It went unnoticed precisely because nothing
> could execute the row to find it. Recorded rather than papered over: answering an empty reduce
> with an invented value would be the worse bug. **This is a ruling, and it is small.**

- **`mapv` and `filterv`** — the builder's call, and the right shape. The rows WERE
  `:wat::core::map`/`:wat::core::filter`, which return a **lazy Stream**; a fence has no stream
  machinery and nothing there can consume one, so both were unreachable in every position. The fix
  is not an eager arm under the lazy head — that would make `:wat::rete::core::map` mean something
  different from `:wat::core::map`, silently, against the `Redispatch` contract "same routine as
  `core_name`". wat already ships the eager materializers under their clojure names
  (`wat/seq.wat`: *"mapv / filterv — the eager forms"*), so the ROWS moved to those. `exec_mapv`
  mirrors `eval_mapv` (every exit `Ok(Value::Vec(..))`); `exec_filterv` mirrors the `filterv`
  defclause. All four HOF arms share `compiled_fn_arg`/`eager_items` so they cannot drift.

**STILL BROKEN — ONE ROW, and it needs a RULING rather than an arm:**

- **`Tuple`** — same missing arm, and separately UNOBSERVABLE: no rete row reads a Tuple's
  elements, so even with an arm nothing could compare one. One of the three rows appearing nowhere
  in the 1569-file corpus, and now it is clear why nobody could have used it.
All four HOFs are **lowered** together (`expr_ir.rs:371-374`) and then **executed** by a path that
originally knew exactly one — `exec` routed to `exec_foldl` by name and everything else fell
through to generic arg-eval, where the lambda's parameters were never bound. **Recognised in one
place, wired in another, and nothing checked that the two agree.** `reduce` is now routed too.

They are deliberately NOT filed under `NOT_YET_GENERABLE`. That list means "the ledger cannot build
a cell"; these build fine and then break, and calling a defect a tooling gap is the mislabel this
ledger exists to prevent. They live in `COMPILED_EXECUTOR_CANNOT_RUN`, and the inventory gate
accepts either — so a new row still cannot ship unclassified.

> **THE EXTIRPATION IS THE LEDGER ITSELF — already built, 2026-08-28.**
> `every_rete_ops_row_is_classified` plus the shards require every row to be DRIVEN to a verdict or
> carry a written reason, which is strictly stronger than the arm-existence check that was
> proposed. Arm-existence turned out to be the wrong question: not NECESSARY (`foldl` maps to
> `Unknown` and reaches the executor by its own route) and not SUFFICIENT (an arm can exist while
> the row is unwritable in every position). `holon_rete_ops_have_opexec` is therefore re-scoped as
> the cheap holon-specific check it always was — **not widened** — with its doc now pointing at
> the ledger as the wall.

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

## Inbound notes — the list that did not exist until 2026-08-29

⛔ **THIS SECTION EXISTS BECAUSE TWO REPORTS SAT UNREAD FOR FIVE DAYS.** `~/work/NOTE-*.md` is where
other agents file findings for this one. Nothing pointed at that directory, so nothing was ever
read from it unless someone happened to `ls`. One of the two was a **SILENT WRONG ANSWER** — the
highest-severity class this arc recognises. **When you file or receive a note, add its row here in
the same motion.** This is the same disease as `partire` x7 (item 4): a real finding, tracked
nowhere, indistinguishable from a finding that does not exist.

| note | filed | subject | verdict |
|---|---|---|---|
| `NOTE-rete-a-where-before-a-fact-condition-silently-matches-nothing.md` | 2026-08-24 | a `where` followed by a fact condition matched NOTHING, silently | ✅ **VERIFIED FIXED 2026-08-29** — re-driven, selects the hit (1, was 0). Probe: `wat-scripts/scratch-pad/probe-where-before-fact-condition.wat` |
| `NOTE-rete-cond-lowers-on-the-lhs-but-not-the-rhs.md` | 2026-08-24 | `cond` compiled in a `where` and failed at `compile-all` in a `:then` | ✅ **VERIFIED FIXED 2026-08-29** — re-driven, compiles and fires. Probe: `wat-scripts/scratch-pad/probe-cond-in-a-then.wat` |
| `NOTE-rete-termination-verifier-refuses-provably-bounded-recursion.md` | 2026-08-28 | a guarded counter is refused though it terminates | **CONFIRMED**, minimum ask done — item 9 |
| `NOTE-holon-classifier-contract-is-unenforced-and-the-holon-tag-breaks-it.md` | 2026-08-28 | `#holon` produces holons that cannot round-trip | ⏸ **DEFERRED by builder ruling** — blocks item 7 step 3 |
| `NOTE-experiri-a-ward-that-executes.md` | 2026-08-27 | (not a rete finding — the ward itself) | — |

⚠ **NEITHER FIX WAS MADE BY ANYONE READING THOSE NOTES.** Both were almost certainly collateral
from this arc's inline-constraint and expression-lowering work. **A finding that gets fixed by
accident is not a process that works** — it is the same outcome as luck, and the next one may not
be adjacent to whatever is being built that week.

## The order, and why

**As of 2026-08-28 (LATE — four commits after the stamp above): the inline position is CLOSED.
Every GENERABLE row fires in BOTH positions, 79 of 79 counting the four holon rows that were
driven by hand. The column went 16 -> 68 -> 71 -> 75 across the day, and the last four came from
proving the ledger's own exclusion false.**

✅ **THE LEDGER'S FOUR HOLON ROWS ARE GENERATED AND FIRE — `NOT_YET_GENERABLE` IS NOW EMPTY.
79/79, produced by the instrument rather than claimed by hand.** Its exclusion read *"a holon has
no literal spelling, so the second operand cannot be written as a constant"* — **and that was
simply FALSE.** `#holon <form>` is the literal; a holon holds the same data EDN does, so it spells
the same way (`#holon [1 2 3]`, `#holon {:a 1}`). Builder, on being shown the claim: *"wut… holon
is just another holder for data like edn is."*

**What the exclusion had actually measured was a MISSING LOWERING ARM.** The reader desugars
`#holon X` to `(:wat::holon::literal X)`, which arrived at `expr_ir` wearing a call's clothes and
fell through to *"cannot lower head `:wat::holon::literal`"*. Someone read that refusal as a
property of holons. **A refusal is evidence about the door you knocked on, not the room behind
it** — the same lesson the termination doctrine block taught six hours earlier, in the same file.

**The fix is a CONSTANT FOLD, not a row.** There is nothing to dispatch: the enclosed form is data
captured without evaluation, so its value is fixed by the source text — no environment, no
bindings, no encoding context. `expr_ir` folds it to `Expr::Lit` like any other literal, which
keeps it out of the jump table entirely (a row for a constant would be a runtime dispatch that can
only ever return the same value). Totality is not weakened, it moves EARLIER: `to_holon_inner` is
partial, but here its input is a quoted literal, so any failure is a located diagnostic at
rule-compile rather than at fire time.

**And the planned `Cell` extension was never needed.** The prior entry concluded the refutation was
"a SECOND FIELD" — because one field cannot discriminate a self-comparison. True, and moot: with a
literal rhs the holon rows need exactly what every scalar row needs, one field and one constant.
**I had designed an extension to the instrument to work around a defect in the substrate.** With `v` and `w` both `:wat::holon::HolonAST` on
`:probe::In`:

| row | inline | fence |
|---|---|---|
| `presence?` | FIRES | FIRES |
| `cosine` (`f64::>` … `0.9`) | FIRES | FIRES |
| `dot` (`f64::>` … `1000.0`) | FIRES | FIRES |
| `coincident?` | **was REFUSED — FIXED, now FIRES** | FIRES |

**Seven of eight when measured; eight of eight after the fix below.** The earlier claim on this page and in the breadcrumb —
*"all four FIRE, inline and fence"* — was WRONG, and it was written from a hand-drive whose exact
shape was never recorded. Thresholds above are MEASURED, not guessed: `cosine` 1.0 vs −0.018,
`dot` 4333.0 vs −81.0, `coincident?`/`presence?` return bool directly
(`wat-scripts/scratch-pad/probe-holon-rete-cell-values.wat`).

✅ **`coincident?` INLINE WAS A LIVE DEFECT — FIXED 2026-08-28, and the fix was the LADDER'S TOP
RUNG, not the one-liner.** It is now `1` inline and `1` in a fence. It was the **fifth instance of
the day's pattern**, and the row of the pattern table still holding it.

**The defect.** `coincident?` is `OpClass::Redispatch` (its PARAMS keep core's `HolonAST | Vector`
polymorphism, so they cannot be a rank-1 scheme) but its RETURN is always `bool`.
`expr_is_provably_boolean` trusted `row.ret` only for `Alias`/`Fallback`, because on
`Form`/`Redispatch` rows `ret: ParamType::Bool` was a documented PLACEHOLDER. One value, two facts —
so the genuinely-boolean row was refused rather than believed, invisibly, because it worked in a
`where` fence the whole time.

**⛔ THE ONE-LINE WIDENING IS UNSOUND AND WAS DRIVEN BEFORE IT WAS PROPOSED.** Admitting
`Redispatch` makes `(Tuple/first (Tuple :v 99))` — an `i64` whose row ALSO said `ret: Bool` — a
legal inline constraint that compiles, fires and returns `Ok(0)`: **silently matches nothing.**
Fix-list F's class, reopened on a new row. Do not re-propose it.

**THE CURE — the placeholder now has no spelling.** `ret: Ret::Is(ParamType) | Ret::NoScheme`
(`vocabulary.rs`). 79 rows migrated: 57 `Alias`/`Fallback` → `Is(...)`, 5 → `Is(Bool)`, 17 →
`NoScheme`. **The compiler then named every reader — seven, not the three I had predicted.** Four
were invisible to grep because they reach `ret` through a shared helper
(`classify_fallback_outcome`, called from `expr_ir`, `where_tree` and `runtime`), which is itself
the argument for the type change: a convention cannot make the compiler find its own violations.

- `clause.rs` and `validate.rs`: the `class` test is **DELETED**. Its absence is the point.
- `check.rs`: the `class` test **STAYS**, and that is not an inconsistency — that site builds a
  WHOLE rank-1 scheme, so it is guarding on PARAMS, not on `ret`. `coincident?` is exactly the row
  that separates the two questions, which is why `ret` got its own enum rather than both folding
  into one `Scheme`.
- `params: &[]` keeps the same two-facts shape and is an **AFFIRMATIVE CUT**, stated in `Ret`'s doc:
  no rete row takes zero operands, so it is unambiguous today. If a zero-arity row is ever minted,
  `params` needs this treatment and that paragraph is the notice.

**TWO GATES, BOTH MUTATION-PROVEN:**
1. `a_row_that_declares_bool_is_believed_inline_whatever_its_class` — `coincident?` inline AND
   fence, plus **the soundness twin**: `Tuple/first` must stay REFUSED. Mutating the table makes
   each arm fire on its own (`Got: Ok(0)` for the twin — the F-class signature, in the failure text).
2. `only_the_named_scheme_less_rows_declare_a_return_type` — freezes by NAME which scheme-less rows
   state a return (`and`, `or`, `enum::=`, `enum::not=`, `coincident?`), asserts every
   `Alias`/`Fallback` states one, and carries two non-vacuity floors. The mutation prints both
   offenders in one diff.

**Also learned, and it belongs in the ledger's design:** `enum::=`/`enum::not=` are `Form` rows that
fire inline via `classify_constraint_head`'s NAME-pattern path, which never reads `ret` at all. So
`coincident?` was the only row in the whole table that genuinely returns bool and sat in neither
admission path — a population of one, which is why no count would ever have surfaced it.

1. ~~**4.1 the reachability ledger**~~ — **COMPLETE 2026-08-28**, all 77 rows verdicted, ZERO
   unrunnable. It found six rows that passed every static gate and could not execute (all fixed),
   18 refused inline, and 39 accepted inline that silently match nothing.
2. ~~**One small ruling** — `reduce`'s 2-arity form raises on an empty collection while its row
   declares `total: true`.~~ **ALREADY CLOSED, and this row was STALE THE DAY IT WAS WRITTEN.**
   Shipped `97eac5a38` (2026-08-27) — the rete lowerer refuses the 2-arity form outright
   (`expr_ir.rs:440`) with a located diagnostic naming the totality reason, and it shipped with a
   119-line gate plus two fixtures. Driven 2026-08-28 to confirm: the refusal fires, and
   `total: true` is honest because the partial arity cannot be reached.
   **This is the SIXTH consecutive inherited row in this arc found stale on audit.** The rate is
   not noise — treat every unstruck row here as a claim about the past, not a statement about the
   tree, and check it before you work it.
   (Everything else under this number was already closed: `map`/`filter` became the eager
   `mapv`/`filterv`; `Tuple` got its three accessors; the coverage GATE is the ledger.)
3. ~~**The inline-constraint gap**~~ — **FULLY CLOSED 2026-08-28. The residue this entry once named
   is gone too, and both of its stated reasons were wrong** (struck below). The inline column went
   **16 -> 68 -> 71 -> 75 of 79**, and the four not counted are the holon rows the LEDGER cannot
   generate — driven by hand, they fire in both positions. The wrong-answer half is gone (fix-list F: 39 rows
   that compiled, fired and silently matched nothing, in BOTH engines); the keyword half was a real
   BUG (`rete_type_segment_of` mapped only the uninhabitable capital `Keyword`); and the grammar
   half is admitted — an inline constraint is now any PROVABLY boolean rete expression, replacing a
   shape-set no reader could infer.

   **My stated reason for the grammar split was FALSE and I nearly shipped it as the rationale.** I
   argued indexability; `alpha_tree.rs` indexes only provable equality discriminators and states
   that `< > <= >=` "ride the wildcard edge" — while being admitted inline. Check the premise
   before running the four questions on it.

   ~~**The residue**~~ — **ALL OF IT CLOSED 2026-08-28, and BOTH stated reasons were wrong.**

   · `cond`/`let`/`match` were refused for being *"polymorphic in their body's type"*. Polymorphic
     IN THE BODY means the type is a FUNCTION of the body, and the body is in the AST. The head-only
     test read `row.ret` — a PLACEHOLDER for `Form` rows — and stopped. It is now
     `expr_is_provably_boolean`, a structural proof needing no env, which keeps
     `classify_rete_clause`'s "by SHAPE alone" contract intact. Decidable because rete is closed and
     every row is total. (`ad2286133`)
   · `cond` was not failing a type test AT ALL — the macro expander descended into `where` bodies
     only, so an inline `cond` never expanded to nested `if`. Discriminating probe: wrapping it in a
     provably-bool head SATISFIES the type objection and it was still refused.
   · The bare-keyword rule was called a syntactic ambiguity. `:probe::E::A` carries `::` and a field
     name is a bare identifier, so an enum variant could NEVER have been a field reference — there
     was nothing to disambiguate. And the engine was already deciding it correctly one level down:
     the same comparison nested inside another call FIRED. `bind_field_refs` and
     `compile_operand_expr` ran the same `position(...)` lookup ~120 lines apart in one file and
     disagreed on the `else`. (`b7f54a17f`)


4. ~~**`partire` x7**~~ — **RESOLVED 2026-08-28 by re-audit. The item dissolves into two closures
   and two named cuts.** It lingered a week for a reason worth naming: **it was a TALLY, not a
   finding.** All that was ever recorded is *"Split proposals for `fire/mod.rs` (3), `validate.rs`
   (2), `expr_ir.rs` (1), `arm.rs` (2)"*. The proposals themselves — WHAT to cut — were never
   written down; they died with the vigilia's context. **You cannot act on `fire/mod.rs (3)`.**
   That is why nobody did.

   ⛔ **AND THE TALLY'S NUMBERS WERE WRONG — THEY COUNTED TEST LINES AS FILE SIZE.** Re-grounded on
   PRODUCTION lines (everything before the first `#[cfg(test)]`, or brace-matched for per-item
   gates):

   | file | tally read | production | verdict |
   |---|---|---|---|
   | `src/rete/kernel/arm.rs` | 1124 | **593** | **LEAVE — never was a candidate** |
   | `src/rete/kernel/fire/mod.rs` | 1893 | **1593** (316 test-only) | **LEAVE on the 3; ONE real seam, see below** |
   | `src/rete/expr_ir.rs` | 1719 | **2041** | **CUT — the author already drew the seam** |
   | `src/rete/validate.rs` | 2169 | **1990** | **CUT — three concerns, no seam drawn** |

   ⚠ **I mis-measured this twice before getting it right, and the corrections are the method.**
   First pass counted "everything after the first `#[cfg(test)]`" and reported `fire/mod.rs` as
   1842 test lines — **false**: those are `#[cfg(test)]` on INDIVIDUAL production fns, not one
   trailing test module. Brace-matched, it is 316. A file-size number is worthless without knowing
   which half it measures, which is exactly how the original tally went wrong.

   **THE TWO CLOSURES, affirmative:**
   - **`arm.rs` — LEAVE.** 593 production lines, 2 author seams, one coherent job (intern the arm at
     compile-all and persist it beside the network). It was only ever on the list because 531 lines
     of tests were counted as file size.
   - **`fire/mod.rs` — LEAVE on the three proposed splits.** But the re-audit found **one seam the
     tally never named, and it is machine-checkable**: 316 lines across 8 `#[cfg(test)]` regions —
     including all four `// ── Pass N` reference passes (`alpha_pass`, `root_join_pass`,
     `keyed_join`, `production_pass`) — sit in the same file as the 1593 lines of production helpers
     the ARMED path uses (`extend_token`, `rematch_compiled`, `driver_of`, `fact_holds_under`, …).
     **Two reasons to change**: the reference passes move when the oracle does, the helpers when the
     fused loop does. Real, but small and zero-risk — logged, not scheduled.

   **THE TWO CUTS, named so they are actionable — this is what the tally should have said:**
   - **`expr_ir.rs` → split at `// ── exec` (line 890).** The author drew this seam themselves. Above
     it: building the `Expr` DAG (`lower_in_frame`, `lower_list`, `lower_expr`, the `#holon` fold).
     Below it: evaluating one (`exec`, the opcode jump table). **Lowering and execution are two
     reasons to change**, and 2041 lines is the largest production file in `src/rete/`.
   - **`validate.rs` → three concerns, and NO seam is drawn anywhere in 1990 lines.** Reading its 38
     fns they group cleanly: the **`:when` validator** (`validate_query_when`, `validate_when_entry`,
     `validate_plain_condition`, `validate_typed_clauses`, `validate_clause`), the **`:then`
     validator** (`check_rhs_operands`, `walk_nested_constructors`, `validate_then_form`,
     `reorder_then_kwargs`, `rhs_operand_can_never_resolve`), and the **operand typer**
     (`resolve_operand_type`, `check_operand_field_ref`, `rete_type_segment_of`,
     `keyword_constant_segment`, `is_non_field_keyword`, `describe_operand`). The typer is the
     self-contained one and the one this session edited.

   ⚠ **NEITHER CUT CAN COMPUTE A WRONG ANSWER.** Both are hygiene against live defect work, which is
   the honest reason to rank them low — not a reason to leave them untracked, which is what produced
   this item.

   ✅ **AND ONE PARTIRE FINDING WAS ALREADY DONE, uncounted by this item:** `delta.rs`'s 1774-line
   `fire_fixpoint_delta_armed` (9 passes, 12 nesting levels, 16 top-level mutable locals) was cut
   along its own author-drawn seams — `DESIGN-STONE-partire-fire-loop.md`. `delta.rs` is now 822
   lines beside `acc.rs`, `rules.rs` and `pass/`. **The concrete partire finding shipped; only the
   tally lingered.**
5. **TRACKED DECISIONS ① and ②, and the `CLAUDE.md` delivery gap** — **all three AUDITED against
   the tree 2026-08-28 and all three LIVE AND ACCURATE.** First inherited row this session that was
   not stale; the streak was 6-for-6, so that is worth recording too.

   - **① the cache LRU's panics — MOVED OUT of 278 by builder ruling** (*"this is unrelated to
     rete/278"*). Now
     `docs/arc/2026/04/109-kill-std/NOTE-the-cache-lru-panics-on-a-value-that-arrives-from-durable-storage.md`.
     The merits ruling is re-verified and **SHARPENED: the three panics do not answer the same
     way.** `Lru::new`'s capacity crosses a serialization boundary (`wat/cache.wat:132` —
     `:durable [capacity <- i64]` is the SPEC the resource is rebuilt from), so a stored `0` panics
     at REHYDRATION with no caller in the frame — fallible input. `put`/`get`'s non-hashable key is
     supplied at the call site and the checker rejects it at most of them — a genuine caller bug.
     **Recommendation: convert `new` only, leave `put`/`get`** — a third of the original surface
     churn. Awaiting mandate.
   - ✅ **② `match` map-destructure — CLOSED 2026-08-29. `:md::Point{40,2}` -> 42 now works in a
     rete rule, in BOTH positions.** It was the LAST `v1` refusal in the rete expression core, and
     it fell to the same move as every other denial this arc removed: *"not lowered in v1"* is a
     STATUS, not a reason.

     **The design question answered itself once measured.** The settled sibling
     `(:ns::Type/field ?x)` compiles its index because class AND field are both in the accessor
     head. For `{vx :x}` the field is in the pattern and the class is the subject's — which looked
     like the difference. It is not: **core must dispatch on the receiver at runtime because
     nothing declares it, while a rete `?p` gets its class from the fact pattern's declared field
     type. Rete has MORE static information here, not less.** The refusal had inherited core's
     runtime-polymorphism problem into a place that does not have it.

     **⛔ AND MY FIRST CUT MINTED FIX-LIST F's CLASS FRESH.** It returned "arm does not match" for
     a field the class does not declare. Core RAISES `UnknownField` there — verified, and it
     raises even with a catch-all arm after it. Silently not-matching would have meant the same
     expression answering differently in the two engines, and would have turned a typo into a
     constraint that compiles, fires and matches NOTHING. It raises now, carrying the class and its
     available fields. **That row is the gate's load-bearing one**; mutated back, the failure prints
     `PatternMatchFailed: no arm matched`, which teaches nothing about the typo.

     Accepts ONLY the hash-destructure, matching core's own rule: `{:keys […]}` and plain map
     literals are refused BY NAME so the diagnostic teaches the spelling that works.
     ⚠ **The field index is resolved at match time, not compiled** — `LowerCx` carries no type for
     a slot. Stated in `Pat::Fields`'s doc with exactly what compiling it would take (thread
     `validate.rs`'s `collect_rule_bind_types` into `LowerCx`); a pure win, no semantic change, if
     it ever shows on a profile.
     Gate: `a_match_hash_destructure_binds_fields_in_both_positions`, mutation-proven.
   - **the `CLAUDE.md` delivery gap — CLOSED 2026-08-28**, and written as a POINTER rather than
     either proposal on the table. Both proposals (paste the subset / `@` import) create a second
     copy of the doctrine, and this row exists BECAUSE a second copy went stale; a pointer asserts
     nothing about wat-rs's content and so cannot rot. Full reasoning in
     `NEXT-STRIKES-theater-hunt.md` § "Not perf — a guardrail hole".
     ⚠ **The edit is UNCOMMITTED — holon root git is FROZEN.**
6. ~~**`circumspicere` 1 (grid SPEED half in CI)**~~ — **CLOSED 2026-08-29. It runs, as its own
   `grid-speed` job, gated on per-axis RATIO FLOORS.**

   **BOTH stated reasons for its absence were dead, and the second one had never been measured.**
   The first — *"needs Clara and a JDK the runner lacks"* — expired 2026-08-27 when `parity`
   installed Temurin 21 and a pinned Clojure CLI. The second, offered as the still-good argument,
   was *"a shared runner is noisy, so a wall-clock gate would flap"*. Measured against the recorded
   33-cell grid:

   | | |
   |---|---|
   | tightest cell | **8.50x** (`fanout [40000]`) |
   | median | 22.09x |
   | widest | 59.11x |
   | verdicts | 33/33 `:us`, 33/33 `:match` |

   **We are nowhere near parity, so runner noise cannot flip a verdict** — and that same margin is
   why the gate does NOT test `:winner`: at 8.5x it fires only on catastrophe and **would have
   missed the real 4x regression this arc already found and fixed.** A `:winner` gate would have
   been the obvious choice and a nearly vacuous one.

   **The shape that earns it: a per-axis ratio FLOOR at ~50% of that axis's recorded minimum.**
   A ratio cancels runner speed (both engines, same job, same box — which is what the row's own
   note meant by "a ratio against Clara measured in the same job"), and a 2x trip margin cannot
   flap. Floors are per-axis because the ratios legitimately span 8.5x–59x; they are the artifact,
   and the script says to raise one only with a new recorded grid to cite and never to lower one to
   clear a red.

   `GRID_RUNS=1` in CI is deliberate and stated: the 3-run convention exists so a near-parity
   `:winner` (a ±5% band) is not read off one sample; a 2x floor is settled by one. Cost **2m24s**
   as its own job, so wall-clock stays flat and a red is attributable to the grid.

   **Also gated: `:accuracy :MISMATCH`.** The perf corpus is NOT the where-family the `parity` job
   diffs — a native-vs-Clara disagreement on a perf axis is a wrong answer nothing else here sees.

   **Mutation-proven on three arms** — a floor raised above its cell names the axis, size, ratio and
   floor; an unfloored axis is refused rather than skipped; a short sweep exits 2 (*"a gate that
   cannot fail, not a green one"*). The failure path's EXIT CODE was verified to be 1, not just its
   message — a gate that prints FAILED and exits 0 is the trap this repo's floor discipline exists
   for.

7. **THE HOLON SURFACE IN RETE.** rete carried **4 of ~40** data-shaped holon ops and all four
   were from ONE group (similarity): it could COMPARE two holons handed to it as fields and do
   nothing else — no constructor, no accessor, no shape predicate. Same shape as `Tuple`
   (constructible, unreadable) and `keyword` (thinnest surface) — the third instance of one pattern.

   ⏸ **PARKED 2026-08-28 BY BUILDER RULING — the similarity tooling was the goal and it is DELIVERED.**
*"I think we park the is verbs … maybe the bulk of the remaining holonic rete items for now.. the
similarity tooling is supported which I wanted."*

**What shipped and stands:** the four similarity rows (`cosine`, `dot`, `coincident?`, `presence?`)
are minted, generated by the ledger, and FIRE in both positions; `#holon` folds to a constant so a
rule can compare a field against a literal holon; a `:wat::holon::defrecord` record is a rete fact
and matches on its scalar fields. A rule can ask *"how similar is this holon to that one"* — which
is the holonic question rete was wanted for.

**What is parked, and WHY it is a park rather than a deferral** — the reason is written down, and
it is a finding, not a lack of time:

1. ~~Fix the ledger's false `NOT_YET_GENERABLE`~~ · **DONE** — `#holon` IS the literal; see above.
2. ~~Verify `is-List?` / `is-Tag?`~~ · **DONE, and both are REAL.** `is-List?` answers true on a
   `:wat::holon::List` and false on a Vector; `is-Tag?` answers true on the Tag node, which lives
   at `Bind/left` of a uuid holon. Their all-false columns were **"never fired"**, exactly as the
   record suspected — not verified-negative. Probes: `probe-holon-is-list-and-tag.wat`,
   `probe-holon-is-tag.wat`, `probe-holon-shape-literals.wat`.
3. **Mint the 10 predicates** (not 11 — `is?` plus NINE `is-*?`; the old count was wrong) ·
   **⏸ PARKED. I had minted all ten rows and REVERTED them.** Three reasons, all driven:

   · **They are an unfinished v1 placeholder.** Arc `226-type-predicates-vsa-similarity` states its
     mission as *"Type checking emerges from VSA similarity … **Continuous answer.**"* What shipped
     is `extract_classifier(h) == Some("Map")` — a string compare. Its own SCORE says so:
     *"v1 is STRUCTURAL exact-match on classifier name only … v2+ deferred to stones 226.2+"*.
     **Those stones were never written and the arc has no INSCRIPTION.** Minting them would freeze
     a self-declared placeholder into the rule surface — the hardest place to change it.
   · **The VSA route works TODAY and is what the design asked for.**
     `coincident?(Bind/left(h), Atom("Map"))` gives the same answer;
     `cosine(Bind/left(h), Atom("Map"))` gives **1.0 vs 0.0013** — the continuous answer, for ANY
     classifier including a user's own record class. **So step 3 collapses into step 4**: mint one
     accessor and rules compose the rest, instead of ten rows hard-coding one string each.
   · **The classifier they read is BROKEN at the `#holon` door.** `#holon {:a 1}` builds
     `Bind(String("Map"), …)` where the reader needs `Bind(Atom(String("Map")), …)` — and the code's
     own comment one line above specifies the `Atom`. So `is-Map?` is false, `extract-classifier` is
     None, `from-holon` RAISES, and `#holon [1 2 3]` does not coincide with `to-holon [1 2 3]`.
     A shape predicate over an unreadable classifier is meaningless.

     ⛔ **THAT IS CORE, NOT RETE, and it is DEFERRED by builder ruling** — *"i think we should just
     defer this … until we get back into actually using holon."* Reported out:
     `~/work/NOTE-holon-classifier-contract-is-unenforced-and-the-holon-tag-breaks-it.md`.
     **Do not unpark item 7 step 3 before that note is ruled on.**

     ★ **THE DOCTRINE, from the builder, and it is why this is not cosmetic:** *"bundle implements
     map, vector, list, set — **a type needs to wrap it to declare which kind of bundle this is**."*
     `Bundle` is the SHARED substrate for all four container kinds, so the classifier is not
     decoration — it is the only thing that makes a Bundle a TYPE. A bare Bundle is not an untyped
     vector; it is an untyped *nothing*, and no reader can recover which of the four it meant.

     **And the typing is partial at the LEAVES too, which explains the predicate family's shape.**
     `symbol`/`keyword`/`tag` are `classified(...)`; `i64`/`String`/`bool` are bare native variants
     carrying no classifier. That is exactly why the nine named predicates are
     Map/Set/Vector/List/Tuple/Symbol/Keyword/Tag/Nil and there is **no `is-Int?`, `is-String?` or
     `is-Bool?`** — those leaves have nothing to read. **The family's membership is a consequence of
     which parts of the encoding happen to be typed, not a decision anyone made.**

4. **Accessors** — `Bind/left`/`Bind/right` (total via `Option` → `Fallback`) and
   `Bundle/first`/`Bundle/children` (RAISE on the wrong variant). Two accessors, one family, two
   partiality conventions — core's inconsistency, which rete would inherit. **Also parked**, but
   note that `Bind/left` is now the highest-value row on this list, since it is the one that makes
   the VSA route expressible in a rule.

5. **`Bundle` and the `:panic` capacity mode — FOLDED IN FROM ITEM 8, 2026-08-29** (builder:
   *"i think we put #8 into #7 … more holon stuff we need to address now that we've matured
   wat"*). It was tracked as its own item because the RULING is runtime-wide; it belongs here
   because the SUBJECT is the holon surface, and splitting them meant the holon queue read as
   shorter than it is.

   `Bundle` is the ONLY holon op that cannot get a `total: true` row. Under the default `:error`
   mode it returns `(Result :- [HolonAST CapacityExceeded])` and **the type system forces
   handling** — proven: `is-Map?: parameter #1 expects HolonAST; got (Result :- [...])`. Under
   `:panic` it aborts instead.

   ⚠ **A CLAIM I MADE AND THE BUILDER CORRECTED — do not repeat it.** I said
   `set-capacity-mode!` is callable at runtime, so the mode was non-deterministic. **False.**
   Driven: inside `:user::main` it is `unknown function` — a LOAD-TIME DIRECTIVE collected by the
   entry-file pass, exactly as the builder said. The determinism objection is void.

   **What the correction buys:** because the mode is fixed BEFORE `compile-all` runs, the rete
   compiler may READ it — so a `Bundle` row can be `Fallback` under `:error` and refused with a
   located diagnostic under `:panic`. That is a compiler reading a load-time fact, not legality
   varying by config.

   **The surviving argument for killing `:panic`**, and it stands without rete: `:error` already
   forces handling at COMPILE time; `:panic` trades that wall for a runtime crash. Blast radius
   ~6 files (`src/config.rs`, `src/process/boot/mod.rs`, `src/runtime.rs`,
   `tests/collection/bundle_capacity.rs` + 2 fixtures, and
   `probe_plain_panic_produces_structured_edn.wat`, which uses the panic as a VEHICLE to test
   structured-EDN panic rendering and would need another trigger). **Not started. The RULING is
   the builder's; the rete-side consequence is ours.**

   ⚠ **AND `:panic` IS NOW THE SECOND PANIC-VS-ERROR ROW IN A WEEK.** The cache LRU
   (`docs/arc/2026/04/109-kill-std/NOTE-the-cache-lru-panics-on-a-value-that-arrives-from-durable-storage.md`)
   and `edn::write` (fixed 2026-08-29 — the failure channel existed one frame up) were both this
   shape. The pattern each time: an abort where the substrate could already carry an error, and
   in both cases the stated blocker had quietly stopped being true. Worth ruling `:panic` with
   those two on the table rather than alone.

⚠ **`is-Tag?` HAS A REACHABILITY WRINKLE worth keeping** if this is ever unparked: a Tag holon only
ever occurs as `Bind/left` of a uuid holon, so a rule can only hold one if HOST wat put it in a
field. Reachable, but not constructible inside a fence.

8. **THE TERMINATION VERIFIER REFUSES A CLASS OF PROVABLY BOUNDED RECURSION.** Reported by
   claude-compute (the main x grok-rete integration branch) 2026-08-28 as
   `~/work/NOTE-rete-termination-verifier-refuses-provably-bounded-recursion.md`. **Weighed against
   this tree and CONFIRMED** — every citation checked, and the refusal reproduced by driving:

   ```
   N(k+1) :- N(k), (where (< ?k 500))     -> RuleSetMayNotTerminate. It terminates at k=500.
   ```

   The cyclicity test is structural (reachability over fact-type edges) and does not read the
   `where` fence. **The refusal is correct by the verifier's own stated claim** — it proves the
   absence of ONE shape — but a bounded counter is the first thing anyone writes in recursive
   Datalog-with-arithmetic, and until now the ONLY record that it is refused-though-terminating was
   prose inside an unrelated fixture's header.

   **DONE: the minimum ask.** The class is now named in `stratify.rs`'s own "WHAT IT CANNOT SEE"
   block, beside the two holes that were already stated there — which is the model the report itself
   named.

   ★★ **THE eBPF PRIOR ART WAS READ 2026-08-29, AND IT ARGUES AGAINST THE ANNOTATION.** The
   direction recorded here — *"a FORM the verifier can CHECK, eBPF's `bpf_loop()` move, the bound
   as a verified argument"* — was written from the IDEA of eBPF. We have actually shipped a rete
   engine on eBPF (`holon-lab-ddos/veth-lab/filter-ebpf/src/main.rs`, the 1.3M-pps XDP scrubber),
   and it does not work that way.

   **It uses no `bpf_loop`, and nothing declares a bound. Every bound is STRUCTURAL:**
   - **The state is fixed-size.** `DfsState { stack: [u32; 16], fields: [u32; 32], … }` (`:227`).
     A push is guarded `if state.top < 16` and the index is MASKED — `state.stack[(state.top & 0xF)]`
     — and the comment says why: *"Walker masks with `& 0x1F` to PROVE index bounds for the
     verifier."* The mask is not defensive; it is how the bound is proven.
   - **The step ceiling belongs to the HOST, not the program.** *"The kernel enforces a max of 33
     tail calls, giving us up to 32 DFS steps"* (`:842`). The program never states a budget.
   - **There is no loop to bound.** *"The BPF verifier sees this as a ~100-instruction
     straight-line program with 2-3 map lookups and NO LOOPS"* — one DFS step per tail-called
     program, each verified independently.
   - **Branches were HOISTED OUT of the walk** so the remaining step is a bounded lookup: all nine
     packet fields are pre-extracted before the tail call, turning a 9-way dispatch × 20 iterations
     (9^20 paths, past the verifier's 1M limit) into `fields[node.dimension]`.

   ⛔ **AND THE LAB'S RETE HAS NO FIXPOINT AT ALL** — rules are condition→action over packet data,
   no consequent feeds back, one forward pass per packet. So it is prior art for BOUNDED TRAVERSAL,
   and explicitly not for bounded *derivation*. Saying so matters: it is the difference between
   citing it and actually reading it.

   **WHAT THAT IMPLIES HERE, and it reframes this item's own open question.** Rete already has the
   eBPF-shaped mechanism: **`max_fire_rounds` IS the 33-tail-call ceiling** — a host-enforced
   ceiling the program does not declare. This item's open question *"does a bound interact with or
   subsume `max_fire_rounds`?"* answers itself against the prior art: in eBPF the ceiling is the
   ONLY budget, and a `:bound` on the rule would be a SECOND mechanism doing the first one's job —
   the two-places-per-row defect this arc keeps pulling out (FM 30).

   What rete lacks is the OTHER half: eBPF's state is fixed-size, and a fixpoint's fact memories
   are not. **So the honest next question is not "how does the author declare a bound" but "can a
   derivation's state be made structurally finite the way `[u32; 16]` is"** — and if it cannot, the
   round cap is already the answer and the refusal is already correct. That is a real design
   question with a real precedent behind it, and it is NOT the annotation this entry used to name.

   ⛔ **DO NOT PROPOSE AN ESCAPE HATCH.** Two were already refused by builder ruling (a `rune:`
   marker — *"no magic comments"*; and `Termination::Asserted [why <- String]` — *"their strings are
   their reason for themselves?"*). An author's string is not a proof. The direction the design
   already names is a FORM THE VERIFIER CAN CHECK — eBPF's `bpf_loop()` posture, the bound as an
   argument it reads. Open questions belonging to whoever takes it: does a bound interact with or
   subsume `max_fire_rounds`; is per-rule or per-cycle the right granularity; and an imported Export
   carries no AST, so a static bound is meaningless there and the round cap stays the only guard.

   **TWO DIAGNOSTIC DEFECTS FOUND WHILE DRIVING IT — BOTH FIXED 2026-08-28, both mutation-proven:**
   1. ~~The message asserts *"the fixpoint can never converge"*~~ — **FALSE for the guarded
      counter; it converges at k=500.** R29 `RVINA ERVDIT`. The verifier computes a derivation
      graph and does not compute convergence, so the diagnostic was asserting what the analysis
      never established. It now says the rounds are UNBOUNDED and names itself *a refusal to
      certify, not a proof of divergence* — and volunteers that `(where (< ?k 500))` is refused
      too, though it terminates, so the reader meets the narrowing instead of discovering it.
   2. ~~With a fn-headed `:then` the message names the **FUNCTION** as the offending fact type~~ —
      *"derives `:bc::mk-next` … and `:bc::mk-next` feeds back into this rule's own `:when`"*,
      where `mk-next` appears nowhere in the `:when`. **The DETECTION was always right**;
      `computed` was built from `fact_type_head` (the raw head) while `produced` beside it resolved
      through `sym`. Both now use `produced_type`. One resolver, two fields.

   **⛔ WHY DEFECT 2 SURVIVED, AND IT IS THE REUSABLE PART.** The gate existed —
   `a_mint_hidden_inside_a_rete_fn_body_is_refused` — and asserted `rule`, never `fact-type`. It
   **held the wrong value in its hand and only ever looked at the field that was right.** A gate
   that reads a subset of the structured error it already parsed is FM 28 in a new position: a
   count cannot see a value defect, and neither can a partial field check. Both fields are asserted
   now, and the mutation prints `left: "fm::bump" / right: "fm::N"`.

   **The class now has a HOME THAT CAN GO RED**, which prose never could:
   `tests/rete/probe_arc278_termination_guarded_counter.wat` +
   `a_bounded_counter_is_refused_too_and_the_message_does_not_claim_divergence`. This is the very
   fixture the report said had been rewritten around the refusal, restored as a gate. If anyone
   ever teaches the verifier to read the fence, that test fails — which is the notification the
   narrowing closed, not a regression.

   **AND THE DOCTRINE BLOCK WAS WORSE THAN "STALE IN OUR FAVOUR" — it CONTRADICTED the function
   twenty lines below it, and two of its three evidence rows were false.** Struck and rewritten
   2026-08-28 after driving all three. It concluded *"no exploit found, the shape is guarded by
   adjacent fences"* while `rete_fn_body_mints`'s own doc-comment says *"THE HOLE THIS CLOSES, and
   it was demonstrated before it was fixed … it compiled clean and ran to the round cap."* Measured:
   · row 1 (`i64::+` → "is not total") is TRUE and a genuine body fence.
   · row 2 (`total fallback` → "is not a rete primitive") is **MIS-ATTRIBUTED** — that refusal names
     the FN, not the body's op. All three probes used a plain `:wat::core::defn`, so the table
     measured ONE door (Law A) three times and reported it as three independent fences.
   · row 3 (`constructs a record at all` → "`kwargs-construct` is not pure") is **FALSE today** — a
     rete defn whose body is `(:bc::N :k (…::i64::+ k 1 :undefined 0))` declares clean and reaches
     the cyclicity check.
   **The lesson, promoted:** *a refusal is evidence about the door you knocked on, not about the
   room behind it.* Three probes failing for an unrelated reason read as safety for a full day.

   **Zero programs in the corpus trip the verifier today** (report's measurement, and consistent
   with our own green floor). That is exactly when this class is cheapest to widen.

9. **A GOLDEN THAT PINS AN INTERPRETER LINE NUMBER — FOUR false reds in one day, across TWO source files, none of them
    behaviour.** `tests/diagnostics/probe_diagnostic_value_snapshot_in_errors.rs`'s five goldens
    pin `:location #wat.core/Span {:file "src/runtime.rs" :line N}`. On 2026-08-28 that `N` moved
    **three times** — 25722→25793→25799→25802 — and **every move was a COMMENT**
    (`classify_fallback_outcome` gaining a doc note; `eval_quote` gaining one). Each cost a full
    375-second floor to discover, and each fix was one integer in five files.

    **The value being pinned is `rust_caller_span!()`** — the sentinel meaning *"no recoverable
    USER source location"*. What deserves assertion is that the location IS that sentinel (file
    `src/runtime.rs`), never WHICH LINE of it. A line number there teaches a reader nothing and
    taxes every edit above it: a gate testing its own accident.

    ⚠ **FOURTH OCCURRENCE, 2026-08-29, AND IT WIDENED THE CLASS.**
    `tests/process/probe_supervisor_select_lost__process_panics.edn` pins **`src/freeze.rs`**
    line 1521, and a one-line edit there moved it to 1522. So this is not one golden pinning one
    file — `src/runtime.rs` and `src/freeze.rs` are both pinned, by goldens in different suites,
    and any edit above either line costs a 375-second floor to discover. **A survey of which
    goldens pin a `src/*.rs` line is now part of the work**, not an afterthought: the three known
    sites were found by breaking them, which is the worst way to enumerate a class.

    **The likely cure** is in `assert_edn_matches_file!` — normalise a `src/*.rs` span's `:line` on
    both sides — but that macro backs every golden in the repo, so it is its own strike and needs
    its own measurement of what else pins such a line. Not started. The three data points and the
    reasoning are in the probe file's own staleness trail, which is where someone hitting the red
    will actually be standing.

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
