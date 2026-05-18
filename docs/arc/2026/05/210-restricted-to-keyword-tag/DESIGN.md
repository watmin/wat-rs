# Arc 210 — `:restricted-to` keyword tag on `def-restricted` + `defn-restricted`

**Status:** OPEN 2026-05-18.

**Priority:** **BLOCKING arc 209 Stone A.** Defservice's generated code emits `defn-restricted` forms for `:service::-start-thread` + `:service::-start-process` substrate-internal entries. If we lock the new keyword shape FIRST, defservice generates the right form from day 1. If we lock LATER, defservice's generated code uses the legacy positional shape and needs sweeping.

**Pedigree:** Arc 198 minted `def-restricted` (substrate primitive) + `defn-restricted` (defmacro sugar) with positional `[prefixes]` second-arg shape. The shape works; the tests pass. Arc 209 surface convergence (collapsed `:state` / `:admin` / `:user` / `:handlers` keyword-tagged sections) surfaced an inconsistency: every other substrate form uses keyword-tagged sections for semantic parameters; `def-restricted`'s prefixes vector is the lone positional holdout.

## The crack arc 210 closes

Today's `(:wat::core::def-restricted :name [prefix-vec] expr)` form encodes the whitelist by POSITION, not by KEYWORD. Readers see `[:wat::kernel::]` and must KNOW the substrate's positional convention to recognize it as the allowed-caller whitelist.

Every other keyword-tagged substrate form (defservice's `:state`/`:admin`/`:user`/`:handlers`; defmacro's typed-AST params; let's binding vectors; etc.) names sections by keyword. `def-restricted`'s positional encoding is inconsistent.

Per `feedback_simple_is_uniform_composition`: the substrate's surface should be uniform. Bringing `def-restricted` into line with the keyword-tagged convention IS the simplest possible composition.

## Goal — the locked form

**Substrate primitive (`def-restricted`):**
```scheme
(:wat::core::def-restricted :my::kernel::restricted-fn
  :restricted-to [:my::kernel::]
  <expr>)
```

**Defmacro sugar (`defn-restricted`):**
```scheme
(:wat::core::defn-restricted :my::kernel::restricted-fn
  :restricted-to [:my::kernel::]
  [p <- :wat::core::i64  q <- :wat::core::i64] -> :wat::core::i64
  <body>)
```

The `:restricted-to` keyword tag goes between the name and the expression/signature. Position-flexible (anywhere between name and body); convention is "name, then keyword sections, then body."

## Why both, not just `defn-restricted`

User direction 2026-05-18 (B path): substrate primitive AND sugar both get the keyword tag. Consistency all the way down.

Reasoning:
- If `def-restricted` (primitive) keeps positional `[prefix]` while `defn-restricted` (sugar) uses `:restricted-to`, readers grepping for either see different shapes. The sugar would expand to a positional form; the substrate's surface would lie about the user-facing convention.
- One form to learn; one form to read; one form to grep.
- Tiny ripple — only 5-6 test sites in `tests/wat_arc198_def_restricted.rs` use these forms directly today (verified via pre-flight grep).

Per `feedback_substrate_owns_not_callers_match`: the substrate-primitive surface should match the user-facing surface; sugar's job is to ADD ergonomics, not to MASK substrate dissonance.

## Out of scope (affirmatively)

- **Multi-keyword sections** beyond `:restricted-to`. Arc 210 ships THIS keyword only; if future restriction mechanisms add new sections (e.g., `:allowed-callers-via-cap`), those land in their own arcs.
- **Renaming the keyword.** `:restricted-to` was the orchestrator's reach in arc 209 pseudocode; the user assessed it as good; the name locks here.
- **Changing the prefix-vector semantics** (namespace-prefix match per arc 198). Arc 210 is naming-only; prefix-vector meaning unchanged.

## Slicing (single slice)

| Slice | What | Notes |
|---|---|---|
| **1 — substrate primitive + sugar + test migration** | (1) `src/check.rs` `parse_def_restricted` accepts `:restricted-to` keyword-tagged form (preserving positional form as deprecated path? OR replacing it outright — slice BRIEF decides per `feedback_attack_foundation_cracks`). (2) `wat/core.wat:221-227` defmacro updates to accept `:restricted-to` keyword param. (3) `tests/wat_arc198_def_restricted.rs` ~5-6 sites migrate. (4) `wat/runtime.wat` consumer check (one sweep grep). (5) Workspace cargo test green. | Single atomic slice. Single sonnet sweep. SCORE inscribes complete migration. |
| **2 — closure paperwork** | INSCRIPTION + USER-GUIDE entry mentioning the keyword form + 058 row + DESIGN status CLOSED + cross-reference to arc 209 Stone A unblocking. | Standard closure. |

## Substrate touchpoints

- `src/check.rs` — `parse_def_restricted` parser; possibly `infer_def_restricted` if the parse-tree shape changes
- `wat/core.wat:221-227` — the `defn-restricted` defmacro pattern
- `tests/wat_arc198_def_restricted.rs` — 5-6 call sites (verified by pre-flight grep)
- `wat/runtime.wat:17-32` and any other wat-side file using `defn-restricted` — sweep target (zero hits per pre-flight grep)

## Connection to broader work

**Forward chain:**
```
Arc 210 closes (substrate + sugar use :restricted-to keyword tag)
            ↓
Arc 209 Stone A drafts (spawn-program defmacro + walker reshape)
            ↓
Arc 209 Stones A → B → C → D ship in order
            ↓
Arc 203 closure → Arc 170 closure → Lab reconstruction
```

Arc 210 is small + precursor; lands cleanly before defservice's generated code needs the new shape.

## Discipline carry-forward

This arc embodies two meta-disciplines surfaced in arc 209 prep:

1. **Assess the reach.** When the orchestrator reaches for unfamiliar syntax in pseudocode sketches, that's the substrate-as-teacher operating at the authoring layer. The reach is a signal — substrate vocabulary surfacing in new contexts. Assess via four-questions; good reaches get inscribed; bad reaches get caught + corrected. The `:restricted-to` reach was good; this arc inscribes it.

2. **Substrate-primitive surface matches user-facing surface.** Per `feedback_substrate_owns_not_callers_match`: when both layers exist, the substrate-primitive shouldn't be positional-positional while the sugar is keyword-tagged. The substrate IS the spec; the sugar adds ergonomics on top; both should look consistent at the surface readers grep.

Cross-references:
- INTERSTITIAL § 2026-05-18 Convergence #13 — the collapse insight + state-as-self + Rust convergence that surfaced this dissonance
- arc 198 (def-restricted minting) — what this arc tweaks
- arc 209 DESIGN — the consumer pressure (defservice's generated code needs the new shape)
- `feedback_simple_is_uniform_composition` — N identical compositions IS simple; keyword-tag uniformity is the simplest composition
- `feedback_substrate_owns_not_callers_match` — the discipline behind locking BOTH layers, not just the sugar
