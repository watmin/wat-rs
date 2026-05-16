# Arc 198 INSCRIPTION — Declared restriction (wat-side + Rust-side)

**Status:** SHIPPED 2026-05-16. Two slices complete; loop closed on arc 170 Stone B's ad-hoc precursor.

## What this arc gave the substrate

Arc 198 minted **declared access control on bindings**: a primitive that lets any callable name carry an allowed-caller-prefix whitelist, enforced at compile-time by the type checker. The mechanism is symmetric across the wat ↔ Rust boundary — same storage, same walker, two declaration surfaces.

The pattern compresses to: *a name exists, is callable, but the walker only permits call sites whose enclosing definition matches the prefix list.* Convergent design — every language with structured visibility has independently arrived here:

| Tradition | Form |
|-----------|------|
| **Rust** | `pub(crate)` / `pub(super)` — caller-scope visibility |
| **Clojure** | `^:private` metadata + `defn-` shortcut |
| **Erlang** | `-export([...])` — module-internal vs exported |
| **Common Lisp** | symbol externals vs internals via packages |
| **Java** | `private` / `package-private` |
| **wat-rs (arc 198)** | `def-restricted` (wat) + `#[restricted_to(...)]` (Rust) — both populate one HashMap |

Inverted from Erlang's `-export` (which declares what IS exported), arc 198 declares **who CAN call** — more expressive than binary public/private. A single name can be callable from multiple specific namespaces without substrate dispatch on call site.

## The two declaration surfaces

### Wat-side (slice 1 — commit `24d3b0d`)

```scheme
;; substrate primitive
(:wat::core::def-restricted
  :my::name                              ;; the symbol being bound
  [:wat::kernel:: :wat::test::]          ;; allowed-caller prefix list
  <value-expr>)

;; defmacro sugar over def-restricted
(:wat::core::defn-restricted
  :my::fn-name
  [:wat::kernel:: :wat::test::]
  (x :i64) -> :i64
  body)
;; expands to (def-restricted :my::fn-name [:wat::kernel:: :wat::test::]
;;              (fn (x :i64) -> :i64 body))
```

### Rust-side (slice 2 — commits `51c69a1` → `045d16f`)

```rust
#[restricted_to(":wat::kernel::Thread/join-result", ":wat::")]
fn eval_kernel_thread_join_result(...) -> Result<Value, RuntimeError> { ... }

#[restricted_to(":my::fn", ":wat::", ":my::specific::caller")]
pub(crate) fn some_other_primitive(...) -> ... { ... }
```

Variadic positional string-literal args. First arg = wat name; remaining = prefix list.

## Prefix matching rules (uniform across both surfaces)

- Entry ending in `::` (e.g., `:wat::kernel::`) → **namespace prefix match** (caller FQDN must start with this prefix)
- Entry NOT ending in `::` (e.g., `:wat::kernel::specific-fn`) → **exact FQDN match**
- Empty whitelist `[]` → no callers allowed (every call fails)

## Architecture end-to-end

```
                ╔═══════════════════════════════════════╗
                ║  defined_value_restrictions HashMap   ║
                ║    (sym, mirrored to env)             ║
                ║   String → Vec<String>                ║
                ╚═══════════════════════════════════════╝
                            ▲              ▲
                            │              │
        ┌───────────────────┘              └──────────────────┐
        │  wat-side                          Rust-side        │
        │  (slice 1)                         (slice 2)        │
        │                                                     │
   def-restricted /                            #[restricted_to(...)]
   defn-restricted                                  │
        │                                           ▼
        │                              inventory::submit! at fn site
        │                                           │
        │                                           ▼
        │                          freeze.rs step 6.8 iterates
        │                          inventory::iter::<RestrictionEntry>
        │                                           │
        └───────────────────────────────────────────┘
                            │
                            ▼
              ╔═════════════════════════════════════════╗
              ║   walk_for_def_restricted_call          ║
              ║   (check_program hook)                  ║
              ║   Caller FQDN against prefix list       ║
              ╚═════════════════════════════════════════╝
                            │
            mismatch       ▼        match
                CheckError::DefRestrictedCallerNotAllowed
```

One storage, one walker, two declaration surfaces. Loop closure: arc 170 Stone B's ad-hoc `validate_join_result_user_namespace` rule is retired — its 2 substrate primitives (`Thread/join-result` + `Process/join-result`) are now declared via `#[restricted_to(...)]` and protected by the same generic mechanism.

## Slice & stone breakdown

### Slice 1 — wat-side primitive + sugar

| File | Commit | What |
|------|--------|------|
| BRIEF.md / EXPECTATIONS.md / SCORE.md | `6eba1f2` / `24d3b0d` | Mint `:wat::core::def-restricted` + `:wat::core::defn-restricted` |

Outcome: 6/6 SCORE rows PASS; 5/5 tests; ~75 min sonnet (predicted 60-90); AST recognized by head keyword (option (c) — neither new variant nor extended Def); per-binding `HashMap<String, Vec<String>>` storage in CheckEnv + SymbolTable.

### Slice 2 — Rust-side complement (4-stone decomposition)

| Stone | Commit | Predicted | Actual | What |
|-------|--------|-----------|--------|------|
| Original monolithic | (superseded) | 180-300 min | (killed in reading) | Bundle: proc-macro + inventory + migration + rule deletion |
| Stone 1 — inventory wiring | `51c69a1` | 60 min | 25 min | `inventory` dep + `RestrictionEntry` struct + freeze.rs step 6.8 iteration |
| Stone 2 — proc-macro attribute | `6775510` | 90 min | 40 min | `#[restricted_to(...)]` via `Punctuated<LitStr>`; codegen emits `::inventory::submit!` |
| Stone 3 — apply to *_join-result | `fe2e0eb` | 30 min | 12 min | Annotate `eval_kernel_{thread,process}_join_result`; both walkers fire transition |
| Stone 4 — loop closure | `045d16f` | 45 min | 7 min | Delete Stone B's ad-hoc rule + variant + hook (~140 net LOC); update Stone B's 4 tests |
| **Slice 2 totals** | — | **225 min** | **84 min** | Decomposition shipped at 37% of monolithic prediction |

### Slice 2 BRIEFs/EXPECTATIONS/SCOREs

- `BRIEF-STONE-1-INVENTORY-WIRING.md` / `EXPECTATIONS-STONE-1-INVENTORY-WIRING.md` / `SCORE-STONE-1-INVENTORY-WIRING.md`
- `BRIEF-STONE-2-PROC-MACRO-ATTRIBUTE.md` / `EXPECTATIONS-STONE-2-PROC-MACRO-ATTRIBUTE.md` / `SCORE-STONE-2-PROC-MACRO-ATTRIBUTE.md`
- `BRIEF-STONE-3-APPLY-TO-JOIN-RESULT.md` / `EXPECTATIONS-STONE-3-APPLY-TO-JOIN-RESULT.md` / `SCORE-STONE-3-APPLY-TO-JOIN-RESULT.md`
- `BRIEF-STONE-4-LOOP-CLOSURE.md` / `EXPECTATIONS-STONE-4-LOOP-CLOSURE.md` / `SCORE-STONE-4-LOOP-CLOSURE.md`
- `BRIEF-RUST-ATTRIBUTE.md` / `EXPECTATIONS-RUST-ATTRIBUTE.md` (SUPERSEDED — original monolithic BRIEF preserved with SUPERSEDED headers per `feedback_inscription_immutable`)

## Tests across the arc — 19/19 green

| File | Count |
|------|-------|
| `tests/wat_arc198_def_restricted.rs` (slice 1) | 5 |
| `tests/wat_arc198_slice2_stone_1_inventory_wiring.rs` | 1 |
| `tests/wat_arc198_slice2_stone_2_attribute.rs` | 3 |
| `tests/wat_arc198_slice2_stone_3_apply.rs` | 2 |
| `tests/wat_arc170_stone_b_walker_collapse.rs` (updated in Stone 4 to assert `DefRestrictedCallerNotAllowed` instead of Stone B's `drain-and-join` substring) | 4 |
| **Arc 198 total** | **15** |
| (plus arc 170 Stone A regression) | (+4 → 19) |

Workspace baseline maintained throughout: 3 stable failures (t6 unquote, totally_bogus, startup_error) + 1 lifeline flake (rotation band). Zero regressions across all 5 stones.

## Discipline lessons inscribed

### Test-first caught a real bug — Stone 4

Initial attempt asserted `err.contains("allowed-caller whitelist")` — fragment from arc 198's Display impl. But Stone B's tests use `format!("{:?}", e)` (Debug rendering), where Display's wording doesn't appear. The variant name `DefRestrictedCallerNotAllowed` is what's visible in Debug output. Test failed BEFORE deletion ran — caught the mismatch; corrected the assertion to use the variant name; THEN proceeded with deletion. Per `feedback_test_first`: see-fail-first prevents post-deletion archaeology.

### Decomposition vs monolithic prediction

The originally-monolithic slice-2 BRIEF predicted 180-300 min. That prediction was the warning signal — bundled too much (proc-macro + inventory + migration + rule deletion + test updates). User correction: decomposed into 4 stones; sonnet shipped each in 7-40 min. Total slice 2 time: 84 min — 37% of the monolithic upper-band. **The decomposition is the speed.** Per `feedback_iterative_complexity`: small bounded stones beat one-shot multi-piece changes.

### "Simplicity is at the surface"

User sharpening 2026-05-16: *"is this nothing more than things with simple surfaces composed into a thing with a simple surface?... it's always simple on the surface."* The `inventory` crate is complex underneath but exposes two simple primitives (`submit!` + `iter::<T>`). Counted as SIMPLE on the four-questions check. Saved as updated `feedback_simple_is_uniform_composition`.

### Stay in arc until inscribed

User correction 2026-05-16: *"why is this 199 and not a continuity of 198?"* The Rust-side complement of slice 1's wat-side mechanism IS arc 198 continuation — not a new arc number. Saved as `feedback_stay_in_arc_until_inscribed`. The reflex to mint a fresh arc number is the deferral pattern in disguise.

## What arc 198 enables next

- **Future substrate primitives** that need restriction declare via `#[restricted_to(...)]` at the fn site — no manual `env.defined_value_restrictions.insert` plug-in dance. Inventory wiring auto-registers.
- **Future wat-level user code** that wants module-scoped helpers declares via `(def-restricted ...)` or `(defn-restricted ...)`. Same mechanism, same walker enforcement.
- **Arc 170 Stone G** (when reached) — the arc 117/133 sibling-binding walker machinery is independent of arc 198 (different concern: in-scope sibling-deadlock detection vs cross-scope visibility). Its retirement is its own future stone.
- **Arc 170 bracket combinator chain** (Stones C-H) — Client/Server type pairs + `run-threads` / `run-processes` macros. Bracket primitives that emerge will likely declare their own restrictions via `#[restricted_to(...)]` — arc 198's mechanism is the canonical way going forward.

## Cross-references

- Original architectural commitment: `docs/arc/2026/05/170-program-entry-points/INTERSTITIAL-REALIZATIONS.md` (2026-05-15 entries on the bracket-combinator design conversation; 2026-05-16 entries on closure of the design phase)
- Stone B's caller migrations (~40 sites) preserved at commit `2a071f0`; tests at `tests/wat_arc170_stone_b_walker_collapse.rs` (updated in arc 198 Stone 4 but still verifies the same enforcement)
- arc 198's walker `walk_for_def_restricted_call` lives at `src/check.rs` (slice 1)
- `RestrictionEntry` struct lives at `src/restriction_entry.rs` (slice 2 Stone 1)
- `#[restricted_to(...)]` proc-macro lives at `crates/wat-macros/src/lib.rs` (slice 2 Stone 2)

---

The substrate refuses; the user does the work; we ship the hard part because that's what we do.

Arc 198 inscribed.
