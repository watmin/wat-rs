# STONE F — the `String/` verbs leave the instance-method namespace

DRAWN + BRIEFED 2026-08-26 against `fa3d29df6`.
**PRIOR ART — read E-iv's commit message first** (`22453b9b6`); its open ruling on producer
provenance does NOT bind here (none of these five verbs is a producer — verified). Closest code to
copy: `src/intrinsic/string.rs:222` (`trim`, the shim shape) and
`wat-scripts/fixes/rename-keyword-to-its-home.wat` (the codemod shape).

**This stone FINISHES Stone E.** `23efc6056` moved the lowercase family to `:wat::string::*` and left
the capital one standing. `56eb6ab3a` deleted `string_ops.rs`. What remains is five verbs squatting in
a namespace that means something else.

## The move

```
:wat::core::String/*  ->  :wat::string::*
  concat · starts-with? · ends-with? · contains? · empty?          (5, from the DISPATCH table)
:wat::rete::core::String/*  ->  :wat::rete::string::*              (5, forced by the naming invariant)
```

## ⛔ THE ONE CONTRACT DECISION — the namespace SURVIVES; the squatters leave

`:wat::core::String/<name>` is **not** a function namespace. It is the **instance-method namespace
that `extend-type` generates**, and it is live and correct. Proved, `tests/types/probe_arc293_4c_extend_type_adapter_dup.wat.bad`:

```clojure
(:wat::core::extend-type :wat::core::String :t::DupTagged
  (tag [self] -> :wat::core::i64 1))          ;; registers :<wat::core::String>/tag
```

**Builder's ruling, 2026-08-26:** *":wat::string::* is the home for string funcs, :wat::core::String
is meant to be a type, :wat::core::String/* would imply 'instance methods' for strings."*

So this stone does **not** kill `String/`. It evicts five plain functions that were never methods.
The name `:wat::core::String/tag` must still be mintable by `extend-type` when the stone lands, and
proving that is an acceptance row — not an afterthought.

⚠ **The bare TYPE `:wat::core::String` does not move.** The trailing `/` is the whole discrimination.
**STOP-1.**

## ⛔ THE HOME IS NOT COMPLETE — `empty?` has no twin, and the BOOTSTRAP calls it

Measured by RUNNING, because `--check` cannot answer this: `src/resolve/walk.rs:268` waves every
`:wat::`-prefixed head through unchecked, so a name that does not exist type-checks clean.
Control probe committed at `wat-scripts/scratch-pad/probe-string-home-completeness.wat`.

```
String/concat        -> :wat::string::concat        OK   same handler already (runtime.rs:5935)
String/starts-with?  -> :wat::string::starts-with?  OK   same handler already
String/ends-with?    -> :wat::string::ends-with?    OK   same handler already
String/contains?     -> :wat::string::contains?     OK   same handler already
String/empty?        -> ⛔ UnknownFunction               NO TWIN. Inline arm, runtime.rs:5969.
```

Four of five uppercase arms already call `intrinsic::string::eval_string_*` — the identical function
the registered lowercase verb uses. **`empty?` is the only verb that would be lost**, and it is not
reachable another way: the polymorphic `:wat::core::empty?` refuses a String by construction, and its
own error enumerates its arms — `Vector`, `HashMap`, `PersistentMap`, `PersistentVector`, `HashSet`,
`List`. String is not among them and never was.

⛔ **`wat/core.wat:1775` and `wat/core.wat:1868` call `:wat::core::String/empty?`.** The FIRST file
loaded depends on the one verb with no twin. **Register `:wat::string::empty?` in PHASE 1 or the
bootstrap cannot migrate at all.**

## ⛔⛔ THE TOOLING THIS STONE WOULD SILENTLY KILL — read this twice

`wat/lint.wat`'s `concat-head?` (`:308-322`) is the `concat-abuse` rule's entry predicate. It
recognises a concat call by **string-literal comparison against `ast-name`** — two literals:

```clojure
(:wat::core::= n ":wat::core::string::concat")     ;; ⛔ ALREADY DEAD — retired by 23efc6056
(:wat::core::= n ":wat::core::String/concat")      ;;    live, and THIS STONE RETIRES IT
```

**Measured this session, by running each name:** `:wat::core::string::concat` raises
`UnknownFunction`. So the rule is already half-dead. **After this stone it matches nothing any
program can write** — and no codemod will catch it, because a keyword-only rewrite correctly refuses
to touch a string literal.

**And its positive control is vacuous.** All three fixtures —
`wat-tests/lint.wat` (8 sites), `tests/lint/probe_arc277_lint_concat_abuse.wat`,
`tests/lint/probe_arc277_1c_concat_format_autofix.wat` — spell the abuse as
`:wat::core::string::concat`, **the dead name**. The rule's green test proves it detects a spelling
that cannot occur. Its only real-world arm has no coverage at all.

This is Stone C's lesson inverted: C found *negative* tests disarmed by a retirement; this is a
*positive* control that only ever fired on a corpse.

**What the stone owes:** `concat-head?` recognises the LIVE name `:wat::string::concat`; the dead
literal is deleted, not accumulated; the three fixtures are migrated to the live name so the gate
proves a real defect is caught. Deleting the dead arm will turn those tests RED first — **that is
the gate working, not a regression.** `wat/lint.wat` also compares `":wat::core::string::interpolate"`
(likewise dead; live twin `:wat::string::interpolate`, verified) — same treatment, same stone.

## The ground — measured by channel, 2026-08-26

```
:wat::core::String/{the five}     .wat corpus 371 · .rs 41 · docs/** 13 (NEVER MOVES) · other 3
:wat::rete::core::String/{five}   .wat corpus  75
dotted form :wat.core/String/      0            ← E-ii's finding; RE-CHECK AFTER
```

The bulk of `String/concat` sits in `wat-scripts/perf/grid/where-*.wat` — the rete where-compiler
perf grid. This family is the one the where-perf work is written in.

**The six tooling surfaces that know these names:**

```
src/runtime.rs:5925-5993        5 alias arms (11 refs) — DELETE; 4 already delegate, empty? is inline
src/check.rs:17367-17415        5 TypeScheme rows — copy the shape at :17527 (`:wat::string::trim`)
src/rete/vocabulary.rs:624-665  5 rows — the lowercase mirror shape is at :669, :1091, :1353
src/rete/purity.rs:602-607      pure_det hand-list, 5 entries
src/rete/purity.rs:778-782      total hand-list, 5 entries
src/rete/expr_ir.rs:1255+       already maps BOTH spellings to one IR node
src/intrinsic/mod.rs:405-410    `pub(crate) mod string` is pub(crate) SOLELY for these arms
src/remedy/retirement.rs:304    the RetirementEntry row shape — E-iv's five rows are the model
```

**PHASE 3 OWES FIVE `RetirementEntry` ROWS**, one per verb, `retired: ":wat::core::String/<v>"` →
`replacement: ":wat::string::<v>"`. Acceptance row 2 measures exactly this: the old spelling must
produce a check error that NAMES its replacement, not a bare "unknown". A retirement with no row
is a name that just vanishes, and the next reader re-mints it innocently.
`[[feedback_retiring_a_name_disarms_every_bare_is_err_test]]`

★ **Two of these DELETE rather than migrate, and that is the point of the stone.**
`src/rete/purity.rs:291` already whitelists `:wat::string::` **by prefix**. The ten hand-list entries
become redundant the moment the verbs move — remove them, do not rewrite them. And `mod string`
reverts to private once nothing outside the registry calls its handlers. A hand-list deleted beats a
hand-list migrated. `[[feedback_a_gate_over_two_hand_lists_is_a_hand_list]]`

**`RETE_MODULES` needs NOTHING** — `:wat::rete::string` is already admitted (measured). Do not add an
entry nothing forces; E-iii's brief demanded one and the rider correctly refused.

## ⛔ THE NEGATIVE FIXTURE IS AT RISK

`tests/function/probe_arc237_stone3_p09.wat.bad:7` calls `:wat::core::String/empty?` in **executable**
position, and its test asserts a bare `is_err()`
(`tests/function/probe_arc237_stone3_guard_ensure.rs:154-158`):

```rust
assert!(result.is_err(),
    ":ensure :fn arg type :String != declared return :i64; should fail type-check; got Ok");
```

Stone C retired a name and **silently disarmed eleven tests of exactly this shape**, floor green
throughout. Migrate this fixture in Phase 2, and in Phase 3 **prove it still fails for its own
reason** — paste the diagnostic. If it names a retirement rather than the `:ensure` type mismatch,
the fixture is disarmed and the stone is not done. **STOP-3.**

## ⛔ PHASE ORDER — the bootstrap calls the verb with no twin

```
PHASE 1   register :wat::string::empty? + its rete mirror.  BOTH SPELLINGS LIVE.  Nothing moves.
PHASE 2   both corpora move by codemod.  Both spellings still work.
PHASE 3   retire.  Delete the five arms, the ten hand-list entries, the dead lint literals.
```

`wat/core.wat` is the first file loaded. Retire while it still uses these and **every program fails to
load.** Verify an ORDINARY program `--check`s clean at each phase boundary. ⚠ Never use
`wat --check wat/core.wat` as that signal — it fails at baseline for an unrelated reason, and Stone D
shipped exactly that as an acceptance row that was unsatisfiable by construction.

## Your role

cwd `/home/john/work/holon/wat-rs`; `pwd` first. **Ending your turn ENDS you** — every command
FOREGROUND. **No sub-agents. No git worktrees.** Do not commit, push, revert, or `git checkout`.
⛔ **Do not run `git stash` in any form** — not push, not `-u`, not pop, not list. For a before/after
comparison, copy to `/tmp` and diff there.

You may run `cargo build --release`, `cargo build --release --all-targets`,
`./target/release/wat --check|--grep <f>`, `./target/release/wat <f>`, and targeted
`cargo nextest run --release -E '<filter>'`. **Not** the floor, **not** clippy — the orchestrator
weighs those centrally. ⚠ `cargo test` is not a diagnostic: it runs N tests in one process, which
`src/host/test_runner.rs:48-55` documents as unsupported.

**Structure:** `:wat::string::empty?` is a thin `#[wat_intrinsic]` shim in
`src/intrinsic/string.rs`, copying `eval_string_trim` at `:222`. It is pure ∧ deterministic ∧ total,
so it takes a plain `@example`, not `@example-norun`. The four existing handlers are already correct
— do not rewrite them.

**Corpus by wat-fix RULES codemod** — copy `wat-scripts/fixes/rename-keyword-to-its-home.wat`.
Population from **wat-grep, not text**: a text count includes comments and string literals a codemod
must never touch. Dry-run on a `/tmp` copy, read the diff, apply, prove idempotence (re-run = 0
changes).

**The rete naming invariant fires** — `rete_name == core_name` with `::rete::` spliced after
`:wat::`, tested. Expect `tests/rete/datamancer.rete.edn` to need regeneration (its `:abi` hashes
every `rete_name`); use the documented command and diff to confirm **only** `:abi` changed.

## STOP triggers — each REJECTS. Ship nothing; surface the gap.

1. **STOP-1 — you would move the bare TYPE `:wat::core::String`.** Only slash-VERBS move, and only
   the five named above.
2. **STOP-2 — you would rewrite an `extend-type`-generated `String/<method>` name.** That namespace
   is live and correct. If the codemod's population includes anything but the five verbs, stop.
3. **STOP-3 — the negative fixture fails for the wrong reason after Phase 3.** Named above.
4. **STOP-4 — the pairing gate needs a `NAMING_RULE_EXCEPTIONS` entry**, or `RETE_MODULES` needs one.
5. **STOP-5 — a room's line number does not hold.** Written against `fa3d29df6`.

## Acceptance — every row measures a MECHANISM, every bar satisfiable

```bash
# 1. all 5 verbs RUN under the home spelling. Extend the committed probe to cover empty?.
./target/release/wat wat-scripts/scratch-pad/probe-string-home-completeness.wat; echo "EXIT=$?"   # 0

# 2. the old spelling is a CHECK error naming its replacement.
./target/release/wat --check /tmp/old-string.wat; echo "EXIT=$?"          # non-zero + remedy names :wat::string::

# 3. ★ extend-type STILL mints String/<method> — the namespace survived. Paste the diagnostic.
./target/release/wat --check tests/types/probe_arc293_4c_extend_type_adapter_dup.wat.bad 2>&1 | head -c 300
#    must still name DuplicateDefine on :<wat::core::String>/tag — NOT a retirement.

# 4. the negative fixture fails for ITS OWN reason. Paste the diagnostic.
./target/release/wat --check tests/function/probe_arc237_stone3_p09.wat.bad 2>&1 | head -c 300

# 5. ★ the lint is ALIVE, not merely green — the fixtures now spell the live name.
#    These five ids are VERIFIED PRESENT in .floor/latest/raw.log; do not substitute guesses.
cargo nextest run --release -E 'test(deftest_wat_tests_lint_detects_concat_abuse) + \
  test(deftest_wat_tests_lint_no_false_positive_concat) + test(probe_arc277_lint_concat_abuse) + \
  test(probe_arc277_1c_concat_format_autofix) + test(probe_arc277_1d_concat_fix_position_gate)'
grep -c ':wat::string::concat' wat-tests/lint.wat tests/lint/probe_arc277_lint_concat_abuse.wat
grep -c ':wat::core::string::concat' wat/lint.wat                          # 0 — the dead literal is gone

# 6. ★ NON-VACUITY for the lint: break the door. Point concat-head? at a bogus name, re-run row 5,
#    confirm RED, restore, confirm GREEN. Report both outcomes. A gate that cannot fail is not a gate.

# 7. bootstrap + gates.
./target/release/wat --check /tmp/trivial.wat; echo "EXIT=$?"             # 0
cargo nextest run --release -E 'test(naming_rule_tests) + test(naming_rule_exceptions_are_exactly_the_documented_eleven)'
cargo nextest run --release -E 'test(every_wat_scripts_file_loads_on_the_current_runtime)'

# 8. BOTH renderings, ALL extensions, AFTER the work — classify every survivor by hand.
git grep -oE ':wat::core::String/[A-Za-z0-9?!-]+' -- ':!docs' | sort | uniq -c
git grep -oE ':wat\.core/String/[A-Za-z0-9?!-]+'  -- ':!docs' | sort | uniq -c
git grep -oE ':wat::rete::core::String/[A-Za-z0-9?!-]+' -- ':!docs' | sort | uniq -c
#    surviving `String/tag`-shaped names are CORRECT. Surviving verbs are not.

# 9. the bare TYPE did not move — boundary-anchored, before AND after, prove equality.
git grep -oE ':wat::core::String(::|/)?' -- ':!docs' | grep -vE '(::|/)$' | wc -l

# 10. the hand-lists SHRANK rather than moved.
grep -c 'String/' src/rete/purity.rs                                      # 0 in both lists

cargo build --release && cargo build --release --all-targets
```

## Report back with

- Each command's actual output, naming the command that produced each number.
- **Row 3 and row 4 diagnostics in full** — the extend-type control and the negative fixture.
- **Row 6's broken-door result** — the lint RED with the door broken, GREEN with it restored.
- The wat-grep population vs any text count, and the difference explained.
- Whether `mod string` reverted to private, and what forced it if not.
- Everything the rete invariant forced; whether `RETE_MODULES` needed an entry, with the measurement.
- The cascade's waterfall, per phase.
- **Anything this brief got wrong.** What you did NOT do, and why.
