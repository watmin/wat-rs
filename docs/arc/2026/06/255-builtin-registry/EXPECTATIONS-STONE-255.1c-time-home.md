# EXPECTATIONS — STONE 255.1c-time

Written **before** the strike.

## Scorecard

| # | what | the command that checks it | expected |
|---|---|---|---|
| 1 | the home exists and is linked | read `src/intrinsic/mod.rs` | `mod time;` declared beside `mod bytes;` — without it the submissions never link |
| 2 | **the split is REAL** | `(:wat::runtime::metadata-of :wat::time::now)` and `(… :wat::time::to-iso8601)` on the built binary | `:determinism` reads **`Nondeterministic`** for one and **`Deterministic`** for the other, **actual output pasted** |
| 3 | the arms are GONE | `grep -c '":wat::time::' src/runtime.rs` | **0** dispatch arms remain (the `5939–6016` block is deleted) |
| 4 | every name still answers | `(:wat::runtime::metadata-of …)` on a sample across the family | `Some[{…}]`, never `None` — a `None` means the name registered under the wrong fqdn |
| 5 | no new enums | `git diff src/intrinsic/mod.rs` | only the `mod time;` line — **no new `Purity`/`Arity`/`Category` variant** |
| 6 | not dead code | `cargo clippy --release --all-targets` | zero warnings **and no `#[allow(dead_code)]`** |
| 7 | bodies MOVED, not rewritten | read the diff | each handler body is its `runtime.rs` original modulo the signature shim; **any logic change is a finding** |
| 8 | build | `cargo build --release` | exit 0 |
| 9 | blast radius | `git diff --stat` | `intrinsic/time.rs` (new) · `intrinsic/mod.rs` (1 line) · `runtime.rs` (deletions). **Nothing else.** |
| 10 | **floor** | orchestrator's own `scripts/floor.sh` | zero new failures vs the pre-strike baseline; a changed count **either way** is a finding |

**Row 2 is the stone.** Home #1 registered four rows that were all `Pure`+`Deterministic` — a set
that cannot falsify the metadata contract. If both sides of `time` report the same determinism, this
stone bought a file move and nothing else. **Row 3 is its shadow:** if the arms are still in
`runtime.rs`, the names are registered *and* still dispatched by the old path — two sources, which is
the exact asymmetry the arc exists to remove.

## Runtime prediction

**45–70 minutes.** 41 handlers is real volume, but the work is uniform and the template is exact; the
cost is in classifying each row's two axes honestly and in the doc-comment prose, not in logic.
Predicted overrun: STOP-2 (a category with no home) or a handler that will not move cleanly.

Time-box: 140 minutes.

## Trap doors — named in advance

- **Classifying from the list in the brief instead of from the body.** The brief's split is
  orientation. A `*-ago` helper that turns out to be pure arithmetic over a *passed-in* instant is
  `Deterministic`, whatever its name suggests. The name is not the evidence.
- **Minting a `RuntimeCategory` variant to make a row compile.** That is STOP-2 wearing a
  convenience's clothes — the category set is a closed domain and widening it is a ruling.
- **"Improving" a handler while moving it.** The carve is observationally inert. A clearer error
  message, a tightened type check, a removed clone — each is a behaviour change smuggled into a move,
  and it makes the floor's verdict unreadable.
- **Forgetting `mod time;`.** Everything compiles, clippy is clean, and the registry is empty —
  because `inventory::submit!` only links what is declared. Row 1 and row 2 both catch it; row 8
  alone would not.
- **Deleting the arms without checking for a second caller.** A `time` handler fn may be called from
  somewhere other than its dispatch arm. The compiler catches a *removed* fn; it does not catch a fn
  left behind with one caller gone. Row 6 (`dead_code`) is the net.

## What this stone does NOT claim

It does not close the soundness hole — the blanket-accept at `resolve/walk.rs:257` is untouched and
`255.1b-iv` is still ahead. It un-ignores none of the nine gates. It does not carve `core::i64` or
`core::f64` and takes **no** position on hot-path perf (the arithmetic arms dispatch at
`runtime.rs:5036`, above the registry guard at `:5608`, and this stone does not move them). It does
not make `rete/purity.rs` a projection — it may only *reveal* that the two disagree (STOP-3).

The honest claim is: **one more family is nameable, queryable and reflectable; its 41 arms are gone
from the central match; and for the first time the registry carries a row that is not
`Pure`+`Deterministic`.** Any report claiming more than that is overclaiming.
