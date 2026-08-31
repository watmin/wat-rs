# BRIEF — excursus 002 stone 1: a Peer may not escape the scope that created its Handle

Make the checker reject a `Peer` that escapes a scope which created that service's `Handle` via
`<svc>/start`. Two escape sites this stone: the value of a `let` (1a) and the return of a function
(1b). A peer that escapes with the handle still owned by the CALLER is untouched — that is every
`conn`/`dial` helper in the corpus and it must keep compiling.

Read `DESIGN.md` beside this file first; it carries the rule, the measured blast radius, and one
collision you will hit.

## Read in order, and why you are being sent there

1. **`wat-scripts/scratch-pad/probe-handle-to-surface-relation.wat`** — the disconfirming probe,
   already green. It proves the checker can derive a service's surface types from a `Handle`, and
   it carries BOTH acceptance shapes side by side:
   `:hs::conn-is-safe-the-caller-owns-the-handle` (must keep compiling) and
   `:hs::dial-and-drop-is-the-real-escape` (must stop compiling). These are your targets.
2. **`src/check.rs:1805-1822`** — case 1b's site. `let body_ty = infer(body_ast, env, &locals, …)`
   then `assignable(&body_ty, &scheme.ret, …)`. `locals` (the params) and `scheme.ret` are
   co-present here; this is the exact site that produced the probe's `ReturnTypeMismatch` naming a
   param type and a return type together, so the information you need is in hand.
3. **`src/check.rs:7749-7810`** — `infer_let` and `process_let_binding`, building `cumulative`.
   Case 1a's site: at the point the body is inferred, `cumulative` holds the bindings.
4. **`src/check.rs:7579-7640`** — `MUST_USE_TYPES` / `MUST_USE_PARAMETRIC_HEADS` and
   `is_must_use_type`. The exemplar for a hardcoded checker wall in this codebase: how the list is
   documented, why it is hardcoded rather than a type-def field, and the comment register expected.
5. **`wat/service.wat:1021`** (`addr-ty`) and **`:2873`/`:2877`** (`handle-fields`,
   `handle-record`) — how a Handle carries its service's `(Address :- [Op Reply T])`. Useful if you
   need the Handle→surface direction; note the probe's error shows a Handle is PARAMETRIC
   (`(:c2::alpha::Handle :- [:wat::kernel::Shared])`), so match on the HEAD, never a bare path, or
   you will miss every handle in the corpus.

## Sketch — the shape, not the code

At each site: (a) collect the services CREATED in this scope — the `<svc>/start` calls; (b) ask
whether the escaping type contains a `(Peer :- [S::Op S::Reply])` whose surface is satisfied by one
of them; (c) if so, raise the new error naming the creating call's span and the escape's span.

Relate peer→service through the SURFACE (`:app::Alpha::Op` names surface `:app::Alpha`;
`:app::alpha/start` is a service that `:satisfies :app::Alpha`), not by string-matching FQDNs.

## Blast radius

`src/check.rs`, plus the one file that owns `CheckErrorKind` for the new variant. **No runtime
change**: `src/runtime.rs`, `eval_let_tail` and the trampoline are out of bounds. No change to
`LociDiedError` or the severed sentinel. No `.wat` corpus edits except the rune in STOP-3.

## Measured targets — verify these, do not take them from this brief

The census says the wall must reject exactly **2** sites and accept **16**. Re-run it yourself:

```
grep -rn --include=*.wat -E '\-> \(:wat::kernel::Peer' tests/ wat-scripts/ wat/ wat-tests/ examples/
```

The 16 include all three stdlib `stdio-connect-{out,err,in}`. If your wall rejects any of those,
the rule has been keyed on the parameter instead of on creation — re-read DESIGN.md's table.

## STOP triggers

**STOP-1** — if a scope's `<svc>/start` calls cannot be collected at either site, STOP and report
what is missing. Do not approximate with a name-shaped heuristic (e.g. "any call whose FQDN ends in
`/start`") — a heuristic wall that is right most of the time is worse than none.

**STOP-2** — if the wall rejects ANY of the 16 safe sites, STOP. That is the rule being wrong, not
the corpus. Report which site and what type it saw.

**STOP-3 — the collision, and it is the one that will surprise you.**
`tests/services/probe_severed_reaches_the_client.wat:68` (`:sev::dial-and-drop`) is the subject of
a live floor gate proving an owner-drop reaches the client as `Severed`. **The wall stops it
compiling.** That gate must keep working: give it a `rune:` exemption naming this wall and stating
why the construction is deliberate. If you find yourself weakening the rule or deleting the gate to
get green, STOP — that trades a proof for a green build, and the gate is the only thing standing
between a future regression and a silent mute.

**STOP-4** — case 2 (a peer leaving via a TAIL CALL) is NOT this stone. The checker has no notion
of tail position. If you find yourself adding one, STOP: that is stone 2 and it is undrawn.

## Prior comparable result to copy for shape

`docs/excursus/2026/08/001-sns-sqs/SCORE-stone-5-surface-guard-reach.md` — the same kind of work
(widening a checker guard's reach), including how the SCORE reports a deliberately-red floor and
keeps its ARM.
