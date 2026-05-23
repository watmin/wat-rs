# Sub-DESIGN — Stone 233.2.j — Migrate 5 producers + eval_inner TrackedValue cascade

**Status:** ACTIVE (2026-05-23 late late). Sub-DESIGN under arc 233 Stone 233.2 chain (j → k → l).

**Driver:** Stone 233.2.k cannot retire `Value::Tracked` while the 5 producers continue to construct it. Stone 233.2.j is the bridge: it eliminates production of `Value::Tracked` by flipping `eval_inner`'s return type to `TrackedValue`, allowing producers to return `TrackedValue::new(value, provenance)` directly. The variant becomes unreachable at every constructor site; 233.2.k deletes it; 233.2.l seals the meta-class via proc-macro.

User direction 2026-05-23 late late:

> *"we just debated the need for a Rust macro that eliminates the problem we're chasing now... i never want to experience this path again — i want to prove that we walked here and prove we never need to come here again"*

This sub-DESIGN articulates the cascade shape so the BRIEF is substrate-informed per FM 2 + FM 2-bis.

## Current substrate state (post-233.2.h/i)

```rust
// Public boundary (post-233.2.i)
pub fn eval(ast, env, sym) -> Result<TrackedValue, RuntimeError> {
    let value = eval_inner(ast, env, sym)?;
    Ok(match value {
        Value::Tracked { inner, provenance } => TrackedValue::new(*inner, provenance),
        other => TrackedValue::from(other),
    })
}

// Internal workhorse (unchanged since pre-233.2)
pub(crate) fn eval_inner(...) -> Result<Value, RuntimeError> {
    // ~600+ lines of dispatch
}

// Producers (5 fns; 18 wrap sites total)
fn eval_keyword_from_string(...) -> Result<Value, RuntimeError> {
    Ok(Value::Tracked { inner: Box::new(kw), provenance: RuntimeBuilt {...} })
}
// + eval_from_holon (10 arms), eval_edn_read, eval_kernel_recv, eval_kernel_try_recv

// Consumer
impl ValueSnapshot {
    pub fn of(v: &Value) -> Self {
        ValueSnapshot {
            type_name: v.inner().type_name(),
            rendered: render_value(v.inner(), 0),
            provenance: v.provenance(),   // ← only meaningful for Value::Tracked
        }
    }
}
```

**The cascade scope (substrate-informed via crawl):**

| Surface | Count | Change shape |
|---|---|---|
| `eval_inner(...)` call sites in src/runtime.rs | **383** | `let v = eval_inner(...)?` → `let v = eval_inner(...)?.value_owned()` (or `.value()` if borrow OK) |
| Producer fn signatures | **5** | `Result<Value, _>` → `Result<TrackedValue, _>` for keyword/from-string + from-holon + edn::read; recv/try-recv stays `Result<Value, _>` (nested-in-Option-in-Result) but their internal `Value::Tracked` constructor → `TrackedValue::new` then `.value_owned()` to fit the nested shape |
| `eval_inner`'s own body | 1 fn | return type changes; ~30 internal Ok(Value::...) sites become `Ok(TrackedValue::from(Value::...))` (or via `Value::into_tracked()` helper) |
| `ValueSnapshot::of(&Value)` signature | 1 fn (call sites: ~80+) | `&Value` → `&TrackedValue` OR add new constructor `ValueSnapshot::of_tracked(&TrackedValue)` and migrate RAISE sites incrementally |
| `eval` boundary (line 4629-4640) | 1 fn | Simplifies: returns `eval_inner(...)?` directly (no unwrap-and-rewrap) |
| `edn_shim.rs` eval_edn_read producer | 1 site | Signature flips; wrap converts at boundary |
| Value::Tracked variant + Value::inner() + Value::provenance() | **stays** until 233.2.k | Becomes unreachable (no producer creates it; transparency arms become dead code) |

## Doctrine — what this enables structurally

**Pre-state trap-door class:** `match v { Value::specific_variant(...) }` silently mis-dispatches `Value::Tracked`-wrapped values. Discipline-only enforcement (FM 2-bis); Stone 233.2.f closed two sites, audit found ~15-40 more. The class is **alive and reproducing** (3+ trap-door incidences this session).

**Post-state structural seal:** With `eval_inner` returning `TrackedValue`, every caller must extract `.value()` or `.value_owned()` to dispatch. The extraction is explicit. The pattern-match-on-Value happens against a `Value` that **structurally cannot be `Tracked`** (because `TrackedValue` is a struct, not a variant; its `.value` field is bare `Value` minus the retired `Tracked` arm). 233.2.k deletes the variant; 233.2.l forbids future wrapping-variant additions via proc-macro.

**The annihilation:** the SITUATION that produces the trap-door (a Value variant that wraps another Value) ceases to exist. Same shape as ZERO-MUTEX ("the situation that produces the failure is never constructed"). Per FAILURE-ENGINEERING.md ✅✅✅ standard.

## Shape decision — the cascade is atomic

**Four questions on splitting (j-i: cascade flip / j-ii: producer migration):**

| Question | Single-stone | Split (j-i + j-ii) |
|---|---|---|
| Obvious? | YES — one cascade; 383 sites; producers flip in same atomic commit | NO — j-i would ship green-tree intermediate where eval_inner returns TrackedValue but producers still emit Value::Tracked; awkward conversion layer |
| Simple? | YES (atomically) — substrate-as-teacher iterates the 383 sites; mechanical | NO — j-i adds a transient conversion layer that j-ii removes; net work increases |
| Honest? | YES — names the cascade as what it is | NO — splits a single decision into two ceremonial stones |
| Good UX? | YES — sonnet executes substrate-as-teacher per FM 15 | NO — two sonnet flights for one architectural change |

**Verdict: single-stone cascade.** Per Section 5 stepping-stone discipline — splitting only helps when the smaller piece's foundation enables the next. Here j-i would ship a transient state that j-ii immediately removes; no foundation is set.

## Migration plan (sonnet brief outline)

### Step 1 — Add `Value::into_tracked()` adapter helper

```rust
impl Value {
    /// Convert a bare Value into TrackedValue with Provenance::Unknown.
    /// Used by eval_inner's leaf arms (literals: IntLit, FloatLit, etc.)
    /// that don't have producer-attached provenance.
    pub fn into_tracked(self) -> TrackedValue {
        TrackedValue::from(self)
    }
}
```

(May already exist via `TrackedValue::from(Value)` per 233.2.h; verify and reuse.)

### Step 2 — Flip eval_inner signature

```rust
pub(crate) fn eval_inner(...) -> Result<TrackedValue, RuntimeError> {
    match ast {
        WatAST::IntLit(n, _) => Ok(Value::i64(*n).into_tracked()),
        WatAST::FloatLit(x, _) => Ok(Value::f64(*x).into_tracked()),
        // ... ~30 leaf arms ...
    }
}
```

Cargo enumerates the ~30 internal Ok(Value::...) sites; mechanical wrap with `.into_tracked()`.

### Step 3 — Sweep 383 callers via substrate-as-teacher (FM 15)

```rust
// Before
let v = eval_inner(&args[0], env, sym)?;
match v { ... }

// After
let v = eval_inner(&args[0], env, sym)?.value_owned();
match v { ... }
```

Substrate-as-teacher iterates. Each cargo build round names the next batch; sweep one category per round; fail-count drops to zero.

### Step 4 — Migrate the 5 producers

```rust
// Before (eval_keyword_from_string at src/runtime.rs:7371)
Ok(Value::Tracked {
    inner: Box::new(kw),
    provenance: Provenance::RuntimeBuilt { producer: "...", call_span: list_span.clone() },
})

// After
Ok(TrackedValue::new(kw, Provenance::RuntimeBuilt {
    producer: ":wat::core::keyword/from-string",
    call_span: list_span.clone(),
}))
```

Same shape for: eval_edn_read (edn_shim.rs:227), eval_from_holon (10 arms in runtime.rs:14420-14660).

For recv/try-recv (runtime.rs:19788, 19865) — the wrap is **inside** a Value::Result(Arc::new(Ok(Value::Option(Arc::new(Some(tagged)))))). The `tagged` becomes `TrackedValue::new(v, prov).value_owned()` for the inner Value-typed slot. Producer's outer return remains `Result<Value, RuntimeError>` because the Value here is the OUTER Result/Option chain. **Provenance is lost at recv/try-recv in 233.2.j's shape** — this is a planned regression that 233.2.e (AST-derived provenance) revisits via a different mechanism. Document as honest delta.

### Step 5 — Simplify eval boundary

```rust
// Before (runtime.rs:4629-4640)
pub fn eval(...) -> Result<TrackedValue, RuntimeError> {
    let value = eval_inner(ast, env, sym)?;
    Ok(match value {
        Value::Tracked { inner, provenance } => TrackedValue::new(*inner, provenance),
        other => TrackedValue::from(other),
    })
}

// After
pub fn eval(...) -> Result<TrackedValue, RuntimeError> {
    eval_inner(ast, env, sym)
}
```

### Step 6 — ValueSnapshot::of signature

Two options:

**Option (a):** flip signature `fn of(v: &Value)` → `fn of(tv: &TrackedValue)`. All ~80+ RAISE sites adjust. Largest ripple.

**Option (b):** add `ValueSnapshot::of_tracked(tv: &TrackedValue)` alongside existing `of(v: &Value)`. Existing `of` keeps `Provenance::Unknown` (per Stone 233.2.a's bare-Value contract). RAISE sites that have TrackedValue use `of_tracked`; sites with bare Value keep `of`. Smaller blast radius.

**Recommendation:** Option (b). It matches the bare-Value-gets-Unknown contract already shipped in 233.2.a. RAISE sites can migrate incrementally; arc 233 doesn't need all 80+ sites to flip at once.

### Step 7 — Verify Value::Tracked is unreachable

```bash
grep -rn "Value::Tracked\s*{" src/  # should show ZERO construction sites
grep -rn "Value::Tracked\s*{" src/ | grep -v "^src/runtime.rs:[0-9]*:.*//.*"  # exclude comments
```

Pattern-match arms on `Value::Tracked` may still exist (e.g., Hash/Eq/Display impls per Stone 233.2.a transparency). They become unreachable but stay until 233.2.k deletes the variant. Document.

## Scope verification (FM 2-bis probe plan)

Write `tests/probe_stone_233_2_j_producer_migration.rs` with these contracts BEFORE the BRIEF:

1. **Producer return type:** `eval_keyword_from_string` returns `Result<TrackedValue, RuntimeError>` (via TypeId or trait check)
2. **Producer provenance attached:** call `:wat::core::keyword/from-string "foo"` via `eval(...)`; result.provenance() matches `RuntimeBuilt { producer: ":wat::core::keyword/from-string", ... }`
3. **eval_inner cascade:** `eval_inner` return type is `Result<TrackedValue, RuntimeError>` (compile-time check via const fn or trait)
4. **Value::Tracked unreachable:** static scan asserts zero `Value::Tracked { ... }` construction sites in src/ (excluding comments + match arms)
5. **ValueSnapshot::of_tracked round-trip:** producer-tagged TrackedValue → ValueSnapshot::of_tracked → Display includes provenance string

Probe lands FAILING pre-stone; flips PASS post-stone. Permanent regression guard.

## Calibration prediction

| Substrate stone | Shipped | Sonnet wall-clock |
|---|---|---|
| 233.2.i (eval signature flip, 107 files) | 8164629 | 64 min |
| 233.2.j (eval_inner flip + 5 producers + ValueSnapshot, ~400 sites) | this stone | **predict 90–150 min Mode A; 240 min STOP** |

Larger blast radius than 233.2.i (3.6× call sites) but narrower file scope (mostly src/runtime.rs internal). Substrate-as-teacher cascade per FM 15 — cargo enumerates batches.

## Trap-door audit (FM 2-bis pre-flight checks)

- [x] `eval_inner` is internal — no public-API ripple beyond eval boundary (✓ grep confirmed 0 external callers)
- [x] Producer wrap sites enumerated — 18 sites across 5 producer fns (✓ grep `RuntimeBuilt`)
- [x] ValueSnapshot::of consumer count — ~80+ sites (✓ grep `ValueSnapshot::of`)
- [ ] Verify Value::Tracked match arms in Hash/Eq/Display impls stay unreachable post-migration (probe-time check)
- [ ] Verify `recv` / `try-recv` provenance loss is honestly named in SCORE (planned regression; arc 233.2.e revisits)
- [ ] Verify no tests pattern-match on `Value::Tracked` (lib tests at 25505, 25564, 27077, 28369, 29079, 29290, 30473, 30478 all use `.inner()` — should be fine; double-check)

## Builds on / unblocks

**Builds on:**
- 233.2.h (TrackedValue struct minted; `new` + `from` + `value` + `value_owned` + `provenance` methods available)
- 233.2.i (eval boundary returns TrackedValue; cascade pattern proven)
- 233.2.f (apply Tracked-unwrap defect documented as worked example; same class)

**Unblocks:**
- 233.2.k (variant retirement — once no producer creates Value::Tracked, deletion is safe)
- 233.2.l (proc-macro structural seal — requires Value enum with no current wrapping variants)
- arc216 stone1 7 probes (task #496 — same trap-door class; auto-resolves when variant deleted)

## Four-questions verdict

1. **Obvious?** YES — the cascade is the substrate forcing one motion to eliminate the variant class
2. **Simple?** YES — each piece is atomic (signature flip + mechanical .value_owned() at callers + 5 producer constructor swaps + boundary simplification + ValueSnapshot::of_tracked addition). Composition is uniform.
3. **Honest?** YES — names the regression at recv/try-recv (provenance loss until 233.2.e); names the unreachable Value::Tracked match arms; names cascade scope upfront
4. **Good UX?** YES — sonnet executes substrate-as-teacher iteration; orchestrator scores against cargo green; probe locks in the regression guard

PROCEED.

## Cross-references

- `DESIGN-STONE-233.2.md` — sub-stone table (this stone is row 233.2.j)
- `DESIGN-STONE-233.2.g.md` — Shape A pivot that mandated TrackedValue mint (233.2.h)
- `DESIGN-STONE-233.2.h.md` — TrackedValue struct mint (foundation this stone builds on)
- `scratch/FAILURE-ENGINEERING.md` — the doctrine driving annihilation-not-patch
- `docs/SUBSTRATE-AS-TEACHER.md` — cascade-iteration pattern (FM 15)
- `docs/COMPACTION-AMNESIA-RECOVERY.md` § FM 2-bis — probe-before-BRIEF discipline
- `INTERSTITIAL-CLIFFNOTES.md` § Currently — chain progression j → k → l
