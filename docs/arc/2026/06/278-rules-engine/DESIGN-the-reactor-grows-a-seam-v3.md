# DESIGN — the reactor grows a seam (v3)

**Stone R1, third draft.** v1 died on a wrong type. v2 died on a wrong *proof*. This one changes
**where the helper goes**, and nothing else.

## WHAT KILLED V2

Four `peers_bijection` EDN goldens **snapshot `service.wat` line numbers** — they literally carry
`:line 896`. The helper was inserted before `defservice`, everything below shifted **+13**, and the
floor went red for a reason with nothing to do with behaviour.

⛔ **My contract decision was the wrong instrument.** I called the floor *"necessary and sufficient
for faithfulness."* It is **necessary and not sufficient**: it conflates *behaviour changed* with
*lines moved*, and any insertion above line 896 trips it.

## ★ THE FIX — and it is probed

**Append the helper AFTER the macro, at the end of the file.** Nothing before it moves; the goldens
hold.

That requires the macro's template to call a `defn` defined *later in the same file*.
`wat-scripts/scratch-pad/probe-template-calls-a-later-defn.wat`, **3/3**:

```
expanded=84 ; verdict=FORWARD-REFERENCE-OK
```

Reasoning said this was fine — the template is quasiquoted data, expanded at the *use* site, by which
time the file is registered. **"Reasoning said fine" is exactly what killed v1 and v2**, so it is
measured.

## ★ AND THE GOLDENS BECOME THE INSTRUMENT

They are not a nuisance. **Four tests that go red if any line above 896 moves** is precisely the
tripwire this stone needs: it proves the helper landed where it was supposed to.

So the proof splits honestly:

| question | instrument |
|---|---|
| did behaviour change? | the floor — 5210 non-golden tests, all expanding through this macro |
| did anything shift? | **the four bijection goldens, staying green** |

Neither alone is sufficient. Together they are.

## THE WORK — unchanged from v2

One parametric `defn`, **at the end of the file**:

```wat
(:wat::core::defn :wat::service::send-keep-serving? :- [R O]
  [peer <- (:wat::kernel::Peer :- [:R :O])  payload <- :R] -> :wat::core::bool
  <the four arms, verbatim>)
```

Signature proven at `probe-send-seam-parametric.wat` (`SEAM-EXPRESSES`, 3/3).

**Five call sites**, at their *current* numbers — nothing has shifted, because v2 is not committed:
`1659 1697 1784 1811 1854`.

### ⛔ THE EXCLUSIONS — five lines, and nine that were never candidates

| lines | why |
|---|---|
| `1828` | `Stopped → **true**`, discarded by `do` — different disposition |
| `1939 1950` | arm bodies are the serve loop's recursive tail calls / `nil` |
| `2006 2012` | `send self` status, all arms `nil` |
| `2025 2045 2100 2239 2324 2368 2420 2470 2620` | **never candidates** — peers-allowed/denied, malformed-reply, five client-face send-then-recv methods, child-main status |

★ **v2's row 2 predicted six remaining sends. There are fifteen.** I named four *groups* and five
*lines*, then predicted four — and never counted the nine that were never in scope. **Row 2's number
is 15.**

## ⛔ THE ONE CONTRACT DECISION

**The extraction changes no behaviour, and moves no line above 896.**

Two clauses because the proof needs two instruments, and v2 proved that having one is having none.

## FILES

`wat/service.wat` only. No codemod, no stash, no `src/`.

## OUT OF SCOPE = REJECTED

- **The drop.** R2. The helper is `[peer payload] -> bool`; a rate-gated drop needs the rate and seed
  from the durable record, so R2 widens it — *drop? before the send, still returning `true`, because
  a drop is not `Stopped`*. Wrapping at the five sites would defeat the seam.
- **Patching the goldens.** If end-placement works they stay green. If it does not, **the goldens are
  their own stone** — they encode a line to assert a span, which breaks on any edit above them.
- **Any arm disposition.** `Closed → true` is the vanished-waiter contract (`service.wat:64`).

## THE PROOF

1. **★★ The four bijection goldens stay GREEN.** Nothing above 896 moved. This is the placement's
   proof and it did not exist in v2.
2. **★★ The floor, after the edit.** `5214/5214`.
3. **★ The five sites are gone.** `grep -n 'kernel::send'` → **15 lines**: the helper, five
   exclusions, nine never-candidates. **Each named.**
4. **The circuit.** `distinct=8000; dup=0`, five runs.
5. **What R2 needs**, stated not built.
