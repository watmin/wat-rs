# DESIGN-STONE — the surface speaks at expand time (kill the guess AND keep the check)

> **Status: DRAWN 2026-08-04. ⛔ CONDITIONAL — the mechanism is UNPROVEN.** Step 0 is a
> disconfirming probe and it gates everything below it. If the probe fails, this stone dies and the
> runtime constant stays; that outcome is a result, not a setback.
>
> Ordering — the one thing that could have killed it outright — is **MEASURED and clear** (§ Ordering).

## What happened, and why a fix needs a second pass

`defservice`'s codegen named an op's `RequestTooLarge` constructor by **concatenation** —
`<proto-base>::<OpPascal>Response::RequestTooLarge` — never by reading the type the author declared.
Every service in the corpus happens to name its response `<OpPascal>Response`, so the guess was right
by luck everywhere it was reached. One scratch probe named it otherwise and the loader gate caught it
(R64 `QVOD TVEBAMVR, NOS TVETVR`).

The cure shipped as a Rust-emitted runtime constant — `(def :<S>::<OP>-RESPONSE-TYPE "<base>")` from
`member.ret`, a sibling of `build_op_budget_constants`. **The naming defect is genuinely gone.** But
it cost something:

| | before | after the constant |
|---|---|---|
| name source | a guess | `member.ret` — correct |
| in generated source | a **literal keyword** | a runtime `String` + an EDN decode |
| loader can check it | **yes** | **no** — no literal survives to check |
| per-op facts crossing to wat | 1 constant | 2 constants |

**A compile-time catch became a runtime one.** Honest qualifier, because it is smaller than it
sounds: the old check verified *our guess*, not the contract — where the guess was right it confirmed
that a correctly-guessed name exists, which is worth little. We lost a guess-check, not a
contract-check. Still a loss, and the constant-per-fact trend is the other half of the cost.

## The claim: nothing here is unknowable at compile time

The response type is a **token in the source** — `(op-name [params] -> :Ret …)` in the surface's
`:features`. Rust reads it at *registration* (`SurfaceMember::Method { ret, .. }`; that is exactly
what Path B and the constant both use). Registration is one phase **after** macro expansion, which is
where `op-methods` needs it.

So the gap is **a missing channel, not missing information.**

## ★ The channel already exists and already carries this exact traffic

Grounded 2026-08-04, by reading `src/macros/expand.rs`:

- `MacroRegistry` is threaded **`&mut`** through `expand_all_with` and every helper
  (`:25`, `:34`, `:111`, `:214`, `:302`).
- **State survives form-to-form during expansion:** `registry.register(def, …)` at `:116` and `:327`;
  `registry.get(head)` / `registry.contains(head)` at `:365`, `:492`, `:515`. A defmacro registered
  while expanding one top-level form is looked up while expanding a later one. **That is a
  form-to-form channel, already load-bearing.**
- **A `defsurface` is already destructured at expand time** — `hoist_surface_messages`
  (`:212-233`) walks its `:messages` vector. That is the write site, already open.

**Precedent, verbatim from the record:** the reserved-prefix fix was *"a flag on `MacroRegistry`
(already threaded everywhere by `&mut`) … **6 lines, ZERO call-site cascade** … The flag rides the
reference already in scope — that's the whole lesson."* Its **first attempt threaded a bool through
~12 call sites and was reverted.** Do not repeat that shape.

## The build

1. At `defsurface` expansion, record `surface → { op → declared response type }` on the registry.
2. At `defservice` expansion, `op-methods` and `serve-op-arms` look the surface up by its
   `:satisfies` name and **splice a literal keyword** for the ctor.
3. The loader sees a literal again — and it is now **correct-by-construction**, not
   correct-by-lucky-guess. Strictly better than the state before the bug was found.
4. **`build_op_response_type_constants` and its call site are DELETED**, along with the EDN-decode
   workaround in both wat sites (a computed keyword cannot be a call head — that constraint
   disappears once the name is literal again).

## ⛔ STEP 0 — THE DISCONFIRMING PROBE, AND IT GATES EVERYTHING

**Do not build any of the above until this passes.** Ten lines: a `defsurface` that causes an entry
to be written to the registry during its expansion, and a `defservice` **after** it that reads the
entry back during *its* expansion. Assert the read succeeds and carries the right value.

**If the read comes back empty, STOP — the stone is dead and the constant stays.** The whole design
rests on one unverified claim (that a defsurface's expansion-time write is visible to a later
defservice's expansion), and everything else here is already grounded. Prove the one thing.

*Orchestrator's note: this probe is owed BY ME, before any brief is written. It is the exact shape
`FM 2-bis` exists to force, and this arc has paid for skipping it.*

## § Ordering — MEASURED 2026-08-04, and it is not a blocker

The only condition that breaks a registry channel is a service expanded **before** its surface.

- **Same file (~120 services):** an anchored scan (declaration lines only, `^(` at column 0, matching
  each `defservice`'s own `:satisfies` target against that file's `defsurface` line) found **zero
  violations** — the surface always precedes its service.
- **Cross-file: exactly 8, all stdlib**, and all with a **contractual** load position:
  `wat/telemetry/{span,journal}.wat` ← `wat/telemetry.wat`; `wat/query/{mem,sqlite-store}.wat` ←
  `wat/query.wat`; plus 2 `tests/services/` probes against stdlib surfaces, which load after the
  stdlib by construction. A stdlib file's load position lives in `src/stdlib.rs` and is gated by
  `:wat::deporder::verify-stdlib`.

*(Kept visible: a first, unanchored version of this scan reported ten "SERVICE-FIRST" violations. All
ten were the pattern matching `:satisfies` inside a COMMENT and `defservice` in prose. Verify a
pattern can only match what you mean before quoting its count — the third time in one session.)*

**So the empty-map case is unreachable today.** That makes the refusal arm cheap and exact rather
than defensive clutter.

## ⛔ STOPs

- **⛔ Do not build before Step 0 passes.**
- **⛔ On an empty map, REFUSE — located, naming the surface and the service. Never fall back to the
  concatenation.** A fallback restores the guess for exactly the case the guess is wrong in, and it
  would be silent. If this arm ever fires it is a load-order bug and must say so.
- **⛔ Do not thread a new parameter through call sites.** Put the map on the reference already
  threaded `&mut`. The invasive version of this was built once and reverted.
- **⛔ Do not remove `<S>::<OP>-MAX-REQUEST-BYTES` in this stone.** It is surface-scoped, so anyone
  holding the surface holds it — it may have readers outside the macro. Folding it into the map is a
  FOLLOW-ON with its own grounding (who reads it?), not a freebie.
- **⛔ Do not let this stone delay the twin fix or the committed test.** `RequestMalformed` at
  `wat/service.wat:1183-1185` still guesses, and the generated paths still have no standing gate.
  Both are needed whichever mechanism supplies the name.

## The four questions

Run against **the current state** (runtime resolution via a constant):

| | |
|---|---|
| **Obvious?** | NO — a reader of `service.wat` sees a string assembled into an EDN tag and decoded; nothing says why a constructor cannot simply be called. |
| **Simple?** | NO — one declared name reaches its use through a Rust emitter, a wat constant, a tag transformation, and a decode. |
| **Honest?** | **Marginal.** It no longer lies about the name. But it moved a check the loader could make into a path nothing exercises, and shipped without the test that would notice. |
| **Good UX?** | NO — a wrong response type is now a runtime `unknown tag`, not a located compile error. |

And against **the proposal**: obvious (the macro reads what the author declared), simple (one channel,
one source, literal output), honest (the loader checks a real name again), good UX (a located error at
the earliest possible moment). Conditional entirely on Step 0.

## What this is really about

Two per-op facts now cross from a surface to wat, each by its own hand-built carrier. A third would
be a third. **The map is the channel those carriers are approximating** — and the same one the
builder reached for unprompted: *"so we need to hoist up the features?"* The instinct was right; the
existing `hoist_surface_messages` was the wrong lever, because it hoists declarations for
registration rather than handing anything to a later form. This is that instinct pointed at the
mechanism that actually carries form-to-form state.
