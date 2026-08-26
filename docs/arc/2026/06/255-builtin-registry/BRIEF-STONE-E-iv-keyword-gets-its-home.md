# STONE E-iv — keyword gets its home: `:wat::keyword::`

DRAWN + BRIEFED 2026-08-26 against `9dd54e58a`.
PRIOR ART — **read E-iii's commit message first** (`9dd54e58a`); its "a gate that LOADS is not a gate
that RUNS" finding is why this brief has a step the earlier ones lacked. Closest code to copy:
`src/intrinsic/hashset.rs`, `wat-scripts/fixes/rename-core-set-and-list-to-their-homes.wat`.

## The move

```
:wat::core::keyword/*  ->  :wat::keyword::*
  from-string · to-string · to-symbol · to-type-form · to-type-form-colon      (5, from the DISPATCH table)
```

**One flavor, so the plain name — no marked/unmarked question here.** `keyword` is a SCALAR type, and
every one of its siblings already has a home: `bigint · bytes · char · f64 · i64 · rational · regex ·
string · time · uuid`. This is the last scalar without one. Nothing is reserved against it.

⚠ **`:wat::core::keyword` — the bare TYPE — does not move.** Arc 251's `wat.type/keyword`. The
trailing `/` is the whole discrimination. **STOP-3.**

## The ground — measured, and THREE things differ from E-iii

```
colon-form files    70   →  .wat 50 · .rs 15 · .edn 3 · .bad 1 · .expr 1
dotted-form files    0   ← checked (E-ii's finding); re-check AFTER
wat/core.wat        13   ← THE BIGGEST BOOTSTRAP EXPOSURE OF THE FAMILY. E-iii had zero.
rete/vocabulary.rs   4      rete/expr_ir.rs 0      macros/eval.rs 5      rete/purity.rs 2
```

**FIVE extensions, and `.expr` is one I have not seen before** —
`tests/types/probe_stone_233_2_k_variant_retired_let_keyword.wat.expr`. A census scoped to `.wat`
and `.rs` misses `.edn`, `.bad`, AND `.expr`. Re-run the all-extension census AFTER the work.

## ⛔ THE NEGATIVE FIXTURE IS AT RISK — unlike the last two stones

`tests/function/probe_diagnostic_non_vector.wat.bad` uses a keyword verb in **executable** position,
and its test asserts a bare `is_err()`
(`tests/function/probe_diagnostic_dynamic_keyword_invocation.rs`):

```rust
assert!(result.is_err(), "non-vector spread arg (i64) must error at eval");
```

Stone C retired a name and **silently disarmed eleven tests of exactly this shape**, floor green
throughout. **Migrate this fixture in Phase 2**, and in Phase 3 **prove it still fails for its own
reason** — `--check` it and paste the diagnostic. If it names a retirement rather than the
non-vector-spread defect, the fixture is disarmed and the stone is not done. **STOP-2.**

## ⛔⛔ TWO FILES SPELL THE RETE FORM DIRECTLY — and the load gate CANNOT see them break

```
wat-scripts/scratch-pad/probe-cond-rete-scorecard.wat
wat-scripts/scratch-pad/probe-cond-rete-where.wat
```

These carry `:wat::rete::core::keyword/…`, so the substring `:wat::core::keyword/` **never appears in
them** — invisible to a text census, and E-iii's equivalent file produced **zero Match facts** so it
was invisible to the structural one too.

**E-iii's finding, and it is the reason this section exists:** that file began **crashing at runtime**
the moment its `RETE_OPS` row was renamed, and BOTH gates stayed green — `--check` does not run a
program, and `every_wat_scripts_file_loads_on_the_current_runtime` checks and loads without invoking
`main`. **A gate that proves a file LOADS does not prove it RUNS.**

So: after the rete rows move, **RUN both files** (`./target/release/wat <path>`), do not merely
`--check` them, and report both exit codes. Handle them with **exact-match** codemod rules, not a
prefix rule, so nothing adjacent is swept up.

## ⛔ PHASE ORDER — 13 occurrences in `wat/core.wat`, the largest yet

```
PHASE 1   register.  BOTH SPELLINGS LIVE.  Nothing moves.
PHASE 2   corpus moves by codemod.  Both still work.
PHASE 3   retire.  Delete the old machinery.
```

`wat/core.wat` is the FIRST file loaded. Retire while it still uses these and **every program fails
to load**. E-ii hit exactly this when a bootstrap `defmacro` body used a migrating verb missing from
`is_pure_total`'s F5 allow-list — **that list has 5 keyword entries here**, so dual admission during
the migration window is likely required. Check `wat/*.wat` for keyword verbs inside `defmacro` bodies
before Phase 3.

Verify an **ORDINARY program** still `--check`s clean at each phase boundary. ⚠ Never use
`wat --check wat/core.wat` as that signal — it fails at baseline for an unrelated reason and Stone D
shipped it as an acceptance row that was unsatisfiable by construction.

## Your role

cwd `/home/john/work/holon/wat-rs`; `pwd` first. **Ending your turn ENDS you** — every command
FOREGROUND. **No sub-agents. No git worktrees.**

⛔ **DO NOT RUN `git stash` IN ANY FORM** — not push, not `-u`, not pop, not list. A rule about the
COMMAND, not an entry: `stash@{0}` holds irreplaceable work and a LIFO push/pop *looks* safe right up
until one mistake destroys it. For a before/after comparison, copy to `/tmp` and diff there.

Do not commit, push, revert, or `git checkout`.

You may run `cargo build --release`, `cargo build --release --all-targets`,
`./target/release/wat --check|--grep <f>`, `./target/release/wat <f>`, and targeted
`cargo nextest run --release -E '<filter>'`. **Not** the floor, **not** clippy.
⚠ **`cargo test` is not a diagnostic** — it runs N tests in one process, which
`src/host/test_runner.rs:48-55` documents as unsupported.

**Structure:** `src/intrinsic/keyword.rs`, thin `#[wat_intrinsic]` shims copying
`src/intrinsic/hashset.rs`. Algorithms stay where they are. Non-pure-det verbs need `@example-norun`;
confirm which of the five are pure∧det rather than assuming.

**Corpus by wat-fix RULES codemod** — slash-form; copy
`wat-scripts/fixes/rename-core-set-and-list-to-their-homes.wat`. Population from **wat-grep, not
text**. Dry-run on /tmp, read the diff, apply, prove idempotence.

**The rete naming invariant fires** — `rete_name == core_name` with `::rete::` spliced after
`:wat::`, tested; 4 rows. `RETE_MODULES` may or may not need `:wat::rete::keyword::` — **it is needed
only if `RETE_OPS` actually has a keyword row that forces it.** E-iii's brief demanded an entry
nothing forced and the rider correctly refused. **Measure before adding.** Expect
`tests/rete/datamancer.rete.edn` to need regeneration (its `:abi` hashes every `rete_name`); use the
documented command and diff to confirm only `:abi` changed.

## STOP triggers — each rejects

1. **STOP-1 — the pairing gate needs a `NAMING_RULE_EXCEPTIONS` entry.**
2. **STOP-2 — the negative fixture fails for the wrong reason after Phase 3.** Named above.
3. **STOP-3 — you would move the bare TYPE `:wat::core::keyword`.** Only slash-verbs move.
4. **STOP-4 — a room's line number does not hold.** Written against `9dd54e58a`.

## Acceptance — every row measures a MECHANISM, every bar is satisfiable

```bash
# 1. all 5 verbs RUN under the new spelling — a scratch-pad probe asserting a result for each.
./target/release/wat wat-scripts/scratch-pad/<probe>.wat; echo "EXIT=$?"        # 0

# 2. the old spelling is a CHECK error naming its replacement.
./target/release/wat --check /tmp/old-kw.wat; echo "EXIT=$?"                    # non-zero + remedy

# 3. the negative fixture fails for ITS OWN reason — paste the diagnostic.
./target/release/wat --check tests/function/probe_diagnostic_non_vector.wat.bad 2>&1 | head -c 300

# 4. the two rete-direct files RUN, not merely check.
./target/release/wat wat-scripts/scratch-pad/probe-cond-rete-scorecard.wat; echo "EXIT=$?"
./target/release/wat wat-scripts/scratch-pad/probe-cond-rete-where.wat;     echo "EXIT=$?"

# 5. pairing gate + bootstrap.
cargo nextest run --release -E 'test(naming_rule_tests)'
./target/release/wat --check /tmp/trivial.wat; echo "EXIT=$?"                   # 0
cargo nextest run --release -E 'test(every_wat_scripts_file_loads_on_the_current_runtime)'

# 6. BOTH renderings, ALL extensions, AFTER the work — classify every survivor.
git grep -lE ':wat::core::keyword[:/]' -- ':!docs' | sed 's/.*\.//' | sort | uniq -c
git grep -lE ':wat\.core/keyword/'     -- ':!docs' | sed 's/.*\.//' | sort | uniq -c

# 7. the bare TYPE did not move — boundary-anchored, before AND after, prove equality.
git grep -oE ':wat::core::keyword(::|/)?' -- ':!docs' | grep -vE '(::|/)$' | wc -l

cargo build --release && cargo build --release --all-targets
```

## Report back with

- Each command's actual output, naming the command that produced each number.
- **The negative fixture's diagnostic in full**, and **both rete-direct files' exit codes**.
- Everything the rete invariant forced — and whether `RETE_MODULES` needed an entry, with the measurement.
- Which door produced the retirement error.
- The wat-grep population vs any text count, and the difference explained.
- The cascade's waterfall, per phase.
- Anything the brief got wrong. What you did NOT do, and why.
