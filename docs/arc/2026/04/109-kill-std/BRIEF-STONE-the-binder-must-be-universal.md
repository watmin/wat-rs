# BRIEF — STONE: the call-site binder must be UNIVERSAL

Ten root-level substrate `eval` forms refuse a call-site param-spec that the CHECKER accepts. Make
them peel it, once, at the one place they all pass through.

Design: `DESIGN-STONE-the-binder-must-be-universal.md` — **read it first**; it carries the measured
reproduction, why the scope is not "generic verbs", and the pinned contract.

## Read in order — every line number verified this session

1. **`src/runtime.rs:6835-6845`** — the dispatch cluster. TEN thin arms, each
   `":wat::eval-…!" => eval_form_…(args, env, sym, list_span)`. The `args` passed down still carry
   the param-spec. **This is where the peel belongs.**
2. **`src/types.rs:4793`** — `peel_param_spec(args) -> (Option<&[WatAST]>, &[WatAST])`. The one door,
   27 callers. `:- []` peels to `Some(&[])`, never `None`.
3. **`src/runtime.rs:28841`** — `eval_form_ast`'s `if args.len() != 1`. The shape of the refusal.
   Four siblings do the same at `:29053`, `:29360`, `:30891`, `:30920`.

## The ten forms — all of them, not the five that are easy to find

```
eval-ast!   eval-with-defs!   eval-step!   eval::walk   eval-edn!
eval-file!  eval-digest!      eval-digest-string!   eval-signed!   eval-signed-string!
```

⚠ **Only five carry a `takes exactly N argument` message.** The other five count differently and a
grep for that phrase will not surface them. Walk the dispatch cluster; do not trust the message.

## Sketch — fill it, do not invent a different shape

Peel ONCE, before the family dispatches:

```rust
// the ten root-level eval forms all pass through here; peel the param-spec so every
// helper below receives a clean `args` — and so form eleven inherits the fix.
":wat::eval-ast!" | ":wat::eval-with-defs!" | ":wat::eval-step!" | ":wat::eval::walk"
| ":wat::eval-edn!" | ":wat::eval-file!" | ":wat::eval-digest!" | ":wat::eval-digest-string!"
| ":wat::eval-signed!" | ":wat::eval-signed-string!" => {
    let (_binder, args) = crate::types::peel_param_spec(args);
    match head { /* the existing ten arms, now on the peeled args */ }
}
```

**Discard the binder at runtime.** It is TYPE information; the checker has already consumed it (the
failing call type-checks clean today). The runtime needs only the value arguments.

⛔ **Do NOT edit the ten helpers.** Peeling per-helper is ten edits, ten chances to miss one, and no
guarantee for the eleventh form. The design pins this: **one peel, one place.**

## The probe you create

`tests/types/probe_stone_binder_is_universal.rs` with a co-located `.wat`. Four rows:

1. **generic, non-empty binder** — `(:wat::eval-ast! :- [:wat::core::i64] <ast>)` evaluates and
   returns a typed `Result`. *This is the load-bearing row; it is the one that fails today.*
2. **non-generic, EMPTY binder** — `(:wat::eval-edn! :- [] "42")` behaves EXACTLY as
   `(:wat::eval-edn! "42")`. `:- []` ≡ absent is arc 109's ruling.
3. **no binder at all** — `(:wat::eval-edn! "42")` unchanged. *The no-regression row.*
4. **a second form** — pick any of the eight you did not use above and repeat row 2, proving the fix
   is structural rather than one-armed.

## Blast radius

`src/runtime.rs` — the dispatch cluster only. Plus your probe and fixture. **No changes to the ten
helpers, to `types.rs`, or to `check.rs`.**

## STOP triggers — rejection criteria. Ship nothing on the row; report it.

1. **The ten arms cannot be grouped** (different signatures, guards, or an arm that is not a simple
   call). Report which, verbatim. Do NOT peel per-helper as a workaround — that is the shape the
   design cut.
2. **`peel_param_spec` is not reachable from that scope**, or its visibility blocks the call. Report
   the exact compiler error. *(Precedent: a `pub(crate)` gap on a sibling helper cost a stone earlier
   in this session — surface it, do not widen anything silently.)*
3. **A non-empty binder on a NON-generic form.** Determine what the CHECKER does with it today and
   **report the behaviour — do not change it.** Whether that should be refused is a language ruling,
   not this stone's.
4. **Any row 1-4 requires touching `check.rs`.** The call already type-checks; if it does not for
   your probe, that is a different defect. Report it.

## Method

Verify with `cargo nextest run --release -E 'test(probe_stone_binder)'` and by running your `.wat`
fixture through `target/release/wat`. Report those numbers. Run everything in the FOREGROUND and
block on it — your turn ends when the numbers are in your hands, not when a command is launched.
Do NOT run the full floor or clippy; the orchestrator runs those centrally.

Do not commit, push, stash, or amend. Leave the git index empty. You may not spawn sub-agents.

## Report

The four probe rows with actual results; `git diff --stat`; confirmation that no helper was edited;
what the checker does with a non-empty binder on a non-generic form; and any surprise.
