# BRIEF — the inert clause is refused

**Read `DESIGN.md` beside this first.** It carries both probe results (the clause is inert in **both**
directions), why probe 1b was deliberately not run, and the contract: **delete the inert declaration, do not
implement the author's intent.**

## The work, in one paragraph

`:deadline-ms` on a `:satisfies`-mode `defservice` is **measured to govern nothing** — not the caller's wait, not
the service's own outbound calls. Refuse it at macroexpansion with a diagnostic naming the caller-side
alternative (`call-by-deadline`), then **delete** the single real declaration in the corpus
(`circuit.wat:2083`), which is behaviour-preserving because it governs nothing, and **file** the latent risk its
comment describes.

## The rooms

| where | why you are going there |
|---|---|
| `wat/service.wat:2442–2449` | where `:deadline-ms` is read: *"Optional … Default 10000. Never off"*, then `deadline-ms-node`. ⭑ Read how the clause-map is consulted; your guard goes where `:satisfies` mode is already known. |
| `wat/service.wat:413–415` | *"A service EITHER mints its own protocol (`:ops`) OR wears a surface's (`:satisfies` + `:impls`). Exactly one of {`:ops`, `:satisfies`}"* — **this is how you test the mode.** |
| `wat/service.wat:889–922` | ⭑ **THE WORDING FAMILY.** The three `:peers` refusals, each a `foldl` + `:wat::core::macro-error` with a concatenated message ending in the remedy. Yours is the **fourth** in this family and should read like a sibling. |
| `wat-scripts/fanout/circuit.wat:2079–2083` | `:fanout::publisher` — the **one** real declaration. ⭑ Read the comment above it; then **delete the `:deadline-ms 120000` line only.** |
| `wat-scripts/scratch-pad/probe-deadline-ms-is-callee-derived.wat` | probe 1: the callee-side inertness, already committed. |
| `wat-scripts/scratch-pad/probe-deadline-ms-governs-nothing-outbound.wat` | probe 2: the outbound inertness, new. ⚠ **Both declare `:deadline-ms` and will now be REFUSED.** They are the evidence for this stone, so they must be adapted — see below. |

## ⛔ The two probes are now illegal, and that is the interesting part

Both existing probes *declare* `:deadline-ms` in `:satisfies` mode, which the refusal forbids. So they cannot
stay as they are.

> **Convert each into a refusal fixture, or drop the clause and keep them as the record of what was measured.**

⭑ **Preferred:** keep their measured output in this stone's SCORE (quoted), and turn **one** of them into the
refusal fixture (it already has the exact shape the refusal must reject). **Do not silently delete the
evidence** — the numbers `10000 vs 300`, twice, are why this stone exists.

## The diagnostic

Mirror the `:peers` family and end with the remedy:

```
<fqdn>: :deadline-ms is declared but governs nothing in :satisfies mode (measured: neither the caller's
wait nor this service's own outbound calls) — put the deadline at the CALL SITE with
(:wat::service::call-by-deadline peer op <ms> <fallback>), or drop the clause.
```

⚠ Check `macro-error`'s call shape and the string-concat idiom against `:889–922` rather than trusting that
snippet. **My sketches have been wrong seven times in this campaign**, once causing the exact trap-door I had
ranked #1.

## ⭑⭑ The proof

| what | expected |
|---|---|
| ⭑ the refusal fires | a `#wat.macro/MalformedTemplate` naming the fqdn, `:satisfies`, and `call-by-deadline` |
| ⭑ `circuit.wat` compiles again after the deletion | `--check` exit **0** |
| ⛔ behaviour unchanged by the deletion | happy `distinct=8000;dup=0`; the publisher still completes its join |
| the `:ops` path is untouched | an `:ops`-mode service with `:deadline-ms` still compiles (**or** say you could not build one cheaply) |

## Verify

- `./scripts/floor.sh`, **Summary line** → green. ⚠ The count is **5244** and may move; state the new number.
- ⛔ **Never a piped exit code** (a type-error run this session reported `$?` = 0 through `| head`; true exit 3).
- `--check` all three: `sqs.wat`, `circuit.wat`, `sns-fanout.wat` → exit 0.
- `cargo nextest run --release --no-run`; `cargo clippy --release --workspace --all-targets -- -D warnings` → 0.
- Happy `circuit.wat 2000 4 3 8192 true 1000` → `distinct=8000;dup=0`; chaos `… 0 0 7 0 0 0 0 0 500` → exit 0.
- `git diff --stat -- src/` **EMPTY**.
- Everything heavy through `./scripts/capped.sh --limit 8g`.

## STOP triggers

1. **STOP-1 — do NOT convert the publisher's caller to `call-by-deadline`.** Trap-door 2: the fix for
   `circuit.wat` is a **deletion**. Implementing the author's intent is a behaviour change owing its own
   measurement, and the FINDING is the handoff.
2. **STOP-2 — do not refuse it in `:ops` mode.** Unmeasured, 0 corpus sites. If the guard cannot be scoped to
   `:satisfies`, STOP and say why.
3. **STOP-3 — do not delete the clause from the macro.** It may be honoured in `:ops`; refusing the
   measured-inert case is the honest rung.
4. **STOP-4 — do not delete the probes' measured numbers.** They are this stone's evidence; carry them into the
   SCORE.
5. **STOP-5 — if any corpus service other than `circuit.wat:2083` trips the refusal**, STOP and name it. My
   census says there is exactly one; a second means my census was wrong and I want to know.

## Shape to copy

`the-dial-declares-its-peer/SCORE.md` — the last refusal added to this family, including how it proved the rule
did not reject correct code. And `NOTE-a-callee-deadline-ms-is-inert.md` for what was already established.
