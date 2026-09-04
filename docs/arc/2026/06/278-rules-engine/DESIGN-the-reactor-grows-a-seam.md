# DESIGN — the reactor grows a seam

**Stone R1.** One extraction. No behaviour change. It is the seam three walls have been standing on.

## WHY

Three faults have been ruled unreachable this campaign, and all three live in the same place:

| wall | why |
|---|---|
| **3d** — drop the reply after the arm | no userland form; the send is inside the serve loop |
| **the select pool** | `selectables` exists only inside the generated quasiquote |
| **server-side handle killing** | same place, same reason |

`wat/service.wat` sends a reply at **ten sites in the template**, and they are two shapes:

```wat
(:wat::core::match (:wat::kernel::send <PEER> <PAYLOAD>)
  (:wat::kernel::SendOutcome::Sent   true)
  (:wat::kernel::SendOutcome::Closed true)     ;; vanished waiter — keep serving
  (:wat::kernel::SendOutcome::Stopped false)   ;; the world is stopping
  ((:wat::kernel::SendOutcome::Lost _c) true))
```

**Identical but for the peer expression and payload.** Four sites take `peer` from an `Option`; four
take `(second (nth selectables idx))`; two are the `Stopped`/`Hibernated` status sends.

⚠ **The file is not what I called it.** I said *"3120 lines containing exactly one top-level form"*
and used that as a reason to defer. It is **nine top-level forms** — eight small type declarations
and the macro — and **1422 of the 3120 lines are comments.** Both halves of my objection were wrong.

## WHAT IT DELIVERS

One top-level `defn` beside the eight already there, and ten call sites in the template that use it:

```wat
(:wat::core::defn :wat::service::send-keep-serving? :- [R]
  [peer <- (:wat::kernel::Peer :- [:wat::core::Never :R])  payload <- :R] -> :wat::core::bool
  <the four-arm match, verbatim>)
```

`solvere`'s move: the repeated concern pulled out. Nothing else.

★ **And it is where the rest of chaos lives.** A drop gated inside that helper is *after the arm,
before the reply-send* — the tracker's row two, verbatim, the only placement that produces
work-done-and-caller-unaware. The vanished-waiter path (`Closed → keep serving`) becomes
**assertable** instead of inherited. And `selectables` is in scope at four of the sites.

**This stone builds none of that.** It builds the seam.

## ⛔ THE ONE CONTRACT DECISION

**The extraction changes no behaviour.**

Not "changes little." The four `SendOutcome` arms map identically at every site, and the floor is the
proof: **5214 tests all expand through this macro.** That is the strongest guard any stone this
campaign has had — a mistake here is not subtle, it is total and immediate.

If any behaviour changes, it is a bug, not a refinement.

## ⛔ THE ASSUMPTION I DID NOT PROBE — and it is the first thing to check

**Can a parametric top-level `defn` take a peer and a payload of the protocol's reply type, and be
called from generated code?** The payload type differs per service (`~proto-reply-ty-ann` in the
template), so the helper must be parametric over it.

`:- [R]` parameterisation exists (`Directed :- [R]`, `Vector :- [T]`), so the form should be
expressible. **I did not write the probe.** Ten minutes of context remained and I chose to ship the
absence rather than a guess — `experiri`'s rule. It is **STOP-1**, and it is the executor's first act.

If it cannot be expressed, the seam needs another shape — a macro-level helper, or the drop woven at
ten sites — and that is a finding, not a workaround to improvise.

## FILES

`wat/service.wat` only.

⛔ **STDLIB, frozen into the binary at build time.** `wat/fix.wat`'s **BOOTSTRAP / STASH-DANCE**
header governs. Read it before starting; it has been in front of us for four stones and this is the
first that needs it.

## OUT OF SCOPE = REJECTED

- **The drop itself.** Next stone. A refactor and a feature in one strike is two stones braided, and
  this campaign has paid for that.
- **Touching the `Stopped`/`Hibernated` sends** (`:2006`, `:2012`) unless they share the shape. If
  they differ, leave them and say so.
- **Changing the four-arm dispositions.** `Closed → keep serving` is the vanished-waiter contract
  (`service.wat:64`); `Stopped → false` is the world stopping. Both stay.

## THE PROOF

1. **★★ The floor.** `5214/5214`. Every test expands through this macro; nothing else is needed to
   know the extraction is faithful, and nothing less is sufficient.
2. **★★ The sites are gone.** `grep -n 'kernel::send' wat/service.wat` shows the call **only inside
   the helper** — plus whatever `Stopped`/`Hibernated` sites were correctly left alone, named.
3. **★ The circuit.** `distinct=8000; dup=0`, five runs.
4. **The seam can carry a drop.** State plainly whether the helper's signature has what a rate-gated
   drop would need, or what it would take to widen it. **Do not build it** — say whether it fits.
