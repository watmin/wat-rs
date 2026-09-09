# DESIGN — the unknowable state

**Stone 3d.** The reply-drop. The only fault that produces work-done-and-caller-unaware, and the
first that can move `seen-dups`.

## WHY

3c measured its own half of the tracker's table:

```
disrupts=24 ; seen-firsts=8000 ; seen-dups=0        ×5
```

24 severs, zero absorbed redeliveries — **because a `defservice` is a serializing actor and a
disrupt alarm fires *between* arms**, so a client-side sever can never land mid-claim. Predicted with
that mechanism before anything was built, and confirmed.

The table's other row has never been taken:

| drop lands | work happened? | caller knows? | duplicate on retry? |
|---|---|---|---|
| before dispatch | no | no | no ← **3c, measured** |
| **after the arm, before the reply-send** | **yes** | **no** | **YES** ← this stone |

## WHERE IT LANDS — and the code already has a comment for it

`Outcome::Continue` carries `reply <- (Option :- [R])`. An arm can advance its state, emit its
sends, and **return no reply.** Proven userland: `probe-reply-drop-is-userland.wat` →
`call2-RETURNED=LOST`. No reactor surgery; `wat/service.wat` stays the one-form macro.

Drop the reply of **`:fanout::seen`'s `claim`**, after the ledger write:

1. The ledger records the seq — **the work happened.**
2. The worker gets `LOST` — **it cannot know whether it landed.**
3. `circuit.wat:402-409` already handles this, and says so:

```wat
((:wat::kernel::RecvOutcome::Lost _cause)
  ;; Do not ack. If the claim landed, vis + Dup absorb.
  (:wat::core::Tuple q0 <redial seen> outs0))        ;; outs0 — no outcome emitted
```

4. No ack → visibility expires → **redelivered** → another worker claims the same seq → **`Dup`**.

★ **That comment describes a path that has never executed.** This stone is the first thing that
takes it. `experiri`: a surface that cannot be reached is a promise the system does not keep — and
this one is load-bearing prose about how duplicates are absorbed.

## ⛔ THE ONE CONTRACT DECISION

**The drop lands AFTER the ledger write, and the placement is the whole fault.**

- Drop **before** the write → the claim never landed → the retry is a `First` → no duplicate.
  That is 3c's row, already measured at zero.
- Drop **after** the write → the claim landed, the caller cannot know → **`seen-dups > 0`.**

Both must be built and both measured. **The stone proves the tracker's table rather than citing it**
— a 2×2 whose two cells must differ, or the placement was never the variable.

## ⚠ THE PREDICTION, WITH ITS MECHANISM — I expect this to find a real defect

Follow the path with `distinct` in hand:

- Worker A claims → **`First`** → ledger written → reply dropped → A does not ack, and emits
  **`outs0`** (no outcome).
- Visibility expires. Worker B receives the same message → claims → **`Dup`** → `first? = false` →
  **B also emits no outcome.**
- **Nobody emits an outcome for that message.**

**I predict `distinct < 8000` — loss.** Not a duplicate: a *stranding*. The consumer claims before it
emits, so a lost claim-reply converts at-least-once delivery into at-most-once processing.

If that is what happens it is **a real defect in the idempotent consumer, found by chaos** — which
is the entire reason this fault domain exists. If `distinct = 8000` instead, something recovers it
and I want to know what, because I cannot see it from here.

★ Stated with the mechanism deliberately: **every prediction this campaign that named only a number
has died, and all three that named a mechanism have held.**

## FILES

`wat-scripts/fanout/circuit.wat` only — `:fanout::seen` gains `drop-rate-bp`, `drop-seed`, and a
`drop-after-write?` flag on its `:durable`, all defaulting to no-drop.

**No `wat/service.wat`. No `src/`. No codemod.**

## OUT OF SCOPE = REJECTED

- **Fixing whatever the prediction finds.** If claim-before-emit strands messages, that is the *next*
  stone. **This stone's job is to make the defect visible, not to repair it** — and repairing it in
  the same strike would mean shipping a fix whose failure was never observed.
- **Dropping any other arm's reply.** The queue's `receive` or `ack` produce different faults and are
  not this table's row.
- **`claimed` being `:ephemeral`.** S31, still named and cut.
- **Reactor-level drops.** `wat/service.wat` is a 3120-line single macro; the userland path is proven
  and sufficient.

## THE PROOF

1. **★★ `seen-dups > 0`.** The number 3c could not move. With the drop on, at any rate, it moves.
2. **★★ The placement discriminates.** Drop-before-write → `seen-dups = 0`. Drop-after-write →
   `seen-dups > 0`. **Same rate, same seed, one variable.** If both cells agree, the placement was
   never the variable and the stone has not been demonstrated.
3. **★ `distinct`.** Report it. ⛔ **`distinct < 8000` is the predicted finding, not a failure** —
   report the number and the mechanism, do not repair it here.
4. **Rate 0 unchanged** — `seen-dups=0`, `distinct=8000`, and the floor untouched.
5. **The seed replays** — two runs, same seed, same `seen-dups`.
6. **The floor**, Summary line, `5213/5213`, at the default.
