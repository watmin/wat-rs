# Sub-DESIGN — Stone 233.2.g — Tracked-unwrap class re-evaluation

**Status:** ACTIVE (2026-05-23 night post-compaction). Sub-DESIGN under arc 233 Stone 233.2.

**Driver:**

Stone 233.2.f (commit `51d83e1`) closed two pattern-match sites in `eval_apply` that were silently mis-dispatching `Value::Tracked`-wrapped values. The follow-up audit found **~20-40 more sites with the same shape** across `src/`. The trap-door is **structural**: Rust's pattern matching on `Value::variant` doesn't auto-unwrap the `Tracked` wrapper variant that Shape C introduced in Stone 233.2.a.

User direction 2026-05-23 night:

> *"we strike to kill, we move with confidence - we study every failure we encounter to ensure it never happens again"*

This sub-DESIGN re-evaluates the Shape decision (made in `DESIGN-STONE-233.2.md`) under new evidence. The Shape decision was made BEFORE we knew the ongoing cost of Shape C. Now we know.

## Trap-door measurement findings (this session's audit)

| Surface | Count | Risk |
|---|---|---|
| `require_X` / `expect_X` helpers (time.rs, spawn.rs, io.rs likely) | ~7 confirmed | **HIGH** — uniform shape; every caller of helpers vulnerable when producers return wrapped values of the expected variant |
| `if let Value::variant` silently-skipping conditionals | 3 confirmed (runtime.rs:2843, 2885, 20012) | **HIGH** for register_runtime_defs_form (fn-registration silently misses); **LOW** for variance loop |
| `match v { Value::variant }` inline (RunResult / Struct field parsers) | ~5 confirmed (test_runner.rs) | **MEDIUM** — Struct field values stored by producers could be wrapped |
| Total at-risk substrate sites | **~15-20 confirmed**, **~20-40 estimated post-full-audit** | substrate-wide latent class |

Total `Value::variant` arm starts in src/: **350+** across 14 files. NOT all at-risk (many match on `DiagnosticValue`, `WatAST`, `HolonAST` — different enums with no Tracked variant). True at-risk subset requires per-site context analysis.

**Trap-door incidence this session:**

We fell through this class **3 times** in a single session:
1. "Intentional gap" framing on substrate-symmetry (caught via four-questions)
2. "Arc 234" scope inflation (caught via user challenge)
3. Apply Tracked-unwrap defect (caught during 233.2.d Row 6 honest delta)

Plus the meta-trap I just fell into ("arc 235" instead of "Stone 233.2.g"). **Four trap-doors in one session is data, not coincidence.** The class is alive and reproducing.

## Shape candidates (under re-evaluation)

### Shape A — `TrackedValue` struct wrap [PREVIOUSLY REJECTED]

```rust
pub struct TrackedValue {
    pub value: Value,
    pub provenance: Provenance,
}
// eval returns TrackedValue; pattern matching forced via .value extraction
```

**Original rejection rationale** (per `DESIGN-STONE-233.2.md`):
- "MASSIVE. Touches every place that uses `Value` becomes `TrackedValue`."
- "Practically infeasible without multi-week effort."
- "Collections-as-holons (arc 216) carefully shaped Value semantics; wrapping breaks the HolonAST encoding contract."

**Re-evaluation under new evidence:**
- The "practically infeasible" framing was BEFORE we knew Shape C's ongoing cost
- Collections argument: TrackedValue ONLY at eval boundary; inside collections, bare Value lives; wrapping happens on extraction
- Pattern matching becomes structurally impossible to forget: `match tv { ... }` doesn't compile (tv is a struct); caller must do `match tv.value { ... }` — and `tv.value` is bare `Value` (no Tracked variant exists)

### Shape C — current `Value::Tracked` variant [SHIPPED]

```rust
pub enum Value {
    // ... existing variants ...
    Tracked { inner: Box<Value>, provenance: Provenance },
}
// Discipline: callers do v.inner() before pattern-match
```

**Status:** shipped via Stones 233.2.a/b/c. ~5 producers tag. ~15-20 at-risk consumer sites confirmed (probably ~40 total). Stone 233.2.f closed 2 sites.

**Ongoing cost (now measured):**
- Per-defect-fix labor: ~5 min per site (Stone 233.2.f precedent) — but cumulative across the audit
- Per-future-site discipline: every new match-on-Value author must remember `.inner()`
- Per-future-producer discipline: each new producer's value flows into N consumer sites; coverage gap multiplies
- **Trap-door incidence: 3+ per session.** Calibration: failure rate >> what discipline-only can suppress

### Shape D — match-via-macro convention [NEW CANDIDATE]

```rust
// Custom macro that expands to match v.inner() { ... }
match_inner!(v, {
    Value::i64(n) => ...,
    Value::String(s) => ...,
    _ => ...,
});
```

Convert all ~350 match sites to use macro. Clippy lint forbids direct `match v { Value::variant }`.

**Analysis:**
- Cost: ~350 site sweep (LARGER than Shape A's eval-boundary refactor)
- Convention-based; structural enforcement only via clippy lint
- Adoption pressure on every future author
- Doesn't eliminate Tracked variant from pattern-match exposure

### Shape E — variant privatization [NEW CANDIDATE]

```rust
pub enum Value {
    // ... pub variants ...
    Tracked { inner: Box<Value>, provenance: Provenance },  // hide from external API
}
impl Value {
    pub fn bare(&self) -> &Value { self.inner() }  // recursive unwrap
}
// External crates: forced to .bare() before match (Tracked variant non-matchable from outside)
// Internal: still vulnerable
```

**Analysis:**
- Wat-rs IS the substrate — most match sites are INTERNAL. Privatization helps external consumers but doesn't fix internal discipline gap
- Doesn't address the trap-door class for the ~15-20 internal sites already at-risk
- Half-measure

## Four-questions per shape (atomic YES/NO; any NO disqualifies)

| Shape | Obvious? | Simple? | Honest? | Good UX? | Verdict |
|---|---|---|---|---|---|
| **A** (TrackedValue struct + retire Value::Tracked variant) | YES (eval-boundary; bare Value inside collections) | NO (large refactor; touches every eval site + helper) | YES (class-eliminated structurally; pattern-match on bare Value cannot miss Tracked) | YES (after refactor: no discipline burden) | **CONDITIONAL** — Simple fails for the transition; structural after |
| **C** (status quo + sweep) | YES (mechanical) | YES per site | **NO** (class survives; per-site discipline ongoing; trap-door incidence rate confirmed >1 per session) | NO (every future author must remember) | **REJECTED on Honest** |
| **D** (match-via-macro + clippy lint) | NO (macro semantics; lint configuration; gradual adoption) | NO (larger sweep than A) | partial (convention; not structural without lint enforcement) | NO (every match becomes macro call) | **REJECTED on Obvious** |
| **E** (variant privatization) | YES | YES | **NO** (internal-only fix; doesn't address the ~15-20 confirmed at-risk internal sites) | partial | **REJECTED on Honest** |

**Only Shape A passes Honest.** It fails Simple on the transition cost — but Simple measures the SHIPPED state, not the transition. Post-transition, Shape A is simpler than Shape C (no discipline burden; structural enforcement).

The transition cost is real but bounded. The trap-door class under Shape C is **unbounded** (every future producer + every future match site).

## Verdict: Shape A

**Pick:** Shape A (TrackedValue struct wrap; retire Value::Tracked variant).

**Rationale:**
1. **Only shape that passes Honest** under four-questions
2. **Trap-door class eliminated structurally** — Rust compiler prevents the bug pattern; not discipline-dependent
3. **Transition cost bounded** vs Shape C's unbounded ongoing cost
4. **Honors user's failure-engineering frame** — "ensure it never happens again" requires structural cure, not convention

**Cost honesty:**
- Reverts SHIPPED stones 233.2.a (Value::Tracked variant + transparency contracts), partially 233.2.b/c (producer wrapping mechanism — keep the producer-naming work; change the wrapping type)
- New struct `TrackedValue { value: Value, provenance: Provenance }` minted
- eval signature: `Result<TrackedValue, RuntimeError>` (was `Result<Value, RuntimeError>`)
- All eval callers: extract `.value` before pattern-match (mechanical; ~?? sites — needs audit)
- Helper signatures (require_X, expect_X): take `TrackedValue` → extract `.value` internally → no `.inner()` discipline needed
- Tests: paperwork — existing probes adapt to new return type; semantics preserved

## Execution decomposition (stones 233.2.h+)

**Stone 233.2.h** — Mint `TrackedValue` struct + adapter trait
- New type `TrackedValue { value: Value, provenance: Provenance }`
- Helper `From<Value> for TrackedValue` (provenance = Unknown)
- Helper `TrackedValue::new(value, provenance)`
- Helper `TrackedValue::value(&self) -> &Value`, `value_owned(self) -> Value`, `provenance(&self) -> &Provenance`
- Existing `Value::Tracked` variant STAYS for now (parallel; not yet retired)
- Lib tests baseline maintained

**Stone 233.2.i** — Flip eval signature
- `eval(...)` returns `Result<TrackedValue, RuntimeError>` (was `Result<Value, RuntimeError>`)
- All eval call sites update: most do `eval(...)?.value` (extract bare Value)
- Helpers (require_X, expect_X) take TrackedValue; extract `.value` internally
- Internal pattern matches stay UNCHANGED (matching on Value, not TrackedValue, after extraction)
- Substrate-as-teacher cascade per FM 15

**Stone 233.2.j** — Migrate producer wrapping from `Value::Tracked` to `TrackedValue`
- Producers (keyword/from-string, from-holon, edn::read, recv, try-recv) return TrackedValue
- Wrap return at producer: `TrackedValue::new(value, Provenance::RuntimeBuilt { ... })`
- Tests / probes adapt to new return path
- Stone 233.2.f's apply unwrapping becomes trivial (already extracts value)

**Stone 233.2.k** — Retire `Value::Tracked` variant + `Value::inner()`
- Variant removed from Value enum
- `.inner()` helper retired (no Tracked to unwrap)
- Transparency contracts in Eq/Hash/Display/HolonRepresentable simplified
- All `.inner()` call sites removed (~21 today)
- Final compile-clean confirms structural enforcement: any future `match v { Value::variant }` works correctly because there's no Tracked variant to miss

**Stone 233.2.e (re-scoped)** — AST-derived provenance for TrackedValue
- Original 233.2.e plan (AST-derived for Literal + SymbolBound) ships on the new TrackedValue substrate
- No structural change vs originally planned; just adapts to new shape

## Out of scope (affirmative scope-bounding)

- **Shape A internal storage** — `Value` retains its variant structure; TrackedValue ONLY wraps at the eval boundary + producer outputs
- **Collection element provenance** — Vec<Value> elements remain bare; no per-element wrapping
- **Cross-boundary serialization** — TrackedValue is local context; bare Value crosses boundaries (mirrors current Value::Tracked transparency contract via inner())
- **Performance optimization** — TrackedValue is a struct (stack); same heap cost as current Box<Value>; revisit if hot-path measures bad
- **arc 232 / Stone 232.0a / defprotocol** — independent; resumes after arc 233 ships
- **Errors-as-EDN** (Stone 233.3) — independent; ships on TrackedValue substrate
- **holon-rs** — NOT touched (Provenance is wat-side concern; holon-rs's HolonAST stays bare)
- **HARD CUT** — no Value::Tracked aliases; remove cleanly

## Risks + honest deltas

- **Stone 233.2.i is the big one** — eval signature flip cascades widely. Substrate-as-teacher iteration; ~100-300 site updates probably. Calibration: 90-180 min sonnet upper bound. Consider sub-slicing if probe-driven enumeration is too coarse.
- **Existing test probes that assert on `Value::Tracked` directly** — Stone 233.2.a's transparency probes test the Tracked variant. They retire OR adapt to TrackedValue's equivalent contract.
- **Stone 233.2.j producer migration** — touches the 5 already-shipped producers; their SCORE docs stay as historical record; new SCORE for the migration
- **Honest cost reveal:** this is reverting + reshaping shipped Stones 233.2.a/b/c. Per `feedback_inscription_immutable`: old SCORE docs stay; new stones add forward-correction. We don't hide the reshape.

## FM 2-bis probe per execution stone

Per `feedback_probe_before_BRIEF`:

- **Stone 233.2.h probe:** unit test asserting TrackedValue::value() / TrackedValue::provenance() / From<Value> contracts
- **Stone 233.2.i probe:** integration test asserting eval-via-startup_from_source returns TrackedValue (compile-shape check)
- **Stone 233.2.j probe:** flow a runtime-built keyword through apply WITHOUT explicit .inner() (now compiles correctly because TrackedValue extraction forced at boundary)
- **Stone 233.2.k probe:** grep src/runtime.rs for `Value::Tracked` — must find ZERO matches post-retirement

Each probe fails pre-stone; passes post-stone.

## Resequencing arc 233 stone chain

| Stone | Original plan | Revised plan |
|---|---|---|
| 233.2.a | ✓ Mint Provenance + Value::Tracked | (shipped; retired in 233.2.k) |
| 233.2.b | ✓ Tag keyword/from-string | (shipped; producer migrated in 233.2.j) |
| 233.2.c | ✓ 4-producer sweep | (shipped; producers migrated in 233.2.j) |
| 233.2.d | ✓ Uniform list_span | (shipped; unchanged) |
| 233.2.e | AST-derived provenance | RE-SCOPED to ship on TrackedValue substrate |
| 233.2.f | ✓ apply Tracked-unwrap | (shipped; trivially-redundant after 233.2.j but stays as historical) |
| **233.2.g** | (this sub-DESIGN) | **PICKED Shape A** |
| **233.2.h** | NEW | Mint TrackedValue + adapter |
| **233.2.i** | NEW | Flip eval signature → TrackedValue |
| **233.2.j** | NEW | Migrate 5 producers to TrackedValue |
| **233.2.k** | NEW | Retire Value::Tracked variant + .inner() |
| 233.3 | Errors-as-EDN | unchanged (ships on TrackedValue) |
| 233.4 | INSCRIPTION | unchanged |

Arc 232 (defprotocol) resumes after arc 233 ships.

## Cross-references

- `docs/arc/2026/05/233-substrate-errors-as-values/DESIGN-STONE-233.2.md` — original Shape A/B/C dialogue; Shape A rejection rationale (now re-evaluated)
- `docs/arc/2026/05/233-substrate-errors-as-values/SCORE-STONE-233.2.f.md` — apply Tracked-unwrap fix; the catalyst for this re-evaluation
- `docs/COMPACTION-AMNESIA-RECOVERY.md` § FM 11 — no deferral; FM 2-bis — probe-before-BRIEF
- `feedback_any_defect_catastrophic` — substrate trust binary
- `feedback_no_known_defect_left_unfixed` — the class IS the defect
- `feedback_refuse_easy_solutions` — Shape C's sweep was L2 reach; Shape A is the L4 honest answer
- `feedback_four_questions_inline` — decision protocol; ran inline here
- `project_failure_engineering` — eliminate the class, not the symptom
- `feedback_sonnet_writes_substrate` — sub-DESIGN is orchestrator-direct; sonnet enacts via 233.2.h+ stones

## What this unblocks

- **Stone 233.2.h BRIEF** — TrackedValue mint; sonnet enacts
- **Future producer additions** — automatically wrapped at TrackedValue boundary; no discipline burden
- **Future match-on-Value sites** — structurally safe (no Tracked variant to miss)
- **Stone 233.2.e** — ships on the right shape
- **Arc 232 resume** — defprotocol builds on the cure, not the symptom
