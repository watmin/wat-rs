# DESIGN — stone 3: a param is an owning binding, downward

## What is actually left

Stones 1 and 2 close more than they were drawn for. Measured this session, not assumed:

- **Road 4** — the handle created in an ARGUMENT expression, peer carried out by a tail call
  (`(let [c (conn (svc/start …))] (ping c))`) — **is already caught** by `HandleTailEscape`. The
  wall sees the `/start` as creating within the let's scope.

**One road is open, and it is measured open:**

```wat
(:wat::core::defn :r3::drive-param [h <- :r3::alpha::Handle] -> :wat::core::i64
  (:wat::core::let [c (:r3::conn h)]
    (:r3::ping c)))                     ;; tail call — this frame dies, and `h` with it

(:wat::core::defn :r3::create-in-argument [] -> :wat::core::i64
  (:r3::drive-param (:r3::alpha/start …)))   ;; the handle is a TEMPORARY argument
```

Type-checks clean under both walls. Severs at runtime — 4 runs `Lost`, 1 run the mute `Closed`.

Neither wall sees it because **the creating scope and the escaping scope are different functions.**
Stone 1 asks "does a peer escape the scope that CREATED the handle" — the callee did not create it.
Stone 2 asks the same of a `let`. The callee's `h` is a *param*, which both walls deliberately treat
as safe, because that is what makes `conn(h)` legal.

## The rule — and the insight is that DIRECTION matters

The two walls are not two rules. They are one rule asked in two directions, and the answer differs:

| direction | shape | safe when |
|---|---|---|
| **upward** — peer returned to the caller | `conn(h) -> Peer` | the CALLER owns the handle, so it outlives the call. A param is a **borrow** here |
| **downward** — peer into a tail call | `(drive c)` in tail position | THIS frame is not the owner. A param **is** an owning binding here, because the frame dies before the callee runs |

So stone 3 is a **widening of stone 2, not a third wall**:

> For the DOWNWARD escape only, a `Handle`-typed **parameter** counts as an owning binding, exactly
> as a `let` binding that called `/start` does.

Stone 1 must NOT be widened. Widening it rejects every `conn` helper in the corpus — that was the
error the corpus corrected before stone 1 shipped, and it stays corrected.

## ★ The contract decision, and its honest cost

Whether a param is the SOLE owner depends on the caller, and the checker is local. So this rule is
**conservative, deliberately**: it rejects a function that takes a handle param and tail-escapes a
peer of it, *even when* the caller still holds the handle and the program is in fact safe.

That is the right trade — soundness over completeness — but it must be stated, not discovered:

- **Every rejection is a real severing OR a rune-able false positive.** Never a missed severing.
- The false-positive shape is: caller holds the handle across the call AND the callee tail-escapes a
  peer. **Census it. If live code hits this, that is a finding to report, not a nuisance to rune** —
  and if it turns out common, the trade is wrong and the stone should STOP rather than ship.

## Where the walls stand after this

| direction | creating scope | param scope |
|---|---|---|
| upward (return / let value) | ⛔ stone 1 | ✅ legal — `conn(h)` |
| downward (tail call) | ⛔ stone 2 | ⛔ **stone 3** |

Three of the four cells walled; the fourth is legal on purpose. That is the whole invariant — *a
peer must not outlive its handle* — expressed locally, without lifetimes or linear types.

## What this does NOT do, affirmatively

- **It does not fix the reap.** `eval_let_tail` still drops its scope when the tail-call signal
  leaves it, and `apply_function`'s `call_env` is still loop-local. TCO is untouched and correct;
  the same severing reproduces with NO tail call at all (`:sev::dial-and-drop`, an ordinary return),
  which is why the reap was never the defect.
- **It does not make the runtime notice reliable.** `Severed` is measured racy (4/5 here, 6/10 in
  the tightest shape) and stays a backstop.
- Out of scope = REJECTED: any runtime change; any widening of stone 1; `LociDiedError`.
