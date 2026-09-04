# DESIGN — the reactor grows a seam (v2)

**Stone R1, re-drawn.** The first draft was struck down on STOP-1. Both errors were mine and both
are corrected here **from a read, not from recollection.**

## WHAT V1 GOT WRONG

| v1 said | truth |
|---|---|
| `Peer :- [Never R]` | **`Peer :- [R O]`** — `send` projects `I`; `[Never …]` is the **timer** orientation |
| "ten sites, two shapes" | **five sites, one shape** |
| "3120 lines, one top-level form" | nine forms; 1422 lines are comments |

## ★ THE BLOCKER IS CLEARED — probed this time, before the brief

`wat-scripts/scratch-pad/probe-send-seam-parametric.wat`, **3/3**:

```
sent=yes ; verdict=SEAM-EXPRESSES
```

A two-parameter top-level `defn` takes `(Peer :- [R O])` and a payload of type `:R`, and
`:wat::kernel::send` type-checks inside it. Called with a real peer, it sends.

Form precedent: **50 parametric defns** exist, including `:wat::core::foldl-spec :- [T U]`
(`wat/seq.wat:277`) — two parameters.

⚠ **What the probe can and cannot see.** It calls with a *client* peer (`Peer :- [Op Reply]`, so
`R=Op`). The serve loop's peer is `Peer :- [Reply Op]`, so `R=Reply`. **Same constraint — the payload
must be the peer's `I` — and the same signature.** It proves the signature, not the template's call
sites.

## THE FIVE SITES — one shape, read at `1854`

```wat
(:wat::core::match (:wat::kernel::send <PEER> <PAYLOAD>)
  (:wat::kernel::SendOutcome::Sent   true)
  (:wat::kernel::SendOutcome::Closed true)     ;; vanished waiter — keep serving
  (:wat::kernel::SendOutcome::Stopped false)   ;; the world is stopping
  ((:wat::kernel::SendOutcome::Lost _c) true))
```

**`1659 1697 1784 1811 1854`.** Nothing else.

### ⛔ THE EXCLUSIONS — named, so they are not swept

| lines | why excluded |
|---|---|
| `1828` | bool-shaped but **`Stopped → true`**, result discarded by `do`. A **different disposition** |
| `1939 1950` | same logic, but arm bodies are the serve loop's **recursive tail calls** / `nil` — not bool |
| `2006 2012` | `send self` status; **all four arms `nil`** |

Sweeping any of these in would change behaviour, which the contract decision forbids.

## ⛔ THE ONE CONTRACT DECISION

**The extraction changes no behaviour.** The four arms map identically at all five sites, and
**every one of 5214 tests expands through this macro** — a mistake is total and immediate, not
subtle.

★ **And the floor is proof only AFTER the edit.** The executor refused to run it on the unextracted
corpus last time — *"a green floor of the unextracted corpus is not proof of an extraction"* — which
was right, and my v1 EXPECTATIONS invited exactly that green.

## FILES — and no BOOTSTRAP dance

`wat/service.wat` **only**: one new top-level `defn` beside the eight already there, and five call
sites in the same file.

★ **No codemod, no corpus migration, no stash.** Stone C established the pattern: no new `fix.wat`
verb means no chicken-and-egg. The helper and its callers are in one file; `cargo build --release`
freezes the new stdlib.

## OUT OF SCOPE = REJECTED

- **The drop.** R2. A refactor and a feature in one strike is two stones braided.
- **The four excluded site groups.** Named above; excluding them is the correct call and doing it
  *silently* is not.
- **Changing any arm disposition.** `Closed → true` is the vanished-waiter contract
  (`service.wat:64`); `Stopped → false` is the world stopping.

## THE PROOF

1. **★★ The floor, after the edit.** `5214/5214`. Necessary and sufficient — no service avoids this
   macro.
2. **★★ The five sites are gone.** `grep -n 'kernel::send' wat/service.wat` → the call inside the
   helper, plus the **four named exclusions**, each named in the report.
3. **★ The circuit.** `distinct=8000; dup=0`, five runs.
4. **The seam can carry a drop.** With `Peer :- [R O]` in hand, say what a rate-gated drop would need
   — the rate and seed live in service state, and the helper is a plain `defn`. **Say it; R2 builds it.**
