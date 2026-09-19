# EXPECTATIONS — the check skips what the build already proved

Scored against `DESIGN.md`. Write `SCORE.md` beside it.

## ⛔ THE NULL IS THE MOST LIKELY CORRECT OUTCOME, AND IT IS A FULL DELIVERY

Tier B is **0 for 2**. If the widened witness (row 1) finds a single `:wat::`-owned body probing a
user-declarable name, the right delivery is **the door, the stdlib fn, the user name, the program
that exposed it — and nothing landed.** Do not weaken the query to reach a zero. Do not narrow the
battery to programs you expect to pass. A third refutation, *named precisely*, is worth more than
121 ms, and it retires a question that has now cost three attempts.

⚠ **Row 1 is scored before anything else, and row 2 is not attempted unless row 1 is uniformly
zero.** A SCORE that lands the elision without the battery fails outright.

## Rows

| # | what | how it is judged |
|---|---|---|
| 1 | ⭐ **The widened witness** | ≥6 adversarial programs (DESIGN names the classes). For EACH: the program, and the TRACE-derived count of `:wat::`-owned bodies probing a user-declarable name. ⛔ Query `TRACE`, never the report's `user-reachable` column — that column aggregates the user's own body and answers a different question. Handle parametric heads (`(:wat::core::Seqable :- [:T])/seq`) and `src/runtime.rs`-attributed user companions; **say that you did.** |
| 2 | **The elision** | Only if row 1 is uniformly zero. Partitioned by the bake-time function set, ⛔ **never by `body.span().file`** — that elides user-type companions (DESIGN item 1). Rides `boot_cache.rs`; no second cache. |
| 3 | ⭐ **Observational identity** | `InferCtx.next` replayed so user bodies see identical type-var ids, **or** a demonstration that ids cannot reach user-visible output. State which, and show the evidence. ⚠ Also confirm `enclosing_rets` / `enclosing_fns` / `enclosing_handle_params` are balanced per body rather than assuming it. |
| 4 | ⭐ **Controls for the elided three** | A control per sweep that a **USER** body is still swept by it. ⛔ Judged by MUTATION, as B was: break each on purpose, show the control reddens, revert. A control asserted but not broken does not score. |
| 5 | **8c and 8d's `forms` half untouched** | B's two controls still green. `validate_def_positions_in_forms` still runs. |
| 6 | **The verdict is not circular** | Say which world derived the verdict and why a stdlib verdict computed there is valid for a *different* user program. "The cold boot that wrote the snapshot" is an answer that needs its own justification — give it. |
| 7 | **Measurement** | Warm-cache `WAT_BOOT_CENSUS=phases`, before and after, same host, single run declared as such. Report the pipeline wall, not only the three leaves. |
| 8 | **Floor** | `scripts/floor.sh`, release. Paste the **Summary line** verbatim + `.floor/<stamp>/`. ⛔ A red: do not re-run, capture whole, name the exact arm. There is no known flake. Clippy counted over the WHOLE output, not a window. |

## What would make this stone wrong

Check each; say what you found.

- **The battery is not adversarial.** Six programs that all define a record prove one thing six
  times. Each must attack a *different* door.
- **A zero produced by a broken query.** Non-vacuity: state the trace-record count and the number
  of distinct `:wat::`-owned bodies seen (it was 31009 / 2177 for my one program). A zero with 12
  records is an instrument failure.
- **The elision changes diagnostics.** A user program with a type error must produce byte-identical
  output before and after. Show one.
- **The elision changes what a `wat/` edit does.** Edit a stdlib body to be ill-typed; the build
  fingerprint must change and the error must still land. ⛔ If a stale verdict can hide a real
  stdlib break, that is a silent-hole finding and outranks the speedup.
- **8f's elision hides an error that only appears with a user program present.** This is the
  refutation shape. Row 1 exists for it; do not let row 2's success argue it away.

## Deliverable

`SCORE.md`: the per-program witness table, what landed (or the refutation), the mutation evidence
for row 4, before/after census, floor Summary + `.floor/` path.

⚠ Paths in any pulsare message are repo-root-relative (`wat-rs/docs/…`).
