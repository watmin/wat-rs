# Sub-DESIGN — Stone 233.2 — Value-level Provenance tracking

**Status:** ACTIVE (2026-05-23 evening). Sub-DESIGN under arc 233; sliced into sub-stones 233.2.a → 233.2.d.

**Driver:** the four-questions on Stone 233.2's scope revealed that AST-derived-only provenance fails the Honest test (it doesn't close the runtime-built case the user explicitly named). Option B (full Value-level provenance) is the right scope; the cost dissolves under proper slicing.

## What 233.2 actually does

Every Value can carry Provenance — a structural record of WHERE the value came from. Errors that include a `ValueSnapshot` (per Stone 233.1) extract this provenance and render it in the diagnostic. The runtime-built case ("this keyword was built by `keyword/from-string` at line 5") becomes legible without source-reading.

## Provenance enum (the data shape)

```rust
#[derive(Debug, Clone)]
pub enum Provenance {
    /// Unknown — default for Values without explicit provenance.
    Unknown,

    /// Literal — value appeared as a literal in source.
    /// E.g., `:foo` in `(let [k :foo] ...)` has `Literal { span: <:foo's span> }`.
    Literal { span: Span },

    /// SymbolBound — value resolved from a Symbol lookup; the binding_span
    /// is where the binding was defined; head_span is where the symbol
    /// appeared in the call.
    /// E.g., in `(let [k :foo] (k 1 2))`, when `k` is looked up at the call,
    /// the value has `SymbolBound { binding_span: <:foo's span>, head_span: <k's span> }`.
    SymbolBound { binding_span: Span, head_span: Span },

    /// RuntimeBuilt — value was constructed by a producer function at runtime.
    /// E.g., `(keyword/from-string s)` returns a keyword with
    /// `RuntimeBuilt { producer: ":wat::core::keyword/from-string", call_span: <the call's span> }`.
    RuntimeBuilt { producer: &'static str, call_span: Span },
}
```

`&'static str` for `producer` is fine — producer names are compile-time constants.

## Implementation shape — the design fork

Three candidate shapes for "where does Provenance live on a Value":

### Shape A — Wrap Value in `TrackedValue` struct

```rust
pub struct TrackedValue {
    pub value: Value,
    pub provenance: Provenance,
}
```

Every place that uses `Value` becomes `TrackedValue`. Eval returns `TrackedValue`; let bindings store it; collections like `Vec<Value>` become `Vec<TrackedValue>`.

**Cost:** MASSIVE. Touches every match arm + every conversion + every primitive + every test that pattern-matches Value. Practically infeasible without multi-week effort. Plus collections-as-holons (arc 216) carefully shaped Value semantics; wrapping breaks the HolonAST encoding contract.

**Verdict:** REJECTED. Cost too high; semantic disruption.

### Shape B — Per-variant Optional Provenance field

```rust
pub enum Value {
    bool(bool, Option<Provenance>),
    i64(i64, Option<Provenance>),
    wat__core__keyword(Arc<String>, Option<Provenance>),
    // ... ~30 variants, all get a new field
}
```

**Cost:** Large. Every variant grows; every match arm updates.

**Verdict:** REJECTED. Inconsistent + invasive + most variants don't need provenance (literal i64 has spans via AST; runtime-built i64 is rare).

### Shape C — New `Value::Tracked` wrapper variant ← RECOMMENDED

```rust
pub enum Value {
    // ── existing variants unchanged ──
    bool(bool),
    i64(i64),
    wat__core__keyword(Arc<String>),
    // ... all 30+ variants unchanged

    // ── NEW ──
    Tracked {
        inner: Box<Value>,
        provenance: Provenance,
    },
}
```

One new variant. Provenance is OPT-IN: producers that care wrap their return value in `Value::Tracked`; producers that don't care emit bare values.

**Transparency contract:** `Value::Tracked` is structurally transparent for behavior but visible at error-construction time:

- **Eq / Hash / PartialEq:** unwrap Tracked recursively; equality compares inner values (Tracked is metadata, not identity)
- **Display / Debug:** transparently render inner Value (Tracked wrapping doesn't affect display)
- **HolonRepresentable serialization:** transparently serialize inner Value (provenance is local-context metadata; not part of the data wire format)
- **Pattern matching:** helper `Value::inner()` returns &Value (unwrapping Tracked); helper `Value::provenance()` returns Provenance (Unknown if not Tracked); match arms use the appropriate helper

**Cost:** Small. One new variant. Each `match` on Value gets a default Tracked arm that delegates to inner (or sonnet writes helper macros). HolonRepresentable + Display + Eq + Hash + a few other infrastructure traits need transparency updates.

**Verdict:** RECOMMENDED. Smallest invasive change; opt-in semantics; provenance is structurally addressable when needed.

## Four-questions on Shape C

| Question | Verdict | Why |
|---|---|---|
| Obvious? | YES | One new variant; opt-in; transparent for most paths |
| Simple? | YES | Single point of change in Value enum; helper fns centralize unwrap logic |
| Honest? | YES | Acknowledges provenance is metadata (not identity); doesn't pretend Value-without-provenance is impossible |
| Good UX? | YES | Producers opt in; consumers query if they care; default behavior unchanged |

## Sub-stone sequencing

| Stone | Purpose | Size | Status |
|---|---|---|---|
| 233.2.a | **Mint Provenance enum + Value::Tracked variant + transparency contracts** — Add Provenance enum (4 variants: Unknown, Literal, SymbolBound, RuntimeBuilt). Add Value::Tracked variant. Implement transparency: Eq/Hash/PartialEq/Display/Debug/HolonRepresentable all unwrap Tracked. Add `Value::inner()` + `Value::provenance()` helpers. ValueSnapshot::of extracts provenance from Tracked. Lib tests all pass (baseline maintained); no actual producers wrap yet (Provenance always Unknown in real use). | medium (one variant + transparency sweep) | ✓ SHIPPED at `7cfeff1` |
| 233.2.b | **Tag at the keyword/from-string producer** — `eval_keyword_from_string` wraps return value in `Value::Tracked` with `Provenance::RuntimeBuilt { producer: ":wat::core::keyword/from-string", call_span }`. Probe asserts error message includes producer info when a runtime-built keyword reaches NotCallable. The minimum-viable producer tag. | small | ✓ SHIPPED at `9cc278c` |
| 233.2.c | **Tag at additional producers** — `eval_from_holon`, EDN-reader, mailbox-recv, possibly more. Each producer site small + isolated. Honest delta if a producer's source-span isn't available cleanly. | medium (one site per producer; sweep) | ✓ SHIPPED at `c0f41f6` |
| **233.2.d** | **Substrate-symmetry — uniform `list_span` threading.** ~245 dispatch arms gain uniform `list_span: &Span` parameter per canonical template. Pure plumbing sweep; closes the asymmetry surfaced during 233.2.c's `eval_edn_read` plumb. See [DESIGN-STONE-233.2.d.md](DESIGN-STONE-233.2.d.md). | large (mechanical; ~245 sites; substrate-as-teacher iteration) | ✓ SHIPPED at `c4dc8f4` |
| **233.2.f** | **apply Tracked-unwrap defect fix.** Two `.inner()` insertions in `eval_apply` (src/runtime.rs:7433 + 7438). Defect surfaced via Stone 233.2.d Row 6 honest delta. | small | ✓ SHIPPED at `51d83e1` |
| **233.2.g** | **Tracked-unwrap class re-evaluation — sub-DESIGN.** Post-233.2.f audit found ~15-40 more sites with same pattern shape across substrate. Four-questions inline (in sub-DESIGN) re-evaluates Shape A vs C vs D vs E. **Verdict: Shape A** (TrackedValue struct wrap + retire Value::Tracked variant). Structural class-elimination chosen over per-site discipline sweep. See [DESIGN-STONE-233.2.g.md](DESIGN-STONE-233.2.g.md). | DESIGN dialogue (orchestrator-direct) | PENDING — sub-DESIGN landed; awaits user concur before execution stones |
| **233.2.h** | **Mint `TrackedValue` struct + adapter trait.** Parallel to existing `Value::Tracked` variant (not yet retired). Lib tests baseline maintained. | small | PENDING — blocked on 233.2.g concur |
| **233.2.i** | **Flip eval signature.** `Result<Value, _>` → `Result<TrackedValue, _>`. Substrate-as-teacher cascade across eval call sites. Helpers extract `.value` internally. | large | PENDING |
| **233.2.j** | **Migrate 5 producers** from `Value::Tracked` wrapping to `TrackedValue::new(...)`. Trivially obsoletes Stone 233.2.f's apply fix (structural; not per-site). | medium | PENDING |
| **233.2.k** | **Retire `Value::Tracked` variant** + `Value::inner()` helper. Final compile-clean confirms structural enforcement. | medium | PENDING |
| **233.2.e** | **AST-derived provenance** — Literal + SymbolBound populated on TrackedValue substrate. Re-scoped from prior 233.2.d slot; ships AFTER structural refactor lands. | medium | PENDING — blocked on 233.2.k |

233.2 closes when 233.2.a-k + 233.2.e ship + the umbrella INSCRIPTION (which lands at 233.4 or whenever 233.3 completes).

**First resequencing (2026-05-23 night, post-compaction):** the provisional pre-compaction slicing had 233.2.d = AST-derived provenance. Post-compaction four-questions on Stone 233.2.c's substrate-symmetry surfacing collapsed the "AST-derived first" framing — `SymbolBound`'s `head_span` cannot be populated honestly on a substrate where 56% of dispatch arms drop the call-site span. Substrate-symmetry foundation precedes AST-derived population. Old 233.2.d (AST-derived) shifts to 233.2.e; new 233.2.d takes the uniform `list_span` work.

**Second resequencing (2026-05-23 night post-233.2.f audit):** Stone 233.2.f's apply Tracked-unwrap fix closed two sites; audit surfaced ~15-40 more with same shape. Four-questions inline (in `DESIGN-STONE-233.2.g.md`) re-evaluated the Shape decision under new evidence. **Verdict: Shape A** (TrackedValue struct wrap + retire Value::Tracked variant). Structural class-elimination chosen over per-site discipline sweep. Stones 233.2.g-k execute the structural pivot; Stone 233.2.e re-scopes to ship on the new substrate. Per `feedback_inscription_immutable`: Stones 233.2.a-c stay as historical record; the SCORE docs preserve the Shape C work that informed the structural pivot.

## What sonnet ships per sub-stone

**233.2.a:** Type mint + transparency. NO behavioral change to existing tests. Establishes the substrate scaffolding.

**233.2.b:** ONE producer tags (the most-load-bearing one). New probe demonstrates the runtime-built case now teaches.

**233.2.c:** Sweep additional producers (each isolated, each with own probe verifying provenance attachment).

**233.2.d:** AST-source-tracking for let-bindings + literal-source tracking for direct keyword/string/value uses.

Each sub-stone is independently verifiable + shippable. Calibration discipline holds.

## Trap-door audit (lessons from arc 232.0)

- **NO invented syntax.** Provenance enum variants use existing Rust struct shape (named fields).
- **NO made-up primitive names.** Helpers `inner()` and `provenance()` are simple; not borrowing from any other system.
- **Implementation shape locked HERE** (Shape C). Sub-stones don't get to reopen the wrap-vs-field-vs-variant question. If they hit a wall, STOP + report; don't quietly switch shapes.
- **HolonRepresentable transparency is load-bearing.** Sonnet verifies serialization unaffected — Tracked unwraps when crossing the wire.
- **Eq/Hash transparency is correctness-critical.** Two Values that differ only in provenance MUST be equal (otherwise HashMap/HashSet behavior breaks). Tests must verify this.

## Risks + honest deltas

- **HolonRepresentable** is a trait used across arcs 216+. Sonnet must verify Tracked transparency works for every variant that implements it.
- **Pattern matching** in existing code may need explicit Tracked arms (or use helpers). Sweep is mechanical but touches many files.
- **Performance:** Tracked introduces one heap allocation per producer-tagged value. Acceptable for v1 — producers are infrequent relative to total Value construction. v2 might intern Provenance via Arc if needed.
- **Cross-boundary transport:** Values serialized across thread/process boundaries lose Tracked wrapping (the inner Value goes; provenance is local). Acceptable v1 limitation; cross-boundary provenance is its own future arc if needed.

## Open question (for user)

**Producer scope for 233.2.b:** start with `:wat::core::keyword/from-string` alone (highest payoff; the case from arc 232.0)? Or tag all known producers in one sub-stone (233.2.b/c collapse into one)?

My read: ship 233.2.b with just `keyword/from-string`; the calibration win + the probe proof land cleanly; 233.2.c sweeps remaining producers with the pattern established.

## Cross-references

- `docs/arc/2026/05/233-substrate-errors-as-values/DESIGN.md` — umbrella; gets updated to reference this sub-DESIGN
- `docs/arc/2026/05/233-substrate-errors-as-values/SCORE-STONE-233.1.md` — ValueSnapshot field is the slot Provenance fills
- `docs/arc/2026/04/109-kill-std/INVENTORY.md` § O — the original backlog entry; three-case table directly motivates the Provenance variants
- arc 216 — collections-as-holons; HolonRepresentable trait that needs transparency
- arc 215 — Value-construction semantics
- `feedback_refuse_easy_solutions` — the discipline that drove Option A → Option B correction
- `feedback_sonnet_writes_substrate` — protocol; sonnet writes substrate; orchestrator briefs + scores
