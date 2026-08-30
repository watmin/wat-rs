# STONE P6-c-0 — the disposition census, and its instrument

> Read `NOTE-p6c-is-a-campaign-not-a-stone.md` first. It measured the population (136 arms, 142
> FQDNs) and names the three hazards this census exists to resolve. **This stone moves no arm.**

## The work

Produce a per-FQDN disposition table for every head dispatched by `src/runtime.rs`'s giant match
(`5365-6884`), and **commit the instrument that produced it** to `wat-scripts/hunt/`.

The output is the wave plan for the rest of P6-c. Nothing is homed here.

## ★ THE CONTRACT DECISION — the unit is the FQDN, not the arm

A census that counts **arms in one match** answers the wrong question. `:wat::config::set-redef!`
has TWO dispatch sites — `runtime.rs:2655` (freeze time, mutates `sym.redef_allowed`) and `:5481`
(eval time, a deliberate no-op, because `sym` is immutable there). Both correct; both documented.
**A sweep that saw only the eval arm would home a no-op and leave the behaviour behind.**

So: for each FQDN, find **every** site that dispatches it, across `runtime.rs` and anywhere else.
`[[feedback_a_slot_with_two_implementations_is_two_slots]]`

⚠ **And the preamble is a dispatch site.** `:wat::rete::insert` is served by a pre-match
`if head == …` short-circuit at `:5341`, not by an arm. A line-anchored grep over the match body
cannot see it. P6-a's rider found this and wrote it down; do not re-find it the hard way.

## The four dispositions

```
INTRINSIC-READY   the handler already takes (args: &[WatAST], env, sym, list_span) — registers as-is
NEEDS-SHAPE       a real handler, but its signature does not fit `#[wat_intrinsic]`'s BINDING emit,
                  which passes `env, sym, list_span` UNCONDITIONALLY (wat_intrinsic.rs:726).
                  `:wat::program::env` → `eval_program_env(args, list_span)` is the worked example.
                  Needs the H-1a/H-1b treatment first.
SPECIAL-FORM      takes its arguments UNEVALUATED. `:wat::stream::lazy` says so in place
                  ("a SPECIAL FORM (capture-don't-eval)"); `:wat::holon::literal` is on
                  `eval_apply`'s SPECIAL_FORMS list. Destination is P6-a's
                  `#[wat_special_form_impl]`, NOT `#[wat_intrinsic]`.
MULTI-SITE        dispatched at more than one site or phase. Names every site and which is live.
```

⚠ These are **not** the O-iv four-valued axis. That axis asked *"can `apply` reach it?"*; this one
asks *"where does its implementation belong?"*. A verb can be BINDING (unreachable by `apply`) and
still be perfectly INTRINSIC-READY. **Do not import the other axis's verdicts.**

## STOP triggers — each REJECTS. Ship nothing on that row and report.

1. **A head fits none of the four.** That is a fifth disposition and it is a finding — name it,
   describe it, stop. Do not widen a category to swallow it.
2. **You cannot determine a head's disposition by reading.** Say so and list it UNKNOWN with the
   reason. An UNKNOWN you named is worth more than a guess you classified.
3. **The instrument disagrees with a hand-read control.** The control wins; report both. Build the
   control FIRST — pick five heads across five namespaces, classify them BY HAND, and only then run
   the instrument over them.
4. **You find yourself editing `runtime.rs`.** This stone moves no arm. If a fix seems necessary to
   classify something, report it instead.

## Acceptance

```
 0. ★ THE HAND-READ CONTROL FIRST — five heads, five namespaces, classified by reading, written
      down BEFORE the instrument runs. Then the instrument over the same five. Agreement or not,
      both are reported. [[feedback_impose_the_check_and_read_the_screams]]
 1. ★ THE INSTRUMENT IS COMMITTED to `wat-scripts/hunt/` with a header stating WHAT IT CANNOT SEE.
      A number whose instrument lives in a temp dir is a number nobody can reproduce.
 2. ★ THE ARM COUNT IS RE-DERIVED INDEPENDENTLY. The NOTE says 136 arms / 142 FQDNs and shows the
      three wrong answers it got first. Reach it by your own route; a disagreement is a FINDING and
      mine is not privileged.
 3. ★ EVERY FQDN CARRIES A DISPOSITION or an explicit UNKNOWN-with-reason. No blanks.
 4. ★ EVERY MULTI-SITE FQDN NAMES ALL ITS SITES, with which is live and in which phase. At least
      one exists (`:wat::config::set-redef!`); if you find exactly one, say how you looked.
 5. ★ THE PREAMBLE IS READ. `:wat::rete::insert`'s short-circuit at :5341 appears in your table.
 6. ★ THE WAVE PLAN falls out: namespaces grouped, each with its disposition split and a size.
      Recommend a FIRST wave and say why that one.
 7. cargo build --release --all-targets — clean (you should not have changed anything that builds;
      say so if the diff is docs + hunt only, which is the expected shape).
```

## How to work

- Work only in `/home/john/work/holon/wat-rs`. `pwd` first. Never operate on a `.claude/worktrees/` path.
- **Everything FOREGROUND. Ending your turn ENDS you** — nothing wakes you, no notification is coming.
- **You may not spawn sub-agents.**
- The full floor and clippy are the orchestrator's; do not run them.
- No `git stash`, in any form. Do not commit, push, revert, or create a worktree.
- ⚠ Your own added prose must not contain the literal pattern your instrument greps for.

## Report back with

The hand-read control and the instrument's answer on the same five. Your independent arm count and
how you got it. The full disposition table. Every multi-site FQDN with its sites. The wave plan with
your recommended first wave and the reason. Then the honest deltas — what surprised you, what you
could not classify, and what your instrument cannot see.
