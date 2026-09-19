# EXPECTATIONS — an oversized reply is not a death

Scored against `DESIGN.md`. Write `SCORE.md` beside it.

## The null

⛔ **"`FrameTooLarge` cannot be reclassified without breaking consumer X"** is a full delivery —
name X, show what it sees today, land nothing. `classify_peer_error` is shared, and a silent
behaviour change for a service or script would be a worse bug than the one being fixed.

## Rows

| # | what | how it is judged |
|---|---|---|
| 1 | ⭐ **Blast radius FIRST** | Every consumer of `classify_peer_error` / `PeerDeath::Lost`-from-`FrameTooLarge`, and what each sees before and after. ⛔ A reclassification landed without this table does not score. |
| 2 | **The reclassification** | `FrameTooLarge` stops meaning death on the spawn-process branch. The peer is alive. |
| 3 | ⭐ **The coordinator survives** | A worker sending an oversized reply must not kill the bracket. Disposition (a) RETRY-bounded, or better if row 3 of the DESIGN allows it without touching the surface. |
| 4 | ⭐ **Control by MUTATION** | The oversize probe currently kills the run; after this it must not. Revert the reclassification → the probe kills the run again. **Show both.** |
| 5 | **Non-vacuity** | The frame really exceeded the cap (the reason string names it), and the item's fate is stated — retried, dropped, or reported. |
| 6 | **The surface wall, counted** | State plainly that this is the **fourth** stone blocked by `(Vector :- [O])`. ⛔ Do not widen `map`/`each`. |
| 7 | **Scope wall** | Thread tier, suppression, lineage-Admin, queue knobs untouched. Say what you did not do. |
| 8 | **Floor** | `scripts/floor.sh`, release. Summary verbatim + `.floor/<stamp>/` + the tree it ran against. A red: do not re-run, capture whole, name the arm. Clippy over the WHOLE output. |

## What would make this stone wrong

- **A silent behaviour change for a non-bracket consumer.** Row 1.
- **Retrying a deterministic fault with no bound** — `FrameTooLarge` reproduces exactly; the
  wall-clock bound must be the stop and the SCORE must say so.
- **Treating it as `Malformed`.** They are different facts: `Malformed` is ambiguous between wire
  damage and sender garbage; `FrameTooLarge` is unambiguously a property of the payload.
- **Widening `map`'s return type** to make (b) fit. That is the builder's call, not this stone's.
- ⚠ **Claiming the DoS is closed when only the bracket path is.** A service already answers
  `Rejected`; a *script* using `select` over spawn peers may still die. Say which paths are covered.

## Deliverable

`SCORE.md`: the blast-radius table, the reclassification, the mutation evidence both ways, the
item's fate, the surface-wall count, floor Summary + `.floor/` path + tree statement.

⚠ Paths in pulsare messages are repo-root-relative (`wat-rs/docs/…`).
