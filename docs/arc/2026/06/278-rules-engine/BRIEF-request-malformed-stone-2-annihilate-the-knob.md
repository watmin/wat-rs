# BRIEF — Stone 2: ANNIHILATE the `:sanitize-requests` knob. Sanitization is UNCONDITIONAL.

> Builder-ruled 2026-07-25, on being shown the opt-in default: *"annihilate it — what are we even stalling
> for here?… who the fuck would opt into crashing on bad input — this is fucking retarded — why would this
> ever be an option to consider."*
>
> **He is right and the orchestrator's Stone-1 brief is what created the knob.** It scoped the sweep away
> and pre-authorized a STOP on the cascade. A knob whose off-position is *"crash on malformed input"* is not
> a choice — it is a **non-option surfaced as one**, which arc-278 doctrine explicitly forbids
> (`feedback_never_surface_a_non_option_even_to_reject_it`). And the corpus sweep is what `wat/fix.wat`
> exists for (R21: *"we use wat-fix to unfuck the farm — do not fear refactors"*).

## The state of the hole

Stone 1 (`0efaa5b7`) built and PROVED the wall — both tiers, attacker refused with a named variant, victim
served. Then defaulted it **off**. No existing service opts in, so **the denial of service is live across the
entire corpus right now.** `wat-scripts/scratch-pad/probe-arc278-wire-dos-service-killed.wat` reproduces the
kill today, unchanged.

## The work

1. **DELETE the `:sanitize-requests` clause** from `defservice` — the clause parse, the `:all`/`:none`
   handling, the located macro-error for a bad value, and every mention. No knob, no default, no escape
   hatch. **Do not deprecate it. Delete it.**
2. **Generate the guard UNCONDITIONALLY** in `guarded-arm` for every op of every service.
3. **`:RequestMalformed` becomes MANDATORY on every op-Response enum** — the exact standing of
   `:RequestTooLarge` under ruling A, checker-forced, with the same located error when absent.
4. **Sweep the corpus.** ~301 sites across ~108 files (Stone 1's count — re-derive it yourself, don't trust
   it): the variant onto every op-Response enum, and the arm onto every caller's exhaustive match.

## The sweep is a CODEMOD, not hand-edits

Per the injected `CLAUDE.md` convention and R21 — a structural rewrite across many `.wat` files is a
**self-hosted wat-fix codemod**, never hand-edits and never python/sed:

- Framework: `wat/fix.wat` (`fix-source` walks the form tree via `read-string` → `with-children`;
  span-faithful edits via `ast-span`/`ast-end-span`/`fix-text-apply`).
- Copy a recorded migration as the shape from `wat-scripts/fixes/*.wat` — `response-record-to-enum.wat` is
  the closest sibling (it already rewrites response enums).
- Write `wat-scripts/fixes/<migration>.wat`, **dry-run on a `/tmp` copy and `diff` it** to verify the rewrite
  is exactly the intended structural change, then apply to the real corpus listing EVERY path. Idempotent
  (re-run = 0 changes). **Commit the codemod as the recorded migration.**
- If the codemod must ship alongside the `src/`+`service.wat` change that makes the old form illegal, read
  `wat/fix.wat`'s header STASH-DANCE note — it is the supported path.

**The caller-side arms are a compiler worklist, not an audit.** The exhaustive-match rule (no wildcard arm)
means the checker names every site that needs the new arm. Work it to zero; do not go hunting.

## ⚠ THE ACCEPTANCE GATE — the existing DoS probe, UNMODIFIED, must flip

`wat-scripts/scratch-pad/probe-arc278-wire-dos-service-killed.wat` **does not opt in and must not be edited.**
It currently prints:

```
"attacker good  => Ok"
"attacker BAD   => LOST (peer gone)"
victim: connect REFUSED — service is GONE
```

After this stone it must show the attacker refused with a named `RequestMalformed` and **the victim served** —
with the probe's source untouched. That is the whole proof: a service that did nothing, asked for nothing, and
opted into nothing is now safe by construction.

Land it as a `deftest` so it cannot regress. Stone 1's opted-in gates
(`wat-tests/service-request-malformed.wat`) stay green; with the clause deleted they should need only the
clause line removed.

## STOP triggers

1. If deleting the clause reveals a service that **cannot** satisfy the guard (a genuinely un-validatable
   request type) — STOP and report it with the type. Do NOT re-introduce an escape hatch; that is the
   builder's ruling to make, not a workaround to ship.
2. If the codemod cannot express the rewrite structurally — STOP and report. Do not fall back to hand-edits
   or sed.
3. If the sweep surfaces a **real** pre-existing bug (a service whose declared request shape does not match
   what its callers actually send) — STOP and report it. That is a genuine find and must not be papered over
   by loosening a declaration.

## Do NOT

- Do not weaken, delete, or loosen any assertion or declaration to reach green.
- Do not keep the knob "for compatibility." There is no compatible position — the old default is the bug.

## Gate

- The unmodified DoS probe flips to safe, as a `deftest`, **both tiers**.
- `:sanitize-requests` appears nowhere in the tree (grep it to zero).
- The codemod committed under `wat-scripts/fixes/` and idempotent on re-run.
- `cargo build --release` clean; `cargo nextest run --release` — **Summary line VERBATIM**. Floor: **4175
  passed, 314 skipped** (expect it to move by the new gates only).
- FOREGROUND only. **Do NOT commit** — the orchestrator weighs by their own re-run and commits.

## Your report

The codemod path + its dry-run diff summary; the real site count you derived; the unmodified-probe
before/after quoted; confirmation `:sanitize-requests` is gone tree-wide; the verbatim Summary line; any STOP.
