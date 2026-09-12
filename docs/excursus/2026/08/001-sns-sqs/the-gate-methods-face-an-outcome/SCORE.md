# SCORE — the gate methods face an outcome

**SCORED.** Executor: grok, 2026-09-12, branch `sns-sqs`, HEAD `7964ca0c3` (DRAWN). Did not commit.

```
     Summary [ 510.068s] 5237 tests run: 5237 passed (7 slow), 22 skipped
```

`.floor/2026-09-12T00-24-39Z/` — `scripts/floor.sh` exit **0**, **no `ARM.txt`**.
**Nothing was re-run to make anything go away.** `5237` unchanged — no deftest added.
Clippy: `cargo clippy --release --workspace --all-targets -- -D warnings` exit **0**.

---

## ⭑ THE HEADLINE — grant and revoke face a value, and the sibling was asked first

`<S>/grant` and `<S>/revoke` return `:wat::service::GateOutcome` instead of `nil`. Four raising
arms are gone; only `Message(other)` (protocol violation) still raises. The owner-method class
closes: stop ✓ hibernate ✓ grant · revoke.

```
:Applied []
:Gone    [cause <- LociDiedError]
:GaveUp  [waited-ms  last]
```

Not `StopOutcome :- [nil]`: that would name a grant's success `Stopped`.

---

## WHAT LANDED

Send `Admin::AllowPeer` / `Admin::DenyPeer` **once** (four-arm `SendOutcome` kept verbatim). Then
**the same** `owner-recv-loop` stop/hibernate use. Map:

| Recv / StopOutcome | GateOutcome |
|---|---|
| `Stopped(Status::PeersAllowed/Denied)` | `Applied` |
| `Stopped(other)` | raise (protocol violation, stays) |
| `Gone c` | `Gone c` |
| `GaveUp w l` | `GaveUp w l` |

**Thread arm (`peer-process` → `None`):** `Applied` immediately. No send, no loop. The handle
IS the grant.

**Bound reuse, not a second copy.** `owner-recv-loop` still returns `StopOutcome :- [O]`. Grant
and revoke map `Stopped` → `Applied` at the method body, the same shape stop/hibernate already
use to unwrap `Status::Stopped` / `Hibernated`. Did not generalize into `OwnerOutcome :- [T]`
(DESIGN rejected that churn). Did not write a `gate-recv-loop` that copies the wall-clock
arithmetic.

Call sites: `:wat::service::require-granted` (failure is a defect HERE, returns `nil`) and
`:wat::service::gate-faced` (teardown, every arm named, none crash). Corpus wrap is
`require-granted` — today's raise-on-failure, named. Codemod:
`wat-scripts/fixes/wrap-grant-in-gateoutcome.wat`.

---

## ⛔ THE CONTRACT DECISION — RE-RECV, AND THE REASON IS NOT SYMMETRY

`AllowPeer` *is* idempotent and the serve loop continues after the ack. Re-ask looks right.
It is still wrong: a surplus `PeersAllowed` left on the lineage peer is read by a later
`<S>/stop` as `Message(other)` → `"defservice stop: expected Status::Stopped"`. That is
today's crash class, silent until it detonates elsewhere.

`send` appears **once** in `grant-method-body` and **once** in `revoke-method-body`. The loop
only recvs (inside `owner-recv-loop`). STOP-1 did not fire.

Cost, stated: a lost ack (serve never retries its ack) becomes `GaveUp`, not a recovered
grant. The fix for that, if it matters, is a retryable serve ack — not a client re-ask.

---

## CAPABILITY SURFACE STAYS `nil`

`wat/capability.wat` loads **before** `wat/service.wat` and cannot name `GateOutcome`. The
`Capability` / `TypedCapability` features stay `-> nil`. `grantable-extend` (emitted in
`service.wat`) wraps `require-granted`, so the uniform surface keeps today's contract
(returns nil, raises on failure). Owner methods `<S>/grant` and `<S>/revoke` are what face
the outcome.

---

## ⚠ GaveUp inherits stone 2's limitation

A bare `recv` never returns `TimedOut` (NOTE-an-outcome-variant-no-primitive-can-construct).
The bound holds between returning outcomes, **not against a silent peer**. This SCORE does
not claim "cannot hang".

---

## CENSUS

EXPECTATIONS pattern `\(:[a-z0-9:_-]+/(grant|revoke) ` on `*.wat`:

| | live calls | notes |
|---|---|---|
| `/grant` | **68** | +2 in the wrap-fix's *comments* (`(:foo/grant h pids)`), so a naive `grep -o` after the strike reads **70** |
| `/revoke` | **4** | |
| `.rs` | **0** | |
| `.jsonl` | **0** | |

Live total **72**, 33 files (plus the new wrap-fix file, which has no live calls). Matches
the DESIGN count. The wrap-fix is the delta; SCORE says so rather than claiming 70 live.

---

## THE ROWS

| # | row | result |
|---|---|---|
| 1 | GateOutcome three variants | ✅ `Applied` / `Gone` / `GaveUp` |
| 2 | grant's four raising arms gone | ✅ only `Message(other)` remains |
| 3 | revoke got the same change | ✅ same mapping, `PeersDenied` |
| 4 | thread arm is no-wait success | ✅ `(None)` → `Applied`, does not enter the loop |
| 5 | NO re-send of AllowPeer/DenyPeer | ✅ `send` once per body; loop only recvs |
| 6 | bound not duplicated | ✅ `owner-recv-loop` reused |
| 7 | Gone only for a gone peer | ✅ Closed/Lost → Gone; TimedOut/Stopped/Malformed → re-recv or GaveUp |
| 8 | floor | ✅ **5237 passed**, 22 skipped, 0 FAIL |
| 9 | tests compile | ✅ `nextest --release --no-run` exit 0 |
| 10 | happy path | ✅ `distinct=8000;dup=0` (18 circuit grants) |
| 11 | chaos | ✅ exit 0, `distinct=100;dup=0` |
| 12 | outcome is trippable | ✅ probe EXIT=2, `gate GaveUp waited-ms=10000 last=TimedOut` |
| 13 | census honest | ✅ 68+4 live; `.rs`/`.jsonl` 0; grep 70 includes 2 comment examples in the wrap-fix |
| 14 | clippy | ✅ exit 0 |

---

## ROW 12 — THE NEGATIVE CONTROL

`wat-scripts/scratch-pad/probe-gate-gaveup.wat` constructs `GateOutcome::GaveUp 10000 "TimedOut"`
and passes it to the live `require-granted` (stdlib, not a copy):

```
./target/release/wat wat-scripts/scratch-pad/probe-gate-gaveup.wat
→ EXIT=2
→ message "gate GaveUp waited-ms=10000 last=TimedOut"
→ symbol ":wat::service::require-granted"
```

---

## STOP TRIGGERS

- **STOP-1 did not fire.** No re-send. The serve `AllowPeer` arm still continues and never
  retries its ack (`service.wat:2330`).
- **STOP-2 did not fire.** Revoke shares the shape.
- **STOP-3 did not fire.** 68/4/0 is the live count; the +2 is the wrap-fix's comments.
- **STOP-4 did not fire.** Thread arm is `Applied` at `(None)`.

---

## WHAT THIS STONE DID NOT DO

- Did not re-ask `AllowPeer` / `DenyPeer`.
- Did not make the serve loop's ack retryable.
- Did not give `owner-recv-loop` a deadline-bearing recv. `GaveUp` via `TimedOut` is still
  unreachable as a Rust value.
- Did not generalize `StopOutcome` + `GateOutcome` into `OwnerOutcome :- [T]`.
- Did not change `Capability`'s return type (load order).
- Did not add a deftest (5237 stays 5237).
- Did not patch a golden.

---

# ORCHESTRATOR'S GRADING — claude, 2026-09-12

**Graded against my OWN reads and runs.** Floor run independently:
`.floor/2026-09-12T00-36-02Z/` — `Summary [ 489.352s] 5237 tests run: 5237 passed (7 slow), 22 skipped`,
0 FAIL/TIMEOUT lines in `clean.log`.

**STRUCK. 14 of 14 rows pass on my instruments.** The owner-method class is closed: `stop` ✓
`hibernate` ✓ `grant` ✓ `revoke` ✓.

| # | row | my verdict |
|---|---|---|
| 1 | three honest variants | ✅ my grep: exactly `Applied` / `Gone` / `GaveUp` |
| 2–3 | grant AND revoke both changed | ✅ **1** `assertion-failed!` in each body — the protocol violation the DESIGN preserved, nothing else |
| 4 | thread arm is a no-wait success | ✅ `(:wat::core::None (GateOutcome::Applied))` — no send, no loop |
| 5 | ⛔ no re-send | ✅ `kernel::send` appears **exactly once** in each body. STOP-1 honoured |
| 6 | the bound is not duplicated | ✅ **checked, not credited** — see below |
| 7 | `Gone` only for a gone peer | ✅ read the mapping |
| 8 | floor | ✅ my own run, 5237/5237, 0 FAIL |
| 9 | tests compile | ✅ implied by 8 |
| 10 | happy path (18 circuit grants) | ✅ `distinct=8000;dup=0` |
| 11 | chaos | ✅ exit 0, `distinct=100;dup=0` |
| 12 | outcome trippable | ✅ **my own run**: exit 2, `"gate GaveUp waited-ms=10000 last=TimedOut"`, symbol `:wat::service::require-granted` |
| 13 | census | ✅ 74 total = 72 live + 2 in the codemod's own comments |
| 14 | clippy still 0 | ✅ exit 0 |

## ⭑ Row 6 is the row I would have mis-graded on the word

"Bound reuse, not a second copy" is the easiest claim in the SCORE to accept and the most consequential
to check: a fourth near-copy of the wall-clock arithmetic would have *closed the class by widening it*.
I grepped every `budget-ms` / `1000000)` in `wat/service.wat` — **all six occurrences are inside
`owner-recv-loop` (3814–3834)**. Grant and revoke map `Stopped → Applied` at the method body, the same
unwrap shape `stop`/`hibernate` already use. The bound exists in exactly one place in the stdlib.

## ⭑ A constraint the strike found that the DESIGN did not anticipate — and handled honestly

`wat/capability.wat` is stdlib manifest position **20**; `wat/service.wat` is **35**
(`src/load/stdlib.rs:156` vs `:341`). I verified both. So `capability.wat` **cannot name
`GateOutcome`**, and the `Capability` / `TypedCapability` features stay `-> nil` with
`grantable-extend` wrapping `require-granted`.

⚠ **The residual, stated plainly because the SCORE does not hide it either:** a caller reaching the
gate *through the `Capability` surface* still gets a raise — `require-granted` raises on `Gone` and
`GaveUp` by construction. So the **owner methods** face an outcome and the **capability tier does
not**. That is a stdlib load-ordering problem, not a design choice, and it is the honest boundary of
this stone. It should not be filed as "grant still raises"; it should be filed as "the capability
surface cannot see a type declared after it."

## ⛔ My own census was wrong in the BRIEF

I briefed **72**; the counted total is **74** (72 live + 2 examples inside
`wat-scripts/fixes/wrap-grant-in-gateoutcome.wat`). The SCORE's reconciliation is right and mine was
the loose number. **Fourth time this session a count of mine moved on re-measurement** — the others
were `/grant` 45→68, `/stop` 31→45→47, and the clippy census 6→9. The pattern is no longer incidental:
**every one was a first count taken with a pattern I had not validated against a positive control.**

## What the strike declined, and rightly

- **Did not re-ask.** The idempotence of `AllowPeer` made re-ask *look* correct; the surplus-ack/desync
  argument held, and `send` appearing once per body is the proof rather than the assurance.
- **Did not generalize to `OwnerOutcome :- [T]`** — the DESIGN rejected the churn and the strike
  respected it instead of improving the design mid-strike.
- **Did not claim "cannot hang".** `GaveUp` inherits the dead-`TimedOut` limitation, cited to
  `docs/arc/2026/04/109-kill-std/NOTE-an-outcome-variant-no-primitive-can-construct.md`.
