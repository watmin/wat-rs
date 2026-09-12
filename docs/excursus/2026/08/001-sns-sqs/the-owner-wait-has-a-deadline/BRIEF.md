# BRIEF — the owner wait has a deadline

**Read `DESIGN.md` beside this first.** It contains a probe result that **refutes the blocker recorded
in the 109 NOTE**, and the reason STOP-1 exists.

## The work, in one paragraph

`owner-recv-loop`'s `TimedOut` arm cannot fire, because a bare `recv` never returns `TimedOut` — so a
silent service hangs the owner forever, for all four owner methods at once. Give the wait a real
deadline. The probe says the wat-level route is probably open; **prove or disprove that first**, then
build the one that survives.

## STOP-1 — DO THIS BEFORE ANYTHING ELSE

The 109 NOTE records the blocker as *"`select` refuses to mix an owner handle with `after`'s `Peer`"*,
citing `peers[1] has wrong tier (expected Process)`. **That was a report, not a verified fact**, and a
probe has already shown the *first* failure is a type-parameter mismatch, not a tier one — and that the
coercion `call-by-deadline` uses removes it.

**Settle it:** inside the macro (where `~admin-stop-kw` / `~status-stopped-kw` and the `Handle`'s
transport marker ARE in scope, unlike a standalone probe), build

```
kind (if (peer-wire? lineage) PeerKind::process PeerKind::thread)
tmr  (first (conj (:wat::core::Vector :- [(Peer :- [<Admin> <Status>])])
                  (after kind (Milliseconds budget) <inert Status>)))
(select [lineage tmr])
```

and report **what happens at runtime**:

- **succeeds** → shape (A): `owner-recv-loop` gains a `select`-based deadline, no Rust. Build that.
- **refuses with a tier error** → grok's report is confirmed, (A) is impossible, and shape (B) (a
  `recv-by-deadline` Rust primitive constructing `TimedOut`) is the stone. **Say so with the verbatim
  error** and re-scope.

Either answer is a good outcome. **Guessing is not.**

## ⭑ THE SHAPE IS NOW DETERMINED (added 2026-09-12), except the runtime question

Worked out from `infer_kernel_after` and `call-by-deadline`; you should not have to re-derive it:

```
;; owner-recv-loop gains `inert <- :O`.  `remaining` is already > 0 here: the
;; (<= remaining 0) branch above returns GaveUp before we get here, which is
;; exactly the NonZeroDuration precondition `after` needs.
kind (if (:wat::kernel::peer-wire? peer) PeerKind::process PeerKind::thread)
tmr  (first (conj (:wat::core::Vector :- [(:wat::kernel::Peer :- [:I :O])])
                  (:wat::kernel::after kind (:wat::time::Milliseconds remaining) inert)))
(match (:wat::kernel::select [peer tmr])
  ((ServiceEvent::Message idx m)
     (if (= idx 0) <dispatch m exactly as the recv arms do today>
                   (StopOutcome::GaveUp <elapsed> "deadline")))   ;; idx 1 = the timer
  ((ServiceEvent::Closed idx) (if (= idx 0) (Gone Disconnected) <timer closed: impossible>))
  ((ServiceEvent::Lost idx c) (if (= idx 0) (Gone c)            <timer lost: impossible>))
  …)
```

★★ **AND A SIMPLIFICATION THAT FALLS OUT: `RecvOutcome::TimedOut` IS NOT NEEDED AT ALL.** Set the
timer to `remaining`, and when it fires the budget is *by construction* spent — so that branch returns
`GaveUp` directly. The dead `TimedOut` arm should be **deleted**, not resurrected. Which means the 109
NOTE's "make `TimedOut` primitive-reachable" is **not a prerequisite for this stone** — the painted
brick stays painted, it just stops blocking us. (The NOTE's corpus-wide sweep for other unreachable
variants is still worth running; it is simply not this stone.)

## The rooms

| where | why |
|---|---|
| `wat/service.wat:3811` `owner-recv-loop` | the single loop all four owner methods use. The whole fix lands here. |
| `wat/service.wat:3825` `call-by-deadline` | ⭑ the working precedent: `kind` from `peer-wire?`, timer coerced through a declared vector element type. Copy the mechanism. |
| `wat/service.wat` `stop-method-body` / `hibernate-method-body` / `grant-method-body` / `revoke-method-body` | the four callers. They pass `budget-ms` 10000 today; keep it a parameter. |
| `src/types.rs:1878` | `RecvOutcome`'s variants, incl. the `TimedOut` comment asserting the peer is ALIVE and SILENT — the fact the loop must be able to observe. |
| `src/runtime.rs` `recv_outcome_*` | the constructor family. If shape (B), `TimedOut` joins it here. |

## Blast radius

Shape (A): `wat/service.wat` only — one function body, four call sites already passing a budget.
Shape (B): `src/runtime.rs` (a constructor + the recv path) plus `wat/service.wat`.
**Neither changes `StopOutcome` / `GateOutcome`.** No call-site migration either way.

## Verify

- `cargo build --release` **and** `cargo nextest run --release --no-run`.
- `./scripts/floor.sh`, read the **Summary line**.
- `cargo clippy --release --workspace --all-targets -- -D warnings` → **exit 0** (it is at 0; do not regress it).
- Happy path `… 2000 4 3 8192 true 1000` → `distinct=8000;dup=0`; chaos `… 50 2 2 8192 true 1000 0 0 7 0 0 0 0 0 500` → exit 0.
- ⭑ **THE ROW THAT MATTERS — a silent peer must now be survivable.** Construct a service that
  accepts `Admin::Stop` and never replies (or hold its reply past the budget), call `<S>/stop`, and show
  it returns `GaveUp` with `last="TimedOut"` instead of hanging. **Before this stone that test hangs
  forever**; if you cannot build it, say so — a fix for a hang with no test that hangs is unproven.

## STOP triggers

2. **If `TimedOut` becomes reachable but `GaveUp` still cannot be produced**, stop and report — that is
   the same "variant nobody can trip" defect this track has now hit three times (`GaveUp`, `Verdict`,
   `Malformed`).
3. **If the fix needs the budget to become a constant**, stop. The DESIGN's contract decision is that it
   stays a caller-visible parameter.
4. ⛔ **CORRECTED 2026-09-12 — this trigger was WRONG and is struck.** It read: *"if you find
   yourself editing the four method bodies, stop — the loop is shared."* **You MUST edit them**, and
   here is why, from `infer_kernel_after`'s own contract (`src/check.rs`):

   > *`args[2]`: msg — inferred; its type becomes the output type `O`. Returns `(Peer' :- [nil O])`.*

   `after` needs a value of the **received** type. `owner-recv-loop` is generic over `[I O]`, and a
   generic function **cannot mint an `O`** — so the loop must take `inert <- :O` as a parameter, and
   the four method bodies are the only places that know their concrete `Status` type. That is exactly
   how `call-by-deadline` does it (its caller passes `(~reply-variant-kw (~rtl-ctor-kw 0 0))`).
   Editing the four bodies to pass an inert value is CORRECT, not a smell. The thing that would still
   be wrong is duplicating the *bound* — that stays in one place.

## Shape to copy

`the-owner-faces-an-outcome/` built the loop; `the-gate-methods-face-an-outcome/` proved a change to it
serves all four methods without duplicating the bound. Its SCORE §"bound reuse, not a second copy" is
the property to preserve.
