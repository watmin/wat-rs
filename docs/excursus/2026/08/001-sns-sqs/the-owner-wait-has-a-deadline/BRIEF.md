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
4. **If you find yourself editing the four method bodies**, stop and reconsider: the loop is shared, and
   a per-method change means the fix is in the wrong place.

## Shape to copy

`the-owner-faces-an-outcome/` built the loop; `the-gate-methods-face-an-outcome/` proved a change to it
serves all four methods without duplicating the bound. Its SCORE §"bound reuse, not a second copy" is
the property to preserve.
