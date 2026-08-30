# STONE — ONE PARAM-SPEC 4: ANNIHILATE THE PROSE

> **Builder's ruling:** *"annihilate this — do not educate any reader of this form — the codemod
> comments can exist as a statement of what you may not say."*
>
> The last stone. The wall stands; nothing can *run* the retired form. This removes everywhere the
> tree still *teaches* it.

## The population — and it is bigger than the last count

```
180   .rs sites across the 8 substrate parametric heads
        132  code / embedded wat strings   ← FLOOR-GATED (see below)
         20  ///  published doc comments
         11  //!  module docs
         17  //   internal comments        ← the 48 comment sites have NO oracle
118 of the 180 are in src/runtime.rs alone.
```

⚠ **Stone 3's report said 172. It is 180.** That count used three heads (`Vector|HashSet|HashMap`);
`Option`, `Result`, `PersistentMap`, `PersistentVector` and `Tuple` carry eight more. **This is the
same failure that cost stone 1 a 178-site undercount** — a head list narrower than the population.
**Your census governs; widen past my eight heads and report what you find.**

## ★ THE ONE DISCRIMINATOR — the builder's ruling, made operational

**A site that shows the form as NORMAL USAGE dies. A site whose subject IS the prohibition may
keep it.**

```
KEEP   src/check.rs:12515  row1_infer_list_constructor_bare_keyword_first_arg_now_rejected
KEEP   src/check.rs:12653  the HashSet twin
       — stone 3's own rejection tests. They must CONSTRUCT the bare form to prove it is refused.
       This is the "statement of what you may not say" class.
DIE    everything else: doc comments, module docs, internal comments, test fixtures,
       embedded wat strings that merely USE the form.
```

⛔ **If you cannot tell which class a site is in, it is a STOP.** Do not guess. A rewritten
rejection test silently stops testing the rejection — and the floor stays green, because the form is
still rejected for a different reason.

## ★ THE ORACLE IS UNEVEN, AND YOU MUST KNOW WHICH HALF YOU ARE IN

- **The 132 code/string sites are FLOOR-GATED.** They are evaluated; a mangled rewrite breaks a
  test. The canonical form works end-to-end (verified: `(:wat::core::Vector :- [:wat::core::i64]
  1 2 3)` → `[1 2 3]`), so a correct rewrite stays green and a wrong one goes red.
- **The 48 comment sites have NO ORACLE AT ALL.** Nothing reads them. A wrong edit is invisible
  forever. **Read each one against the code it describes** — which is exactly the discipline that
  found seven doc lies in ten verbs earlier in this session.

## ★ AND AT LEAST ONE OF THEM IS NOW FACTUALLY FALSE

```
src/runtime.rs:3373  /// `{k v ...}` as `(:wat::core::HashMap :wat::type::Infer :wat::type::Infer k v ...)`.
src/runtime.rs:3374  /// An empty `{}` renders as `(:wat::core::HashMap :wat::type::Infer :wat::type::Infer)`
```

**Stone 3 moved that synthesis to `:- [...]`.** This doc does not merely use a retired spelling — it
now describes behaviour the substrate no longer has. **Expect more of these**: any comment
describing the desugar, the ctor's first argument, or `unwrap_type_param_bracket`'s two arms may
have been made false by the wall. **Report every one you find as a lie, not a style fix.**

## STOP triggers — each REJECTS.

1. **A site you cannot classify** as normal-usage vs prohibition-subject.
2. **A comment you cannot check against its code.** Report it rather than rewriting prose you have
   not verified.
3. **A `.wat` file needs editing.** Out of scope — the corpus is clean and its codemod headers are
   blessed by the ruling.
4. **The floor goes red and the cause is not obviously your rewrite.** Report before continuing.

## Acceptance

```
 0. ★ YOUR OWN CENSUS, widened past my 8 heads. Disagreement with 180 is a finding — the last two
      counts here were both wrong (172 by head-narrowing, 178 on stone 1 the same way).
 1. ★ THE KEEP-LIST, named before you edit: every site whose subject is the prohibition.
 2. ★ ALL OTHERS REWRITTEN. Grouped by class (published doc / module doc / internal / code+string).
 3. ★ EVERY COMMENT READ AGAINST ITS CODE — and every one the wall made FALSE reported separately,
      with before/after. `runtime.rs:3373-3374` is the known first; find the rest.
 4. ★ THE REJECTION TESTS STILL REJECT — run them by name and show they pass for the RIGHT reason
      (the bare form, not a mangled one).
 5. ★ Full floor green. Any red traced to a rewrite, fixed; any red not yours, reported.
 6. ★ AFTER: your row-0 census returns only the keep-list.
 7. cargo build --release --all-targets — clean; warnings VERBATIM.
 8. cargo nextest run --release — 0 failed.
```

## How to work

- Work only in `/home/john/work/holon/wat-rs`. `pwd` first. Never a `.claude/worktrees/` path.
- **Everything FOREGROUND. Ending your turn ENDS you** — nothing wakes you, no notification is coming.
- **You may not spawn sub-agents.** Clippy is the orchestrator's; the floor is yours (row 8).
- No `git stash`. Do not commit, push, revert, or create a worktree.

## Report back with

Your census and its command. The keep-list. The rewrites grouped by class. **Every comment the wall
made false, with before/after.** The rejection-test proof. The floor. Then the honest deltas —
especially any site you could not classify.
