# NOTE — a macro-generated program does not survive a source round-trip

> **Found 2026-07-27**, arc 170 execve step 2d, by the floor. STOP-4 of
> `BRIEF-execve-step2-stream-the-program.md` fired exactly as written:
> *"if any real service's forms fail to survive `read-string(ast->source(f))`,
> STOP and report the form."*
>
> This blocks the execve design as drawn. It is not a bug in the boot wire —
> steps 2a–2c are green and banked (`425d7624`, `aa5910e6`).

## The finding

`spawn-program (process)` may be handed forms that were **produced by a macro**,
and those forms carry **hygiene scopes**. Rendering them to source text and
re-parsing produces fresh nodes with no scope, so a reference and its binder stop
matching.

Three floor tests said so, all of them real consumers:

```
probe_arc170_gapj_each_kwargs::each_with_kwargs_tail_fires_every_side_effect_and_returns_nil
probe_arc170_c2_strike1_mixed::c2_strike1_mixed_7_services_5_data_runs_end_to_end
probe_arc170_c2_mixed_macro::mixed_via_macro_runs
```

The diagnostic is exact, and it is the checker naming its own invariant:

> `HygieneScopeDivergence`: reference `kwargs` (scope `{}`) is unbound, but a
> binder `kwargs` exists under a different scope `{952}` — a macro rebuilt this
> binder from its name instead of reusing the node; reuse the original AST node.
> — at `<child>` line 26, from `wat/bracket.wat:666` `collect-loop`

The child is the "macro" in that sentence: parsing rendered text *is* rebuilding a
binder from its name.

## Why the step-2c oracle did not catch it

**It was vacuous in the dimension that mattered, and that is the lesson.**

2c had the child compare the streamed source against `forms_to_source(&forms)`
computed from the forms it inherited. Both sides ran the same printer over the
same in-memory forms, so the strings were equal **by construction**. It proved the
*transport* faithful — bytes in equal bytes out — and could not have detected a
lossy *round trip*, because the parent never re-parsed.

The equality that needed proving was never tested:

```
   tested:      render(forms)          ==  render(forms)      ← trivially true
   needed:      parse(render(forms))   ==  forms              ← the actual claim
```

An oracle that compares a thing to itself is not an oracle. Any future
"is the new path faithful?" check must run the **whole** conversion and compare to
the original, not compare two renderings of the same input.

## What it means for the design

`DESIGN-execve-every-fork.md` says the program crosses as **source text**, on the
grounds that a `Vec<WatAST>` cannot survive `execve`. Both halves are still true —
and together they are now a contradiction for macro-generated programs:

- text loses hygiene scopes
- the address space that holds the scopes does not survive exec

So the program must cross in a form that **carries hygiene**. Three shapes, none
of them chosen:

1. **Encode the AST, not the source.** Cross as EDN-encoded `WatAST` with scopes
   preserved, and reconstruct the nodes rather than re-parse them. Keeps hygiene
   by definition; costs a faithful AST codec, and the `:wat.core/fn` vs
   `:wat::core::fn` problem is why `ast->source` exists in the first place — so
   this needs its own grounding, not an assumption.
2. **Render hygiene into the source.** A scope-annotated form syntax that the
   reader restores. Keeps text as the wire; costs a syntax extension that every
   reader must honour.
3. **Resolve hygiene before crossing.** Rename scoped binders to unique names so
   the text is unambiguous without scope metadata. Cheapest wire; changes the
   program's identity, and whether that is observable needs proving.

## Status and what stands

**OPEN — the execve stone is blocked here.** Steps 2a–2c are banked and green:
the boot wire (derived, registered, guarded), the transport (chunked, acked,
named EOF), and the handshake wired through both fork sides with a real child
booting over it. What 2d proved is that the *payload* is wrong, not the pipe.

The probe (`wat-scripts/scratch-pad/probe-execve-argv-cow-leak.wat`) remains RED
at `2` / `2`, which is correct: nothing has closed the leak yet.
