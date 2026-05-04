# Arc 148 — Arithmetic + Comparison Correction — INSCRIPTION

## The closing

Arc 148 shipped 2026-05-03 across one extended session. Six slices
(1 audit, 2 rename, 3 values_compare buildout, 5 cleanup, 4 boss
fight, 6 closure). Plus four unanticipated side-quest arcs the
cascade surfaced and demanded — arc 132 amend (default time-limit
raised 200ms → 1000ms), arc 149 (Ratio scratch), arc 150 (variadic
`:wat::core::define`), arc 151 (wat-macros wrapper honest-message
stub) — plus a TypeScheme refactor that absorbed sonnet's slice-1
sibling-map back into the inline field where it belonged.

**The arc opened wanting to retire one polymorphic-handler
anti-pattern.** It closed with arithmetic + comparison fully
first-class, the substrate's variadic-define gap closed, the
time-limit safety-net widened, the Ratio architecture sketched
for future capture, and a wrapper-message lie stubbed for cleanup.
The substrate-as-teacher cascade did exactly what it was designed
to do: each slice surfaced the next link.

**The user's framing — owned: "we just made the UX of the language
remarkably better."** That's the load-bearing claim and it's
substantively true. Arithmetic + comparison are now LLM-natural
by construction: bare polymorphic names default; type-locked
variadics for explicit-Type usage; comma-typed leaves reachable
per no-privacy but rarely needed; one consistent rule about when
each shape applies. The substrate stopped requiring users (LLMs!)
to learn surface idiosyncrasies — it now follows the conventions
they already know from Lisp.

## What ships under arc 148

### The numeric arithmetic + comparison architecture (LOCKED + SHIPPED)

For every arithmetic op `<v>` ∈ {`+`, `-`, `*`, `/`}: **8 entities** —
1 polymorphic surface + 1 binary Dispatch entity (`,2`) + 2 same-type
variadic wat fns + 2 same-type binary Rust leaves (renamed in slice
2) + 2 mixed-type binary Rust leaves. **32 names per arithmetic.**

For every comparison op `<v>` ∈ {`=`, `not=`, `<`, `>`, `<=`, `>=`}:
**1 entity** — 1 polymorphic substrate primitive at the bare name,
backed by `values_compare`/`values_equal` for universal same-type
delegation + selective mixed-numeric arms (handled internally by
the comparison helpers, not as named leaves per the comma-typed-leaf
rule). **6 names per comparison.**

**Total numeric arc 148 surface: 4 × 8 + 6 = 38 first-class entities.**
Each queryable via `signature-of` / `lookup-define` / `body-of`.
Each addressable directly per arc 109's no-privacy doctrine.

### LLM-affordance pyramid (the user-facing UX)

```
DEFAULT REACH                    Lisp-natural; type-system enforces
  (:wat::core::+ x y z)          at the binding site
  (:wat::core::< x y)

TYPE-LOCKED REACH                Useful when LLM wants to assert
  (:wat::core::i64::+ x y z)     "this operates only over i64";
  (:wat::core::f64::* x y z)     compile-time error on f64 mixed in

SUBSTRATE ADDRESSING             Reachable per no-privacy; rarely
  (:wat::core::+,2 x y)          needed in everyday code; the
  (:wat::core::+,i64-f64 x y)    substrate being honest about
  (:wat::core::i64::+,2 x y)     routing internals
```

Future LLMs writing wat code can stay at the top tier — the
substrate handles the rest.

### The comma-typed-leaf rule (architectural principle, captured for future)

> Comma-typed leaves (`:<verb>,<type-pair>`) exist iff the underlying
> Rust impls genuinely differ per type-pair.

| Family | Comma-typed leaves needed? | Why |
|---|---|---|
| Arithmetic | YES — `:+,i64-f64`, `:+,f64-i64`, etc. | Rust impls differ (i64+i64 ≠ i64+f64) |
| Comparison | NO | One `values_compare` helper handles universal delegation including mixed-numeric |
| Future Ratio arc | YES — `:+,i64-Ratio`, etc. | Rust impls differ across the numeric tower |

The rule survives arc 148's closure as a CONVENTIONS-level principle
future arcs apply. Documented in DESIGN; honored by every entity
introduced this arc.

### The arity rules (Lisp/Clojure tradition adopted)

Arithmetic:
- `(+) → 0:i64`; `(*) → 1:i64`; `(-) / (/)` → ARITY ERROR
- 1-ary inserts identity-on-left: `(- x) → -x`; `(/ x) → 1/x`
- Type-preserving: `(:/ 5) → 0:i64` (integer truncation; honest)
- 2+-ary folds-left via the binary Dispatch

Comparison: strict binary. `(:< 1 2 3)` is rejected (chained
comparison is meaningless under fold semantics). Chains via
`:and` explicitly.

### Foundation principles established (carry forward beyond arc 148)

1. **The "extend the carrier" pattern.** `Function` struct accretes
   optional fields (`rest_param`, `rest_param_type`, future `docstring`).
   Every signature feature follows this template. Arc 141 (docstrings)
   becomes a pattern-application slice.
2. **Path C — custom inference is honest** when the substrate's
   TypeScheme system can't express the polymorphism cleanly.
   `infer_polymorphic_compare` → `infer_comparison`;
   `infer_polymorphic_arith` → `infer_arithmetic`. Renaming retired
   the anti-pattern framing; the function itself is honest substrate.
3. **The cascade's compounding signature.** Each foundation slice
   pays forward: arc 146 Dispatch + arc 150 variadic define were
   both surfaced + closed before they were needed; the boss-fight
   slice (slice 4 — the largest scope) shipped in 18 minutes once
   the gear was in hand.

## The cascade — what arc 148 surfaced and closed along the way

### Arc 132 amendment — default deftest time-limit raised (200ms → 1000ms)

User-driven during slice 5 verification. The user observed
"shocking amount of failed tests" in `cargo test`; orchestrator
initially speculated multi-threaded contention without measuring;
user pushed back; we measured. Single-threaded canonical view:
clean. Default-thread cargo test: 8-10 flaky timeouts on
computationally-heavy tests + spawn/program tests. The 200ms
default was tight under realistic parallel-test execution.

User direction: raise to 1000ms. Foundation fix shipped in commit
`0a8d6e5`. **Result: 8-10 flakes → 0 flakes**; remaining workspace
failures are documented arc 130 in-progress noise only.

### Arc 149 — Ratio support (scratch arc captured)

User asked about Clojure-style native rationals during the arity-
rule debate. Coherent substrate addition; not arc 148's scope;
future arc when a real lab use case demands exact ratios. The 5
open questions surfaced in this session (mixed Ratio×f64 coercion;
equality across numeric types; canonical form; type-locked variant
semantics; display) are captured at `docs/arc/2026/05/149-ratio-support/DESIGN.md`
so future-self picks up cleanly.

### Arc 150 — variadic `:wat::core::define` (the foundation gap)

The slice 4 boss fight's first attempt discovered that
`:wat::core::define` had never supported variadic rest-params (only
`:wat::core::defmacro` did, since arc 029). 30+ arcs of substrate
work and nobody had needed variadic user functions until arc 148
asked for them. **The substrate-as-teacher pattern surfaced an
arbitrary asymmetry that was always there but never named.**

Per discipline ("eliminate failure domains; don't bridge"),
arc 150 closed the gap at the substrate layer rather than working
around it. Function struct gains `rest_param: Option<String>` +
`rest_param_type: Option<TypeExpr>`; parse_define_signature
mirrors parse_defmacro_signature 1:1; apply_function handles
variadic arity + Vec rest-binding; reflection round-trips the
shape. ~24 min substrate work end-to-end (slice 1 ~19 min +
TypeScheme inline cleanup ~5 min).

The "extend the carrier" pattern arc 150 established now serves
arc 141 (docstrings) + every future variadic surface (format,
log, pipe, etc.).

### Arc 151 — wat-macros wrapper honest-message (stub captured)

User caught a real foundation crack during slice 5 investigation:
the wrapper at `crates/wat-macros/src/lib.rs:722-725` reports
"exceeded time-limit" when a thread completes-but-doesn't-signal
(rare defensive case). The lie: NOT a timeout; thread terminated
without panicking. Honest message would be "thread completed but
did not signal — likely a substrate inconsistency."

Stub arc captured at `docs/arc/2026/05/151-wat-macros-wrapper-disconnected-honest/DESIGN.md`.
Small future fix; not blocking; on the deck.

### TypeScheme inline-field cleanup (mid-arc-150 follow-up)

Sonnet's arc 150 slice 1 used a sibling-map workaround
(`CheckEnv.variadic_rest_types: HashMap<String, TypeExpr>`)
because of an incorrect assumption that mass-edit tooling
(sed/perl/python) wasn't available. Functionally equivalent
but architecturally non-ideal.

Per user direction ("if there is something we deferred we do it
now"), orchestrator verified tool availability empirically (`which`
returned paths; sed -i works; python3 runs), then folded the
sibling map back into TypeScheme as a proper inline field via a
24-line python state-tracker that walked brace-depth from each
`TypeScheme {` opening and inserted `rest_param_type: None,` at
the matching closing brace. **All 215 substrate-primitive
struct-literal sites updated cleanly in ~5 minutes.**

Lesson recorded: sonnet should empirically test tool availability
before claiming a constraint. Cost of testing: ~2 seconds. Cost of
wrong assumption: a follow-up arc to clean up.

## Slice-by-slice ship record

| Slice | What | Wall clock | Mode |
|---|---|---|---|
| 1 | AUDIT — enumerate all 7 polymorphic_* handlers | ~10 min | A clean |
| 2 | Rename per-Type arithmetic leaves to `,2` | ~46 min | A clean |
| 3 | `values_compare` ord buildout (8 new arms; 46 tests) | ~10 min | A clean |
| 5 | Comparison cleanup (10 per-Type leaves retired) | ~25 min | A clean |
| 4 | **Numeric arithmetic migration (the boss fight)** | **~18 min** | **A clean** |
| 6 | Closure — this paperwork | small | A |

**Cumulative slice time: ~110 min sonnet work** for the entire
arithmetic + comparison architecture transition. Plus arc 150's
~24 min for the variadic-define foundation.

The compounding signature — slice 4's boss fight (the largest
scope) shipped in 18 minutes BECAUSE arc 150 + slice 5 + slice 3 +
slice 2 + arc 146 + arc 144 had laid the foundation. Each prior
slice's substrate work paid dividends.

## What the substrate gained — counted

- **38 first-class numeric entities** (32 arithmetic + 6 comparison)
- **8 mixed-type Rust primitives** for arithmetic (mixed-numeric
  promotion)
- **8 same-type variadic wat fns** for arithmetic (using arc 150)
- **4 binary Dispatch entities** for arithmetic (arc 146 pattern)
- **8 new ord arms** in `values_compare` (time + Bytes + Vec + Tuple
  + Option + Result + Vector; Bytes covered by Vec recursion)
- **2 anti-pattern framings RETIRED** — `infer_polymorphic_compare`
  → `infer_comparison`; `infer_polymorphic_arith` → `infer_arithmetic`
- **`eval_poly_arith` RETIRED** — replaced by `eval_arithmetic_variadic`
  + `apply_arith_pair` + `ArithOp` (with arity-rule helpers)
- **10 per-Type comparison leaves RETIRED** (slice 5; the comparison
  surface collapsed cleanly under the universal-delegation rule)
- **`TypeScheme.rest_param_type` inline field** (215 substrate-primitive
  literal sites updated; sibling-map removed)
- **Comma-typed-leaf rule** captured as architectural principle
- **LLM-affordance pyramid** captured as documented UX

## Test surface added

- `tests/wat_arc148_ord_buildout.rs` — 46 tests (ord coverage)
- `tests/wat_arc150_variadic_define.rs` — 16 tests (variadic-define
  semantics)
- `tests/wat_polymorphic_arithmetic.rs` — extended from 20 → 33
  tests (+13 covering variadic + mixed-numeric + identity rules)

**75 new tests across the cascade.** All passing. All deterministic.
All exercise the new architecture end-to-end.

## What this arc does NOT close

Per the user's parallel-track scoping (2026-05-03):

- **Category B — time arithmetic** (`:wat::time::+`, `:wat::time::-`).
  Handler `infer_polymorphic_time_arith` not yet retired; the
  Instant ± Duration patterns intact. Future arc.
- **Category C — holon-pair algebra** (`:wat::holon::cosine`,
  `:dot`, `:coincident?`, `:coincident-explain`, `:simhash`).
  4 polymorphic_holon_* handlers; algebraic surface; future arc.

Both categories are now in the user's parallel track per session
direction. The architectural patterns arc 148 established (Dispatch
entity + per-Type leaves where Rust impls differ; custom inference
where TypeScheme can't express) apply directly when those arcs spawn.

## What this arc unlocks

- **Arc 146 slice 5 closure** — was BLOCKED on arc 148 completion;
  unblocks at this commit. Arc 146's INSCRIPTION can ship.
- **Arc 144 closure** — verification + paperwork queue becomes
  tractable; the polymorphic-handler anti-pattern's last vestiges
  for the numeric families are cleared.
- **Arc 130 reland** — substrate-foundation now stable enough for
  the LRU substrate reshape's RELAND v2.
- **Arc 145 (typed `let`)** — small slice; pattern-application atop
  the now-rich Function struct.
- **Arc 147 (substrate registration macro)** — could fold the 215
  TypeScheme literals into a more maintainable form; arc 150's
  cleanup proved the mass-edit pattern works.
- **Arc 141 (docstrings)** — pattern-application slice atop the
  "extend the carrier" template arc 150 established.
- **Arc 151 (wrapper honest message)** — small substrate fix.
- **Arc 109 v1 closure** — the impeccable foundation milestone the
  user has been chasing for a week. Arc 148's chain link closes
  cleanly.

## Methodology — what worked, what's worth carrying forward

### The substrate-as-teacher cascade

Each slice's pre-flight crawl surfaced the next arc's necessity.
Arc 148 slice 1 (audit) revealed the per-Type leaves at bare names
→ arc 148 slice 2 (rename). Arc 148 slice 4's first attempt
discovered the variadic-define gap → arc 150. Arc 132's 200ms
default surfaced under multi-binary contention → arc 132 amend.
Sonnet's mass-edit assumption failure → TypeScheme cleanup +
recorded discipline lesson.

The cascade compounds. Foundation work is genuinely the fast path
when "fast" means "shipping the right thing without bridging."

### The four questions discipline

`Obvious / Simple / Honest / Good UX` — applied to every
architectural call this arc. Stop at first NO. The discipline
prevented multiple potential anti-patterns (unifying-comma scheme;
slash-stacking; sibling-map permanence; comparison comma-typed
leaves; variadic-as-defmacro path).

### The protocol works

§ 7 pre-flight checklist (substrate-informed BRIEF + scorecard
EXPECTATIONS + FM 9 baselines pre-spawn + STOP-at-first-red +
2× time-box + non-overlapping work queued). Honored across all
sweeps. Calibration tightened with each sweep — slice 4's
prediction was 60-90 min Mode A; actual ~18 min. The discipline
has earned the trust that lets the rhythm hold.

### Sonnet's track record this arc

5 sonnet sweeps (slices 1, 2, 3, 5, 4 — slice 6 is orchestrator
work). All Mode A clean. Cumulative wall-clock: ~110 min for the
entire arithmetic + comparison transition. Per-slice honest
deltas surfaced (3 in slice 1; 4 in slice 3; 0 in slice 5; 3 in
slice 4) and addressed within scope or in clean follow-ups.

The trust-but-verify discipline holds: orchestrator independently
re-runs load-bearing rows; sonnet's claims verified before SCORE
ships. Zero shipped workarounds.

## The user's words — captured for the record

> *"we just made the UX of the language remarkably better"*

> *"if there is something we deferred we do it now"*

> *"the fewer of these 'comma typed funcs' the better — they are a
> crutch to make a strongly typed lisp feel more lisp-y. i'm
> chasing UX gains to make it easier for LLMs to produce wat code
> organically while being forced to live with the constraints the
> language is imposing"*

> *"this whole 109 endeavor is meant to find flaws in our system...
> when we find a crack in the system we ask the questions and
> choose the durable long term solution no matter how much time it
> takes to build it"*

> *"sounds you like found the path — let's map it out and watch
> sonnet beat the dungeon"*

> *"close it out — nice job"*

> *"we just did a hell of a side quest — got some incredible loot
> — let's give sonnet the equipment for this next fight — let's
> see how they handle the boss a second time"*

The work is real. The methodology is real. The foundation
strengthens with every arc. The user shapes the discipline; the
substrate teaches the next link; sonnet executes the briefs;
orchestrator verifies + records.

## Cross-references

- **Inside arc 148**: DESIGN.md (the locked architecture);
  AUDIT-SLICE-1.md; SCORE-SLICE-2.md (rename); SCORE-SLICE-3.md
  (values_compare); SCORE-SLICE-5.md (comparison cleanup);
  SCORE-SLICE-4.md (boss fight)
- **Cascade arcs**: arc 146 (Dispatch entity precedent + slice 5
  closure unblocked here); arc 150 (variadic-define foundation —
  surfaced + closed mid-arc); arc 149 (Ratio scratch); arc 132
  amend (commit `0a8d6e5`, time-limit raise); arc 151 (wrapper
  honest message stub)
- **Discipline**: COMPACTION-AMNESIA-RECOVERY.md § 7 + § 12 +
  FM 4 + FM 9 + FM 10 (all honored)
- **Foundational artifacts updated this arc**: DESIGN.md (numerics
  architecture locked); USER-GUIDE.md (variadic functions
  subsection added by arc 150 closure; arithmetic + comparison
  surface added by this slice 6); FOUNDATION-CHANGELOG.md (lab
  repo; arc 148 row to be added by slice 6)

## Status

**Arc 148 closes here.** The polymorphic-handler anti-pattern for
arithmetic + comparison is RETIRED. Every numeric op is a
first-class entity. The LLM-affordance pyramid ships per-DESIGN.
The cascade arcs (132 amend, 149, 150, 151, TypeScheme cleanup)
are all on disk.

The methodology IS the proof. The rhythm held. The dungeon yielded
its loot.

**Onward to arc 109's v1 closure.** The impeccable foundation
milestone the user has been chasing approaches by another major
chain link's worth.

---

*the cascade compounds. the foundation strengthens. the substrate
teaches. the user directs. the methodology proves itself.*

**PERSEVERARE.**
