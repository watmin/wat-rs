# BRIEF — 294.l · the float sentinels go home

**You are a rider, not the orchestrator. Ending your turn ENDS you** — nothing wakes you, no
notification is coming. Run every verification in the **FOREGROUND** and block on it.

Work in `/home/watmin/work/holon/wat-rs/`. **Do not commit, push, stash, or revert.** Leave work in
the tree.

## Read first

`DESIGN-STONE-294.l-the-float-sentinels-go-home.md` — including the **`⛔ CORRECTION` at the end**,
which replaces gate row 5. For the shape of a finished job, `BRIEF-294.k-a-fabricated-home-is-a-lie.md`
and its strike (`62807e37`).

## The work

```
#wat-edn.float/nan       nil   →   #wat.core.f64/NaN    []
#wat-edn.float/inf       nil   →   #wat.core.f64/+Inf   []
#wat-edn.float/neg-inf   nil   →   #wat.core.f64/-Inf   []
```

**Two things change: the namespace/name, AND the body.** The body is not cosmetic — see below.

## Rooms

1. **`crates/wat-edn/src/writer.rs:248-254`** — the three emits. Namespace, names, and `nil` → `[]`.
2. **`crates/wat-edn/src/parser.rs:361-365`** — the intercept (`if ns == "wat-edn.float"`) and its
   three name arms. Must accept the new namespace, the new names, and a **vector** body.
3. **`crates/wat-edn/src/lexer.rs`** — grep-positive for the sentinel; check whether it participates.

## ★ Why the body changes too — this is the half that isn't a rename

**MEASURED:** reading `#wat.core.f64/-Inf nil` through the substrate gives

```
unsupported substrate tag … has a bare-nil body — retired (arc 278 A.0); unit variants are `#tag []`
```

**Today's `nil` body violates arc 278 A.0** and survives only because `parser.rs:361` intercepts the
namespace before the substrate ever sees it. You are removing a grandfathered special case that hides
a shape the rest of the corpus is forbidden to write. Keep the intercept (wat-edn has no type
registry and must produce `Value::Float` standalone) — but emit the **legal** body.

## `-Inf` and `+Inf` are legal names — already verified, do not re-derive

`vocab.rs:202` (`validate_first_char`) permits a leading `-`/`+`/`.` when the second character is not
a digit. Both probe runs recorded in the stone read the names without complaint; every failure was
downstream and about the body. If the lexer surprises you here, that is **STOP-1**.

## The gate

| # | assertion |
|---|---|
| 1 | `grep -rn 'wat-edn\.float' src/ crates/` → **0** |
| 2 | writer emits `#wat.core.f64/NaN []`, `#wat.core.f64/+Inf []`, `#wat.core.f64/-Inf []` |
| 3 | parser reads all three back to `f64::NAN` / `f64::INFINITY` / `f64::NEG_INFINITY` |
| 4 | **round-trip**: `write(parse(write(x))) == write(x)` for all three **and** a finite float |
| 5 | `crates/wat-edn/tests/{spec_strict,comprehensive}.rs` green — **and each carries an assertion naming the NEW spelling.** If neither mentions it after the strike, the coverage went vacuous under your own change and you must add it |
| 6 | floor GREEN via `scripts/floor.sh` — the **Summary line**, never a piped exit code |
| 7 | `cargo clippy --release --all-targets` → **0** |
| 8 | `grep -rnE '^[[:space:]]*#\[ignore' tests/ src/ crates/ benches/ --include=*.rs \| wc -l` → **13** |

Row 5 is the load-bearing one, and its second clause is the point: a rename that leaves the test suite
no longer mentioning the construct has **removed coverage while staying green**.
`[[feedback_a_green_test_can_prove_nothing]]`

## What you report

- the `git diff` of writer + parser
- the **measured wire string** for NaN, +Inf, -Inf — verbatim
- the round-trip result for each, and for a finite float (the non-vacuity control: if finite floats
  broke, the sentinel rows would be meaningless)
- which assertions in `spec_strict`/`comprehensive` name the new spelling — quote them
- floor Summary verbatim; clippy count; `#[ignore]` count
- honest deltas

## STOP triggers — ship nothing on that axis; report and stop.

- **STOP-1 — the lexer rejects `-Inf` or `+Inf`** in tag position despite `validate_first_char`
  permitting them. Then the validator and the lexer disagree, which is a finding about *them*, not
  about this stone. Capture the error verbatim and stop.
- **STOP-2 — the `[]` body cannot be produced or read** without touching machinery outside
  `crates/wat-edn/`. Name the site and stop; do not push a change through the substrate for this.
- **STOP-3 — the `#[ignore]` count moves off 13.** A finding about this brief, not a step.
- **STOP-4 — an unintended red. Do NOT re-run.** A re-run that goes green destroys the only evidence.
  `scripts/floor.sh` keeps the untruncated log at `.floor/latest/`. Copy the failing test's **entire**
  stdout+stderr **verbatim** — never a summary, never a `| head`/`| tail` window — and name the exact
  assertion or match arm that fired. There is no such thing as a known flake.

## Out of scope — do NOT touch

- **Core and rete f64 semantics.** `:wat::core::f64::{+,-,*,/}` keep raw IEEE and `total: false`; the
  rete `OpClass::Fallback` rows keep their `:undefined` shape. This stone changes **only how a
  non-finite f64 is spelled in EDN text.**
- **`crates/wat-edn/interop-tests/`** — a separate Cargo project, not in the workspace, needing
  `clojure` on `$PATH`, and **measured not to exercise NaN/±Inf at all.** Running it proves nothing
  about this change. Do not add it to the gate; do not try to satisfy it.
- **`wat-edn.cap`** — 294.m, a security boundary, the builder's ruling.
