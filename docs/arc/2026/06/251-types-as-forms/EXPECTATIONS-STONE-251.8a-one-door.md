# EXPECTATIONS — STONE 251.8a

Written **before** the strike so the result cannot move the goalposts.

## Scorecard

| # | what | the command that checks it | expected |
|---|---|---|---|
| 1 | the door exists and is total | read `crates/wat-reader/src/identifier.rs` | `namespace()` returns `&str`, never an `Option`; `is_reference()` documented as `namespace() != BOUND_NAMESPACE` |
| 2 | **the door is not vacuous** | the new RED probe, mutated then restored | probe RED with `is_reference` broken; GREEN restored. **The mutation result is reported explicitly** |
| 3 | the four hand-rolls are gone | `grep -n "contains('/')" src/resolve/normalize.rs src/macros/expand.rs src/runtime.rs` | **0** |
| 4 | `$bound` is reserved | read `src/resolve/reserved.rs` | `":$bound::"` present in `RESERVED_PREFIXES`, doubled-colon form |
| 5 | `as_str()` is untouched | `git diff crates/wat-reader/src/identifier.rs` | no change to `as_str`'s body or signature |
| 6 | nothing else moved | `git diff --stat` | ≤ 6 files: `identifier.rs`, `reserved.rs`, the 3 files holding the 4 sites, + the probe |
| 7 | build | `cargo build --release` | exit 0 |
| 8 | lint | `cargo clippy --release --all-targets` | zero warnings |
| 9 | the discriminator probe still holds | `./target/release/wat wat-scripts/scratch-pad/probe-251-keyword-vs-colon-quoted-symbol.wat` | exit 0; `:foo`, `:my.app/status`, `:wat.core/+` |
| 10 | **the floor has not moved** | orchestrator's own `scripts/floor.sh`, Summary line read by hand | zero new failures — and a *changed count in either direction* is a finding, not a pass |

Row 2 is the load-bearing one. Rows 3 and 10 are the ones a plausible-but-wrong strike passes.

## Runtime prediction

**15–25 minutes.** Small surface (189-line file + one const list + four one-line call sites), no
new types, no signature changes. Predicted overrun cause, if any: STOP-1 — `namespace()` needing
something `Identifier` cannot see from `&self`.

Time-box: 50 minutes (2× upper bound).

## Trap doors — named in advance

- **The `$bound` string could be spelled two ways.** The namespace is `$bound`; the *reservation
  entry* is `":$bound::"` because `is_reserved_prefix` strips a leading `:` and matches
  doubled-colon prefixes. If those two spellings get conflated, the reservation silently matches
  nothing and row 4 passes while protecting nothing. **This is the vacuous-gate shape** — row 4 is
  a *read*, not a grep, for exactly this reason.
- **`is_reference()` vs `reference?`.** The intueri cast named the wat-surface verb `reference?`.
  Rust has no `?` in identifiers, so the Rust-side spelling is `is_reference`. That is a rendering,
  not a second decision — if a future wat-facing verb ships, it is `reference?`.
- **A fifth `contains('/')` site.** The design measured four. If a fifth exists (a different
  spelling, a `find('/')`, a `split('/')` used as a test), it is in scope and must be collapsed
  too — and the miscount is worth reporting, because "four" came from one grep and this arc has
  been bitten repeatedly by a pattern that could not reach the thing.
- **The probe could pass on a tautology.** Asserting `namespace(bare("x")) == "$bound"` when
  `namespace` is implemented as "return `$bound` if there is no `/`" is close to restating the
  implementation. The probe must also assert the *reference* direction on a real namespaced
  identifier, and the mutation in row 2 is what proves it can fail at all.

## What this stone does NOT claim

It does not implement symbols. The namespace is **derived**, not stored; `Identifier` still holds
one `name` string. The claim is exactly: *four hand-rolled string tests became one door with a
name, and the door's signature is the contract 8b will swap the storage behind.* Any report that
says more than that is overclaiming, and the design says so in its own 8a bullet.

It does not fix #95. The dotted call head remains unchecked until 251.8c.
