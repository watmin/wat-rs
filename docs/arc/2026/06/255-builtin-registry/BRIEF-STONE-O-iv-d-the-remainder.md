# STONE O-iv-d — the remainder, and the end of the sweep

> The last O-iv wave. Read `BRIEF-STONE-O-iv-c-2-the-holon-sweep-atom.md` for the shape and the
> design's **"A FOURTH DISQUALIFIER"** section for the full disposition axis.

## The work — 14 verbs, not 26

The shell census said 26. Classified against all **four** disqualifiers the sweep has since learned,
across `uuid` · `kernel/ambient` · `string` · `reflect` · `bytes` · `witness` · `time` · `regex` ·
`math` · `list` · `char` (93 handlers total):

```
MIGRATABLE?   14        BINDING   67        ARG-SPAN   12
```

⚠ **A CANDIDATE LIST from a pattern.** Verify each by reading. If your count differs, that is a
finding — O-iv-c-1 refused five my brief demanded, and O-iv-c-2 refused one for a reason I had never
named.

★ **Already checked for you: none of the 14 appears in `eval_apply`'s `SPECIAL_FORMS` list.** That
check is now mandatory before classifying anything — a name on that list is already ruled
un-dispatchable by someone who wrote down why, and O-iv-c-2's `:wat::holon::literal` was exactly
that case.

```
:wat::core::List                              ← the valuable one: VARIADIC
:wat::uuid::v4 · :wat::uuid::nil
:wat::kernel::stopped? · sigusr1? · sigusr2? · sighup?
:wat::kernel::reset-sigusr1! · reset-sigusr2! · reset-sighup!
:wat::string::declare-acronyms
:wat::intrinsic::variadic-args-measurement
:wat::time::now
:wat::math::pi
```

## What is different about this wave

**Twelve of the fourteen take zero arguments.** A 0-arg ALGEBRA fn is `fn f() -> Result<Value,
EvalBreak>` — no `&Value` params at all. O-iii's rider explicitly handled the n=0 case in the
generated arg-list (it noted avoiding "a dangling-leading-comma bug"), so the machine supports it —
but this is the first wave to exercise it at scale. **If n=0 misbehaves, that is a finding about the
generator, not a reason to skip the verb.**

**`:wat::core::List` is VARIADIC** — `args: &[WatAST]`, evaluating each element. It migrates to the
`&[Value]` ALGEBRA shape:

```rust
-pub(crate) fn eval_list_of(args: &[WatAST], env, sym, _span) -> … {
-    for arg in args { items.push_back(eval_inner(arg, env, sym)?.value_owned()); }
+pub(crate) fn list_of(vals: &[Value]) -> … {
+    for v in vals { items.push_back(v.clone()); }
```

**This is the most valuable verb in the wave** — `(apply :wat::core::List …)` is a real thing to want,
and it is the first *variadic* ALGEBRA migration in the arc. Do it carefully and prove it with a
3-element splat.

⚠ **SEVERAL OF THESE ARE EFFECTFUL, and three MUTATE.** `uuid::v4` and `time::now` are
non-deterministic; `reset-sigusr1!` / `reset-sigusr2!` / `reset-sighup!` clear global signal flags.
Migration does not grant a new capability — every one is already callable directly — but say in your
report that you noticed, and check each one's `@Purity`/`@Determinism` still reads correctly
afterward.

## ⚠ A finding to REPORT, not fix: `:wat::string::declare-acronyms` is a no-op stub

```rust
pub(crate) fn eval_string_declare_acronyms(_ns, _acronyms, _env, _sym, _span) -> … {
    Ok(Value::Unit)     // both arguments accepted and discarded
}
```

It takes two arguments, ignores both, and always returns `Unit` — with runes on `_env`, `_sym` AND
`_span`. Migrating it is trivial and harmless. **Whether a verb that does nothing should exist at
all is not this stone's question** — record it and move on.

## STOP triggers — each REJECTS. Ship nothing and report the gap.

1. **A verb trips any of the four disqualifiers** — `env`/`sym` beyond arg-eval, `<arg>.span()`,
   unevaluated-args, or a `SPECIAL_FORMS` entry. Leave it, name it, say which.
2. **The n=0 generated door misbehaves.** Report it as a generator finding; do not work around it.
3. **Any behaviour changes** — value or error text, direct or through `apply`.
4. **A verb's `@Purity` or `@Determinism` becomes wrong.** Several of these are effectful; the doc
   must still say so.

## Acceptance

```
 0. ★ YOUR OWN DISPOSITION TABLE, before migrating. Every disagreement with my 14 reported.
 1. ★ EVERY MIGRATED VERB REACHES apply — scratch .wat, `--check` clean, before and after.
      BEFORE: O-iv-a's "registered, but no handler taking EVALUATED arguments…". AFTER: answers.
 2. ★ `(apply :wat::core::List [1 2 3])` RETURNS A 3-ELEMENT LIST. The first variadic ALGEBRA
      migration in the arc — prove the splat, not just that it dispatches.
 3. ★ THE 0-ARG DOOR WORKS. `(apply :wat::math::pi [])` and one effectful 0-arg verb.
 4. ★ DIRECT CALLS BYTE-IDENTICAL for every migrated verb, before and after, diffed.
      `git show HEAD:<path>` for the pre-image — never `git stash`.
 5. ★ PURITY/DETERMINISM INTACT. `(:wat::runtime::metadata-of <verb>)` for `uuid::v4`,
      `time::now`, and a `reset-*!` — before and after, and say what each declares.
 6. cargo build --release --all-targets — clean; report any warning verbatim.
 7. cargo nextest run --release -E 'test(intrinsic) + test(apply) + test(uuid) + test(kernel)'
```

## How to work

- Work only in `/home/john/work/holon/wat-rs`. `pwd` first. Never operate on a `.claude/worktrees/` path.
- Everything FOREGROUND. Ending your turn ENDS you. Land the numbers.
- `cargo build`, scoped `cargo nextest`, `./target/release/wat` — yes. The full floor and clippy are
  the orchestrator's.
- No sub-agents. **No `git stash`, in any form.** Do not commit, push, revert, or create a worktree.
- New scratch `.wat` → `wat-scripts/scratch-pad/`, `--check` clean.

## Report back with

Row-by-row: command, actual output, PASS/FAIL. Your disposition table with every disagreement. The
effectful verbs you noticed. Then the honest deltas.
