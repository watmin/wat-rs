# EXPECTATIONS — STONE 255.1c-kernel-stdio

Written **before** the strike.

## Scorecard

| # | what | the command that checks it | expected |
|---|---|---|---|
| 1 | the home is linked | read `src/intrinsic/mod.rs` | `mod kernel_stdio;` beside `mod time;` — without it, submissions never link and the registry stays empty |
| 2 | **★ THE FIRST EFFECTFUL ROWS** | `(:wat::runtime::metadata-of :wat::kernel::println)` on the built binary | `:purity` reads **`Effectful`**, actual output pasted, plus one `Pure` row for contrast |
| 3 | **the cross-check AGREED, unedited** | `cargo test --release pure_declared_matches_is_effectful_op` | green — **and neither the declaration nor `is_effectful_op` was changed to make it green** (`git diff` proves it: `runtime.rs:25164` untouched) |
| 4 | all six answer | `metadata-of` on each of the six | `Some[{…}]` six times, never `None` (a `None` = registered under the wrong fqdn) |
| 5 | arms gone | `grep -cE '^\s+":wat::kernel::(println\|pprintln\|eprintln\|epprintln\|readln.\|read-frame)" *=>' src/runtime.rs` | **0** — anchored on `=>`, not a bare substring |
| 6 | no new enums | `git diff src/intrinsic/mod.rs` | only the `mod kernel_stdio;` line — no new `Category`/`Purity`/`Arity` variant |
| 7 | not dead code | `cargo clippy --release --all-targets` | zero warnings, no `#[allow(dead_code)]` in the new module |
| 8 | bodies MOVED, not rewritten | read the diff | each body is its `services/` original modulo the signature shim; any logic change is a finding |
| 9 | **stdio still routes the same way** | the hermetic/stdio tests in the floor | no change in where output lands — same fn, reached differently |
| 10 | build | `cargo build --release` | exit 0 |
| 11 | blast radius | `git diff --stat` | `intrinsic/kernel_stdio.rs` (new) · `intrinsic/mod.rs` · `runtime.rs` · `services/` only |
| 12 | **floor** | orchestrator's own `scripts/floor.sh` | zero new failures vs **4399 passed / 263 skipped**; a changed count either way is a finding |

**Row 2 is the stone** — 48 `Pure` / 2 `Preserving` / **0 `Effectful`** is a column that has never
been able to be wrong. **Row 3 is its twin, and it is the harder one**: the cross-check going green
is worthless if either side was nudged to agree. The value is two independently-derived answers
meeting, and the `git diff` is what proves they did.

## Runtime prediction

**30–50 minutes.** Six verbs, one body module, an exact template. The cost is in classifying purity
and determinism honestly per verb and in the doc prose — not in code. Predicted overrun: STOP-4, the
service-tier routing turning out to be less transparent than "same fn, different door."

Time-box: 100 minutes.

## Trap doors — named in advance

- **★ Editing the declaration until the cross-check passes.** The single way this stone can produce
  a green that means nothing. If `is_effectful_op` and a body-derived declaration disagree, that
  disagreement IS the finding — the first real one the column has ever been able to produce.
- **Looking for the bodies in `src/kernel/`.** They are not there. `src/kernel/` holds
  `address.rs`/`listener.rs`/`peer.rs`/`spawn.rs` — concurrency and networking. stdio lives in
  `crate::services::`. Home #2's brief made the mirror-image mistake and it cost a delta.
- **Maintainer rationale in the `///` block.** `render-doc` prints that block to users. It shipped
  once today and the byte-identical goldens caught it. Rationale goes in `//`.
- **Scope creep into the rest of kernel.** 49 arms are in that namespace and 43 of them are not this
  stone. Concurrency, networking, signals, errors, handles, capability — each is its own stone.
- **A bare-substring arm grep.** `grep -c '":wat::kernel::println'` also matches error text and type
  names. Row 5 anchors on `=>` because that imprecision already cost one delta.
- **Forgetting `mod kernel_stdio;`.** Everything compiles, clippy is clean, and the registry is
  empty. Rows 1 and 2 both catch it; row 10 alone would not.

## What this stone does NOT claim

It carves six of `:wat::kernel::`'s 49 arms and leaves 43. It does not close the soundness hole —
the blanket-accept at `resolve/walk.rs:257` is untouched and `255.1b-iv` is still ahead. It un-ignores
none of the nine gates. It does not touch `is_effectful_op`, `rete/purity.rs`, or the `core::i64`
hot path.

The honest claim: **six stdio verbs are nameable, queryable and reflectable; their arms are gone from
the central match; and for the first time the registry carries rows that are not `Pure` — checked
against an independent derivation that could have disagreed.**
