# STONE P6-c-1 — the macro honours the tail the handler DECLARES

> Not a wave. A generator stone that decides how big the P6-c campaign is.
> Read `NOTE-p6c-is-a-campaign-not-a-stone.md` and P6-c-0's committed instrument first.

## The measurement that drew this

P6-c-0's census: 129 of 148 giant-match FQDNs are NEEDS-SHAPE. Bucketed by the tail each arm
**actually passes** (`wat-scripts/hunt/p6c-disposition-census.py --json`):

```
100   (list_span, env, sym)          pure ORDER
 14   arity < 3                      the handler declares only SOME context params
  4   (sym, list_span)               subset AND order
────  118 of 129 = "honour the declared tail"
  1   (env, sym, list_span.clone())  eval_apply takes an OWNED Span — by-value, NOT order
~10   extra non-context args         a shared helper parameterised per verb — genuinely per-arm
```

## ★ THE CONTRACT DECISION — and it is a REMOVAL, not a liberty

**The macro emits the context tail in the order the handler DECLARES, passing only the params it
declares.** Derived, not chosen, from a contradiction already in the macro:

```
NativeHandler       fn(&[WatAST], &Span, &Environment, &SymbolTable)   (args, span, env, sym)
                                                                       src/intrinsic/mod.rs:162
the emitted shim    fn(args, list_span, env, sym)                      wat_intrinsic.rs:924-928
the runtime's arms  eval_x(args, list_span, env, sym)                  100 of them
the handler CALL    #fn_name(args, env, sym, list_span)                :726 — THE ONLY OUTLIER
```

**The registry's own handler type, the generated shim, and the hundred arms all agree.** The macro
takes that shape in and then *permutes* it to call the handler. Nothing derives that permutation —
it is an assumption the emit makes about something the handler already declares by its parameter
types, and `sniff_args` already walks those types and throws the information away (`seen_context`
is a bool).

So this stone **deletes an assumption**; it does not widen a rule. Same shape as Stone P7 — a
classifier too narrow for a declaration that was already there — and stronger, because here a
canonical order already exists to derive from.

## The work — part 1, THE MACRO ALONE (the control)

`sniff_args` records the context tail as a sequence (which of `&Environment` / `&SymbolTable` /
`&Span` appeared, in declared order); `emit`'s BINDING arm passes exactly those, in that order.

★ **Then `cargo build --release --all-targets`, and the whole tree must compile UNCHANGED.** All 380
existing handlers declare `(env, sym, span)`, which the new rule still honours — so a green build
here is the control proving the change is a superset. **If anything breaks, STOP.**

## The work — part 2, prove it by homing TWO verbs with ZERO signature change

Not a wave. Two, one of each shape, chosen because neither is multi-site:

```
ORDER    :wat::form::matches?      eval_form_matches(args, list_span, env, sym)   runtime.rs:5599
SUBSET   :wat::program::cpu-count  eval_program_cpu_count(args, list_span)        runtime.rs:5590
```

Register each as `#[wat_intrinsic]` and delete its arm. **The callee's `fn` signature line must not
change** — that is the whole claim. `git diff` on the callee must show the attribute and doc added
and *no parameter list edited*.

⚠ Homing needs a real `///` doc block (`@added`, `@ret`, ≥1 `@example`/`@example-norun`, purity,
determinism, category) and a declared arity — that is the H-1a/H-1b treatment and it is per-verb
work. Two is the proof; the waves are not yours.

## The work — part 3, the instrument must stop lying

`p6c-disposition-census.py` classifies against the OLD hardcoded order. After part 1 it would report
118 false NEEDS-SHAPEs. **Update it to the new rule, re-run it, and report the new distribution** —
how many of the 148 are now INTRINSIC-READY. A committed instrument that outlived the rule it
encodes is worse than none. `[[feedback_an_instrument_must_outlive_the_number_it_produced]]`

## STOP triggers — each REJECTS. Ship nothing on that row and report.

1. **Part 1 breaks any existing handler.** The change is not a superset; report it rather than
   special-casing.
2. **A behaviour changes anywhere.** Same handler, same arguments, same order of evaluation — only
   the generated call site moves. Any diagnostic or value that differs is a finding.
3. **The interaction with Stone P7 bites.** P7 made a fn whose ONLY param is `&Span` an ALGEBRA
   handler. A BINDING handler wanting a span-only tail must therefore still lead with
   `&[WatAST]`/`&WatAST`. If you find a shape where the two rules disagree, STOP and report it —
   do not resolve it by tie-break.
4. **You find yourself editing a callee's parameter list.** That is the thing this stone exists to
   make unnecessary. If a verb needs it, that verb is not one of the two — say so.

## Acceptance

```
 0. ★ CONFIRM THE DERIVATION YOURSELF before changing anything: NativeHandler (mod.rs:162), the
      emitted shim (wat_intrinsic.rs:924-928) and the runtime arms all read (args, span, env, sym);
      the handler call at :726 does not. Quote all four. If they do not say that, STOP — the stone
      rests on it.
 1. ★ PART 1 ALONE COMPILES THE WHOLE TREE UNCHANGED. Report the command and result before part 2.
 2. ★ BOTH SHAPES NOW WORK, proven by a handler that declares each:
        a. a tail declared (span, env, sym)  — the 100-arm shape
        b. a tail declared (span) only       — the subset shape
 3. ★ AND THE ILLEGAL SHAPE IS STILL REFUSED. A BINDING handler declaring a context param that is
      none of env/sym/span must still be rejected with a real message. Paste it.
 4. ★ TWO VERBS HOMED, CALLEE SIGNATURES UNTOUCHED. `git diff` on each callee shows no parameter
      list changed. Both reachable and correct after: call each, before and after.
 5. ★ THE INSTRUMENT UPDATED AND RE-RUN. New label distribution over the same 148, reported next to
      the old one. Say how many became INTRINSIC-READY.
 6. ★ REGISTRY POPULATION +2 (380 → 382) and the giant match −2 arms.
 7. cargo build --release --all-targets — clean; report any warning VERBATIM.
 8. cargo nextest run --release -E 'test(intrinsic) + test(reflection) + test(program) + test(form)'
```

## How to work

- Work only in `/home/john/work/holon/wat-rs`. `pwd` first. Never operate on a `.claude/worktrees/` path.
- **Everything FOREGROUND. Ending your turn ENDS you** — nothing wakes you, no notification is coming.
- **You may not spawn sub-agents.**
- `cargo build` and scoped `cargo nextest` are yours; the full floor and clippy are the orchestrator's.
- No `git stash`, in any form. Do not commit, push, revert, or create a worktree.
- New scratch `.wat` → `wat-scripts/scratch-pad/`, `--check` clean.

## Report back with

The four quoted signatures from row 0. Part 1's build result, before part 2. Both shape proofs and
the refusal message. The two `git diff`s showing untouched parameter lists. The old and new
instrument distributions side by side. Then the honest deltas — especially anything about the P7
interaction, and how many of the ~10 extra-arg arms you now think are genuinely per-arm.
