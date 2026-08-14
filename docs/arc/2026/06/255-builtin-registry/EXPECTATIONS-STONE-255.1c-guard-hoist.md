# EXPECTATIONS — STONE 255.1c-guard

Written **before** the strike.

## Scorecard

| # | what | the command that checks it | expected |
|---|---|---|---|
| 1 | **baseline captured FIRST** | the report's own ordering | a ns/op figure for `i64::+` dispatch **taken before the hoist**, best-of-≥5, spread stated |
| 2 | the hoist is where it belongs | read the diff | an `if let Some(handler) = …lookup(head)` **above `match head {`**, not as its first arm |
| 3 | the old guard arm is GONE | `grep -n "registry().lookup" src/runtime.rs` | the `h if …lookup(h).is_some()` arm deleted; **one** consult point, not two (the old arm looked up twice) |
| 4 | **post-hoist number** | the same harness, same session | a second ns/op figure, best-of-≥5, spread stated |
| 5 | **★ REGISTERED NOW WINS** | insert a literal arm for a registered name above the match, build, run, revert | returns **`"ff"`**, not `"SHADOWED"` — the inverse of the HEAD differential |
| 6 | the loop was not optimised away | the report | an explicit sentence on how the work was kept alive, plus a figure that is not implausibly small |
| 7 | inert | `git diff` | no behaviour change; the harness is `#[ignore]`d |
| 8 | build | `cargo build --release` | exit 0 |
| 9 | clippy | `cargo clippy --release --all-targets` | zero warnings, no new `#[allow]` |
| 10 | blast radius | `git diff --stat` | `src/runtime.rs` **only** |
| 11 | **floor** | orchestrator's own `scripts/floor.sh` | zero new failures vs **4399/4399**; a changed count either way is a finding |

**Rows 1 and 4 together are the stone** — the pair, in that order, is the entire deliverable. **Row 5
is its twin**: without it, "registered wins" is an assertion; the differential at HEAD returns
`"SHADOWED"`, so the same experiment returning `"ff"` is the proof the hoist did what it claims.

## Runtime prediction

**35–60 minutes.** The code change is ~5 lines; nearly all the time is harness construction and
getting a stable number. Predicted overrun: STOP-2 — the harness cannot isolate dispatch from its
own setup cost.

Time-box: 120 minutes.

## What I predict the number will say — recorded now so I cannot rationalise later

A `HashMap<&'static str, _>` get with std SipHash over a ~20-character key is **on the order of
20–40 ns**. Interpreted wat dispatch per call is plausibly **hundreds of ns**. So I expect a
**measurable but not catastrophic** regression on `i64::+` — single-digit to low-tens percent.

**I am recording this because I might be wrong in either direction, and the number rules, not this
paragraph.** If it comes back a 3× regression, that is a real finding and the hoist may not be the
right shape. If it comes back indistinguishable from noise, that is equally a finding and the perf
gate the design has feared since June was cheaper than assumed. Either way the builder rules on a
measurement.

## Trap doors — named in advance

- **Measuring after the change and calling it a delta.** The single most likely way this stone
  produces a worthless result. Row 1 exists solely to force the ordering.
- **A stale binary.** A restored source with an unrebuilt binary reports the *previous* build. This
  exact error was made against this exact question earlier today, produced `"SHADOWED"` where the
  baseline should have read `"ff"`, and survived a full round before being caught. `cargo build
  --release` between every source change and every run.
- **The optimiser deleting the loop.** A bench that measures nothing reports a beautiful number.
  Row 6 exists for this.
- **One sample.** A shared machine's noise exceeds the effect being measured. Best-of-≥5, spread
  reported.
- **"Registered wins" asserted rather than shown.** Row 5. The HEAD differential is on record
  returning `"SHADOWED"`; the post-hoist run must return `"ff"` from the same experiment.
- **Scope creep into a carve.** The moment a family's arms start moving, this stone has become
  something else. Zero arms are deleted here except the guard itself.

## What this stone does NOT claim

It carves no family and registers no name. It does not touch the blanket-accept, the resolver, the
checker, `IntrinsicRegistry`, or any `.wat`. It does **not** change the hash function or introduce
`phf` — if the number argues for that, it is the builder's ruling on a measurement, not something
this stone pre-authorises.

The honest claim is: **the registry is now consulted before the literal table, a literal arm can no
longer shadow a registration, and here is exactly what that costs the arithmetic hot path.**
