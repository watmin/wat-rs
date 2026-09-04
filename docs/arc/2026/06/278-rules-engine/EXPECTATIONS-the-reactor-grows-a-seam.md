# EXPECTATIONS — the reactor grows a seam

Written **before** the strike. Re-run by me.

| # | what | expected |
|---|---|---|
| 1 | ★★ **the floor** | `5214/5214`. Every test expands through this macro — this row is both necessary and sufficient for faithfulness |
| 2 | ★★ **the sites are gone** | `grep -n 'kernel::send' wat/service.wat` → the call **only inside the helper**, plus any correctly-excluded site, **named** |
| 3 | ★ the circuit | `distinct=8000; dup=0`, five runs |
| 4 | ★ the parametric form expressed | the first-act probe, and its result |
| 5 | the four arms unchanged | `Sent`/`Closed` → true, `Stopped` → false, `Lost` → true |
| 6 | scope | `wat/service.wat` only; no `src/`, no drop |
| 7 | the seam's reach, stated | whether it can carry a rate-gated drop as-is — **said, not built** |

## ⛔ ROW 1 IS UNUSUAL: THE FLOOR IS THE WHOLE PROOF

Normally a green floor proves the absence of a regression and little else. Here it proves the
extraction is **faithful**, because *every one of 5214 tests expands through this macro.* There is no
service in the corpus that avoids it.

That also means a red here is not a hint — it is the extraction being wrong, and the failing test
names which shape diverged.

## RUNTIME PREDICTION

**60–120 minutes**, most of it the BOOTSTRAP dance rather than the edit. The extraction itself is two
shapes at ten sites.

## TRAP-DOOR RISKS

1. **BOOTSTRAP.** The stdlib is frozen into the binary at build time, so the tool cannot boot to fix
   its own stdlib mid-flight. `fix.wat`'s header is the supported path. First use this campaign.
2. **Quasiquote hygiene.** `~`-splices and symbol-nodes. The file's 1422 comment lines are largely
   *about* this; the reasoning is written down.
3. **`:2006` and `:2012` may not share the shape** — status sends, not reply sends. Excluding them is
   correct if so; excluding them silently is not.
4. **The helper's peer type.** Shape A takes `peer` from an `Option`; shape B takes
   `(second (nth selectables idx))`. If those are different peer types, the parametricity is over two
   things, not one — that is STOP-1 territory and a real finding.

## WHAT WOULD MAKE ME REJECT A GREEN REPORT

- Row 2 reported as "extracted" without the grep output.
- Any of the four arms changed — including "simplifying" `Closed → true`, which is the vanished-waiter
  contract at `service.wat:64`.
- The drop built in this stone.
- A site excluded without being named.
