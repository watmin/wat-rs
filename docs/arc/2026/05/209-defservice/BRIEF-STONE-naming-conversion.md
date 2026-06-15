# BRIEF — Stone: PascalCase ⇄ kebab-case naming-conversion tooling

Executor: Shadowdancer (sonnet). Anchor: `/home/watmin/work/holon/wat-rs/` (verify `pwd`; ONLY here;
ignore `.claude/worktrees/`). Run `cargo test`/`cargo build` PLAINLY (no setsid/timeout). Trust your
own build over rust-analyzer. **Do NOT commit — the Inquisitor weighs.**

## Work in one paragraph
Build the full bidirectional PascalCase⇄kebab converter and thread the forward direction into
defservice. Three pieces, each placed by the OP-PLACEMENT rubric: `pascal->kebab` (Rust intrinsic,
macro-needed), `to-uppercase` (Rust primitive), `kebab->pascal` (wat helper). Then replace the bare
`to-lowercase` in `wat/service.wat`'s op-name derivation with `pascal->kebab` so multi-word ops name
correctly (`:GetObject` → `get-object` / `get-object-request`). Spec + algorithms:
`docs/PASCAL-KEBAB-CONVERSION.md`. Full design: `DESIGN-STONE-naming-conversion.md` (this dir).

## The model to copy
`eval_string_to_lowercase` in `src/string_ops.rs` + its wiring is the EXACT mold for the two Rust
ops. Read it first, then mirror it twice.

## Piece 1 — `:wat::core::string::pascal->kebab` (Rust intrinsic, on is_pure_total)
- `eval_string_pascal_to_kebab` in `src/string_ops.rs`: input String → boundary BEFORE each
  uppercase char except position 0, downcase every char, segments joined by `-`. (`GetObject` →
  `get-object`; `Get` → `get`; `GetV2` → `get-v2`; digits ride the current word.) Pure + total.
- Wire the four sites like to-lowercase: check scheme (`String->String`) in `src/check.rs`; runtime
  dispatch arm in `src/runtime.rs`; **add the head to `is_pure_total` in `src/macros/eval.rs`** (the
  defservice macro calls it at expand time — this entry is load-bearing).

## Piece 2 — `:wat::core::string::to-uppercase` (Rust primitive)
- `eval_string_to_uppercase` in `src/string_ops.rs`: `s.to_uppercase()`. Pure + total. Same four-site
  wiring as to-lowercase EXCEPT **do NOT add to `is_pure_total`** (no macro calls it).

## Piece 3 — `:wat::core::string::kebab->pascal` (wat helper)
- New file `wat/string.wat`, added to the `src/stdlib.rs` embedded list **after `core.wat`** (a new
  `include_str!("../wat/string.wat")` entry in order). It holds wat-level string helpers; this is its
  first.
- `(:wat::core::defn :wat::core::string::kebab->pascal [s <- :wat::core::String] -> :wat::core::String …)`
  — `split` on `"-"` → for each segment, `to-uppercase` the first char (`subs seg 0 1`) + keep the
  rest (`subs seg 1 (length seg)`) → `concat`; join the capitalized segments. The algorithm is in
  `docs/PASCAL-KEBAB-CONVERSION.md` (the `kebab->pascal` + `capitalize` sketch) — port it; it now
  works because `to-uppercase` (piece 2) exists.

## Piece 4 — thread into defservice (`wat/service.wat`)
- The op-name derivation does `op-lower (:wat::core::string::to-lowercase op-str)` in the
  constructors foldl (~line 387) AND the methods foldl (~line 443). Replace BOTH `to-lowercase` with
  `pascal->kebab`. (Records/variants stay PascalCase — concat, untouched.) Single-word ops must still
  work (`Get` → `get` — pascal->kebab of a single word == lowercase).

## Gate (run all; report verbatim from YOUR runs)
```
cargo test --release -p wat --test probe_arc209_naming_conversion        # 2 passed (both directions + roundtrip; defservice :GetObject)
cargo test --release -p wat --test probe_arc209_c3_defservice_client_face # 1 passed (single-word defservice still works)
cargo test --release -p wat --test probe_string_to_lowercase             # 1 passed (sibling intact)
cargo test --release -p wat --lib -- --test-threads=1                     # zero NEW failures (baseline 36)
cargo test --release -p wat --test nursery -- --test-threads=1            # zero NEW (baseline 4)
cargo test --release --workspace --no-run                                 # compiles
```

## STOP triggers (REJECT — surface; do not improvise)
1. `pascal->kebab` can't be added to `is_pure_total` / the defservice macro can't call it → STOP
   (that's the whole point — it must be macro-reachable).
2. `wat/string.wat`'s load position means `kebab->pascal` can't see the Rust string primitives →
   STOP (the Rust intrinsics register before any `.wat` loads; surface if the order fights you).
3. Threading `pascal->kebab` regresses a single-word op (`Get` must stay `get`) → STOP.
4. Tempted to make `kebab->pascal` a Rust intrinsic for symmetry → STOP — the rubric says wat helper
   (no macro needs it; it composes). Keep it in wat.

## Blast radius
`src/string_ops.rs`, `src/check.rs`, `src/runtime.rs`, `src/macros/eval.rs`, `src/stdlib.rs`,
`wat/service.wat`, new `wat/string.wat` + the probe. NO changes to defprotocol/the registries/
`assignable`/dispatch.

## Return
Report: the two Rust eval fns + their wiring sites, the `is_pure_total` entry (pascal->kebab only),
the `wat/string.wat` home + its stdlib.rs position, the service.wat thread (both sites), every gate
command's counts from YOUR runs, and any honest delta. Do NOT commit.
