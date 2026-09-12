# SCORE — the gate outcome outlives its file

**SCORED.** Executor: grok, 2026-09-12, branch `sns-sqs`, HEAD `116a89ccf` (DRAWN). Did not commit.

```
     Summary [ 515.338s] 5237 tests run: 5237 passed (7 slow), 22 skipped
```

`.floor/2026-09-12T02-40-08Z/` — `scripts/floor.sh` exit **0**, **no `ARM.txt`**.
**Nothing was re-run to make anything go away.** Two earlier floors were red and are kept (below). `5237` unchanged — no deftest added.
Clippy: `cargo clippy --release --workspace --all-targets -- -D warnings` exit **0**.

---

## ⭑ THE HEADLINE — the capability tier returns a value

`GateOutcome` is registered in `src/types.rs`. `wat/capability.wat` names it. `Capability/grant` and `TypedCapability/revoke` on a live process handle print:

```
#wat.service.GateOutcome/Applied []
#wat.service.GateOutcome/Applied []
PROBE=0
```

`wat-scripts/scratch-pad/probe-capability-grant-returns-gateoutcome.wat`. The floor does not execute this surface (zero userland callers). The probe is the proof.

The name did not change. Zero `kernel::GateOutcome`.

---

## WHAT LANDED

| where | change |
|---|---|
| `src/types.rs` | `:wat::service::GateOutcome` — `Purity::Pure`, no type params. `Applied` is **Tagged with empty fields** (see below). `Gone [cause <- LociDiedError]`. `GaveUp [waited-ms <- i64, last <- String]`. |
| `wat/service.wat` | `defenum` deleted. `grantable-extend` no longer wraps `require-granted`; it returns the owner method's outcome. `typedcap-extend` is still **bodiless** — the sibling check. |
| `wat/capability.wat` | all four `grant`/`revoke` features `-> :wat::service::GateOutcome`. No `-> :wat::core::nil` on them. |
| `wat/core.wat` | generated `::grant-worker` / `::revoke-worker` face `GateOutcome` with a match (same three arms as `require-granted`). **Two lines, same file length** as HEAD (2152). |

`require-granted` and `gate-faced` stay. `grep` on `wat/service.wat`: **5** hits. Caller's choice. STOP-4 did not fire as a deletion.

Blast: 4 files, +46/−15, plus the scratch probe. Uncommitted.

---

## STOP-1 / STOP-2 / STOP-3 / STOP-4

- **STOP-1 did not fire.** No second file needs `CallOutcome` or `StopOutcome` moved. They remain `defenum`s in `service.wat`. `git diff src/types.rs` does not mention either.
- **STOP-2 did not fire.** `capability.wat` already named `:wat::kernel::Address` (Rust-registered). It now names `GateOutcome` the same way. It does not depend on any other post-20 wat declaration.
- **STOP-3 — the sketch said `Unit`; the defenum was Tagged empty.** `:Applied []` is `parse_defenum`'s Tagged-with-Vector path, even when the vector is empty. First probe: `"GateOutcome::Applied is not a tagged variant"`. Registration is `EnumVariant::Tagged { name: "Applied", fields: vec![] }` so the existing `(GateOutcome::Applied)` ctor/match spelling is unchanged. Purity, field names, and the empty-tagged shape all carry over. The sketch's `Unit` was wrong; the defenum is the contract.
- **STOP-4 did not fire as unused.** Helpers survive. The generated worker cannot *call* `require-granted` (see first red).

---

## ⭑ typedcap-extend is the sibling, and it was already bodiless

`typedcap-extend` is `(:wat::core::extend-type ~handle-bare-name ~typedcap-ty)` — satisfaction edge only. Runtime dispatch serves `TypedCapability/grant|revoke` off `grantable-extend`'s bodies. Unwrapping `grantable-extend` is the runtime change for **both**. The TypedCapability feature signatures were widened to match. The probe calls both surfaces.

---

## TWO CAPTURED REDS (not re-run)

### 1. Load order — `.floor/2026-09-12T02-15-37Z/`

```
     Summary [ 513.528s] 5237 tests run: 5236 passed (7 slow), 1 failed, 22 skipped
```

Arm: `wat::kernel test_stdlib_load_order::verify_stdlib_has_no_load_order_violations` — `left: 2, right: 0`.

The two violations (named, not guessed):

```
#wat.deporder/Violation {:referencer "wat/core.wat" :referencer-pos 0
                         :definer "wat/service.wat" :definer-pos 34
                         :symbol ":wat::service::require-granted"}
```

twice (grant-worker and revoke-worker). `core.wat` is stdlib position 0. Wrapping the generated calls with `require-granted` is a load-order violation even as a quasiquote. **Do not name a later defn from core.wat.** The generated worker faces `GateOutcome` (Rust-registered) with kernel `assertion-failed!` instead. GaveUp's message is `last` only — `string::interpolate` in that quasiquote would be the same class of violation.

### 2. Golden spans — `.floor/2026-09-12T02-27-24Z/`

```
     Summary [ 516.306s] 5237 tests run: 5232 passed (7 slow), 5 failed, 22 skipped
```

Arms: five EDN-golden macro-error tests (`probe_arc249_threading`, `probe_arc258_stone2b_macro_error`, two `probe_arc279_format`, `wat_core_cond::cond_refuses_missing_else`). Actual cause in the first:

```
:file "wat/core.wat" :line 1430
:wat::core::first: WatAST List has 0 child(ren); no child at index 0
```

Extra lines in `core.wat` shifted later macro-body spans. **Goldens were not patched.** Restored HEAD's line count (2152); the face-match sits on the original two `call-form` lines.

The green floor is `.floor/2026-09-12T02-40-08Z/`. The two reds remain on disk.

---

## THE ROWS

| # | row | result |
|---|---|---|
| 1 | enum Rust-registered | ✅ `grep -n '":wat::service::GateOutcome"' src/types.rs` — **1** hit |
| 2 | defenum gone | ✅ `defenum :wat::service::GateOutcome` count **0** |
| 3 | name did not change | ✅ zero `kernel::GateOutcome`. Count rose because capability.wat and types.rs now *name* it, plus the probe — not a rename. |
| 4 | capability surface faces the value | ✅ four features `-> :wat::service::GateOutcome` |
| 5 | ⭑ capability-tier grant RETURNS a value | ✅ probe prints `GateOutcome/Applied []` twice (Capability/grant, TypedCapability/revoke) |
| 6 | both extends unwrapped | ✅ `grantable-extend` returns the outcome; `typedcap-extend` never had bodies |
| 7 | helpers survive | ✅ `require-granted`/`gate-faced` count **5** in `service.wat` |
| 8 | purity preserved | ✅ `Purity::Pure` |
| 9 | floor | ✅ **5237 passed**, 22 skipped, 0 FAIL. Two prior reds captured, not re-run. |
| 10 | tests compile | ✅ floor ran them |
| 11 | clippy still 0 | ✅ `-D warnings` exit **0** — `large_enum_variant` did not wake |
| 12 | happy path | ✅ `2000 4 3 8192 true 1000` → `distinct=8000;dup=0` HAPPY=0 |
| 13 | chaos | ✅ `50 2 2 8192 true 1000 0 0 7 0 0 0 0 0 500` → `distinct=100;dup=0` CHAOS=0 |
| 14 | StopOutcome/CallOutcome NOT moved | ✅ neither appears in `git diff src/types.rs` |

---

## WHAT WAS NOT DONE

- Did not commit.
- Did not add a `wat-tests` deftest (5237 stays).
- Did not move `CallOutcome` / `StopOutcome`.
- Did not rename to `:wat::kernel::GateOutcome`.
- Did not delete `require-granted` / `gate-faced`.
- Did not patch goldens.

---

# ORCHESTRATOR'S GRADING — claude, 2026-09-12

**Graded against my OWN reads and runs.** Floor: `.floor/2026-09-12T02-53-10Z/` —
`Summary [ 492.834s] 5237 tests run: 5237 passed (7 slow), 22 skipped`, 0 FAIL. Clippy exit **0**.
Happy path `distinct=8000;dup=0`; chaos `distinct=100;dup=0`. Blast: 4 files, +46/−15.

**STRUCK. 14 of 14 rows pass on my instruments** — including the one a green floor could not have given.

## ⭑ Row 5 is the row, and it is proven

I ran the probe myself:

```
#wat.service.GateOutcome/Applied []
#wat.service.GateOutcome/Applied []
```

The capability tier **returns a value** through both `Capability/grant` and `TypedCapability/revoke`.
EXPECTATIONS said a green floor would be *weak* evidence here because there are zero userland callers, and
that held exactly: the floor never executes this surface. **The probe is the entire proof, and it exists
because the row demanded it before the strike began.**

| # | row | my verdict |
|---|---|---|
| 1 | Rust-registered | ✅ 1 hit in `src/types.rs` |
| 2 | `defenum` gone | ✅ count 0 |
| 3 | name unchanged | ✅ **zero** `kernel::GateOutcome` |
| 4 | features widened | ✅ zero capability `grant`/`revoke` still `-> nil` |
| 5 | ⭑ tier returns a value | ✅ **my own run**, both surfaces |
| 6 | both extends | ✅ `grantable-extend` no longer wraps `require-granted` (0 hits in its body); `typedcap-extend` was always bodiless |
| 7 | helpers survive | ✅ 5 hits — unused-but-deliberate, as STOP-4 required |
| 8 | purity | ✅ `Purity::Pure` |
| 9 | floor | ✅ my own run, 0 FAIL |
| 10 | tests compile | ✅ |
| 11 | clippy 0 | ✅ my own run — `large_enum_variant` did not wake |
| 12–13 | happy / chaos | ✅ `8000/0` · `100/0` |
| 14 | `CallOutcome`/`StopOutcome` not moved | ✅ absent from `git diff src/types.rs` |

## ★ STOP-3 caught MY sketch being wrong, and chose correctly

My BRIEF sketched `EnumVariant::Unit("Applied")`. `:Applied []` parses as **Tagged with an empty field
vector**, and `Unit` breaks the existing `(GateOutcome::Applied)` spelling — surfaced as
*"GateOutcome::Applied is not a tagged variant"*. The strike matched **the defenum, not my sketch.**
I verified: the registration reads `name: "Applied"` (Tagged), not `Unit`. ⭑ **The existing declaration
is the contract; a sketch in a brief is a convenience.** That is the right precedence and I had it
backwards.

## ★★ `wat/core.wat` — position 0 — had to change, which I did not foresee

The generated `grant-worker`/`revoke-worker` must now face a `GateOutcome`, and they **cannot call
`require-granted`** (it lives in `service.wat`, position 34). So the match is inlined. A
`test_stdlib_load_order` gate caught the violation **by name** — `left: 2, right: 0`, with both
`Violation` records printed. Good news buried in a red: **this class is already gated.**

## ⛔ AND THE SECOND RED EXPOSES AN UNDOCUMENTED CONSTRAINT — now filed

Adding lines to `core.wat` shifted macro-body spans and broke **five EDN goldens** whose names mention
*threading*, *format* and *cond* — none of which sound like `core.wat`. The strike **restored HEAD's exact
line count (2152, which I verified both sides) and patched no goldens.** Correct.

But `wat/core.wat` now carries an invisible rule: **you may not change its line count.** This is the
**second** instance today (the earlier stone reverted `src/freeze.rs` rather than patch a `:1521` golden),
and the class was already noted for Rust. I have extended
`docs/arc/2026/04/109-kill-std/NOTE-a-golden-that-pins-a-rust-line-number.md` with the wat-side sibling,
because the file most exposed to edits — manifest position 0 — is the one most tightly pinned, and the
next person will learn this from five unrelated reds.

## What I'd credit above all

**Two reds captured, named, and left on disk; nothing re-run to make anything go away; no golden patched
in either.** Three opportunities to make a red disappear quietly, three refusals.
