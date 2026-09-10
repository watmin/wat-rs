# BRIEF — no rule that cannot compile

## The work

Build a lint gate that requires every `wat-scripts/` file declaring rete rules to **compile** them,
then make it green: delete nine files whose rules cannot compile, repair three user-fn fences in
two recorded migrations, and re-ground one false citation in `src/runtime.rs`.

**Read `DESIGN.md` beside this file first.** It pins the one contract decision — the gate has
**zero** exemption categories — and explains the driver that needs no binary and no `main`.

## Read in order

| room | why you are here |
|---|---|
| `docs/arc/2026/06/278-rules-engine/strike-no-rule-that-cannot-compile/DESIGN.md` | the contract decision, the algorithm, what is affirmatively out of scope |
| `docs/arc/2026/06/278-rules-engine/the-fence-says-what-the-clause-cannot/FINDING-loading-is-not-compiling.md` | the measurement, and the two traps that made the census wrong twice first |
| `tests/lint/rete-compile-census.sh` | the working instrument. Your gate must agree with it |
| `tests/lint/wat_scripts_fixes_load.rs` | ⭐ the sibling gate — same corpus, one step short. Copy its walk, its sharding argument, and its `startup_from_source` driver |
| `tests/lint/every_wat_bad_fixture_actually_fails.rs` | the closed-category discipline, and the shard pattern that keeps a 268-file walk inside nextest's budget |
| `wat/rete/compile.wat:455-470` | the four-axis gate itself — `is-pure`/`is-det`/`is-total`/`is-rete` and `first-failing-axis` |
| `src/rete/vocabulary.rs:709-717` | `:wat::rete::core::String/contains?` — `pure`, `deterministic`, `total`. The primitive the repair uses |
| `src/freeze.rs:924` | `startup_from_source` |
| `src/runtime.rs:25959` | `apply_function` |
| `src/runtime.rs:5364-5378` | the citation to re-ground; read the whole comment, the claim is architectural |

## The eleven — in scope is exactly these paths

**DELETE (9):**
```
wat-scripts/fixes/rete-truth-maintenance-probes/neg.wat
wat-scripts/scratch-pad/experiri-then/then-law-a-core-head.wat
wat-scripts/scratch-pad/experiri-then/then-law-a-core-not.wat
wat-scripts/scratch-pad/probe-arena-rich-graph.wat
wat-scripts/scratch-pad/probe-rete-predicate-termination-routes.wat
wat-scripts/scratch-pad/probe-rules-rich.wat
wat-scripts/scratch-pad/probe-stop-a-where-arith-path.wat
wat-scripts/scratch-pad/probe-where-cond-fence-execution-split.wat
wat-scripts/scratch-pad/probe-where-shape-spread.wat
```

**REPAIR (2):** `wat-scripts/fixes/to-faithful-clojure-net.wat` (fences calling `:fix::has-ns?`
and `:fix::type-shaped?`), `wat-scripts/fixes/to-faithful-clojure-rete.wat` (`:fix::head-keyword-str?`).

Every other file in the corpus is out of scope. The list is the partition.

## Sketch

```rust
// Per file: source + a synthesized entry defn. startup_from_source never evals `:user::main`,
// so a codemod's side effects cannot fire — which is why this needs no temp-file `sed`.
let mut src = std::fs::read_to_string(path)?;
src.push_str(&driver_defn(&namespaces_declared_in(&src)));   // calls compile-all over collect-rules
let world = startup_from_source(&src, /* … as wat_scripts_fixes_load.rs does … */)?;
let f = world.symbols().get(":census::run").expect(…).clone();
apply_function(&f, …)   // Err, or a CompileOutcome that is not Compiled, is the finding
```

## Blast radius

One new file under `tests/lint/`; the 9 deletions; 3 fence lines in 2 `.wat` files plus their
explanatory comments; one comment in `src/runtime.rs`. No change to `wat/`, to
`src/rete/`, to `rete-compile-census.sh`, or to any corpus file not named above.

## STOP triggers

These partition the territory — deletions, repairs, population. No two overlap, and every case
falls in exactly one.

**STOP-1 (deletions) — a file on the DELETE list is driven by a test.** The list was measured as
having zero references from `tests/`, `src/`, `wat/` other than prose. If any file turns out to be
*executed* by a test, stop on that file, keep it, and report which test. Do not delete something
that is load-bearing because a list said so.

**STOP-2 (repairs) — you cannot show the repaired fence is equivalent to the function it replaces.**
⛔ There is no before/after diff available: these codemods have never run, so "before" produces
nothing. Equivalence must be shown the other way — read the function body, drive the old function
and the new fence expression over the same inputs, and show they agree. If they cannot be made to
agree, stop and report; a silent semantic change to a recorded migration is worse than a broken one.

**STOP-3 (population) — the gate reds on a file NOT among the eleven.** The census was measured at
`d7aa7c9ae`. A twelfth file means the census was wrong or the corpus moved. Stop, report the file
and its message, and do not extend the delete list on your own judgment.

## The open question — answer it either way, with evidence

`src/runtime.rs:5371` claims the `where`-traverses-`dispatch_keyword_head_value` path is
*"proven by `probe-stop-a-where-arith-path.wat`"*. That file's fence is refused —
`:wat::core::i64::+` is **not total** — so it compiles nothing and traverses nothing.

**The citation is void. The CLAIM may still be true, and that is the question.** Either ground it
on something that actually executes and rewrite the sentence to cite that, or state plainly in the
comment that the traversal is unproven. **Do not simply delete the sentence** — it is load-bearing
architecture prose about why the namespace gate comes first. If the claim turns out false, that is
a finding worth more than this whole strike; say so.

## Working rules

- `cargo nextest run --release`, never `cargo test`. The floor is `./scripts/floor.sh`.
- ⛔ **Run the floor in the FOREGROUND and do not end your turn while it is running.** If your
  tooling forces it into the background, poll it to completion and read `.floor/latest/clean.log`
  yourself. A floor nobody read is not a floor. This has now failed three times in this project.
- Read the `Summary` line from the captured log; never a piped exit code.
- One cargo build at a time — `pgrep -af 'cargo|nextest'` first.
- **Commit only on green.** Stage explicit paths. `git commit -F -` with a quoted heredoc.
- On any red: do NOT re-run. Copy the failing test's whole stdout and stderr verbatim, name the
  exact assertion arm, surface it.
