# EXPECTATIONS — time crosses the boundary

Written **before** the strike. Re-run by me on a quiet box. The result cannot move these.

| # | what | command | expected |
|---|---|---|---|
| 1 | ★ **all four cells cross** | `./target/release/wat wat-scripts/scratch-pad/probe-nonzeroduration-crosses-the-wire.wat` | `immediate=[ok:0]; upto=[ok:250000000]; duration-CONTROL=[ok:1000000]; instant-EXEMPLAR=[ok:1000000]; verdict=NonZeroDuration-CROSSES-THE-WIRE` |
| 2 | ★★ **zero is refused AND the service lives** | a peer sends `UpTo` with a zero payload, then a valid call on the **same connection** | first: `RequestMalformed`, `expected` naming `NonZeroDuration`. second: **`ok:`**. ⛔ If the second call returns `LOST`/`CLOSED`, that is STOP-3 |
| 3 | negative control — the arms discriminate | a `:wat::time::Duration` target handed an EDN `String` | `EdnCoerceError`, `got=String`. A blanket-accept fails the stone |
| 4 | negative duration still refused | a `Duration` target handed `Integer(-1)` | refused, consistent with `time.rs:351` |
| 5 | encode untouched | `git diff src/edn/render.rs` | no hunk at `:4158-4160` |
| 6 | blast radius | `git diff --stat` | **`src/edn/render.rs` only** |
| 7 | the doc table is not left lying | `sed -n '2230,2250p' src/edn/render.rs` | three new rows; the table matches the arms |
| 8 | the floor | `scripts/floor.sh`, **Summary line** | `5207 run / 5207 passed`. **Green, no exceptions this time** |

## ⛔ ROW 2 IS THE STONE. ROW 1 IS ONLY PLUMBING.

Row 1 makes Stone B *possible*. Row 2 is what makes it *worth doing*.

Stone A's wall is rung 3 for a literal zero and **rung 2 for a computed one** — a runtime panic that
surfaces as `LociDiedError/Panic` and, at process locus, **kills the child**. A peer sending a zero
across the wire is the computed case by definition. This stone is the first place the zero wall can
answer a remote caller with a *typed refusal* instead of a corpse.

**A refusal that kills the connection is the panic wearing a different hat.** The second call on the
same connection is the whole row; without it, row 2 is not evidence.

## RUNTIME PREDICTION

**25–45 minutes.** Three arms in one `match`, an exemplar three lines away, no new types, no
cascade. If this exceeds 90 minutes something is wrong with the DESIGN's premise — say so rather
than pushing through.

## TRAP-DOOR RISKS

1. **`Duration` and `NonZeroDuration` both accept `Edn::Integer`.** They are distinguished only by
   the *target*, which is correct for a typed coercion — but it means a mistyped arm silently
   produces the wrong variant. Row 1's two separate cells (`upto` vs `duration-CONTROL`) exist to
   catch exactly that; they must be read as two results, not one.
2. **The untyped decoder at `:2138` already has the `Instant` arm.** Copying it into the typed path
   may look redundant. It is not — `instant-EXEMPLAR` fails today, which proves the two paths are
   separate.
3. **`edn_shape_name` may not need editing.** If you find yourself extending it, the diagnostic is
   probably being built somewhere you have not read yet. Say where.
4. **This stone touches the path every service message travels.** Row 8 is not a formality — a
   coercion mistake here is not local to time types.

## WHAT WOULD MAKE ME REJECT A GREEN REPORT

- Row 2 without the second call on the same connection.
- Row 1 reported as one line rather than four cells.
- Row 3 or 4 not run — arms that accept everything pass row 1 and are worthless.
- Row 5 showing an encode hunk. `Instant` proved encoding was never the blocker; changing it means
  the diagnosis was wrong and the DESIGN needs re-drawing, not patching.
- The floor from a piped exit code rather than the Summary line.
