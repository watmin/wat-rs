# BRIEF — STONE N RELAND 1: the generated `::Op::` send, and the tool that cannot read

> Stone N largely LANDED. Read its SCORE first. This closes one residue, and the residue has a
> bootstrap trap wrapped around it.

## WHERE IT STANDS — measured centrally, not reported

```
floor   M:  2447 passed · 2773 failed · 18 TIMED OUT
        N:  4765 passed ·  473 failed ·  0 timed out      <- the spawn path WORKS again
corpus  bare spelling 2039 -> 91, and the 91 are comments + historical wat-fix scripts
stdlib  freezes with 0 type-check errors
probe   5/5 green
```

**Every one of the 473 is the same reason** — `positional variant construction is retired` — in two
populations:

```
117  :wat::kernel::StdOut::Op::Write        ┐ GENERATED ::Op:: client sends
 31  :wat::query::Store::Op::EnsureSchema   │
 27  :wat::kernel::StdIn::Op::…             ┘
 38  :probe::Echo::EchoResponse::Ok         ┐ test-file enums the wrap did not reach
 36  :probe::Outcome::{Stopped,Message,     │ (SCORE reports one crash-resume mid-run)
     Lost,Closed} ×4                        ┘
111  :wat::core::match                        downstream of the above
```

## ★★★ THE CRITICAL PATH IS ONE SITE, AND IT UNBLOCKS THE TOOL

`readln` routes through `StdIn::Op::ReadFrame`. **The rename driver reads its worklist with
`readln`.** So the codemod cannot be re-run until the generated `::Op::` construction is fixed —
`wat/fix.wat:23`'s chicken-and-egg, one layer deeper. Fix that, rebuild, and the tool can finish the
`:probe::` population itself.

## ⛔ FIRST ACT: `macroexpand`. NOT grep.

**CLAUDE.md rule 4** — *"Debugging a MACRO? READ THE EXPANDED FORM FIRST. A confusing error from a
`defservice`/`defsurface` form is almost never a mystery in the macro's logic; it is something the
macro EMITTED that you have not looked at."*

★ **Worked example, and it is the orchestrator's, from one hour ago:** I spent six commands grepping
`wat/service.wat` for the site. What I established by grep is only that it is NOT there —
`op-variant-kw` has exactly ONE construction, `service.wat:2080`, and it already emits the map form
`(~op-variant-kw {:req req})`. The binary is newer than both `service.wat` and `stdio.wat` (not a
stale freeze), and the refusal message is not a mislabelled catch-all (four distinct reasons,
measured). **Expand the form and read what was emitted.**

⚠ **AND IT MAY NOT BE IN WAT AT ALL.** `src/types.rs:2563 synthesize_surface_protocol` builds the
`::Op` enum in RUST (`:2990  let op_name = format!("{}::Op", surface.name);`). If the CONSTRUCTION
is likewise Rust-emitted, **no `.wat` codemod can ever reach it and a `src/` change is required.**
That is authorized — see STOP-2.

## THE WORK

1. **Expand** a `defsurface`-with-`:features` (StdOut is the smallest) and find where
   `<Surface>::Op::<Op>` is CONSTRUCTED. Report the site before changing it.
2. **Fix it to the map form** — in `.wat` if it is a template, in `src/` if it is Rust codegen.
3. **Rebuild**, and confirm `readln` works: a program that reads stdin must not die.
4. **Re-run the codemod** over the remaining worklist (the `:probe::` population and anything the
   crash-resume skipped), then **re-run it a second time** — idempotence went unmeasured twice now.
5. Report the floor's delta per population, not a total.

## STOP TRIGGERS

- **STOP-1 — grep is used as the diagnostic.** Expand the form. The orchestrator already spent six
  commands proving grep cannot find this.
- **STOP-2 — a `src/` change is AUTHORIZED here, and scope-limited.** If the construction is
  Rust-emitted, change it. Do NOT work around it in wat, and do NOT soften the refusal — the wall is
  correct and every one of the 473 is a real site. Touch only what emits the construction.
- **STOP-3 — the wall is weakened to reduce the count.** The 473 are the worklist, not the problem.
- **STOP-4 — idempotence goes unmeasured a third time.** Run the codemod twice; the second run must
  change nothing.
- **STOP-5 — the `:probe::` population is hand-edited.** R21. It is the codemod's, once the tool can
  read again.
