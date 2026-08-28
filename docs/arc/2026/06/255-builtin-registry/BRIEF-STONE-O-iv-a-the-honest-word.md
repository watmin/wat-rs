# STONE O-iv-a — `apply` tells the truth about what it cannot reach

> Read `DESIGN-STONE-O-one-declaration-feeds-both-doors.md`'s **O-iv DECOMPOSED** section first —
> it carries the one contract decision and the measured ripple.

## The work

`:wat::core::apply` reports **"unknown function"** for **331 of 380** registered verbs. They are not
unknown. The registry holds their name, their arity, their doc, their examples. They work when
called directly. What they lack is a value-level door, and *that* is the true statement:

```
(:wat::f64::max-of 3.0 9.0 41.0)                    →  Some [41.0]           ← registered, works
(:wat::core::apply :wat::f64::max-of […])           →  "unknown function"    ← a lie
```

You make `eval_apply` ask the registry before it raises. If `lookup_entry` finds the name, the verb
exists and the error must say what is actually wrong.

★ **This is the same defect as `walk.rs:268` with the sign flipped, and arc 255 exists to kill
both.** The blanket-accept says YES to names that do not exist; `apply` says NO to names that do.
Each answers from a private picture instead of from the registry.

## The ONE contract decision — a NEW variant, already ruled

Mint `RuntimeErrorKind::NotValueDispatchable { name: String }` (name it better if you see a better
one — say so in your report if you change it). **Do not reuse `MalformedForm`.** Step 7 uses that for
special forms, and it is tempting, but the form here is not malformed: the verb is registered, the
call is well-formed, and a reader who sees `MalformedForm` goes hunting a syntax error that is not
there.

⚠ **This message is PERMANENT, not transitional.** 243 handlers take `env`/`sym` — they are BINDING
and can never be splatted, because `apply` holds `Value`s and a BINDING handler consumes `WatAST`s.
The sweep stones (O-iv-b/c/d) shrink the population that hits this path; they never empty it. Write
the message as a statement about a real boundary, not as a "not yet".

## Rooms — verified against `dd5494256`

```
src/runtime.rs:10761    eval_apply step (d)      — `// (d) Nothing found — UnknownFunction`, the raise you gate
src/runtime.rs:10755    eval_apply step (c)      — dispatch_substrate_impl; step (d) is immediately after
src/value/signal.rs:191 pub enum RuntimeErrorKind — where the variant goes; put it next to UnknownFunction
src/value/signal.rs:189 #[derive(wat_edn::ToEdn)] — the EDN wire form is DERIVED; you write no EDN arm
src/value/signal.rs:584 the Display `match self`  — ★ THE ONLY non-exhaustive site; measured, not guessed
src/value/signal.rs:587 the UnknownFunction arm   — copy its shape, including how it uses `prefix`
src/intrinsic/mod.rs:360 lookup_entry(name)        — how you ask whether the name is registered
```

## Implementation sketch

```rust
    // (c) substrate arithmetic / dispatch-impl verbs (pre-evaluated path).
    if let Some(result) = dispatch_substrate_impl(head_kw.as_str(), &combined) {
        return result;
    }

    // (d) Registered, but with no value-level door. Stone O-iv-a — `apply` used to call
    // these "unknown function", which is false: the registry holds the name. A BINDING
    // handler takes `&[WatAST]` and evaluates its own arguments; `apply` has already
    // evaluated its arguments and holds `&[Value]`, so there is no AST left to hand it.
    if crate::intrinsic::registry().lookup_entry(head_kw.as_str()).is_some() {
        return Err(RuntimeError::new(
            list_span,
            RuntimeErrorKind::NotValueDispatchable { name: head_kw.as_str().to_string() },
        ).into());
    }

    // (e) Genuinely not registered anywhere — UnknownFunction, and now it means it.
    Err(RuntimeError::new(list_span, RuntimeErrorKind::UnknownFunction(...)).into())
}
```

Message text — say the true thing and point at what the caller can do:

```
:wat::f64::max-of is registered but cannot be reached through apply:
it takes its arguments unevaluated, and apply has already evaluated them. Call it directly.
```

Read `signal.rs:587`'s `UnknownFunction` arm before writing yours — it shows how `prefix` is used
and how the retirement `remedies_for` lookup is folded into message text. **You do not need
`remedies_for`**: a registered name is not a retired one.

## Blast radius

`src/value/signal.rs` (one variant, one `Display` arm) and `src/runtime.rs` (`eval_apply`'s step (d)
split into (d) and (e)). Nothing else. **No new registry field, no macro change, no intrinsic file
change, no change to `dispatch_substrate_impl`.**

## STOP triggers — each REJECTS. Ship nothing and report the gap.

1. **The new variant needs an arm anywhere other than `signal.rs:584`.** That count was measured by
   imposing a throwaway variant and reading the compiler: exactly one site. If the compiler names a
   second, STOP and report where — the ripple was mis-measured and the design should know.
2. **A genuinely-unknown name stops saying "unknown function".** `(apply :wat::not::a::verb […])`
   must still raise `UnknownFunction`. If your gate swallows it, you have replaced one lie with
   another. This is the row most likely to pass for the wrong reason — prove BOTH branches.
3. **Any verb that WORKS through apply today changes.** The 49 reachable verbs (43 explicit
   `value =` + 6 generated) must be untouched. Step (c) returns before your gate; if you find
   yourself moving it, STOP.
4. **A special form's diagnostic changes.** Step 7 rejects those earlier with `MalformedForm`; your
   gate is after it. If a special form starts reporting `NotValueDispatchable`, STOP.
5. **You reach for `MalformedForm` or widen `UnknownFunction`.** Both were ruled out in the design —
   `UnknownFunction` is a tuple variant deliberately pinned narrow (`signal.rs:587`'s own comment
   says so). If the new variant seems not worth it, STOP and say why rather than reusing.

## Acceptance — run each, report the actual output

```
 0. ★ THE LIE IS GONE, AND THE TRUTH IS IN ITS PLACE.
      ./target/release/wat wat-scripts/scratch-pad/255-stone-o-apply-lies-about-what-exists.wat
    The three rows reading `APPLY=err:unknown function` (`max-of`, `to-uppercase`, `math::sqrt`)
    must now read the new diagnostic. The `:wat::i64::+` and `:wat::vector::length` rows must be
    UNCHANGED — they are reachable and must stay so. Do not edit the probe.

 1. ★ BOTH BRANCHES, PROVEN SEPARATELY — this is STOP-2's positive form.
    Write a scratch .wat under wat-scripts/scratch-pad/ that shows, side by side:
      (apply :wat::f64::max-of […])   -> registered-but-unreachable, the NEW kind
      (apply :wat::not::a::real::verb […]) -> still UnknownFunction, the OLD kind
    Read the EvalError's `kind` field, not just its message prose — a message test cannot tell
    these apart if someone later edits the text.

 2. ★ PROVE THE GATE IS WHAT DID IT. Comment out the `lookup_entry` gate, rebuild, show row 0's
    three rows revert to "unknown function", restore. Confirm the edit LANDED before reading its
    output: a no-op edit prints a meaningless green.

 3. ★ THE 49 REACHABLE VERBS ARE UNTOUCHED.
      ./target/release/wat wat-scripts/scratch-pad/255-stone-o-apply-has-three-broken-doors.wat
    Every row except DOOR2's `(apply max-of [...])` must be byte-identical. That one row changes
    from `ERR` to `ERR` in the probe's own rendering (it prints only "ERR" for any error), so ALSO
    show its underlying kind changed, separately.

 4. ★ SPECIAL FORMS UNCHANGED.
      ./target/release/wat wat-scripts/scratch-pad/255-stone-o-ii-apply-special-forms-still-refused.wat
    Byte-identical — MalformedForm, not the new kind.

 5. ★ THE RIPPLE WAS ONE SITE. Report every file the compiler forced you to touch. If it is more
    than `signal.rs` + `runtime.rs`, that is STOP-1 and a finding.

 6. cargo build --release --all-targets — clean.

 7. cargo nextest run --release -E 'binary_id(wat::wat_lang)' plus any test naming apply or
    unknown-function. Report the Summary lines verbatim.
```

## How to work

- Work only in `/home/john/work/holon/wat-rs`. Verify with `pwd` first. Any path containing
  `.claude/worktrees/` is harness state — never operate on it.
- Everything in the FOREGROUND. Your turn ends when the numbers are in your hands, not when a
  command is launched.
- You may run `cargo build`, `./target/release/wat --check <file>`, `./target/release/wat <file>`,
  and a scoped `cargo nextest run --release -E '<filter>'`. The orchestrator runs the full floor and
  clippy centrally — leave those two alone.
- You may not spawn sub-agents.
- Do not commit, push, stash, revert, or create a worktree. Leave the tree dirty; the orchestrator
  weighs and commits.
- Any new scratch `.wat` goes under `wat-scripts/scratch-pad/` and must `--check` clean — that
  directory is loader-gated.

## Report back with

Row-by-row: the command, its actual output, PASS/FAIL. Then the honest deltas — what surprised you,
what this brief got wrong, what you had to decide that it did not settle. Three riders on this chain
have each caught a real defect in this orchestrator's brief; the last one refuted its opening
premise outright. That is the most useful thing you can hand back.
