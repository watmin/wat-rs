# BRIEF — 294.j · `edn_shim` forgets the algebra

**You are a rider, not the orchestrator. Ending your turn ENDS you** — nothing wakes you, no
notification is coming. Run every verification in the **FOREGROUND** and block on it: your turn ends
when the numbers are in your hands, not when a command is launched.

Work in `/home/watmin/work/holon/wat-rs/`. **Do not commit, push, stash, or revert.** Leave the work
in the tree; the orchestrator weighs and commits.

Read `DESIGN-STONE-294.j-the-shim-forgets-the-algebra.md` (sibling) in full first — it carries the
builder's ruling, the classification, and the measurements this brief rests on. For the shape of a
finished job, read `BRIEF-294.i-opaque-the-death-warrant.md` and its strike (`df6e2e91`).

## The model, in one line — the builder's, and it is the design

> *"`#holon {:a "b"}` represents two atoms, bound together."*

The data is the wire form. `Bundle`/`Bind`/`Atom` is **derived** from it, deterministically, so a
receiver rebuilds the same structure. **The algebra never crosses a wire.** Your job is to stop
`edn_shim` writing it down.

## The cure already exists, and it is total

```rust
holon_to_watast(&HolonAST) -> WatAST        // runtime.rs:20625 — total, no panic/unwrap/expect
crate::wat_edn_bridge::watast_to_edn(&WatAST) -> OwnedValue   // the WatAST↔EDN bijection
```

`holon_to_watast` handles **every** variant — Thermometer and SlotMarker included — lowering each to
the wat source form that constructs it. It is live at 8 call sites in `runtime.rs` and **zero** in
`edn_shim.rs`. You are adopting it, not writing it.

The exemplar is **three lines above the site you are changing**:

```rust
// edn_shim.rs:3728 — WatAST already received this ruling:
Value::wat__WatAST(a) => crate::wat_edn_bridge::watast_to_edn(a.as_ref()),
```

## Rooms, in order — each with why you are being sent

1. **`src/edn_shim.rs:3731`** — `Value::holon__HolonAST(h) => holon_ast_to_edn(h)`. **The strike.**
   This one arm replaces sixteen tag arms.
2. **`src/edn_shim.rs:3937-4045`** — `holon_ast_to_edn`, the 16-arm encoder. It goes.
3. **`src/edn_shim.rs:4062-4230`** — `edn_to_holon_ast` / `edn_to_holon_ast_natural` /
   `edn_holon_tag_to_ast`. The reader trio. **They collapse** — see "the fork is vestigial" below.
4. **`src/edn_shim.rs:2870`** — `if ns == "wat-edn.holon" { … }`, the decode dispatch.
5. **`src/edn_shim.rs:2094-2110`** — the `":wat::holon::HolonAST"` coercion arm and its mode selector
   `tag.namespace() == "wat-edn.holon"`. **A namespace-string comparison selecting a decode mode** —
   after the strike both arms call the same reader, so the selector has nothing to select.
6. **`src/edn_shim.rs:4277-4302`** + **`src/lib.rs:138`** — the three `pub` read/write fns and their
   export line.

## The work

### 1 — the encode arm

```rust
Value::holon__HolonAST(h) => crate::wat_edn_bridge::watast_to_edn(
    &crate::runtime::holon_to_watast(h)
),
```

Exact pathing is yours to resolve — `holon_to_watast` is currently private to `runtime.rs`; making it
`pub(crate)` is in scope. Delete `holon_ast_to_edn` and every `Tag::ns("wat-edn.holon", …)` with it.

### 2 — the reader collapses to ONE

**MEASURED** (`tests/value/probe_arc294_holon_bare_leaf_read.rs`, in the tree, run 2026-08-16):

```
strict  · bare leaf inside a composite    FAIL   ← edn_shim.rs:4068
strict  · tagged leaf inside a composite  PASS   ← control: the harness works
natural · bare leaf inside a composite    FAIL   ← edn_shim.rs:4068, the SAME line
natural · bare leaf at top level          PASS   ← natural tolerance does exist
```

Both reds are one line. `edn_holon_tag_to_ast`'s composite arms recurse through the **strict** reader
unconditionally, so "natural" is top-level-only. The fork exists **only** to compensate for
leaf-wrapping; with the wrapping gone there is one reader. Collapse them and drop the mode selector.

### 3 — the tag is DEAD, not dormant

A decoder that still accepts `#wat-edn.holon/String "x"` leaves the tag dormant, and dormant is how
`.opaque` survived long enough to need a death warrant. **Both halves go**, and gate 5 pins it: that
form must be **refused** after the strike.

### 4 — Thermometer and SlotMarker survive as VERBS

`holon_to_watast` already emits `(:wat::holon::Thermometer v min max)` and
`(:wat::holon::SlotMarker min max)`. You add nothing; verify they render and note what they render to.

### 5 — the goldens

Three files, all `tests/value/wat_arc221b_keyword_dispatcher_completeness__*.edn`. They **will**
change. Regenerate, then **read the diff** and confirm each new value is plain EDN.

### 6 — the probe ships GREEN

`tests/value/probe_arc294_holon_bare_leaf_read.rs` is in the tree, currently RED, **with no
`#[ignore]` and it must gain none**. Rewrite it to the post-strike spec — four rows, none of them
scaffolding:

| row | assertion |
|---|---|
| 1 | a bare leaf round-trips (top level) |
| 2 | a `#holon`-derived structure renders as plain EDN — no `wat-edn.holon` substring anywhere in the output |
| 3 | `#wat-edn.holon/String "x"` is **REFUSED** on decode — the negative control |
| 4 | Thermometer renders to its `(:wat::holon::Thermometer …)` call form — non-vacuity for row 3 |

Row 3's refusal must be checked by **direction plus a positive control** (row 4 proving the decoder
still works), never by matching an error's message text.

## The gate

| # | assertion |
|---|---|
| 1 | `grep -rn 'wat-edn\.holon' src/ crates/ tests/ wat/ wat-scripts/ wat-tests/` → **0** |
| 2 | the seven named fns are gone, and `lib.rs:138`'s export list with them |
| 3 | `Value::holon__HolonAST` renders through `holon_to_watast` + `watast_to_edn` — one arm |
| 4 | the coercion arm has one reader, no mode selector |
| 5 | `#wat-edn.holon/String "x"` refused on decode |
| 6 | Thermometer + SlotMarker render to their call forms |
| 7 | 3 goldens regenerated, diff read, every new value plain EDN |
| 8 | the probe is GREEN with **zero `#[ignore]`** |
| 9 | floor GREEN via `scripts/floor.sh` — read the **Summary line**, never a piped exit code |
| 10 | `cargo clippy --release --all-targets` → **0** |
| 11 | `grep -rnE '^[[:space:]]*#\[ignore' tests/ src/ crates/ benches/ --include=*.rs \| wc -l` → **13**, unchanged |

## What you report

- The full `git diff` of `src/edn_shim.rs`, or every hunk that is not a straight deletion.
- The final `grep -rn 'wat-edn\.holon'` output, verbatim (expected: empty).
- The three golden diffs, before/after, verbatim.
- Floor Summary verbatim; clippy count; the `#[ignore]` count.
- Any test whose expected text you changed, with the before/after. A test asserting
  `#wat-edn.holon/Bind [...]` legitimately becomes one asserting a plain form — **that is the strike
  working.** A test whose *body shape* changed is not; call it out.

## STOP triggers — rejection criteria. Ship nothing on these; report and stop.

- **STOP-1 — `holon_to_watast` + `watast_to_edn` is not total in practice.** The stone claims
  totality from a signature and a panic-audit. If a real value takes a path that panics or loses
  information, **name the value and the arm.** Do not add a fallback; the composition being partial
  changes the design.
- **STOP-2 — a directive must be read back OUT of `#holon`-shaped data.** The builder said
  *"`#wat.holon/Thermometer` is probably the correct name"* for a directive nested in data. This
  stone settles only the *rendering* layer. If you find a site needing the reader-tag spelling,
  that is the builder's ruling — **name the site and stop.**
- **STOP-3 — deleting a `pub` export breaks a consumer.** `read_holon_ast_tagged` /
  `read_holon_ast_natural` are exported at `lib.rs:138` with zero callers in `src/`, `crates/`, and
  `tests/` — measured. That measurement covers **this repo only.** Before deleting, run
  `grep -rn 'read_holon_ast' ../ --include=*.rs` (siblings included). **If anything outside this repo
  names them, KEEPING them is the correct answer** — say so and move on; do not contort the strike
  around it, and do not treat "delete" as an order that outranks what you find.
- **STOP-4 — the `#[ignore]` count moves off 13.** The waterline came down from 200+ over a day of
  deliberate work. If making something pass seems to require an ignore, that is a finding about this
  brief, not a step. **Report it; do not add the ignore.**
- **STOP-5 — a red you did not intend. Do NOT re-run.** `scripts/floor.sh` has already kept the
  untruncated, ANSI-stripped log at `.floor/latest/`. Copy the failing test's **entire** stdout and
  stderr block **verbatim** — never a summary, never a `| head`/`| tail` window — and name the exact
  assertion or match arm that fired. There is no such thing as a known flake; a red is a red.
