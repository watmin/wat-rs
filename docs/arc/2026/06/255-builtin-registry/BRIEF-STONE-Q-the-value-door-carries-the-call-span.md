# STONE Q — the value door carries the call span

> Drawn 2026-08-28 because **O-iv-c is blocked**. Read the design's final section,
> *"O-iv-c IS BLOCKED, AND THE MEASUREMENT REFRAMES THE WHOLE SWEEP"*
> (`DESIGN-STONE-O-one-declaration-feeds-both-doors.md`), before this brief.

## Why this stone exists

The ALGEBRA contract says an ALGEBRA fn takes `&Value` params and nothing else — no `env`, no `sym`,
**no span**. The first two are load-bearing: they are what make a handler need ASTs, so a handler
needing them genuinely cannot be splatted. **The third was an over-reach**, and it caps the sweep:

⛔ **THE CENSUS THAT FIRST MOTIVATED THIS STONE IS RETRACTED — see the design's
`RETRACTED THE SAME DAY` banner.** Three text instruments gave three answers and the third failed
its own control (`eval_f64_max_of` came back SPAN-FREE while reaching `a.span()` inside
`f64_variadic_reduce`, one level down). **Whether a verb can become ALGEBRA depends on what its
HELPERS do, and no scan of a handler's body can see that.**

**Do not restore a number to this brief.** Q is justified by a design argument, not a population:

★ **A span is not binding state.** It is a location, and `apply` already holds one — it just does
not thread it.

## The ONE CONTRACT DECISION

```
ValueHandler  =  fn(&[Value]) -> Result<Value, EvalBreak>
              ->  fn(&[Value], &Span) -> Result<Value, EvalBreak>

ALGEBRA fn    may take a trailing `&Span` after its `&Value` params:
                  fn f(a: &Value, b: &Value) -> Result<Value, EvalBreak>            // still legal
                  fn f(a: &Value, b: &Value, span: &Span) -> Result<Value, EvalBreak>
```

Both doors then supply a real call span: the AST door passes the `list_span` it already has; the
value door passes the one `eval_apply` already holds.

⚠ **This REVERSES Stone O-i's STOP-3** — *"you need a new parameter on `dispatch_substrate_impl`
→ STOP and report."* That was correct for O-i, whose blast radius was one function and whose job was
a guard. It was never a ruling about the parameter forever. You are doing the thing that STOP
forbade, deliberately, and the design records why.

## Rooms — verified against `6d4f43ac0`

```
src/intrinsic/mod.rs        `pub(crate) type ValueHandler` — the type to widen. Its doc comment
                            explains WHY a second slot exists; extend that reasoning, do not
                            replace it.
src/runtime.rs              `pub(crate) fn dispatch_substrate_impl(impl_name, vals)` — the signature,
                            and P1/O-i's arity guard inside it (leave the guard's behaviour alone)
src/runtime.rs:10773        ★ THE ONLY CALLER, inside eval_apply, which HOLDS `list_span`
crates/wat-macros/src/wat_intrinsic.rs   `sniff_args` (stops at the first non-`&WatAST` param — the
                            same trick for a trailing `&Span`) · the ALGEBRA branch that builds the
                            value door and the AST door
19 hand-written value twins — find them with:
    find src -name '*.rs' -exec cat {} + | tr '\n' ' ' \
      | grep -oP '#\[wat_intrinsic\(\s*"\K[^"]+(?="\s*,\s*value\s*=)'
src/intrinsic/vector.rs · map.rs · hashmap.rs · vec.rs · linkedlist.rs · hashset.rs
                            the 38 ALREADY-MIGRATED ALGEBRA fns — they take no span and must keep
                            compiling untouched. That is the proof the trailing span is OPTIONAL.
```

## Blast radius

`src/intrinsic/mod.rs` (the type), `src/runtime.rs` (`dispatch_substrate_impl` + its one caller),
`crates/wat-macros/src/wat_intrinsic.rs` (the sniff + both generated doors), and the 19 hand-written
twins gaining an ignored parameter. **No verb migrates in this stone. No handler's behaviour changes.**

## STOP triggers — each REJECTS. Ship nothing and report the gap.

1. **`dispatch_substrate_impl` has more than one caller.** Measured: exactly one, `runtime.rs:10773`.
   If you find another, the blast radius is wrong — STOP and name it.
2. **An existing ALGEBRA fn must change.** All 38 already-migrated verbs take no span and must keep
   compiling verbatim. If the trailing `&Span` cannot be optional, STOP — a mandatory span would
   mean touching every migrated verb, which is a different stone.
3. **You migrate a verb.** Not this stone. Q makes migration *possible*; O-iv-c and O-iv-d do it.
4. **Any error message or span changes anywhere.** Q threads a span that is currently dropped; it
   does not yet let anyone USE it. Every existing diagnostic must be byte-identical.
5. **You reach for `rust_caller_span!()` to fill a gap.** If a call site has no real span to pass,
   STOP and name it — a synthesized span silently standing in for a real one is the exact defect
   class this arc is built around.

## Acceptance — run each, report the actual output

```
 0. ★ THE 38 MIGRATED VERBS COMPILE UNTOUCHED. `git diff --stat` must show NO change to
    src/intrinsic/{vector,map,hashmap,vec,linkedlist,hashset}.rs. That is STOP-2's positive form
    and it proves the trailing span is optional.

 1. ★ NOTHING'S DIAGNOSTICS MOVED. Re-run every committed probe under wat-scripts/scratch-pad/
    that exercises apply or an arity error:
      255-stone-o-apply-lies-about-what-exists.wat
      255-stone-o-apply-has-three-broken-doors.wat
      255-stone-o-the-value-door-panics-on-arity.wat
      255-stone-o-i-vector-concat-value-door-panic.wat
      255-stone-o-iv-b-collections-sweep-apply.wat
    ALL byte-identical to today. Paste the diffs (expect empty).

 2. ★ A SPAN-TAKING ALGEBRA FN COMPILES AND IS REACHED ON BOTH DOORS. Add ONE throwaway ALGEBRA
    verb taking a trailing `&Span`, show it answers via a direct call AND via apply, then remove it.
    Do not migrate a real verb to prove this.

 3. ★ THE SPAN THAT ARRIVES IS THE CALL'S, NOT A SYNTHETIC ONE. In that throwaway, raise an error
    carrying the received span and show it points at the CALL SITE — different line for two
    different call sites in the same file. A span that is the same for both is `rust_caller_span!()`
    wearing a parameter, and fails this row.

 4. ★ THE 19 HAND-WRITTEN TWINS STILL SERVE apply. Pick three (an i64 arm, an f64 arm, a rational
    arm), call each through apply, identical results to today.

 5. cargo build --release --all-targets — clean.

 6. cargo nextest run --release -E 'test(intrinsic) + test(apply) + binary_id(wat::wat_lang)'
    Summary lines verbatim.
```

## How to work

- Work only in `/home/john/work/holon/wat-rs`. Verify with `pwd` first. Any path containing
  `.claude/worktrees/` is harness state — never operate on it.
- Everything FOREGROUND. Ending your turn ENDS you; nothing wakes you. Land the numbers.
- You may run `cargo build`, `cargo nextest run --release -E '<filter>'`,
  `./target/release/wat --check <file>` and `./target/release/wat <file>`. The orchestrator runs the
  full floor and clippy centrally — leave those two alone.
  ⚠ **`cargo build` is NOT enough on a doc-comment change**: the last stone shipped ten
  `doc list item overindented` clippy errors that `build` and the floor were both blind to. Keep new
  doc prose plainly formatted — no deep-aligned continuation lines in `///` lists.
- You may not spawn sub-agents.
- **No `git stash`, in any form.** `git show HEAD:<path>` for a pre-image.
- Do not commit, push, revert, or create a worktree. Leave the tree dirty.

## Report back with

Row-by-row: the command, its actual output, PASS/FAIL. Then the honest deltas. Every rider on this
chain has caught a real defect in an orchestrator brief — a refuted opening premise, a corrected
census, a stone about to ship its own deliverable unreadable. If a claim here does not survive
contact with the disk, that is the most useful thing you can hand back.
