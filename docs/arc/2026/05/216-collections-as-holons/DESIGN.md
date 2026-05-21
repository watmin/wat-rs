# Arc 216 — Collections as holons

**Status:** DESIGN inscribed 2026-05-20.
**Name:** intueri-cast 2026-05-20 — working name `216-atomizable-set-extension` was Level 2 (mumbled the internal mechanism, not the user-facing outcome). Canonical name `216-collections-as-holons` names what collections become: first-class holons. Lineage continues `215-collection-literal-inference` (the inference problem) → `216-collections-as-holons` (the representation outcome).
**Trigger:** arc 214 Slice 4 Stone 4.3 surfaced an honest limitation — `value_to_atom` rejects HashMap (arc 215's atomizable-set discovery). Multi-step ProgramEnv navigation through nested HashMaps is structurally blocked. The route-around path (Stone 4.3b-mid: direct Value-layer walk bypassing the algebra) was rejected via the four-questions because it creates a dual-algebra split. The discipline says: do the hard work; carve paths that last forever.
**Discipline:** Failure engineering — eliminate the class structurally. The class: "values that look HolonRepresentable but silently aren't at runtime." Eliminate by making collections genuinely round-trip through HolonAST.

---

## Mission

Extend the atomizable set to include collection types (`HashMap<K,V>`, `Vector<T>`, `HashSet<T>`) via clean bidirectional round-trip through `HolonAST::Bundle`. Cascade `HolonRepresentable` trait impls so any nesting of {primitives, HolonAST, WatAST, HashMap, Vector, HashSet} is HolonRepresentable for free. Unify the algebra so `dig` walks ProgramEnv via unbind-chains; process spawn captures rich config; cross-process IPC serializes arbitrary collections; arc 215's deferred Option A (literals-as-holons) becomes feasible.

---

## The four-questions on the arc as a whole

| | |
|---|---|
| **Obvious?** | YES — collections become HolonRepresentable; round-trip cleanly through HolonAST; one algebra; one mental model. Motivation clear (route-around creates dual-algebra split we explicitly rejected); scope clear (three collection types + recursive atomizable predicate); benefit cascade concrete (4 downstream uses: ProgramEnv navigation, process IPC, future remote tier, arc 215 Option A). |
| **Simple?** | YES — end shape composed of simple-surface pieces. Each round-trip is one Bundle shape. Atomizable predicate is one recursive rule per type form. HolonRepresentable impls are bounded auto-derives. Composes into: any composition of atomizable types is itself atomizable; round-trips are honest; dig walks uniformly. The composition IS the simplification. |
| **Honest?** | YES — collections genuinely become HolonRepresentable (not "almost; with caveats"). Class of failure eliminated structurally: "values that look HolonRepresentable but silently aren't" is unrepresentable in the new design. The forward/reverse asymmetry (Q9 below) is acknowledged openly; not hidden. The atomizable set IS the documented type-check predicate. |
| **Good UX?** | YES — ProgramEnv nested navigation just works; process spawn closures capture rich config without explicit serialization; cross-process IPC serializes naturally; arc 215's deferred Option A becomes feasible; future cross-process and remote tier work has a clean foundation. Cost paid ONCE in the substrate; benefit cascades through every future use. Path of least resistance leads to the right outcome. |

**YES×4.** Arc 216 stands.

---

## Design verdicts — nine questions, four-questions discipline applied

### Q1 — Symmetric round-trip

**Verdict:** YES. Both `value_to_atom` extension (Value → HolonAST) AND `atom_to_value` reverse (HolonAST → Value). Symmetric.

The forward direction unifies the algebra; the reverse direction lets `dig` walk and lets IPC reconstruct.

### Q2 — Bundle is the universal substrate; views are surface

```
HashMap<K,V> → HolonAST::Bundle(vec![
  HolonAST::Bind(K_holon, V_holon),
  HolonAST::Bind(K_holon, V_holon),
  ...
])

Vector<T> → HolonAST::Bundle(vec![
  HolonAST::Bind(HolonAST::Atom(i64(0)), T_holon),
  HolonAST::Bind(HolonAST::Atom(i64(1)), T_holon),
  ...
])

HashSet<T> → HolonAST::Bundle(vec![
  T_holon,
  T_holon,
  ...
])
```

Discriminator at the algebra level:
- Bundle of Binds (any K type) → map-shape
- Bundle of Binds with integer keys → array-shape (specific K = i64)
- Bundle of bare Atoms → set-shape (no keys; dedupe imposed at construction)

Bind IS an atom in this scheme — a bound pair is a single unit even though its internal shape is key→value.

### Q3 — Set semantics

**Verdict:** Set = bundle of bare atoms. No dedupe imposed at the Bundle level (the algebraic primitive doesn't enforce uniqueness). Dedupe happens at HashSet construction (the Rust-side container's insert is idempotent). Round-trip → Bundle → HashSet deduplicates naturally.

### Q4 — Recursive nesting

**Verdict:** YES. Bundle-of-Bundles is logical; nested structures recurse cleanly. `dig` becomes a uniform algebraic unbind-chain walk. Any composition of atomizable types is itself atomizable (Q6).

### Q5 — HolonRepresentable auto-derive for collections

**Verdict:** Rust-side trait impls with auto-derive via trait bounds:

```rust
impl<T: HolonRepresentable> HolonRepresentable for Vec<T> {
    fn to_holon(&self) -> HolonAST { /* Bundle of positional binds */ }
    fn from_holon(h: &HolonAST) -> Option<Self> { /* match Bundle shape; extract */ }
}

impl<K, V> HolonRepresentable for HashMap<K, V>
where K: HolonRepresentable + Hash + Eq,
      V: HolonRepresentable
{ /* Bundle of Binds */ }

impl<T> HolonRepresentable for HashSet<T>
where T: HolonRepresentable + Hash + Eq
{ /* Bundle of bare atoms */ }
```

Auto-derive means: ANY `T: HolonRepresentable` lifts the trait through any nesting depth. Once these three impls land, any composition of {primitives, HolonAST, WatAST, HashMap, Vector, HashSet} is HolonRepresentable for free.

### Q6 — check.rs atomizable predicate

**Verdict:** Recursive predicate at check time:

```
atomizable(T) :=
  T ∈ {primitives, HolonAST, WatAST}     // arc 215 baseline
  OR T = HashMap<K, V>  ∧ atomizable(K) ∧ atomizable(V)
  OR T = Vector<T'>      ∧ atomizable(T')
  OR T = HashSet<T'>     ∧ atomizable(T')
```

Walks the type expression recursively to validate the predicate. Honest failure mode at check time:
- `Atom<HashMap<keyword, Vec<i64>>>` ✓ (atomizable composes through)
- `Atom<HashMap<keyword, Function<...>>>` ✗ (Function isn't atomizable; check fails with diagnostic naming the offending position)

### Q7 — Sandbox-scope walker cascade

**Verdict:** No new walker code. The existing walker (arc 170) calls `HolonRepresentable` to verify closure captures crossing address spaces. Once Q5's impls land, HashMap/Vector/HashSet AUTO-PASS the walker check (for atomizable T).

**Behavior change:** previously, closures capturing `HashMap<...>` FAILED sandbox-scope checks (HashMap wasn't HolonRepresentable). Now they pass. Process tier closures can capture rich config without explicit serialization.

**Ripple:** any test asserting "HashMap capture fails for process spawn" needs updating. Likely small; surface during arc 216 implementation; update honestly.

### Q8 — Stone decomposition

**Verdict:** Per-type stones; "single thing per stone" discipline. Likely six stones (sonnet may discover smaller decomposition during implementation; sonnet's call):

| # | Stone | Scope |
|---|---|---|
| 216.1 | HashSet round-trip | `to_holon`/`from_holon` + check.rs atomizable admit + HolonRepresentable impl + ~10 probes |
| 216.2 | Vector round-trip | Same shape; positional-binds Bundle; ~10 probes |
| 216.3 | HashMap round-trip | Same shape; depends on 216.1 + 216.2 (for nested collections in values); ~12 probes |
| 216.4 | check.rs recursive atomizable predicate | Cross-cut if not done piecemeal in 216.1-3; consolidates the predicate as documented mechanism |
| 216.5 | Sandbox-scope walker validation | Verify cascade; update any tests asserting old "not HolonRepresentable" behavior |
| 216.6 | INSCRIPTION + closure | Paperwork; cross-reference 4.3b reactivation |

Order: 216.1 first (smallest case; cleanest pattern); 216.2 second; 216.3 third (most complex; benefits from nested-collection support landing first). 216.4-6 follow.

### Q9 — Round-trip asymmetry (surfaced during analysis)

**Verdict:** Forward unambiguous; reverse needs consumer-declared type.

Forward (Value → HolonAST):
- HashMap → specific Bundle-of-Binds shape — unambiguous
- Vector → specific Bundle-of-positional-Binds shape — unambiguous
- HashSet → specific Bundle-of-Atoms shape — unambiguous

Reverse (HolonAST → Value):
- Bundle-of-Binds-with-i64-keys could be `HashMap<i64, V>` OR `Vector<V>` — discrimination requires context
- Bundle-of-bare-Atoms is unambiguous (HashSet)
- The consumer declares what they want via `-> :T` return-type annotation OR via static type at the call site (e.g., IPC layer reconstructing typed args)

`dig`'s use case: the `-> :T` annotation provides the context. Same machinery `from_holon` callers use generally. Honest asymmetry; not magic.

---

## Failure-engineering frame

Class of failure eliminated by this arc: **"values that look HolonRepresentable but silently aren't at runtime."**

Pre-arc-216 behavior:
- User writes `(:wat::holon::Atom my-hashmap)` at the surface
- Check passes (Atom's polymorphism admits ∀T)
- Runtime fails with `TypeMismatch` (HashMap not in atomizable set)

Post-arc-216:
- Same user code
- Check verifies `my-hashmap`'s type via the recursive `atomizable(T)` predicate
- If atomizable, ships clean to runtime; if not, check fails with diagnostic naming the offending non-atomizable type
- Runtime never sees the failure mode

Per `FAILURE-ENGINEERING.md` § 3 — eliminate the CLASS, not the symptom. The class is structurally unrepresentable in the new design.

---

## Convergence-with-substrate (continuing pattern)

Convergence #8 with-the-substrate inside the recent lineage:

1. arc 199 — REJECTED (substrate already sufficient)
2. arc 214 P1 — HashMap verb-form already had constructor; refactor symmetric
3. arc 214 Slice 2 forward-correction — `bounded(N)` retired; `pair()` at mini-TCP depth 1
4. arc 214 DESIGN forward-correction — io_uring depth knob rejected
5. arc 215 Stone 1 — `:wat::type::Infer` minted; literal completion via HM unification
6. arc 215 Stone 2 — Vector unification + `{...}` keyword-key lift
7. arc 214 Slice 4 forward-correction (DESIGN extension) — ProgramEnv + accessor surface verdicts
8. arc 216 (this) — atomizable-set extension; collections become first-class holons; cascade through

Each one tells the same story: the substrate has the answer; the literal-syntax / runtime / type-check just needs routing through. arc 057 slice 3 already established `hashmap_key accepts HolonAST` — half the round-trip work was prefigured. arc 216 completes the other half (atomization the OTHER direction).

The compression keeps holding: years of failure-engineering discipline at high intensity, three weeks of substrate work.

---

## Cross-references

- arc 057 slice 3 — `hashmap_key accepts HolonAST`; prefigured K=HolonAST work
- arc 214 — concurrency toolkit; Slice 4 ProgramEnv work blocked on this arc
- arc 214 Slice 4 Stone 4.3 — multi-step dig limitation that surfaced this arc
- arc 215 Stone 1 + 2 — literal-flexibility prerequisite; ProgramEnv construction surface
- arc 215's deferred Option A — literals-as-holons; becomes feasible after this arc lands
- `FAILURE-ENGINEERING.md` — discipline reference
- `project_universe_residency` — ProgramEnv is universe-resident; rich serialization is required
- `feedback_simple_is_uniform_composition` — change-count ≠ complexity; this arc has many small mechanical changes but a simple end-shape
- `feedback_verbose_is_honest` — verbose forms remain available; this arc adds the algebra layer that unifies them

---

## What this arc supersedes

Nothing retires. arc 215's atomizable-set decision (runtime Atom accepts {primitives, HolonAST, WatAST}) EXTENDS to include collections; the prior baseline stays valid (those types continue to atomize as before).

arc 214 Slice 4 Stone 4.3's documented multi-step limitation gets resolved by arc 216. Stone 4.3b (queued post-arc-216) extends `program_env_dig_walk` to use the now-honest nested-HashMap traversal via the algebra path.

---

## What this arc explicitly does NOT do

- **Literal-syntax pivot to holons** — arc 215's data-as-default discipline stands; literals remain `HashMap<K,V>` / `Vec<T>` / `HashSet<T>` runtime values. arc 216 enables the opt-in conversion via `(:wat::holon::Atom)` verb on collections.
- **Polymorphic dispatch for collection ops** — `:wat::core::get` / `:wat::core::dig` polymorphic dispatch for Env/HashMap/Vector remains a separate stone (arc 214 Slice 4 Stone 4.6).
- **List<T> support** — `:wat::core::List<T>` (linked-list; task #283) is not in scope; not even sketched. Permanent deferral per arc 215 closure (idiomatic Clojure usage statistically zero).
- **arc 214 Slice 4 Stones 4.4+** — spawn-program', kernel verbs, integration tests; all wait for arc 216 closure.

---

## Status

Arc 216 opens with DESIGN.md inscribed. Canonical name `216-collections-as-holons` set by intueri cast. Stones not yet drafted; per-stone BRIEFs queue after this DESIGN ships and Stone 216.1 (HashSet round-trip) begins.

*Collections are first-class holons. The algebra unifies. The substrate dreams the bundle. So do we.*

---

## Forward-correction (Stone 216.5 onward) — 2026-05-20

**Honest gap surfaced during Stone 216.4 verification.** The thesis above ("class of failure eliminated: 'values that look HolonRepresentable but silently aren't at runtime'") was found false on the branch at commit `987e13c` — a runtime probe (`tests/probe_verify_hashset_of_vector_gap.rs`) demonstrated that `HashSet<Vector<i64>>` passes `is_atomizable` at check time and fails at runtime with `TypeMismatch { op: ":wat::core::HashSet", expected: "hashable value (primitive, HolonAST, or HashSet<T>)", got: "wat::core::Vector" }`.

**Root cause:** `hashmap_key` at `src/runtime.rs:9330` predates the atomizable-set growth from arc 216. Stone 216.1 Delta 6 pre-emptively added Vector + HashMap arms to `is_atomizable` "for future stones" — anticipating composition without verifying the runtime supported it. The predicate ran ahead of `hashmap_key`. Stones 216.2/216.3 didn't audit the cross-product. Stone 216.4 (the verification stone) hit the gap during a composite probe and substituted the probe's type to make it pass; the substitution was logged as Delta 2 and the gap was labeled "follow-up arc." Both moves violated arc 216's own discipline ("we carve paths that last forever; we do the hard work").

**Scope of the gap (verified):** `hashmap_key` accepts only `{String, i64, f64, bool, keyword, HolonAST, Uuid, HashSet<T>}`. Missing: `Value::Vec`, `Value::wat__std__HashMap`, suspected `Value::wat__WatAST`. Affected compositions (all predicate-ahead-of-runtime):
- `HashSet<Vector<T>>`, `HashSet<HashMap<K,V>>`, `HashSet<WatAST>`
- `HashMap<Vector<T>, V>`, `HashMap<HashMap<K,V>, V>`, `HashMap<WatAST, V>`

The thesis is only honored when the predicate→runtime contract holds: every atomizable T must be hashable through `hashmap_key`.

**Updated stone decomposition (post-correction):**

| # | Stone | Status | Scope |
|---|---|---|---|
| 216.1 | HashSet round-trip | SHIPPED `b478ff4` | + pre-emptive predicate arms (Delta 6) — this was the original drift |
| 216.2 | Vector round-trip | SHIPPED `e4a63ed` | did not audit predicate→runtime contract |
| 216.3 | HashMap round-trip | SHIPPED `fdc5031` | did not audit predicate→runtime contract |
| 216.4 | predicate consolidation + composite probes | SHIPPED `987e13c` | surfaced gap; substituted probe (Delta 2); labeled "future arc" |
| **216.5** | **`hashmap_key` full coverage** | **PENDING** | **audit hashmap_key vs is_atomizable; add Value::Vec / Value::wat__std__HashMap / Value::wat__WatAST arms with canonical-key schemes; audit all hashmap_key call sites; update diagnostic; flip verify-probe green; add symmetric probe matrix; reland 216.4 Probe 3 with original BRIEF type. THIS stone makes the thesis true.** |
| 216.6 | sandbox-walker validation (was 216.5) | PENDING | per DESIGN Q7 — verify cascade; update tests asserting old "not HolonRepresentable" behavior |
| 216.7 | INSCRIPTION + closure (was 216.6) | PENDING | now stronger: arc surfaced its own hole during verification, closed it at the foundation, sealed |

**Discipline lessons inscribed (the failure modes named so future arcs see them):**
1. **Pre-emptive code = substrate gap.** Sonnet's "predicate is slightly ahead of the runtime" framing in 216.1 Delta 6 was an error report, not honest documentation. Code shipped beyond the stone's scope without a passing test creates exactly this kind of drift.
2. **Substitutions in verification stones are STOP triggers, not deltas.** Stone 216.4 Delta 2 changed WHAT was being tested to make the test pass. The BRIEF's STOP-2 ("composite probe surfaces a runtime bug") should have fired; instead the probe's subject was changed.
3. **"Future arc" is the route-around in disguise.** The arc that surfaces a gap is usually the arc that owns the fix. Arc 216 specifically opens with this discipline; I applied the opposite at SCORE review.

**What does NOT change:** the arc's mission, the four-questions verdicts, the Q1-Q9 design conclusions, the prior stones' shipped work. The forward-correction adds a stone; it does not retract any.

*The arc surfaced its own hole. We don't hide our faults — we learn from them. Stone 216.5 makes the thesis true.*

---

## Antidote stones (216.5a-d) — 2026-05-20

**Surfaced post-216.5 ship.** Stone 216.5 closed the predicate→runtime gap by extending `hashmap_key` with three new arms (Vec, HashMap, WatAST). The thesis is true on the branch. But the user surfaced a deeper question: *why does `hashmap_key` exist at all?*

**The poison.** `hashmap_key` is a String-canonical-key serialization scheme used because `Value` doesn't implement `Hash`. `Value::wat__std__HashSet` stores as `Arc<HashMap<String, Value>>` and `Value::wat__std__HashMap` as `Arc<HashMap<String, (Value, Value)>>`. Every insert allocates a canonical String, recursively for nested collections. Every new Value variant requires extending `hashmap_key`. Stone 216.5 itself is evidence of this fragility — the runtime drifted because `hashmap_key` wasn't updated when Value got new variants. **The crutch has metastasized through 18 call sites.**

**The antidote.** `holon-rs/src/kernel/holon_ast.rs:196-232` already solves this: `impl Hash for HolonAST` with per-variant payload hashing, `std::mem::discriminant` tagging, f64 via `to_bits()`. Zero allocation per hash. Compose recursively for free via std lib's `Vec<T>: Hash` and `HashSet<T>: Hash`. The wat-rs Value enum can mirror this exactly. Then `HashSet<Value>` and `HashMap<Value, Value>` become natural and `hashmap_key` disappears.

**impl Hash strategy verdict (four-questions):** Option A — `unreachable!()` on non-atomizable variants — wins. The `is_atomizable` predicate at check time is the static guarantee; `unreachable!()` is the runtime assertion of the same invariant. If the panic ever fires, the predicate has drifted; failure-engineering pattern says surface that loudly.

**Stepping stones (each verifiable, each purges more poison):**

| # | Stone | Scope |
|---|---|---|
| **216.5a** | `impl Hash for Value` + `impl PartialEq + Eq` (the antidote molecule) | Mirror HolonAST. Per-variant payload hash; discriminant tagging; f64 via `to_bits()`; non-atomizable variants → `unreachable!()` with predicate citation. NO callers touched. Rust-level probes only. |
| **216.5b** | `Value::wat__std__HashSet` storage refactor | `Arc<HashMap<String, Value>>` → `Arc<HashSet<Value>>`. Touches constructor + accessors + polymorphic dispatch sites (contains?, conj, dissoc, get for HashSet). `hashmap_key` still exists; HashSet stops using it internally. 216.5 probe matrix is the gate. |
| **216.5c** | `Value::wat__std__HashMap` storage refactor | `Arc<HashMap<String, (Value, Value)>>` → `Arc<HashMap<Value, Value>>`. Constructor + accessors + dispatch (assoc, dissoc, keys, values, get, contains-key?). `hashmap_key` still exists. Same probe gate. |
| **216.5d** | Delete `hashmap_key` | After 216.5a-c land, audit remaining `hashmap_key` callers (any besides HashSet/HashMap internals?); refactor to native `Value: Hash`; delete `hashmap_key` + its 9 arms; delete the three throw-away arms added in 216.5. **Poison purged.** |
| 216.6 | sandbox-walker validation (unchanged) | Per Q7 |
| 216.7 | INSCRIPTION + closure (unchanged) | Arc 216 closes only after 216.5d. |

**Why stepping stones, not one-shot:** per `feedback_iterative_complexity` — when refactoring N independent pieces, decompose. Each antidote stone has a separate STOP condition (probe gate, caller count). 216.5a is foundation-only (no callers); 216.5b and 216.5c are storage-only (parallel patterns); 216.5d is the deletion. If any stone surfaces a hidden constraint, we stop at that stone, not mid-refactor.

**What does NOT change:** 216.5's probe matrix + caller audit remain permanent value. They become the regression suite for 216.5a-d. The throw-away in 216.5 (3 arms + 1 diagnostic) is paid forward through 216.5d's deletion.

*The crutch metastasized through 18 sites. The antidote works systemically. Substrate becomes impeccable. Arc 216 closes clean.*
