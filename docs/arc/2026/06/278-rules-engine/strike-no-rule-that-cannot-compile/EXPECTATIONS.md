# EXPECTATIONS — no rule that cannot compile

Written **before** the strike, at `d7aa7c9ae`.

| what | command | expected |
|---|---|---|
| the gate exists and walks the corpus | `cargo nextest run --release -E 'test(compile)'` | the new gate runs; its walk covers **136** rule-declaring files |
| ⛔ **the gate is not vacuous** | plant a fence calling a partial op in one corpus file, re-run | **RED**, naming that file and the axis. Restore ⇒ green. Mutate the file the gate reads, never a copy |
| ⛔ **the gate has NO exemption category** | read the new file | no rune category, no `DECLARED_CATEGORIES`, no skip list. If one appeared, the contract decision was reversed |
| two instruments agree | `tests/lint/rete-compile-census.sh` | after the strike: **cannot compile: 0**. A disagreement between script and gate means one of them is wrong — say which |
| the nine are gone | `git status` / the delete list | 9 deletions, exactly the enumerated paths |
| no fence in the corpus calls a user fn | grep the two repaired files for `rete::where (:fix::` | **0** |
| ⭐ **the codemods RUN — the first executable proof they have ever had** | drive `:fix::convert` on a small input | it produces output. **Record that output in the SCORE**; it is the baseline that has never existed |
| STOP-2's equivalence is shown, not asserted | the SCORE | old function vs new fence expression, driven over the same inputs, agreeing — per input, in the SCORE |
| the citation is re-grounded | `src/runtime.rs:5364-5378` | cites something that executes, **or** states plainly that the traversal is unproven. Not deleted |
| floor | `./scripts/floor.sh`, foreground; `.floor/latest/clean.log` | **0 failed.** Passed **≥ 5493** plus the gate's tests. ⚠ Do not pin an exact total — deleting `.wat` files removes no tests (they are walked anonymously), and a count in a scorecard caps coverage downward |
| clippy | `cargo clippy --all-targets --release -- -D warnings` | rc=0 |

## Runtime prediction

**90–120 minutes.** Two floors dominate. The gate is the bulk of the work; the deletions are
minutes. STOP-2 is the variance — if the three fences cannot be shown equivalent, the strike ends
with nine deletions, a gate, and two files still red, which is a legitimate partial result.

## Trap doors

- ⛔ **Shelling out to the CLI binary.** `every_wat_bad_fixture_actually_fails`'s header says it in
  capitals — *"THE DRIVER IS THE WHOLE QUESTION — DO NOT MEASURE THIS WITH THE BINARY"* — and a
  strike was withdrawn for it once. `rete-compile-census.sh` uses the binary and pays for it with a
  `sed` on `:user::main` to stop codemods writing files. `startup_from_source` never evals `main`.
- ⛔ **Adding a `red-by-design` category "for future negatives."** That is the contract decision
  reversed. Zero members until one is earned.
- ⛔ **`rc` is not a verdict here.** A rete compile `AssertionFailure` still exits 0. The census
  script was wrong this way first.
- ⛔ **Assuming the fence repair is behaviour-preserving.** These codemods have never run; there is
  no "before" to diff. Equivalence must be driven, function against expression. See STOP-2.
- **Making `:fix::has-ns?` total instead of inlining at the fence.** Changes a general function's
  contract for a rete-only reason.
- **Deleting `src/runtime.rs`'s sentence.** The citation is void; the claim may not be.
- **Believing this brief.** The eleven were measured once, by one hand, at `d7aa7c9ae`, with an
  instrument that reported a confident **136/136 OK** before it was fixed. Re-run the census first.
  If the number moved, that is worth more than the strike.
