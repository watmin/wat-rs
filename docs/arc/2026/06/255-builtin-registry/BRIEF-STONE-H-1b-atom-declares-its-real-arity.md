# STONE H-1b — `atom.rs` declares its real arity

> Same stone as **H-1a**, on the remaining file. Read
> `BRIEF-STONE-H-1a-holon-declares-its-real-arity.md` and the design's **"H-1a SHIPPED"** section
> first — H-1a converted 35 verbs, −542/+235, and its results change what you should expect here.

## The work

`src/intrinsic/holon/atom.rs`: **60 handlers**, all declaring `args: &[WatAST]`, **54** hand-rolling
their own arity check. Convert them to declared fixed parameters and let `#[wat_intrinsic]` generate
the check, exactly as H-1a did for the other four files.

## Row 0 — the six, before any conversion

60 handlers, 54 hand-rolled checks. **Name the six that have none and say why.** My counter tripped
on multi-line `#[wat_intrinsic(` attributes and returned six `)]` fragments instead of six names, so
I do not know the answer and will not guess it. Plausibly 0-arg verbs, plausibly something else.

**STOP-0: if a verb has no arity check and is not 0-arg, say so before converting it** — that is a
missing check, not a conversion candidate, and it is a finding.

## ⚠ EXPECT A DOC-LIE WATERFALL — and here is how to enumerate it in ONE build

**All 60 handlers carry a collapsed `@arg args… …` line.** `doc_arg_ret_types_match_checker_scheme`
compares **per-argument** doc types against the checker's scheme; a collapsed line has no
per-argument type, so **that gate currently verifies nothing for any of these 60.** Splitting them
gives it something to check.

On H-1a this fired **five** times out of 35 — real doc lies that had been unseeable. Expect more
here, because atom is larger and the checker knows most of it.

★ **The gate asserts on the FIRST mismatch, so fixing them one at a time costs one build each.**
Do this instead — it is the technique the orchestrator used on H-1a after three iterations:

```rust
// TEMPORARY, in src/intrinsic/mod.rs's doc_arg_ret_types_match_checker_scheme:
} else if i < scheme.params.len() {
    let scheme_ty = typeexpr_to_doc_string(&scheme.params[i]);
    if ty != scheme_ty.as_str() {
        println!("MISMATCH {} arg {} doc=`{}` checker=`{}`", entry.name, i, ty, scheme_ty);
    }
}
```

```bash
cargo nextest run --release --no-capture -E 'test(doc_arg_ret_types_match_checker_scheme)' | grep '^MISMATCH'
```

One build, the complete list. **Then RESTORE the gate verbatim** (`git diff --stat
src/intrinsic/mod.rs` must be empty) and fix the docs. Report the full list either way — it is a
deliverable, not scaffolding.

⛔ **THE CHECKER IS THE AUTHORITY ON A TYPE'S SPELLING — not the corpus majority.** On H-1a the
orchestrator "fixed" a fn-typed `@arg` to `:wat::core::Fn` because four other sites spell it that
way; the checker said `[:wat::core::f64 :-> :wat::core::bool]` and had been two directories away the
whole time. When a doc and the checker disagree, **the doc moves.**

## STOP triggers — each REJECTS. Ship nothing and report the gap.

1. ⛔ **`eval_holon_from_holon` IS OUT OF SCOPE ENTIRELY — do not touch it.** It is genuinely
   range-arity (1 *or* 3 args), it returns `TrackedValue` and stamps
   `Provenance::RuntimeBuilt { call_span }` (Stone G), and it parses a runtime `-> :T` annotation
   that **arc 258.4 retired language-wide** — Stone P6-a corrected two `if` doc comments the same
   day that still described it. Three separate unsettled questions in one handler. Leave it exactly
   as it is and say you did.
2. **Any other genuinely variadic or range-arity verb.** Keep `args: &[WatAST]` and its hand-rolled
   check — that check is honest. Name each one you leave and why. H-1a found none outside
   `from-holon`; do not assume that holds here.
3. **Any behaviour changes.** Value AND error text, before and after, for every verb converted. The
   generated `ArityMismatch` must match the hand-rolled one it replaces exactly.
4. **A verb's real arity is unclear from its doc and body.** STOP and name it rather than picking.
5. **You leave the temporary gate edit in the tree.** `git diff` on `src/intrinsic/mod.rs` must be
   empty at the end.

## Acceptance — run each, report the actual output

```
 0. ★ THE SIX NAMED. Which handlers have no hand-rolled arity check, and why.

 1. ★ METADATA-OF STOPS SAYING -1. All 60 verbs' `:arity` before and after, via a scratch .wat
    under wat-scripts/scratch-pad/ (`--check` clean). Before: -1 for every one. After: the real N,
    except any STOP-1/STOP-2 verb you correctly left variadic — list those separately.

 2. ★ BEHAVIOUR IS BYTE-IDENTICAL. For every converted verb: a success call where constructible,
    and a WRONG-ARITY call for all of them, before and after, diffed. Build the "before" with
    `git show HEAD:<path>` — never `git stash`. The wrong-arity row is the load-bearing one.

 3. ★ THE FULL DOC-LIE LIST. Every MISMATCH the one-build enumeration printed, with what you
    changed each to. If the list is empty, say so — that is a real and surprising result given
    H-1a found five in a smaller file.

 4. ★ WHAT THE COMPILER SAID ABOUT SPANS. Per verb: did `list_span` become unused? Report both
    lists. For each verb where it is STILL used, one line on what for. ⚠ This is a deliverable —
    it is the input that sizes Stone Q, and H-1a's answer (5 unused of 35) already refuted the
    orchestrator's hypothesis once.

 5. ★ THE HAND-ROLLED CHECKS ARE GONE. `grep -c 'args.len() !=' src/intrinsic/holon/atom.rs`
    before and after. Anything non-zero after must be a verb you named under STOP-1/2.
    ⚠ Check your own added prose does not contain the literal pattern — two riders on this chain
    have tripped their own acceptance grep on their own comments.

 6. cargo build --release --all-targets — clean. Report any warning verbatim.
    ⚠ Expect `clippy::too_many_arguments` on any verb with 5+ wat args (the env/sym/span tail makes
    it 8). H-1a carried one as `#[expect(clippy::too_many_arguments, reason = "…")]` — NOT
    `#[allow]`, so it goes red if the signature ever shrinks under the limit. Do the same.

 7. cargo nextest run --release -E 'test(holon) + binary_id(wat::reflection) + test(intrinsic)'
    Summary verbatim.
```

## How to work

- Work only in `/home/john/work/holon/wat-rs`. Verify with `pwd` first. Any path containing
  `.claude/worktrees/` is harness state — never operate on it.
- Everything FOREGROUND. Ending your turn ENDS you; nothing wakes you. Land the numbers.
- You may run `cargo build`, `cargo nextest run --release -E '<filter>'`,
  `./target/release/wat --check <file>` and `./target/release/wat <file>`. The orchestrator runs the
  full floor and clippy centrally — leave those two alone.
- You may not spawn sub-agents.
- **No `git stash`, in any form.** `git show HEAD:<path>` for a pre-image.
- Do not commit, push, revert, or create a worktree. Leave the tree dirty.

## Report back with

Row-by-row: the command, its actual output, PASS/FAIL. The doc-lie list, the per-verb span table,
and every verb you left variadic with its reason. Then the honest deltas. Every rider on this chain
has caught a real defect in an orchestrator brief; H-1a's rider caught its own false grep signal and
reported it. That is the standard.
