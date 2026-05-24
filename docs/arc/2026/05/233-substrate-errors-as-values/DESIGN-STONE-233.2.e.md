# Sub-DESIGN — Stone 233.2.e — AST-derived provenance (Literal + SymbolBound)

**Status:** ACTIVE (2026-05-23 night). Sub-DESIGN under arc 233 Stone 233.2 chain (j ✓ → k ✓ → l ✓ → **e**).

**Driver:** Two `Provenance` variants exist in the enum but have ZERO populate sites: `Literal { span }` and `SymbolBound { binding_span, head_span }`. The j/k/l chain closed the trap-door class (variant + meta-class); now 233.2.e activates the AST-derived provenance machinery on the structurally-sealed substrate. After this stone, every Value flowing through eval_inner carries meaningful provenance — either RuntimeBuilt (producer), Literal (source-coordinates), SymbolBound (binding lineage), or Unknown (escaped contexts only).

This is the **diagnostic-richness payoff** that arc 233 was opened for. Errors raised on let-bound values now name WHERE the binding was defined; errors on literals now point at the source position; the user/LLM gets traceable error context.

## Current state (post-233.2.l)

**Provenance enum** (`src/runtime.rs:~1640`):
```rust
pub enum Provenance {
    Unknown,                                            // populated: default
    Literal { span: Span },                             // EXISTS, ZERO populate sites ← this stone
    SymbolBound { binding_span: Span, head_span: Span }, // EXISTS, ZERO populate sites ← this stone
    RuntimeBuilt { producer: &'static str, call_span: Span }, // populated: 5 producers via 233.2.j/k
}
```

**eval_inner literal leaf arms** (`src/runtime.rs:4496+`) currently discard span via `(_, _)` patterns:
```rust
WatAST::IntLit(n, _) => Ok(TrackedValue::from(Value::i64(*n))),       // span discarded
WatAST::FloatLit(x, _) => Ok(TrackedValue::from(Value::f64(*x))),     // span discarded
WatAST::BoolLit(b, _) => Ok(TrackedValue::from(Value::bool(*b))),     // span discarded
WatAST::StringLit(s, _) => Ok(TrackedValue::from(Value::String(...))),// span discarded
WatAST::Vector(items, _) => ...                                       // span discarded
WatAST::Keyword(k, _) => ... (special-case nil/None)                  // span discarded
```

**Environment storage** (`src/runtime.rs:~1267` post-233.2.k):
```rust
struct EnvCell {
    bindings: HashMap<String, TrackedValue>,  // no binding_span tracked
    parent: Option<Environment>,
}
```

**LetBinding enum** (`src/runtime.rs:~6090`):
```rust
enum LetBinding<'a> {
    Single { name: String, rhs: &'a WatAST },                       // no name_span
    Destructure { names: Vec<String>, rhs: &'a WatAST },            // no per-name spans
    StructDestructure { field_names: Vec<String>, rhs: &'a WatAST },// no per-field spans
}
```

**env.lookup** (`src/runtime.rs:~1288`):
```rust
pub fn lookup(&self, name: &str) -> Option<TrackedValue> { ... }    // no head_span; can't construct SymbolBound
```

## Doctrine — what this enables structurally

**Pre-state:** producer-built values carry RuntimeBuilt provenance; everything else is Unknown. Errors on let-bound symbols say "got: wat::core::i64 `5`" with NO context about WHERE 5 came from. Diagnostic-richness is partial.

**Post-state (233.2.e):**
- Literal `5` in source → `Provenance::Literal { span: <:5 source position> }`
- `(let [x :foo] (some-fn x))` — when `x` flows into a TypeMismatch error, the error names `binding_span: <let line>` + `head_span: <some-fn call site>`
- Producer-built values keep RuntimeBuilt (already populated)
- Only escape-context values (cross-thread receive after let-loss, certain helper-internal Values) keep Unknown

The user/LLM debugging an error sees: WHERE the value was defined, WHERE it was used, and WHAT it was. The diagnostic-richness goal of arc 233 reaches its peak.

## Shape decisions

### Decision 1 — Environment storage shape

**Option A (CHOSEN):** `bindings: HashMap<String, BoundEntry>` where `BoundEntry { value: TrackedValue, binding_span: Span }`. Named struct; binding_span travels with TrackedValue inside EnvCell.

**Option B (REJECTED):** `bindings: HashMap<String, (TrackedValue, Span)>`. Tuple shape; less self-documenting; mirroring TrackedValue's struct shape (decision 2 of Stone 233.2.g) suggests struct.

**Option C (REJECTED):** Store binding_span inside TrackedValue.provenance as initial provenance. Mixes the "as-bound" provenance with the "at-time-of-binding" coordinates. Loses producer-attached RuntimeBuilt provenance when binding fires.

**Verdict:** Option A — `BoundEntry` struct. Honest naming; small surface.

### Decision 2 — env.lookup signature

**Option A (CHOSEN):** `pub fn lookup(&self, name: &str, head_span: &Span) -> Option<TrackedValue>`. The boundary constructs `SymbolBound { binding_span: <stored>, head_span: <passed> }` provenance, wrapping the stored TrackedValue's existing value but replacing the provenance with SymbolBound.

**Option B (REJECTED):** `pub fn lookup(&self, name: &str) -> Option<&BoundEntry>` — caller gets the entry and constructs SymbolBound itself. Spreads the provenance-construction logic across all callers; less canonical.

**Option C (REJECTED):** `pub fn lookup(&self, name: &str) -> Option<(TrackedValue, Span)>` — caller gets both pieces. Same downsides as B.

**Verdict:** Option A. The lookup IS the SymbolBound boundary. All callers pass head_span (which they have from the WatAST::Symbol(_, span) node).

**Subtle:** the stored TrackedValue may already have RuntimeBuilt provenance (e.g., let-bound to keyword/from-string result). Decision: SymbolBound REPLACES the stored provenance at lookup time. Rationale — once the value is bound, its "useful provenance" for diagnostic context is the BINDING coordinates, not the original producer. The producer-context is preserved in commits/SCORE/git-history; the let-binding is the lexical scope.

Alternative considered: chain provenance — wrap stored RuntimeBuilt inside SymbolBound. Rejected: Provenance enum is flat; chaining requires a new enum shape (out of scope for 233.2.e). Forward-compatible note: a future arc may revisit if chained provenance becomes load-bearing.

### Decision 3 — LetBinding shape change

**Option A (CHOSEN):** Add per-name spans:
```rust
enum LetBinding<'a> {
    Single { name: String, name_span: Span, rhs: &'a WatAST },
    Destructure { names: Vec<(String, Span)>, rhs: &'a WatAST },
    StructDestructure { field_names: Vec<(String, Span)>, rhs: &'a WatAST },
}
```

`parse_let_binding` extracts the name's span from the WatAST::Symbol(_, span) node.

**Option B (REJECTED):** Use `rhs.span()` as binding_span placeholder. Dishonest — binding_span should be the NAME's position, not the RHS. SymbolBound's doc says "where the binding was defined" — that's the LHS.

**Option C (REJECTED):** Carry binding_span via a side-channel HashMap. Splits the binding into two structures; brittle.

**Verdict:** Option A — LetBinding carries name spans. Mechanical change; parser already has the spans (just discards them currently).

### Decision 4 — Scope of literal-arm changes

**In scope:** WatAST leaf arms that produce Value at value-position in eval_inner:
- IntLit, FloatLit, BoolLit, StringLit → `Provenance::Literal { span }`
- Vector — when used as expression-position literal `[...]` (per arc 215) → `Literal { span }`
- Keyword special cases (`:wat::core::nil` → Value::Unit; `:None` → Value::Option(None)) → `Literal { span }`

**Out of scope:**
- `WatAST::List` — these are CALL forms (dispatch); their result's provenance comes from the dispatched fn (RuntimeBuilt or SymbolBound or composed). Not a "literal."
- `WatAST::Symbol` — handled by Decision 2 (SymbolBound via env.lookup).
- `WatAST::StructPattern` — value-position is illegal; raises MalformedForm; no provenance needed.

### Decision 5 — eval_let_tail flip

The 233.2.k SCORE noted: "eval_let_tail (tail-call path) still returns Result<Value>; not changed this stone." 233.2.e completes the let-binding provenance coverage by flipping eval_let_tail to `Result<TrackedValue, RuntimeError>` and updating its callers per the same pattern as 233.2.j's eval_let flip.

This restores provenance in the tail-call path (recursive let chains with arc 145 let-in-let optimization).

### Decision 6 — recv/try-recv provenance (honest delta documented)

The 233.2.j Phase 6 honest delta removed the Value::Tracked wrap inside `Value::Result(Arc::new(Ok(Value::Option(Arc::new(Some(tagged))))))` for recv/try-recv. Provenance was lost at the value carrier.

**233.2.e does NOT restore the RuntimeBuilt {producer: ":wat::kernel::recv"} wrap.** That mechanism required Value::Tracked which is permanently retired.

**Indirect coverage via SymbolBound:** when recv's result is let-bound (the common pattern `(let [v (recv ch)] ...)`), pattern-matching against the Option extracts the inner value; the let-binding stores it with binding_span; subsequent lookups attach SymbolBound provenance pointing at the let site. The provenance trace covers the BINDING context, not the original send site (which is in a separate execution universe anyway — the send site's span is in a different process/thread).

**Honest framing:** the original send-site's span is unrecoverable (lives in another execution context). The recv-call's span IS recoverable but the carrier mechanism is gone. The recv call's span CAN be attached via a different mechanism (e.g., wrap the extracted T at the match site), but the user code does the match, not the substrate. Out of scope for 233.2.e.

Document in SCORE: recv/try-recv values that flow through let-binding get SymbolBound provenance (covers the common case); raw extraction stays Unknown; the original producer-site span is permanently lost.

## Implementation surface

### Phase 1 — Literal{span} at eval_inner literal arms (~6-7 sites)

```rust
// Before
WatAST::IntLit(n, _) => Ok(TrackedValue::from(Value::i64(*n))),
// After
WatAST::IntLit(n, span) => Ok(TrackedValue::new(
    Value::i64(*n),
    Provenance::Literal { span: span.clone() },
)),
```

Same shape for FloatLit, BoolLit, StringLit, Vector, Keyword (nil/None special cases).

### Phase 2 — BoundEntry struct + EnvCell shape flip

```rust
pub struct BoundEntry {
    pub value: TrackedValue,
    pub binding_span: Span,
}

struct EnvCell {
    bindings: HashMap<String, BoundEntry>,
    parent: Option<Environment>,
}
```

### Phase 3 — env.lookup signature flip

```rust
pub fn lookup(&self, name: &str, head_span: &Span) -> Option<TrackedValue> {
    if let Some(entry) = self.inner.bindings.get(name) {
        let value = entry.value.value().clone();  // bare Value, clone for owned
        Some(TrackedValue::new(
            value,
            Provenance::SymbolBound {
                binding_span: entry.binding_span.clone(),
                head_span: head_span.clone(),
            },
        ))
    } else {
        self.inner.parent.as_ref().and_then(|p| p.lookup(name, head_span))
    }
}
```

The 4 known lookup call sites add `head_span` argument (mechanical — they all have access to the lookup-site span via the AST node).

### Phase 4 — LetBinding shape + parse_let_binding

```rust
enum LetBinding<'a> {
    Single { name: String, name_span: Span, rhs: &'a WatAST },
    Destructure { names: Vec<(String, Span)>, rhs: &'a WatAST },
    StructDestructure { field_names: Vec<(String, Span)>, rhs: &'a WatAST },
}
```

`parse_let_binding` extracts span from `WatAST::Symbol(ident, span)` (it has access to the span; currently discards).

### Phase 5 — bind_let_binding stores binding_span

```rust
LetBinding::Single { name, name_span, rhs } => {
    let tv = eval_inner(rhs, scope, sym)?;
    Ok(scope.child().bind(name, name_span, tv).build())
}
```

`EnvironmentBuilder.bind(name, binding_span, tv)` — accepts binding_span; constructs BoundEntry.

### Phase 6 — eval_let_tail flip

`fn eval_let_tail(...) -> Result<Value, RuntimeError>` → `Result<TrackedValue, RuntimeError>`. Callers update via `.value_owned()` if they want bare Value (mirrors 233.2.j eval_let pattern).

### Phase 7 — Update render in ValueSnapshot

`ValueSnapshot::Display` already renders all 4 Provenance variants (verified at lines 1780-1810). Verify Literal + SymbolBound render correctly with the new spans (probably already correct; smoke test).

## FM 2-bis probe plan

Write `tests/probe_stone_233_2_e_ast_derived_provenance.rs` with 5 contracts BEFORE the BRIEF:

1. **Literal{span} on i64 literal:** evaluate `42`; assert `tv.provenance()` is `Provenance::Literal { span: <non-unknown> }`
2. **Literal{span} on string literal:** evaluate `"hello"`; same shape
3. **SymbolBound{binding_span, head_span} on let-bound symbol:** evaluate `(let [x 42] x)`; assert `tv.provenance()` is `Provenance::SymbolBound { binding_span: <let line>, head_span: <body x ref> }` with the two spans distinct
4. **SymbolBound from destructure binding:** evaluate `(let [[a b] (tuple 1 2)] a)`; assert `tv.provenance()` is SymbolBound with binding_span pointing at `a`'s position in the LHS pattern
5. **Literal{span} renders in error messages:** trigger a TypeMismatch on a literal-bound value; assert error message contains `at line X col Y` derived from the Literal span

Probe ships FAILING pre-stone (provenance is currently Unknown for all these cases). Sonnet's mission: flip to PASS.

## Calibration prediction

| Stone | Predicted |
|---|---|
| 233.2.e | **90–150 min Mode A; 180 min STOP** |

Larger than 233.2.l (smaller proc-macro stone) but smaller than 233.2.j (the 383-site cascade). Mechanical sweep + signature flips + LetBinding shape change.

**Risks:**
- LetBinding shape change cascades through parser + bind_let_binding + ~4 lookup call sites
- Destructure span-extraction may require care in parse_let_binding
- env.lookup head_span propagation through parent chain (recursive call needs the arg)
- ValueSnapshot::of(&Value) — without TrackedValue, provenance is Unknown; for SymbolBound to surface in errors, RAISE sites need to use ValueSnapshot::of_tracked(&TrackedValue). This is a known cascade — incremental per call site. Out of scope for 233.2.e per Stone 233.2.k's "incremental migration."

## Trap-door audit (FM 2-bis pre-flight)

- [x] Provenance enum variants (Literal + SymbolBound) exist; need only populate sites
- [x] Literal-arm leaf sites enumerated (~6-7 in eval_inner)
- [x] Environment + EnvCell shape post-233.2.k confirmed (HashMap<String, TrackedValue>)
- [x] LetBinding enum confirmed bare-String (needs shape change to carry spans)
- [x] 4 known env.lookup call sites in runtime.rs (need head_span arg)
- [ ] Verify check.rs lookup at 12607 doesn't need changes (likely uses different code path; lookup may not exist there)
- [ ] Verify recv/try-recv stays Unknown (per Decision 6); no scope creep into restoring carrier-level provenance
- [ ] Verify ValueSnapshot::of(&Value) callers don't accidentally get Unknown for previously-Tracked values (incremental migration boundary)

## Builds on / unblocks

**Builds on:**
- 233.2.h (TrackedValue mint) — the provenance carrier
- 233.2.i (eval boundary returns TrackedValue) — boundary shape proven
- 233.2.j (producer migration + Phase 5 fix) — RuntimeBuilt provenance pattern
- 233.2.k (variant retirement + Environment stores TrackedValue) — clean substrate
- 233.2.l (proc-macro seal) — meta-class closure ensures no future wrapping variants

**Unblocks:**
- Stone 233.3 (Errors-as-EDN) — provenance now meaningful for EDN serialization
- Stone 233.4 (INSCRIPTION) — arc 233 closes once 233.2.e + 233.3 land
- arc 232 defprotocol — resumes on the diagnostic-rich + sealed substrate
- arc 233's original thesis ("errors are remarkable") — empirically delivered

## Out of scope (affirmative scope-bounding)

- **Chained provenance** (RuntimeBuilt → SymbolBound when let-bound producer result) — Provenance enum is flat; chain would require new variant; not load-bearing today
- **Carrier-level recv/try-recv provenance** — permanently lost per 233.2.j Phase 6; indirect coverage via let-binding SymbolBound
- **ValueSnapshot::of(&Value) sweep to of_tracked** — incremental migration per 233.2.k; out of scope
- **Destructure slot provenance via deeper analysis** (e.g., per-tuple-element provenance traceable to the source tuple's per-element span) — each slot gets binding_span pointing at LHS pattern; source-element provenance is one level deeper and out of scope
- **holon-rs** — NOT touched
- **HARD CUT** — no deprecation aliases

## Four-questions verdict

| Question | Verdict |
|---|---|
| **Obvious?** | YES — Provenance enum variants exist with documented purpose; this stone wires them up |
| **Simple?** | YES — atomic phases (literal arms + EnvCell shape + LetBinding shape + env.lookup boundary + eval_let_tail flip). Composition is uniform. |
| **Honest?** | YES — recv/try-recv permanent loss documented as honest delta; chained provenance documented as future work; LetBinding shape change is necessary (not approximation) |
| **Good UX?** | YES — errors gain source-coordinates context; let-bound values name their binding site; the diagnostic-richness goal of arc 233 reaches peak coverage |

PROCEED.

## Cross-references

- `DESIGN-STONE-233.2.md` — sub-stone table (this stone is row 233.2.e)
- `DESIGN-STONE-233.2.j.md` — establishes recv/try-recv Phase 6 honest delta this stone addresses indirectly
- `DESIGN-STONE-233.2.k.md` — establishes eval_let_tail honest delta this stone closes
- `SCORE-STONE-233.2.k.md` — Option A Environment storage pattern this stone extends
- `SCORE-STONE-233.2.l.md` — sealed substrate this stone builds on
- `docs/COMPACTION-AMNESIA-RECOVERY.md` § FM 2-bis — probe-before-BRIEF
- `feedback_partial_state_grading` — discipline if STOP-3 fires
- `scratch/FAILURE-ENGINEERING.md` — the doctrine driving the chain
