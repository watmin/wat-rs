# DESIGN — Stone K moves 2–4: the three timing diagnostics leave the test binary

## Why

`benches/perf_arc278_fire_baseline.rs`'s own header states the rule and calls itself **move 1**:

> *"Relocated from `tests/rete/perf_arc278_fire_baseline.rs` (296 Stone K, move 1 — benchmarks live
> in `benches/` + `cargo bench`, not behind `#[ignore]`). Behaviour is UNCHANGED… Only the harness
> changed — `#[test]`/`#[ignore]` became a `[[bench]]` target (`harness = false`, plain `fn main()`)
> so the category is **structural**: a benchmark is not in the test binary at all, and cannot
> inflate the ignore count."*

Three diagnostics in `src/rete/kernel/tests/binding_repr_bench.rs` are still `#[ignore]`d tests.
`#[ignore]` is a convention — a test that *is* in the binary and *is* counted, excused by a string.
`benches/` is the rung above: the mistake cannot be expressed, because the code is not a test.

## Feasibility — settled by the file's own header, not assumed

`binding_repr_bench.rs:3-5`:

> *"`partire` verified this region names ZERO symbols from its host module — no `super::`, no
> `FireSession`, no `to_transient`, no `eval_in`. It builds `Value`/`Arc`/`HashTrieMapSync`
> directly."*

That is exactly the property a `benches/` target needs, since it compiles as a **separate crate**
and sees only `pub` items. `Value` is `pub use`d at `src/lib.rs:375`, so `wat::Value` resolves; the
rest is `std` and `rpds`. **This is why Stone K is possible at all** — a diagnostic that reached
`pub(crate)` internals could not move without widening the API, and that would be a worse trade.

## What moves and what stays — the file holds FIVE tests, not three

| test | disposition |
|---|---|
| `bind_key_construction_vs_map_operation` (`:49`) | **STAYS** — a live gate, not ignored |
| `binding_key_cost` (`:147`) | **MOVES** — `rune:excusare(below-resolution)` |
| `binding_repr_microbench` (`:265`) | **MOVES** — `rune:excusare(no-falsifier)` |
| `binding_cardinality_distribution` (`:407`) | **STAYS** — a live gate, not ignored |
| `token_bindings_representation_dominance` (`:598`) | **MOVES** — `rune:excusare(below-resolution)` |

⚠ An item-level move drops what is not an item: the module header, the shared helpers, and the
`use` block belong to whichever side still needs them. **Read the whole file, not five functions.**
`[[an-item-level-move-drops-what-is-not-an-item]]`

## The one contract decision

**The three `rune:excusare` REASONS survive the move as doc comments.** They are measured evidence —
`below-resolution` carries *"medians 2028.8 ns / 676.3 ns = 3.0×… noise floor 3.5–4.4×… the margin
sits inside the floor"*, and `no-falsifier` carries two named failed attempts at a gate. Once there
is no `#[ignore]`, the rune has no attribute to sit on, but **deleting the reason would destroy the
only record of why these are not gates** and invite a future hand to "fix" them into assertions —
the exact excursion that reddened the floor earlier in this arc.

Whether they remain literal `rune:excusare(...)` text or become prose is the executor's call
against the ward-vocabulary gate; the REASONS are non-negotiable.

## The floor prediction

The three are `#[ignore]`d, so today they are **skipped**, not run: `5480 run / 22 skipped`. After
the move they leave the test binary entirely, so the expectation is **`5480 run / 19 skipped`** —
run count unchanged, skipped down by exactly three. A different number means something else moved.

Nothing pins the skip count (grepped: no `22 skipped`, no ignore-count assertion), and nothing
outside the file names the three tests except `mod binding_repr_bench;` at `tests/mod.rs:224`.

## Out of scope = REJECTED

- **Turning any diagnostic into an assertion.** Their runes say why that failed; `:597`'s margin
  sits inside the measured noise floor and `:264` names two attempts that produced known-false
  gates.
- Re-measuring any of the recorded numbers. A strike that re-runs a bench to satisfy a rule about
  evidence has let the rune drive the work.
- The two live gates.
