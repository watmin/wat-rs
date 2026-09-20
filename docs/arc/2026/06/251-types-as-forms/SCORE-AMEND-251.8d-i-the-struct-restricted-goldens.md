# SCORE — AMEND 251.8d-i: three stale goldens + two stale doc comments

Folded into the stone. **No follow-up commit.** Stone is now `a83eae411` (was `22118e145`).
AMEND brief remains `66f9864d9` on top. **Not pushed.** Floor / workspace clippy / census:
orchestrator's row.

## The red

Three `wat::types struct_restricted` goldens, all the same diagnostic sentence. Structural
fields (`:callee`, `:enclosing-fn`, `:prefixes`, `:location`) were already right. Did **not**
revert `src/check/error.rs:721`.

## Regen, not a hand-typed sentence

Path: `UPDATE_EDN=1 cargo test --release -p wat --test types struct_restricted`
(`assert_edn_matches_file!` in `src/lib.rs` — `UPDATE_EDN` writes pretty EDN after parse).
Three files, one line each. Re-ran without `UPDATE_EDN`: **9 passed**.

## The two comments

`tests/types/struct_restricted.rs` and `tests/kernel/wat_arc198_def_restricted.rs` now say
the `/` discriminator. The kernel file's tests were already green; only a reader would have
noticed.

## Walls I ran (not the floor)

- `cargo test --release --offline -p wat --test types struct_restricted` — 9 passed.
- `cargo clippy --release --all-targets -p wat --offline -- -D warnings` — exit 0.

Floor + workspace clippy + `census.sh --diff`: **not run** (brief: orchestrator, uncontended).
Do not push. Do not start 8d-ii.

---

# ORCHESTRATOR'S WEIGH — independent re-run, 2026-09-20

| row | result |
|---|---|
| ⭐ **THE GATE — corpus census, MY independent run** | ✅ **2145 / 2145 OK — 0 A · 0 B · 0 C · 0 other** |
| idempotence | ✅ 60-file spread of already-converted copies, **second-pass-changed = 0** |
| ⭐ **ZERO tracked `.wat` CONVERSIONS** | ✅ 4 files **A**dded (new fixtures) + 1 **M**odified (`wat/fix.wat`, the engine). No corpus file rewritten. |
| `scripts/floor.sh` | ✅ **5930/5930 passed** (8 slow), exit 0 — **+6, exactly predicted** |
| clippy `-D warnings --all-targets --workspace` | ✅ **0** |
| `census.sh --diff` | ✅ `no STOP-8` |
| fold is INSIDE the stone | ✅ no commit after `a83eae411` touches the goldens, the doc comments, or `check/error.rs` |
| message NOT reverted | ✅ `check/error.rs` keeps the `/` sentence |
| **old prose fully retired** | ✅ `entry ending in` / `without trailing` — **0 hits tree-wide** |
| goldens carry the new sentence | ✅ 3 of 3 |
| goldens **regenerated**, not hand-typed | ✅ `UPDATE_EDN=1 cargo test … --test types struct_restricted`, then re-run clean |
| no published history rewritten | ✅ `origin/main` ancestor, 0 `refs/original` |

## Behaviour re-verified on the built binary, not argued

| probe | result |
|---|---|
| prefix `my.kernel` admits `my.kernel.ThreadOpts/spawn-runner` | ✅ **the counterpart's catch** — the brief's rule would have red the stdlib |
| prefix `my.kernel` denies `my.kernelish/sneaky` | ✅ `whitelist [my.kernel]` shown — no over-match |
| exact `my.kernel/specific-caller` admits its named caller | ✅ |
| entry that is neither keyword nor symbol | ✅ `MalformedForm: ":restricted-to entries must be keywords or symbols"` — the silent `filter_map` drop is dead |

## ⭐ THE BRIEF WAS WRONG TWICE; THE COUNTERPART CAUGHT BOTH

1. **The prefix discriminator.** The brief said *"contains no `/`"*. That is **incomplete**: a prefix
   must admit `ns/fn` **and** `ns.Type/method`, because the old keyword `:wat::spawn::` was a
   character prefix of both. Matching only `entry + "/"` **reds the stdlib**
   (`ThreadOpts/spawn-runner`). Landed as `entry/` **or** `entry.`.
2. **The test surface.** The brief told it to unify the `defn` and `defstruct` paths, then named the
   test selector of only the `defn` one. Three `types::struct_restricted` reds followed. **The
   orchestrator's defect, not the executor's** — its own walls were correctly chosen for the surface
   it was given.

## ⚠ ORCHESTRATOR PROCESS FAULT, RECORDED

Verifying idempotence, the orchestrator ran `pkill -f census8di` to stop its own background census —
**the pattern matched the shell running it**, killing both jobs (exit 144). No damage: the first-pass
census had already completed and printed `DONE 2145`, and the tree was clean. But it destroyed the
in-flight second pass, which is why idempotence is a **60-file sample** here rather than the full
2,145. `[[feedback_never_decide_on_pgrep_f]]` is the standing lesson and it was about matching, not
killing — **the same defect wearing a different verb.**

**VERDICT: ACCEPTED.** 8d-i is done: the codemod is TOTAL and the corpus has not moved.
