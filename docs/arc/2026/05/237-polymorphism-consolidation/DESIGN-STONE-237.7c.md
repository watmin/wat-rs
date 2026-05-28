# Stone 237.7c — `:wat::core::assoc` as a Tier-B intrinsic spanning HashMap + Record

**Follows 237.7b complete** (all four collection ops shipped: length `8100d9d2`,
empty? `e401c183`, contains? `fef2c8d9`, conj `2d3259ae`, get `fad1c1c6`). Same
doctrine (`DESIGN-STONE-237.7-intrinsic-kill.md` + `DESIGN-STONE-237.7b.md` § Slices):
the **intrinsic boundary** — verbs needing `∀T` / multi-Value-variant dispatch /
raw-`Value` inspection MUST be Rust intrinsics; userland can't author them
(closed universe; `:Any` banned 058-030).

## What's different about assoc (and why it's the records-doctrine slice)

`assoc` is **NOT** a `define-dispatch` like the 7b ops were. It is a
**single-impl alias** (arc 146 slice 4, `wat/core.wat:50`) that today maps
`:wat::core::assoc → :wat::core::HashMap/assoc` — Record support is
**ABSENT** from the surface name. Calling `(:wat::core::assoc some-record :field v)`
**FAILS today** because the alias's signature is HashMap-only.

Meanwhile `:wat::Record/assoc` already exists (arc 234 Stone 234.3b, `e91860ee`)
as a polymorphic write verb with the Liskov-correct umbrella signature
`∀T. :wat::Record × :keyword × :T → :wat::Record`. `eval_record_assoc`
(`src/runtime.rs:17129`) **already accepts BOTH base and holonic flavors** —
base arm (lines 17150–17215) rebuilds `struct_form` only; holonic arm
(17218+) rebuilds BOTH `struct_form` AND `holon_form` in lockstep (the PARITY
invariant). Flavor is preserved: base → base, holonic → holonic.

So the work is **promoting the umbrella `:wat::core::assoc` from a HashMap-only
alias to a polymorphic intrinsic that dispatches across two heterogeneous
collection families** — HashMap (Parametric) and Record (umbrella Path). The
per-Type leaves stay; the intrinsic routes into them. This is the
**records-doctrine slice** the 7b DESIGN flagged at line 96.

## The Tier-B twist this slice introduces

The 7b custom arms (`infer_contains`, `infer_conj`, `infer_get`) all matched
arg0 against `Parametric` collection shapes with type args (Vector<T> /
HashSet<T> / HashMap<K,V>). 7c adds a new shape: matching against an **umbrella
path** (`:wat::Record`) with NO type args, where arg2 is free `∀T` (no
unification against an element-type slot).

Recipe per arm:

| arg0 reduced | arg1 expected | arg2 expected | return |
|---|---|---|---|
| `Parametric { head: "wat::core::HashMap", args: [K, V] }` | K | V | `HashMap<K,V>` (type-preserving) |
| `Path(":wat::Record")` | `:wat::core::keyword` | free ∀T (any) | `:wat::Record` (umbrella; flavor-preserved at runtime) |
| else | — | — | teaching `CheckError::TypeMismatch` (`Vector<T> or…` style: `"HashMap<K,V> or :wat::Record"`) |

Return is **type-preserving per arm** (no Option wrap, unlike `get`). For
HashMap the return uses `apply_subst(&coll_ty, subst)` (same as `conj`). For
Record the return is the umbrella `record_ty()` — the per-arm flavor (base vs
holonic) is a runtime property, not a check-time distinction (Liskov: both
flavors satisfy the umbrella).

At runtime, `eval_assoc` matches the raw `Value`:

- `Value::wat__std__HashMap(_)` → reuse `hashmap_assoc_inner` (existing helper,
  `src/runtime.rs:11410`)
- `Value::wat__Record { .. } | Value::wat__holon__Record { .. }` → delegate to
  `eval_record_assoc` (existing function, `src/runtime.rs:17129`, which already
  handles both flavors via its early-return base arm + holonic fallthrough)
- else → teaching `RuntimeError::TypeMismatch`

## Crawl (ground truth, 2026-05-27)

| site | what's there | role in 7c |
|---|---|---|
| `wat/core.wat:50` | `(:wat::runtime::define-alias :wat::core::assoc :wat::core::HashMap/assoc)` | **DELETE** (HARD CUT; the alias is the thing being replaced) |
| `src/check.rs:14271` | retired-`infer_assoc` comment block (arc 146 slice 4) | **REPLACE** with a real `fn infer_assoc(...)` per the recipe above |
| `src/check.rs:19788` | retired-fingerprint comment block (arc 146 slice 4) | KEEP unchanged (the per-Type leaves still register below it); orient by reading |
| `src/check.rs:19714` | `:wat::core::HashMap/assoc` TypeScheme (∀K,V) | KEEP (the per-Type leaf the intrinsic dispatches into) |
| `src/check.rs:20015` | `:wat::Record/assoc` TypeScheme (∀T) | KEEP (the per-Type leaf the intrinsic dispatches into) |
| `src/runtime.rs:5368` | `":wat::Record/assoc" => eval_record_assoc(…)` dispatch arm | KEEP |
| `src/runtime.rs:5822` | `":wat::core::HashMap/assoc" => eval_hashmap_assoc(…)` dispatch arm | KEEP |
| `src/runtime.rs:11410` | `fn hashmap_assoc_inner` | REUSE in `eval_assoc` |
| `src/runtime.rs:11532` | `fn eval_hashmap_assoc` | KEEP (per-leaf entry) |
| `src/runtime.rs:17129` | `fn eval_record_assoc` (handles BOTH flavors) | REUSE in `eval_assoc` |
| `src/runtime.rs:12494` | retired-`eval_assoc` comment block (arc 146 slice 4) | **REPLACE** with a real `fn eval_assoc(...)` |
| (new) `src/check.rs` near `infer_get` | mint `fn infer_assoc` | NEW |
| (new) `src/check.rs::register_builtins` near `length`/`get` ∀T entries | register `:wat::core::assoc` plain ∀T scheme (the custom-arm overrides at infer_list dispatch; the scheme is the fallback rank-1 form for the env.get path) — **mirror 7b-iv exactly** | NEW |
| (new) `src/runtime.rs::eval_list` arm near `eval_get` | wire `":wat::core::assoc" => eval_assoc(…)` | NEW |

## Slicing

**ONE-stone slice.** No internal split — the recipe is thrice-proven (7b-ii /
7b-iii / 7b-iv), the only new shape is the Path-(not-Parametric) arm with free
∀T arg2, and the runtime arms already exist (no new inner helpers needed). One
sweep covers check.rs + runtime.rs + wat/core.wat + the SCORE.

## FM-2-bis probe — what it must prove

`tests/probe_arc237_7c_assoc_polymorphic.rs`. Disconfirming AT HEAD `e435194d`
(today; before substrate work); regression guard after.

| # | Test | At HEAD (today) | After 7c ships |
|---|---|---|---|
| 1 | `assoc_hashmap_returns_hashmap_type_preserved` | PASS via the alias | PASS via the intrinsic |
| 2 | `assoc_hashmap_wrong_key_type_rejected_at_check` | PASS via the alias's scheme | PASS via `infer_assoc` HashMap arm |
| 3 | `assoc_hashmap_wrong_value_type_rejected_at_check` | PASS via the alias's scheme | PASS via `infer_assoc` HashMap arm |
| 4 | `assoc_base_record_returns_base_record_struct_only` | **FAIL** (the alias rejects Record at check) | PASS via `infer_assoc` Record arm + `eval_record_assoc` base early-return |
| 5 | `assoc_holonic_record_preserves_parity` | **FAIL** (same reason) | PASS via `infer_assoc` Record arm + `eval_record_assoc` holonic fallthrough; struct + holon both updated |
| 6 | `assoc_non_collection_arg0_runtime_type_mismatch` | passes some way today (alias's check rejects); should pass post via `infer_assoc` else-arm | PASS — teaching `TypeMismatch` (check or runtime) |

Tests 4 + 5 are the disconfirming load-bearing rows. They MUST be RED at HEAD
(proving the gap) and GREEN post-7c (proving the fix).

Commit the probe BEFORE the BRIEF. Sonnet mirrors it.

## What this stone does NOT do

- `DispatchRegistry` deletion (still 237.7c-or-237.8-adjacent per the 7b DESIGN).
- Arithmetic family (237.8 — concrete defclauses + DELETE widest-contagion).
- USER-GUIDE base-vs-holonic records sentence (owed at the arc 237.9
  INSCRIPTION; not this stone).
- `dissoc` / `keys` / `values` alias-to-intrinsic promotion. These are HashMap-only
  by their nature (Record has no dissoc/keys/values primitive at the umbrella
  level — Record's field set is class-fixed, not mutable; record fields are
  introspected via `:wat::core::record->map` arc 234.3a, not `:keys`). If a
  future records-doctrine stone surfaces a Record-side keys/values/dissoc need,
  it gets its own slice. **Out of arc 237.7c's scope** per the
  one-canonical-path doctrine — don't mint what no caller has demanded.

## Constraints

- Edits in `src/check.rs` + `src/runtime.rs` + `wat/core.wat` only.
- NO holon-rs. NO `DispatchRegistry` deletion. NO touch of the per-Type leaves'
  schemes / eval functions / dispatch arms. NO touch of arithmetic decls.
- Green-gate (momentary, per `feedback_green_gate_lib_and_build` +
  `feedback_sonnet_bash_firewall`): two RAW cargo commands as SEPARATE lines —
  `cargo build --release --tests --workspace` (0 errors) + `cargo test --release
  --lib -p wat` (834+/0). **Do not invoke `./scripts/green-gate.sh`** — the
  Anthropic firewall denies wrapper scripts for sub-agents and orchestrator
  under restricted permission modes (lesson 2026-05-27).
- Probe is the regression guard: `cargo test --release --test
  probe_arc237_7c_assoc_polymorphic` ≥ all-row PASS post-ship.
- HARD CUT discipline — DELETE the alias line; no shim, no "legacy fallback",
  no "if record arm fails, try hashmap." The intrinsic owns it.
