# STONE P5-b — `@yields` gains a SUBJECT, loses its TYPE, and becomes mandatory

> The stone P5-a was the prerequisite for. Read `BRIEF-STONE-P5-a-one-spelling-for-a-function-type.md`
> first — its wall's predicate is this stone's predicate, and its canonical bracket form is what makes
> everything below derivable.

## The two defects, measured

**1. The gate is ONE-DIRECTIONAL.** `yields_type_matches_fn_arg_param` (`src/intrinsic/mod.rs:962`)
opens with `None => continue` — it checks that a *declared* `@yields` is right and **can never see a
callback that declared none.** Its only enforceable subject today is `:wat::intrinsic::yields-witness`,
the fixture written to exercise it.

**2. `@yields` is a parsed SINGLETON** (`crates/wat-doc/src/lib.rs:604`, `DuplicateSingleton`) while
`:wat::kernel::spawn-thread` declares **three** fn-shaped args. Mandating the singleton would force
one directive to stand for three callbacks — the reason P5 split.

The whole population, and it is closed: **`@yields` appears ZERO times in the `.wat` corpus.** Seven
fn-typed `@arg`s across five `#[wat_intrinsic]` entries. Nothing else.

## ★ THE CONTRACT DECISION — the subject arrives, and the TYPE LEAVES

```
@yields <argname> <desc>          ← repeatable, one per value-carrying fn-shaped @arg
```

**Not `@yields <argname> <type> <desc>`.** Dropping the type is the load-bearing half, and the proof
it is redundant is already in the tree: `yields_type_matches_fn_arg_param` is **a test whose entire
job is to assert that two declarations of one fact agree.** A gate that exists to catch drift between
two spellings of the same truth is proof the second spelling should not exist.

★ **This is only possible BECAUSE P5-a landed.** The callback's param type is now mechanically
extractable from the `@arg` string — `[A :-> B]` yields `A` — because there is exactly one spelling.
Before P5-a there were three, one of them not wat, so `@yields` had to carry its own type. **P5-a did
not merely unblock P5-b; it made P5-b smaller.**

Four questions, on keeping the type token: Obvious YES · Simple YES · **Honest NO** — two declarations
of one fact, with a gate confessing the redundancy. Dropping it: all four YES.

**Consequence:** `yields_type_matches_fn_arg_param` is DELETED, not rewritten. There is no longer a
second declaration to disagree with the first. ⚠ **Prove the coverage is not lost** — the arg-type-vs-
scheme comparison is `doc_arg_ret_types_match_checker_scheme`'s job and already covers a fn-typed
`@arg`; the "declares `@yields` but has no Fn param" panic becomes an expand-time error. Show both.

## The mandate, and its exact shape

**Every fn-shaped `@arg` THAT HANDS A VALUE IN must carry a `@yields`. One that hands nothing in must
NOT.** Enforced at expand time — a `compile_error!`, not a test.

★ The predicate is P5-a's wall predicate, free: a canonical `@arg` type is fn-shaped iff it is
bracket-delimited and contains `:->`. And it is **nullary iff it begins `[:->`** — no params, nothing
handed in, so a `@yields` there would be a lie with no referent.

```
MUST carry @yields (5)
  :wat::holon::Hologram/make      filter          [:wat::core::f64 :-> :wat::core::bool]
  :wat::kernel::spawn-thread      prog            [(:wat::kernel::Peer :- [S R]) :-> :wat::core::nil]
  :wat::kernel::spawn-thread      post_spawn_fn   [:wat::spawn::ThreadLaunch :-> :wat::core::nil]
  :wat::kernel::spawn-process     post_spawn_fn   [:wat::spawn::ProcessLaunch :-> :wat::core::nil]
  :wat::intrinsic::yields-witness f               [:wat::core::i64 :-> :wat::core::i64]   ← HAS one; migrate it

MUST NOT carry one (1)
  :wat::kernel::spawn-thread      init_fn         [:-> :wat::core::Record]     ← NULLARY. Nothing is handed in.

LEDGERED, neither (1)
  :wat::kernel::fn-forms          f               :wat::core::Fn   ← P5-a's FN_ARG_ANON_SYMBOL_LEDGER
```

⚠ **Four of these five need a `<desc>` WRITTEN, not copied.** The description is the one thing not
derivable — it says what the callback receives and when. `spawn-thread`'s `prog` receives the
self-peer handle at thread start; the two `post_spawn_fn`s receive a launch record **owner-side after
the spawn**. Read each verb's prose before writing its line; a `<desc>` that restates the type is
worthless, and the type is no longer even there to restate.

## The rooms

```
crates/wat-doc/src/lib.rs:158,163,356,601-644   grammar: DocYields gains `arg`, loses `ty`;
                                                `yields: Option<DocYields>` -> `Vec<DocYields>`;
                                                DuplicateSingleton -> duplicate-SUBJECT rejection,
                                                and an unknown subject (naming no @arg) is an error
crates/wat-macros/src/wat_intrinsic.rs:690,896  emission: one literal -> a slice; AND the expand-time
                                                mandate (the new work — nothing here does this today)
src/intrinsic/mod.rs:388-392,476,528            `yields_type: Option<&'static str>` -> the pair slice
src/intrinsic/mod.rs:962                        DELETE `yields_type_matches_fn_arg_param`
src/intrinsic/reflect.rs:435-438                render: the `Yields:` line, now N lines, and the TYPE
                                                is DERIVED from the @arg for display
```

## STOP triggers — each REJECTS. Ship nothing on that row and report.

1. **A callback takes MORE THAN ONE parameter.** No such arg exists today (verify that yourself). The
   grammar above names one subject and derives one type; a two-param callback needs a grammar decision
   I have not made. **Report it; do not invent a spelling.**
2. **The param type is not extractable from some canonical `@arg`.** The whole contract decision rests
   on it. If it fails anywhere, STOP — the type token cannot be dropped and the stone must be redrawn.
3. **Deleting `yields_type_matches_fn_arg_param` loses coverage you cannot show is held elsewhere.**
   Keep it and report, rather than shipping a hole.
4. **The mandate cannot be enforced at expand time** and you find yourself writing a test instead.
   That is the convention rung where the top rung was the point. Report why.

## Acceptance

```
 0. ★ YOUR OWN CENSUS from `entry.args`, before touching anything: which args are fn-shaped, which
      are NULLARY fn-shaped, which carry @yields. Every disagreement with my 5/1/1 reported.
 1. ★ THE MANDATE IS PROVEN BY BREAKING IT, BOTH WAYS, BEFORE the corpus is fixed:
        a. delete the witness's @yields          -> expand-time compile_error
        b. add a @yields to `init_fn` (nullary)  -> expand-time compile_error
      Paste both verbatim. Restore both. A mandate never seen to refuse is not a mandate.
 2. ★ A THIRD BREAK: a @yields naming an @arg THAT DOES NOT EXIST -> compile_error. Paste it.
 3. ★ THE REPEAT WORKS. `spawn-thread` carries TWO @yields (prog, post_spawn_fn) and compiles —
      the exact shape the singleton made impossible. This is the stone's whole point; show it.
 4. ★ ALL FIVE `<desc>`s ARE WRITTEN FROM THE VERB'S PROSE. Quote the prose you derived each from.
 5. ★ COVERAGE NOT LOST by the deletion — name what now catches each thing the old gate caught.
 6. ★ render-doc's `Yields:` line renders N lines with DERIVED types. Show spawn-thread's output.
 7. ★ THE GOLDEN. `tests/reflection/probe_arc255_spec_complete.rs:107` pins render-doc byte-identically
      and it covers the witness. It WILL move. New bytes in the report; do not edit it silently.
 8. ★ P5-a'S WALL STILL GREEN — `fn_typed_arg_has_one_canonical_spelling`.
 9. cargo build --release --all-targets — clean; report any warning VERBATIM.
10. cargo nextest run --release -E 'test(reflection) + test(lint) + test(intrinsic) + test(kernel)'
```

## How to work

- Work only in `/home/john/work/holon/wat-rs`. `pwd` first. Never operate on a `.claude/worktrees/` path.
- **Everything FOREGROUND. Ending your turn ENDS you** — nothing wakes you, no notification is coming.
  Your turn ends when the numbers are in your hands, not when a command starts.
- `cargo build`, scoped `cargo nextest`, `./target/release/wat` — yes. **The full floor and clippy are
  the orchestrator's**; do not run them.
- **You may not spawn sub-agents.**
- No `git stash`, in any form. Do not commit, push, revert, or create a worktree.
- New scratch `.wat` → `wat-scripts/scratch-pad/`, `--check` clean. Not the session scratchpad.
- ⚠ Your own added prose must not contain the literal pattern any acceptance grep looks for.

## Report back with

Row by row: the command, its actual output, PASS/FAIL. Your census with every disagreement against my
5/1/1. All three compile_errors verbatim. The five `<desc>`s with the prose each came from. What the
golden became. Then the honest deltas — what surprised you, and anything you could not measure.
