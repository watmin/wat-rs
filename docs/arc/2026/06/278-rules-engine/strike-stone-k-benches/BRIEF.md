# BRIEF — Stone K moves 2–4: three diagnostics from `#[ignore]` to `benches/`

## The work

Move `binding_key_cost`, `binding_repr_microbench` and `token_bindings_representation_dominance`
out of `src/rete/kernel/tests/binding_repr_bench.rs` into a `benches/` target with
`harness = false`. Two live gates in that file STAY. Behaviour unchanged — only the harness.

## Read in order

1. `benches/perf_arc278_fire_baseline.rs:1-30` — **move 1, the template.** Its header states the
   rule, the `harness = false` / plain `fn main()` shape, and that behaviour must be UNCHANGED.
   Note it reaches the crate through `use wat::…` — the public API only.
2. `Cargo.toml:250-254` — the `[[bench]]` stanza to copy.
3. `src/rete/kernel/tests/binding_repr_bench.rs` — **read the whole 741 lines**, not the three
   functions. Its header (`:1-5`) records `partire`'s finding that the region names zero symbols
   from its host module, which is what makes the move possible; the shared helpers and the `use`
   block have to be split between the two homes.
4. `docs/arc/.../strike-stone-k-benches/DESIGN.md` — the five-test table and the floor prediction.

## Sketch

```rust
//! benches/binding_repr.rs — Stone K moves 2-4. Relocated from
//! src/rete/kernel/tests/binding_repr_bench.rs. Behaviour UNCHANGED; only the harness.
//! A benchmark is not in the test binary at all, so the category is structural rather
//! than an #[ignore] excused by a string.
//! Run: cargo bench --bench binding_repr
use wat::Value;                       // pub use at src/lib.rs:375
fn main() { binding_key_cost(); binding_repr_microbench(); token_bindings_dominance(); }
```

Then in `Cargo.toml`, one `[[bench]]` with `harness = false`; and in
`src/rete/kernel/tests/binding_repr_bench.rs`, delete the three fns and whatever helpers now have
no caller — keeping everything the two surviving gates still use.

## Carry the reasons across

The three `rune:excusare` strings are **measured evidence** and must survive as doc comments on the
moved functions: `:146`'s lookup/build ratios, `:264`'s two named failed gate attempts, `:597`'s
`medians 2028.8 ns / 676.3 ns = 3.0×` against a `3.5–4.4×` noise floor. Without them a future hand
reads three unasserted benchmarks and "fixes" them into assertions — which is the excursion that
reddened this floor earlier in the arc.

Whether the literal `rune:excusare(...)` text stays or becomes prose is your call against
`no_unknown_ward_rune.rs` — check what that gate scans before deciding, and say which you chose.

## The floor prediction — state it before you run it

Today: `5480 run / 22 skipped`. The three are `#[ignore]`d, so they are among the 22 skipped, not
the 5480 run. After the move: **`5480 run / 19 skipped`**. Run count unchanged, skipped down by
exactly three. Any other number means something else moved — investigate before landing.

## STOP triggers

1. **If any of the three needs a `pub(crate)` item** — STOP. The whole move rests on the file
   naming zero symbols from its host module. Report what it reaches; do NOT widen the public API to
   make a benchmark compile.
2. **If a shared helper is needed by BOTH a moved fn and a surviving gate** — STOP and report it.
   Duplicating it silently forks two copies that will drift; that decision is the orchestrator's.
3. **If the skipped count does not fall by exactly three** — STOP and say what else changed.
4. Do not turn any diagnostic into an assertion. Do not re-measure the recorded numbers. Do not
   touch the two live gates.

## Blast radius

New `benches/binding_repr.rs`, one `Cargo.toml` stanza, deletions in
`src/rete/kernel/tests/binding_repr_bench.rs`. **No `wat/`. No engine code.**

## Prior comparable

`benches/perf_arc278_fire_baseline.rs` is move 1 of this same stone — read its header as the
statement of intent this strike continues.
