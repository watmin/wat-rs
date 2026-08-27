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

## PILE 2 — owned by `NEXT-STRIKES-theater-hunt.md` § "WHAT REMAINS OPEN". Read it there.

**⚠ AUDIT THE LIST BEFORE WORKING IT — 2026-08-27.** Two of these were checked against the tree
and found STALE, which is a finding about the LIST, not the code:

- **`conformare` x9 — STALE.** Neither cited file (`eval_insert.rs`, `arm.rs`) contains
  `rust_caller_span!` any more, and the sites it named now use real wat spans
  (`acc_form.span().clone()`). Verified by BEHAVIOUR, not by grep: a user's malformed `:then`
  reports `:location` at their own file, at the offending operand's line.
- **`intueri` x3, the `validate.rs` row — STALE.** That file now renders through `render_form` and
  says so in its own doc, recording that it *"used to be `{other:?}`"*.

**But probing the stale finding surfaced a LIVE one it did not name**, now fixed: `eval_insert` and
`compiled_rhs` rendered the offending operand into `:got` with Rust `Debug`, so an unbound `?var`
in a `:then` showed the user
`Symbol(Identifier { name: "?nope", scopes: {} }, Span { file: … })` — hygiene scopes and a nested
span, for a typo. Routed to `validate::render_form` (the printer `write-forms` already uses) rather
than growing a second renderer; both halves of the compiled/interpreted differential moved together
because their errors are contracted BYTE-IDENTICAL. Gated by
`tests/rete/probe_arc278_then_operand_rendered_as_source.rs`.

**Still standing, unverified:** `vocare` x6, the two remaining `intueri` rows, `exigere` x1. Check
each against the tree before working it — the hit rate on this list is now 2-for-2 stale.

**None of these can produce a wrong answer** — that is why they sit below the piles above, and
also why they will never be found by a differential. `conformare` x9 is the one with real user
impact.

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

### 4.1 A reachability ledger over `RETE_OPS` — and it must be PER CALL-SITE KIND
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

1. **4.1 the reachability ledger** — small, converts a proven-live defect class into a standing
   gate, and immediately tells us how much dead surface there is.
2. **1.1 interleaved retract** — DONE 2026-08-27.
3. **4.2 the termination verifier** — DONE 2026-08-27. The fn-headed `:then` hole was named, then
   investigated and found to be guarded by three adjacent fences with no exploit constructible. It
   is NOT the head of the list; **3.2 (no CI job checks the arc's own closing condition) is**,
   because every Clara agreement this session was established BY HAND and a parity regression
   still merges fully green.
4. ~~3.2 CI parity~~ DONE 2026-08-27. Next: **1.2 generated rules** (the whole `:then` side and
   multi-rule interaction), then the PILE 2 tail with `conformare` x9 first — it is the only ward
   finding with real user impact.

> ⚠ **A green fuzzer is not an empty list.** 4104 shapes at zero divergences means the engine is
> correct IN THE REGION THESE GRAMMARS REACH. Both spaces are hand-authored; they cover what
> someone thought to encode. That is why the vacuity findings of 2026-08-27 matter more than they
> look — a vacuous dimension shrinks the region without shrinking the number.
