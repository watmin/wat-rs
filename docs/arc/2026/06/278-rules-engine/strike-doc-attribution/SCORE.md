# SCORE — E3, weighed against the orchestrator's own re-run. **CLASS E IS CLOSED.**

> Re-run here at `76e221bbb`.

| # | pre-value | after |
|---|---|---|
| 1 | 3 blocks on one variant; 2 with none | ✅ four variants, one block each — **verified in rendered HTML**, not source |
| 2 | "attached to a different failure" | ⚠ **the ROW was wrong** — see thin spot B |
| 3 | `signal.rs` 9 unresolved | ✅ **0** |
| 4 | nothing runs rustdoc; lint not enabled | ✅ `tests/lint/no_new_broken_doc_link.rs` |
| 5 | — | ✅ 34 **named** `(file, target, sites)` keys, no bare count |
| 6 | — | ✅ **six** arms driven, incl. a genuinely held cargo lock |
| 7 | 50 tree-wide | ✅ **41** |
| 8 | — | ✅ two files |
| 9 | lint 116/116 | ✅ **119/119** |
| 10 | floor 5230/5230 | ✅ `Summary [ 392.687s] 5233 tests run: 5233 passed (1 slow), 21 skipped`, zero FAIL rows |
| 11 | clippy rc=0 | ✅ rc=0 |

## ⛔ Where MY brief was thin — and the first is the one that stings

- **A. ★★ MY KEY SHAPE REOPENED THE HOLE I HAD JUST WARNED ABOUT.** DESIGN's ⚠ cites `purity.rs` —
  *"the gate wanted SET MEMBERSHIP and measured CARDINALITY"* — and then the sketch prescribes a
  `(file, target)` key. **7 of the 34 keys have two sites in one file**, so fixing one of
  `parser.rs`'s two `parse_all` links would have left the gate green: the same defect, one level
  down, inside the cure for it. The rider added a per-key site count and drove it. Its defense is
  correct and worth keeping: **a count scoped inside a name still names the offender**, which is
  precisely what a global count cannot do.
- **B. Row 2's pre-value was measured on text that is not in the file.** The *"names an action the
  author can take"* sentence lives at `outcome.rs:226` and was **correctly placed all along**. What
  was misattached was block 2, the terminate prose. **The work-list row's claim was wrong**, and it
  is the third Class E row this session whose detail did not survive an audit.
- **C. My trap named the wrong risk.** I worried about *duration*; the gate costs ~0 against a
  131.9s lint binary. The hazard is the **nested target-directory lock** — the gate spawns
  `cargo doc` while nextest runs, and an unbounded spawn **blocks** rather than fails. I sent it
  back; the rider drove the real thing by `flock`ing `target/release/.cargo-lock` and quoting
  cargo's own `Blocking waiting for file lock` line in the red.
- **D. Nobody specified the doc build's profile.** Dev inside a release floor would compile a second
  full dependency graph on a machine that only builds release. The gate derives `--release` from its
  own executable path rather than `cfg!(debug_assertions)`.
- **E. `--workspace` is wider than `default-members`** — it pulls `crates/wat-source-derive`, which
  `cargo build` here does not. Zero links today; noted so a future arrival is not a surprise.

## The ruling I owed, recorded at the constant

The bound turns a hang into a red but does not make the gate correct under contention: a lock held
past 300s reds a clean tree. The rider raised that against *"there is no such thing as a known
flake"* and asked for a ruling rather than assuming one. **Ruled: keep the bound, do not give the
doc build its own `CARGO_TARGET_DIR`** —

1. red-when-it-cannot-measure is the **correct** answer; the alternative is the recorded failure of
   a check reporting success without running;
2. it is **not a flake in the doctrine's sense** — a flake fails for *unknown* reasons, which is why
   re-running destroys evidence. This one states and captures its cause, so resolving that cause and
   re-measuring is `extirpare`, not re-run-until-green;
3. the structural cure buys out a condition the operating discipline **already forbids**, at the
   price of a cold dependency compile on every fresh clone.

## What the rider found that neither of us predicted

**Only two of `signal.rs`'s nine broken links were E4's.** Seven were older — 3× `SymbolTable`,
`set_source_loader`, `set_macro_registry`, `EncodingCtx`, `HashError`. The file had been citing four
unreachable items before my strike ever touched it.

And it caught **a broken citation inside its own red message**: cargo prints *"file lock on
**artifact** directory"*, and it had written *"build directory"*. A gate about unverifiable citations
must not ship one.

## Arms not driven, named

`cargo doc` non-zero-exit (needs a broken workspace build); the wrapper's 125/126/127 arm (needs a
fabricated compile-time `env!("CARGO")`); the duplicate-key ledger test (passed on the real ledger);
`profile_flag()`'s debug branch (every run here was release). Each named with its reason.
