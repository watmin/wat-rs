# Arc 073 — Build log + open question for another session

**As of:** 2026-04-28 (active session paused mid-slice-4).

**Why this file exists:** the user is consulting another session to resolve the slice-4 design question below. That session has NOT seen the work shipped here. This file is the cold-start brief.

---

## What's shipped (3 of 4 slices)

### Slice 1 — `HolonAST::SlotMarker` 12th variant
- **holon-rs@7586f66** — Adds `HolonAST::SlotMarker { min: f64, max: f64 }` to the closed algebra. Eleven variants → twelve. Updates `Debug`, `PartialEq`, `Hash`, `canonical_edn_holon` (TAG_SLOT_MARKER = 0x07), and the encoder (which **panics** on SlotMarker — templates are query keys, not encodable values; chose panic over silent zero-vector to avoid masking category-error bugs).
- **wat-rs@fb7650c** — Sweeps the two exhaustive HolonAST match sites in wat-rs:
  - `dim_router::immediate_arity` — SlotMarker = arity 1 (defensive; encoder panics first)
  - `runtime::holon_to_watast` — emits `(:wat::holon::SlotMarker <min> <max>)` as a debug-legible list. **INTENTIONALLY non-round-trippable**: the keyword is not registered as a from-watast constructor, so re-parsing fails. Templates inspectable but unspoofable.

Holon-rs: 245 → 245 tests pass. Wat-rs full test suite passes.

### Slice 2 — `template / slots / ranges` decomposition
- **holon-rs@e5b96c9** — Three pure structural methods on `HolonAST`:
  - `.template() -> HolonAST` — recursive replace Thermometer → SlotMarker
  - `.slots() -> Vec<f64>` — pre-order Thermometer values
  - `.ranges() -> Vec<(f64, f64)>` — pre-order (min, max) pairs
  - 8 new tests cover tuning collapse, range distinguishes, atom distinguishes, pre-order ordering, slots/ranges parallel arity, Thermometer-free degeneration, template re-decomposition (no-op), encode-of-template panic.
- **wat-rs@709ec72** — Three substrate primitives + dispatch + type schemes:
  - `:wat::holon::term::template :: HolonAST -> HolonAST`
  - `:wat::holon::term::slots :: HolonAST -> Vec<f64>`
  - `:wat::holon::term::ranges :: HolonAST -> Vec<(f64,f64)>`
  - Substrate gap closed: `:wat::core::=` now accepts HolonAST pairs via `values_equal` (the closed algebra had `PartialEq + Eq` but wasn't exposed to wat-side equality). Bit-exact structural; distinct from `coincident?` (which encodes + compares cosine).
  - 7 wat-side tests: `wat-tests/holon/term.wat`.

Holon-rs: 254 tests pass. Wat-rs: 87 tests pass.

### Slice 3 — `term::matches?` (fuzzy unification)
- **wat-rs@d86e32c** — One substrate primitive composing slice 2:
  - `:wat::holon::term::matches? :: HolonAST × HolonAST -> bool`
  - Predicate: `template(q) == template(s) ∧ ∀i: |q[i]-s[i]|/range[i] < floor(d)`
  - `floor(d) = sigma(d) / sqrt(d)` — uses substrate's `coincident_floor` machinery (arc 023, arc 024, arc 037)
  - Short-circuits on template mismatch — no encoding required when shapes diverge
  - Degenerate range (min == max) requires bit-exact value equality (no divide-by-zero)
  - 5 wat-side tests: self-match, close-slot match, distant-slot miss, different-template miss, Thermometer-free degeneration

Wat-rs: 87 tests pass (12 in `term.wat` total across slices 2+3).

---

## Where the build paused — slice 4 open question

Slice 4 is `TermStore<V>` — the registered parametric data structure with `new / put / get / len`. The lab cache slice (umbrella 059 slice 1) becomes a three-line shim consuming `TermStore<HolonAST>` (×2) and `TermStore<Vector>` (×1).

**The DESIGN.md (in this same directory) is internally inconsistent on TermStore's mutability shape:**

> "TermStore is a value-up immutable structure (returns new store on `put`); the lab cache slice that needs thread-owned mutable state composes it inside a `LocalCache<Template, …>` or service program of its own choosing — *not* this arc's concern."

vs.

> "The lab cache slice (umbrella 059 slice 1) becomes:
>   TermCache (next-form direction)     :: TermStore<HolonAST>
>   TermCache (terminal-value direction):: TermStore<HolonAST>
>   EncodeCache                          :: TermStore<wat::holon::Vector>
> Three caches; one primitive."

These can't both hold. If TermStore is values-up immutable, the lab cache must wrap it in a mutable cell — not a three-line shim. If the lab cache IS `TermStore<V>` directly (three-line shim), TermStore must be thread-owned mutable.

### Option A — values-up immutable
- TermStore<V> is a pure data structure. `put(form, v)` returns a new TermStore<V>; the old one is unchanged.
- Lab cache wraps it: something like a `:wat::lru::LocalCache<:wat::holon::Template, :Vec<(Slots, V)>>` with hand-rolled put/get logic on top.
- **Pros:** clean values; substrate primitive composes uniformly; testing is trivial; no shared-state semantics to reason about.
- **Cons:** every `put` clones (Arc reference bumps for the unmodified buckets, but the affected bucket needs `Arc::make_mut` style copy-on-write). Fine for occasional updates; questionable for a hot cache that hits put on every call site that produced a coordinate.
- **Cons:** lab cache loses the three-line shim — needs to compose TermStore inside a mutable container, write its own decomposition+matching glue. Defeats the "one primitive, three caches" framing.

### Option B — thread-owned mutable (recommended by the active session)
- TermStore<V> mirrors `:rust::lru::LruCache` (in wat-rs's `crates/wat-lru/src/shim.rs`): a `#[wat_dispatch(scope = "thread_owned")]` newtype wrapping a `HashMap<HolonAST, Vec<(Vec<f64>, V)>>`. `put` mutates in place; `get` returns `Option<V>` and (per Q2's FIFO eviction) doesn't bump LRU order. The thread-id guard makes it scope-safe with zero Mutex.
- **Pros:** lab cache literally IS `TermStore<V>` — three-line shim is real. Matches the existing wat-rs precedent for caches (LruCache is the canonical model).
- **Pros:** O(1) put for new template buckets, O(bucket-size) for existing buckets — no cloning the whole HashMap.
- **Cons:** mutable means consumers reason about reference identity. `:rust::lru::LruCache` already establishes this pattern as "fine in wat" — thread_owned scope prevents cross-thread surprises.
- **Cons:** the DESIGN's "values-up" framing is wrong and needs to be updated.

The active session would update the DESIGN to B and ship slice 4 mirroring the LruCache shape:
- New newtype `WatTermStore` in (probably) `crates/wat-lru/src/shim.rs` or a new sibling crate
- `#[wat_dispatch(path = ":wat::holon::TermStore", scope = "thread_owned", type_params = "V")]`
- Methods: `new(cap_override: Option<i64>) -> Self`, `put(&mut self, form: Value, v: Value)`, `get(&mut self, form: Value) -> Option<Value>`, `len(&self) -> i64`
- sqrt(d) cap derived from ambient router at `new`, with optional override

---

## Cross-session context the other session should know

- **Where `arc 073` came from**: the recognition documented in `holon-lab-trading/BOOK.md` Chapter 70 ("Jesus Built My Hotrod"). The user stopped the active session mid-build of a flat-fuzzy `Vec<(HolonAST, V)>` cache (which proof 018 had prototyped) with: *"the surface is a template... yes — just like Prolog... did we just model neurons into the system?"* That recognition crystallized the substrate's leaf taxonomy as already separating tuning-curve leaves (Thermometer) from exact-identity leaves (Atom/Symbol/String/I64/Bool/F64) — i.e., HolonAST is a Prolog term and Thermometer is a logic variable.

- **The `sqrt(d)` invariant** (load-bearing for slice 4's `cap`): at d=10000 (the substrate's default tier per arc 067), sqrt(d)=100. That's the population's resolution — cells in the same template bucket beyond ~100 entries start having receptive fields that overlap and tuning curves that interfere. The DESIGN's cap is sqrt(d); the FIFO eviction policy keeps the bucket honest with the algebra grid. Below sqrt(d) you have headroom; above sqrt(d) the substrate stops discriminating. This is Kanerva's capacity budget surfaced as a cache parameter — not a knob to tune, an invariant to respect.

- **The lab cache umbrella** (`holon-lab-trading/docs/proposals/2026/04/059-the-trader-on-substrate/059-001-l1-l2-caches/DESIGN.md`) is paused pending slice 4. After slice 4 ships, the lab cache becomes the three-line shim (option B) or a wrapped composition (option A). Either way, no more lab work on caches until arc 073 closes.

- **Proof 018** (`holon-lab-trading/wat-tests-integ/experiment/022-fuzzy-on-both-stores/`) is the prior reference impl that surfaced the categorical flaw. It's preserved as a learning artifact — not the production cache. Its 6 tests (T0/T0b/T0c/T1-T6) become the substrate-tier regression suite for `TermStore` once slice 4 lands (per the test-strategy section of DESIGN.md).

- **Untouched in this session:** `wat-rs/crates/wat-edn/` — the user's parallel side-build of an EDN parser/writer. Workspace `Cargo.toml` adds it as a member; the active session left both alone per user instruction.

---

## Files to inspect (in order)

1. `wat-rs/docs/arc/2026/04/073-term-unification/DESIGN.md` — the spec, with the inconsistency between "values-up" and "three-line shim" preserved verbatim.
2. `holon-rs/src/kernel/holon_ast.rs` — slice 1 and 2 changes in one file; the SlotMarker variant + decomposition methods.
3. `wat-rs/src/runtime.rs` — slices 1, 2, 3 wat-side bits: `eval_term_template / slots / ranges / matches_q`, plus the `holon_to_watast` SlotMarker arm and the values_equal HolonAST arm.
4. `wat-rs/src/check.rs` — type schemes for the four term::* primitives.
5. `wat-rs/wat-tests/holon/term.wat` — the wat-level regression suite (12 tests).
6. `wat-rs/crates/wat-lru/src/shim.rs` — the LruCache shape that option B mirrors.
7. `holon-lab-trading/BOOK.md` Chapter 70 (line ~31374) — the recognition narrative that drove arc 073.

---

## What the user wants from the other session

Resolve the slice 4 mutability shape: A (values-up) vs B (thread-owned mutable). The active session's recommendation is B with reasoning above; the user wants a second opinion before committing the DESIGN update.

When you answer, please consider:
- Does the "three-line lab shim" framing still hold under your option, or does it require revising the umbrella 059 slice 1 expectation?
- If A, what's the lab-side wrapper shape that makes per-call-site cache hits viable performance-wise?
- If B, are there cross-thread / cross-program coordination cases the active session is missing? (`:wat::lru::LruCache` is per-thread — does a TermStore want different scope?)
- A third option I haven't considered?
