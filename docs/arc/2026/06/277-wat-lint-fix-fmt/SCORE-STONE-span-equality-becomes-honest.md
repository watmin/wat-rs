# SCORE — STONE: `Span` equality becomes honest

No commit. Blast radius matched the census: two tests, both bugs the vacuous eq was hiding. No third red.

## The two tests — before / after

**Before.** Each tuple called `rust_caller_span!()` twice, at different lines. Under always-true `Span::eq` the arm proved only that `.span()` / `.span` does not panic.

```
left:  Span { …probe_arc243_stone3_typeerror_pattern_a.rs, line: 83, col: 17, end: None }
right: Span { …probe_arc243_stone3_typeerror_pattern_a.rs, line: 89, col: 13, end: None }
```

**After.** One span bound, cloned into the error, moved as `expected`. `assert_eq!(actual_span, expected_span)` now proves the span the error was constructed with is the span `.span()` returns — the claim the doc comment already made.

Same shape in `probe_arc243_stone6_checkerror_pattern_a::checkerror_span_access_is_single_path`.

`probe_arc243` filter: **21 tests run: 21 passed**. The two named tests are among them. No third failure.

## `span.rs` — `# Equality and hashing` rewritten

```
`Span::eq` compares `file` / `line` / `col` / `end`. A span assertion
means what it says.

Position-independence of AST identity is a `WatAST` requirement, not a
`Span` one. `WatAST`'s `PartialEq` compares structure and skips the span.

`Span::hash` is a no-op. Rust's contract is `a == b ⟹ hash(a) == hash(b)`;
unequal spans may collide. The no-op keeps `WatAST` hashes
position-independent for `canonical_edn_wat`.
```

The false claim (*"it never compares Span values for equality"*) is gone. `Pos` gained `PartialEq + Eq`. `Hash` for `Span` is the same no-op.

`span::tests::span_eq_compares_file_line_col_end` — same file/line/col/end equal; different line, col, or `end` not equal.

## `WatAST` — requirement moved, not dropped

Manual `PartialEq` over the 14 variants: structure compared, span skipped. `Hash` unchanged (still skips span via the no-op).

**Synthetic vs parsed still holds:** `parser::tests::atom_literals` —

```
assert_eq!(crate::parse_one!("42").unwrap(), WatAST::int(42));
```

and the rest of that function (ints, floats, bools, strings, keywords, symbols). Green under `cargo test --release -p wat-reader` (105 lib + 2 totality).

## Comment-only (code left)

`tests/value/probe_runtime_error_one_door.rs:38` — Debug workaround stays. Comment no longer says "`Span` doesn't derive `PartialEq`".

## Commands

| command | result |
|---|---|
| `cargo build --release` | Finished `release` in 20.43s |
| `cargo test --release -p wat-reader` | **105** lib + **2** totality passed |
| `cargo nextest run --release -E 'test(probe_arc243)'` | **21 passed**, 5167 skipped |
| `cargo nextest run --release --test lint` | **118 tests run: 118 passed, 0 skipped** |

Floor and clippy `--all-targets -D warnings` are the orchestrator's.

## What surprised me

Nothing about the blast radius — two tests, as drawn. The `probe_arc243` filter is wider than the two named tests (21 binaries' worth) and all of them passed, which is the STOP-1 measurement I could take without a full floor. `src/runtime.rs:19751` was not in this filter; the census already recorded it as a claim that happens to be true.

---

## ORCHESTRATOR VERDICT — 2026-09-05, weighed against my own re-run

**ACCEPTED, with two orchestrator-side edits.** The census held: no third red.

| what | command | result |
|---|---|---|
| the floor, on the rider's tree as delivered | `scripts/floor.sh` | **5171 run, 5171 passed, 0 FAILED, 17 skipped** |
| doctests (floor runs them first now) | (inside `floor.sh`) | 5 + 3 passed, 0 failed |
| clippy, the half the SCORE left to me | `cargo clippy --release --all-targets -- -D warnings` | ⛔ **1 error** |
| the floor, after both edits | `scripts/floor.sh` | **5171 / 5171**, clippy **0** |

**5170 → 5171 is the right delta**: `span_eq_compares_file_line_col_end`, +1, and nothing else moved.

### Edit 1 — the clippy red

`clippy::items_after_test_module`: `mod tests` was inserted mid-file, leaving `span_prefix`
stranded behind it. Moved the module to the end of `span.rs`. No code change.

### Edit 2 — `WatAST::eq` was one wildcard short of the defect it was fixing

The delivered impl was `match (self, other) { …14 arms…, _ => false }`. Its own neighbour,
`impl Hash for WatAST`, scrutinizes `self` alone and is **exhaustive** — so a new variant is a
compile error there and a silent `false` here. That is the same shape as the bug this stone
closed: a `PartialEq` that answers without looking.

Rewritten to match on `self` alone, `matches!(other, …)` inside each arm. All 14 arms preserved
verbatim; set-difference of enum variants against impl arms is **14 = 14, empty both ways**.

**⚠ SABOTAGE, AND IT CORRECTED ME.** I first wrote that the wildcard version "would compile
silently." The negative control refutes that framing:

```
15th variant added → cargo build -p wat-reader
  against the exhaustive impl : error[E0004] ×4
  against the wildcard impl   : error[E0004] ×4      ← IDENTICAL
```

Four other exhaustive matches in `ast.rs` already catch a new variant. The true difference is
what happens *after* those four are answered: the wildcard version then compiles, shipping a node
that is unequal to itself; the exhaustive one is a fifth error the author must answer by deciding
what equality means. The comment in `ast.rs` now states the measured claim, not the flattering one.

### Not disputed

`Pos` gaining `PartialEq + Eq`; the module doc rewrite (the false *"it never compares Span values
for equality"* is gone); the `Hash` no-op retained with the correct one-directional contract; the
comment-only edit at `probe_runtime_error_one_door.rs:38`.
