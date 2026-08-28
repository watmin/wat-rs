# STONE E-i — the maps get their homes: `:wat::map::` and `:wat::hashmap::`

DRAWN + BRIEFED 2026-08-26 against `5c343e553`.
PRIOR ART — read D's commit message first; five stones refined this method and each one's finding is
a line in this brief: **A-i** `b2d10158f` · **A-ii** `1333e90d0` · **B-i** `ae2330bc1` ·
**B-ii** `870d59898` · **C** `11b85591e` · **D** `1a3f6a703`.

## The move

```
:wat::core::PersistentMap/*  ->  :wat::map::*        326 sites   ← UNMARKED
:wat::core::HashMap/*        ->  :wat::hashmap::*    221 sites   ← marked
verbs (union): assoc · contains-key? · dissoc · empty? · get · keys · length · values
```

**Both flavors survive.** Builder, 2026-08-26: *"these just co-exist in the interim in their
appropriate homes."* This is a spelling migration, not a backend decision.

## ★ WHY PERSISTENT GETS THE UNMARKED NAME — this is the stone's whole point

The builder is moving to persistent-backed collections *"probably in a week or two"*, and the ask was
to make that move **more tractable than it is now**. Naming decides the cost:

- **Persistent → `:wat::map::` now.** When the backend swap lands, this family **never moves again** —
  its name already *is* what the default will be called. Only `:wat::hashmap::` moves, once.
- **If instead both were named for their flavor** (`:wat::persistentmap::` / `:wat::hashmap::`), the
  swap would have to re-partition **both** — 541 extra sites of churn, because the persistent one
  would have to be renamed into the unmarked slot it should have held from the start.

Whatever marker the builder later rules for the non-default flavor, the move is then a **prefix
rename** — the shape this arc has executed five times with a recorded codemod, one stone each.

⚠ **Do NOT claim `:wat::set::` or `:wat::list::`** in this stone or any other until a persistent set
and list exist. They are E-iii's, and the unmarked name must stay free for the flavor that will
become the default. Squatting it guarantees a second migration.

## The seven consumers — named, because B-i's brief claimed zero and the rename found six

```
src/collection/eval.rs   34     the implementations
src/runtime.rs           28     dispatch
src/check.rs             21     type schemes
src/rete/purity.rs       18     pure/deterministic/total axes
src/macros/eval.rs        8     is_pure_total (the macro F5 gate)
src/resolve/normalize.rs  3     ← NEW; the numerics never touched this one
src/rete/vocabulary.rs    1     the rete op table
```

A consumer census before the brief is cheap and **still not sufficient** — three stones running, the
gates found consumers the census missed. Expect one you are not told about; when a gate names it,
that is the system working.

## The corpus — FIVE extensions, because a census scoped to two has bitten twice

```
.wat  122      .rs  9 files/122      .edn  3      .bad  3      .md  1
```

`.edn` is new to this family. `.bad` is invisible to `git ls-files '*.wat'` **by extension** (B-ii),
and `.jsonl` was invisible to a `'*.rs' '*.wat'` bar and took the floor down (C). **Re-run the
all-extension census at the END** — a count taken before the work is not a count taken after:

```bash
git grep -lE ':wat::core::(HashMap|PersistentMap)[:/]' -- ':!docs' | sed 's/.*\.//' | sort | uniq -c
```

## ⛔ PHASE ORDER — `wat/core.wat` uses these 9 times

Same hazard as Stone D. `wat/core.wat` is the FIRST file loaded; retire the old spelling while it
still uses them and the whole substrate fails to load, cascading everywhere.

```
PHASE 1   register the new names.  BOTH SPELLINGS LIVE.  Nothing in the corpus moves.
PHASE 2   move the corpus by codemod.  Both spellings still work.
PHASE 3   retire the old.  Delete the old machinery.
```

Verify the tree builds and an ORDINARY program still `--check`s clean at the end of each phase.
⚠ Do **not** use `./target/release/wat --check wat/core.wat` as that signal — it fails at baseline for
an unrelated reason (`core.wat` is `include_str!`'d and loaded with `Stdlib` privilege; handing it to
the CLI as an entry file parses it a second time, unprivileged, and it collides with itself). Stone D
shipped that as an acceptance row and it was **unsatisfiable by construction**.

## ⛔⛔ THREE NEGATIVE FIXTURES WILL BE DISARMED BY PHASE 3 — this is not hypothetical

Stone C retired a name and **silently disarmed eleven negative tests**; their fixtures used the
retired spelling and their tests asserted only `is_err()`, so each began passing on the RETIREMENT
error instead of the defect it existed to prove, and nothing went red.

I ran that pre-check for this stone and **it fired.** Three fixtures use these verbs in EXECUTABLE
position, and all three of their tests are bare `is_err()`
(`tests/function/probe_arc237_7c_assoc_polymorphic.rs:66-83`):

```
probe_arc237_7c_wrong_key.wat.bad        "assoc HashMap<String,i64> with i64 key MUST reject"
probe_arc237_7c_wrong_value.wat.bad      "... with String value MUST reject"
probe_arc237_7c_non_collection.wat.bad   "assoc with non-collection arg0 (i64) MUST reject"
```

**Migrate all three in Phase 2 with the rest of the corpus**, and in Phase 3 **prove each still fails
for its own reason** — run `--check` on each fixture and paste the diagnostic. If any names a
retirement rather than a type mismatch, that fixture is disarmed and the stone is not done.

## Your role

cwd `/home/john/work/holon/wat-rs`; run `pwd` first. **Ending your turn ENDS you** — every command
FOREGROUND, blocking. **You may not spawn sub-agents.** Do not commit, push, stash, revert, or
`git checkout`; `git stash@{0}` must never be touched. **No git worktrees** — if you need a baseline
fact, state it as a claim about the current tree instead.

You may run `cargo build --release`, `cargo build --release --all-targets`,
`./target/release/wat --check|--grep <f>`, `./target/release/wat <f>`, and single named tests.
**Not** the floor, **not** clippy.

**Structure:** two registry homes, `src/intrinsic/map.rs` and `src/intrinsic/hashmap.rs` — thin
`#[wat_intrinsic]` shims with `///` preambles, copying `src/intrinsic/string.rs`. **The algorithms
stay in `src/collection/`**, which is already the namespace home; this is the same two-home split the
string carve used (registry home = dispatch shim + doc; namespace home = the code).

**The corpus moves by wat-fix RULES codemod** — copy
`wat-scripts/fixes/rename-core-numerics-to-their-homes.wat`. Its traps: `rename-keyword-prefix` is a
**silent no-op** on `::`-terminated prefixes, and the **KEYWORD-ONLY** guard is mandatory. ⚠ These are
**slash-form** names (`HashMap/get`) — a different prefix shape from the numerics' `::`. Handle it or
report that it needs its own rule. Get the population from **wat-grep, not text**
(`wat-scripts/grep/core-numerics-ops.wat` is the shape).

## STOP triggers — each rejects

1. **STOP-1 — a retirement row does not fire.** Prove a retired spelling produces a CHECK-time error
   naming its replacement, and say which door. A prior stone shipped 14 inert rows.
2. **STOP-2 — a negative fixture still fails for the wrong reason after Phase 3.** Named above; three
   are already known. Report the diagnostic.
3. **STOP-3 — you would need to claim `:wat::set::` or `:wat::list::`.** You would not; say so if the
   work seems to want it.
4. **STOP-4 — a room's line number does not hold.** Written against `5c343e553`.

## Acceptance — every row measures a MECHANISM, and every bar is satisfiable

```bash
# 1. the new names RUN (not merely register) — a probe under wat-scripts/scratch-pad/
#    asserting a result for each of the 8 verbs under BOTH new namespaces.
./target/release/wat wat-scripts/scratch-pad/<probe>.wat; echo "EXIT=$?"    # 0

# 2. the old spellings are CHECK errors naming their replacements.
./target/release/wat --check /tmp/old-map.wat; echo "EXIT=$?"               # non-zero + remedy

# 3. the bootstrap still loads — via an ORDINARY program, NOT wat/core.wat directly.
./target/release/wat --check /tmp/trivial.wat; echo "EXIT=$?"              # 0
cargo test --release --test lint every_wat_scripts_file_loads_on_the_current_runtime

# 4. the three negative fixtures each fail for THEIR OWN reason — paste all three diagnostics.
for f in wrong_key wrong_value non_collection; do
  ./target/release/wat --check tests/function/probe_arc237_7c_$f.wat.bad 2>&1 | head -c 300; echo; done

# 5. all-extension census AFTER the work; classify every survivor.
git grep -lE ':wat::core::(HashMap|PersistentMap)[:/]' -- ':!docs' | sed 's/.*\.//' | sort | uniq -c

# 6. codemod idempotence — second run, zero matches.
cargo build --release && cargo build --release --all-targets
```

## Report back with

- Each command's actual output, naming the command that produced each number.
- **The three negative-fixture diagnostics in full** — the row I will read most closely.
- **Which door produced the retirement error.**
- The wat-grep population vs any text count, and the difference explained.
- The /tmp dry-run diff: files, hunks, confirmation every hunk is a prefix rewrite.
- The cascade's waterfall, per phase.
- Anything the brief got wrong. What you did NOT do, and why.
