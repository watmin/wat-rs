# BRIEF — 279.4 · `interpolate` renders anything, and the partial renderer ceases to exist

You are a rider, not the orchestrator. **Ending your turn ENDS you** — nothing wakes you, no
notification is coming, and a Monitor cannot wake you either. Run every verification in the
**FOREGROUND** and block on it: your turn ends when the numbers are in your hands, not when the
command is launched.

Work in `/home/watmin/work/holon/wat-rs/`. **Do not commit, push, stash, or revert.**

## Read first

1. `docs/arc/2026/06/279-format/DESIGN-STONE-279.4-interpolate-renders-anything.md` — the ruling.
2. `docs/arc/2026/06/279-format/EXPECTATIONS-STONE-279.4-interpolate-renders-anything.md` — the scorecard.

## The work in one paragraph

`src/check.rs:13930-13935` already documents `interpolate`'s value slots as accepting *"ANY
str-renderable type (do NOT reject non-String values — **the intrinsic renders them unquoted at
runtime**)"*. The runtime does not do that: it uses `render_unquoted`, a five-arm partial renderer
that raises on anything else. So `(interpolate "{r}" :r <a record>)` type-checks and then raises.
Stone 279.3 minted `render_str_total` — the total renderer, already the implementation of
`:wat::core::str` and `join`. Point `interpolate` at it. `render_unquoted`'s caller count then goes
to **zero**, and you delete it.

## Read in order

1. **`src/string_ops.rs:512-517`** — `render_str_total`, the door. Two arms: top-level `String`
   renders **bare**, everything else through the EDN encoder with the type registry. You are adding
   its third caller; you are not writing a renderer.
2. **`src/string_ops.rs:612`** — the one line that changes, inside `eval_string_interpolate`.
3. **`src/string_ops.rs:519-545`** — `render_unquoted`, its doc comment, and its signature. This is
   what you delete.
4. `src/check.rs:13930-13935` — the comment that has been promising this. Read it; confirm it reads
   true afterwards.

## The three moves

### 1 — Point `interpolate` at the door (`string_ops.rs:612`)

```rust
- let rendered = render_unquoted(eval(val_arg, env, sym)?.value_owned(), OP, val_arg.span())?;
+ let rendered = render_str_total(
+     &eval(val_arg, env, sym)?.value_owned(),
+     sym.types().map(|a| a.as_ref()),
+ );
```

Note it is now infallible — no `?`. If that leaves `OP` or a span binding unused in this scope,
that is expected; clean it up only as far as clippy requires.

### 2 — Delete `render_unquoted` entirely

Not deprecate, not widen, not leave it unused. **Delete the function.** Its `op` and `span`
parameters existed only to build a `TypeMismatch` that can no longer occur. It is `pub`, and its
census is exactly two lines — the definition and the one call site you just changed
(`grep -rn 'render_unquoted' --include=*.rs .`). Nothing outside `src/` uses it.

### 3 — Confirm the checker's comment now reads true

`check.rs:13930-13935` says the intrinsic renders non-String values unquoted at runtime. After move
1 that is true. Read it and confirm; **only reword if it names something that no longer exists.**
Do not rewrite it for style.

## The gate

| # | assertion |
|---|---|
| 0 | ★ **NON-VACUITY, AND YOU DO THIS FIRST.** Write the kept test (row 2 below) **before** touching `src/`, run it, and watch it **FAIL with the raise**. Capture that failure **verbatim**. If it passes before your change, STOP and report — the stone is already done and something else is going on |
| 1 | after the change, that same test **passes**: the record renders with **named fields** (`{:x 1}`, not `{:field-0 1}`) |
| 2 | ★ `(interpolate "hello {name}" :name "world")` → `hello world` — **BARE**. The String arm's control, and the proof nothing started re-quoting |
| 3 | `grep -rn 'render_unquoted' --include=*.rs .` → **0**. The function is GONE, not merely unused |
| 4 | `render_str_total` now has **three** callers: `eval_str`, `eval_string_join`, `eval_string_interpolate` |
| 5 | all **156** existing `interpolate` call sites green — the floor + the `wat-scripts` loader gate cover this; name how you confirmed it |
| 6 | `check.rs:13930-13935`'s comment reads true as written |
| 7 | floor GREEN via `scripts/floor.sh` — read the **Summary line**, never a piped exit code |
| 8 | `cargo clippy --release --all-targets` → **0** |
| 9 | `grep -rnE '^[[:space:]]*#\[ignore' tests/ src/ crates/ benches/ --include=*.rs \| wc -l` → **13** |

Row 0 is the load-bearing row. Without it, rows 1 and 2 could both pass on a stone that changed
nothing. Row 3 is what separates this from a patch.

## Where the kept test goes

`wat-tests/interpolate.wat` is the existing home — an arc-284 `deftest` whose header already says
*"Runtime interpolation: named slots, unquoted render (String/i64), `{{ }}` escape."* Add rows there
in the same shape. You need a record to interpolate: define one with `defrecord` in the fixture and
pass an instance as a kwarg value.

Keep it. Do not put it in `wat-scripts/scratch-pad/` and delete it afterwards.

## STOP triggers — ship nothing on that axis; report and stop

- **STOP-0 — row 0 passes before your change.** Do not proceed. Report what you ran and what it
  printed.
- **STOP-1 — row 2 renders `"world"` with quotes.** Then the String arm is being bypassed. Do not
  special-case strings downstream to fix it — find out why the door's first arm did not fire. If the
  door is being called correctly and it still quotes, that is a finding about `render_str_total`,
  which `str` and `join` also depend on, and it is much bigger than this stone.
- **STOP-2 — deleting `render_unquoted` breaks a caller you did not expect.** The census says two
  lines. If it says more when you run it, the census was wrong: name every site and stop.
- **STOP-3 — an existing `interpolate` site changes its output.** All 156 pass strings today, so
  none should move. If one does, capture it verbatim — it means something is reaching `interpolate`
  that is not a string, and that site's behaviour was silently relying on the raise.
- **STOP-4 — the `#[ignore]` count moves off 13.**
- **STOP-5 — an unintended red. Do NOT re-run.** `scripts/floor.sh` keeps the untruncated log at
  `.floor/latest/`. Copy the failing test's **entire** stdout+stderr **verbatim** — never a summary,
  never a `| head`/`| tail` window — and name the exact assertion or match arm that fired. **There
  is no such thing as a known flake.**

⚠ **On goldens specifically:** if a `.edn` golden under `tests/diagnostics/` fails because a
**line number inside `src/*.rs` shifted**, that is not an adjacent file you must leave alone — a
golden encodes an observation, and updating it to the new observation **is** the work. Update it,
and say in your report exactly which goldens you changed and by how much. Anything else that goes
red is a STOP-5.

## Out of scope — affirmative cuts

- **`str` vs `show` semantics.** Settled: `render_unquoted` already renders a top-level `String`
  bare and `render_str_total`'s first arm is identical. Preserved exactly; only the tail widens.
- **Teaching the checker to track partiality in general.** Task #64, a language-scale question.
- **The `format`/`interpolate` duplication** — two state machines parsing one grammar, one in wat
  (`wat/core.wat:1448`) and one in Rust. Real, recorded in the DESIGN-STONE, and a purity-gate
  question, not a rendering one.
- **The 156 call sites.** They all pass strings today. No migration.
