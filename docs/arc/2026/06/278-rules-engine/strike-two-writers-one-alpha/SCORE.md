# SCORE — D7, weighed against the orchestrator's own re-run

> **STOP-1 FIRED. D7 IS LIVE, NOT LATENT — a native/oracle divergence that silently drops derived
> facts.** No cure was written, correctly. And the instrument my brief told the rider to trust is
> structurally incapable of seeing this defect.

## The verdict

```
native=2 oracle=3        wat-scripts/scratch-pad/d7-two-writers-one-alpha.wat   (MY run)
wide=3  narrow=3         d7-pack-width-controls.wat — angle 2's negative control (MY run)
```

Three `Box` facts in, three `Hit`s expected. The native engine derives **two**; `fire-rules$oracle`
on the identical staged session derives **three**. A derived fact is lost with no diagnostic.

## ⭐ A — THE TRIGGER: PARAMETRIC RECORDS ERASE THEIR TYPE ARGUMENT INTO ONE CLASS

```wat
(:wat::core::defrecord :d7::Box :- [T] [k <- :wat::core::i64  v <- :T])
```

`Box[i64]` and `Box[String]` are **one class** `d7::Box` at runtime. Verified by me:

- `pack_i64_row` (`session.rs:309`) tests **runtime values** — `Box{v:100}` packs and joins
  `class_ids`; `Box{v:"…"}` does not and falls to `alpha_activate_fact`.
- `arm.rs:334` files each node under exactly one `pat.type_head`, so **both writers reach the same
  `aid`**.
- `pass/alpha.rs:130`'s `wm.alpha.insert(aid, els)` then replaces the whole `Arc<Vec<Element>>` and
  discards writer 1's push.

The DESIGN's declared-vs-runtime seam was right; **generics are the constructor for it**, and nothing
in my brief pointed at the type system. Observed at the write site (temporary, reverted):

```
D7-COLLISION aid=0 writer1_pushed=1 discarded_by_replace_with=2 d_alpha_slots=Some([0])
```

`d_alpha[aid]` still holds writer 1's slot indices, which after the replace index **different
elements** — the delta is aliased, not merely short.

## ⛔⛔ B — I POINTED THE RIDER AT AN ORACLE THAT CONSUMES THE CORRUPTED STATE AS ITS OWN REFERENCE

My BRIEF said: *"Arm the existing differential — it already computes `extra`/`missing` for exactly
this invariant, and a non-empty `extra` is the collision, observed."* **That is false, and the rider
measured it false:**

```
D7: fire 0: facts=3 leaf_aids=1 predicted=2 actual=2 extra=[] missing=[]     (three facts in)
```

`record_seed_leaf_vs_alpha` builds `predicted` by `continue`-ing on any fact whose `i64_by_fact[i]`
is `None` (`delta.rs:144-151`) — **the same predicate that decides batch membership.** So `predicted`
re-derives writer 2's output and compares it against writer 2's output. They agree by construction
**while a fact is being dropped.**

**A rider following only that instruction reports "armed it, `extra` empty, no collision" — a
confident false negative on a live fact-dropping bug.** Both DESIGN and BRIEF assert it, so it was
not a slip. This is `[[a-cache-can-make-a-gate-unfalsifiable]]` exactly, and the reason the divergence
was found at all is that the rider reached **outside** the tree to `fire-rules$oracle` —
`[[when-two-engines-disagree-neither-is-the-referee]]`, applied correctly.

## ⭐ C — THE NEGATIVE ANGLES WERE CLOSED PROPERLY, AND THREE WERE READABLE

| angle | result |
|---|---|
| runtime value not `i64` in some instances | **PROVEN LIVE** — parametric record |
| `>I64_ROW_CAP` fields vs fewer | **not reachable** — field count is a class property, so a wide class is *uniformly* unpackable and `ids.is_empty()` (`alpha.rs:119`) skips the batch. Driven anyway: `wide=3 narrow=3` |
| one `aid` under two classes | **not reachable by construction** — `arm.rs:333` pushes each node id under exactly one head; ids are unique |
| `Nature::Struct` vs `Record` | **not reachable, and could not collide** — a Struct writes nothing (refused at the insert door), and upcasting a `defstruct` is refused at check time |

⛔ **Three of my four angles die on a single line of source each.** I presented all four as equally
open, sending a rider to drive what reading settles. The angle that mattered needed the **type
system**, and my brief never looked there.

## ⚠ D — A SECOND ROUTE THE RIDER FOUND, WHICH I DID NOT SEE

`alpha_seed`'s `_ =>` arm does **not** push to `wm.i64_by_fact`, so a Struct or non-Aggregate reaching
`input_facts` would desync `i64_by_fact` from the fact index — and the *next*, perfectly packable fact
would read `packed = false` and be misrouted. The rider could not construct it because of the
`Record` wall, **but** `insert_facts_on_session` only per-element-checks when the argument is a
`PersistentVector` (`insert.rs:221`, `if let Value::wat__core__PersistentVector(pv) = …`) — any other
shape reaching `vector_concat_inner` skips `require_record_fact` entirely. **That `if let` is its own
row.**

## ⛔ E — THE STRIKE WAS FRAMED FOR THE WRONG OUTCOME

DESIGN's *"may close as a bounded negative"*, EXPECTATIONS rows 2–5, and the whole act-two assertion
plan were written for latency. Row 5 — *"neither writer reaped"* — is now **the wrong constraint**:
one of them has to change. Framing a strike around the outcome you expect is how a scorecard stops
being independent of the result; D2's precedent made latency feel like the likely answer and I let
that shape the card.

## Gates

Floor **`5336 tests run: 5336 passed, 21 skipped`**, exit=0 — run by me **after** the two `.wat`
files landed in `wat-scripts/`, which is a gated tree. Lints 210/210. `git diff` empty: no `src/`
change, correctly.

## Per-arm status

| arm | status |
|---|---|
| the collision, via parametric records | **PROVEN LIVE** — driven by the rider, re-driven by me |
| angle 2 (row width) | **proven not reachable** — driven control |
| angles 3, 4 | **not reachable** — dispositive by source |
| the `leaf_occ` differential as a detector | **proven BLIND** to this defect, by construction |
| the second route (§D) | **reachable but not driven** — the `if let` gap is open |
| a cure | **not attempted** — correctly, per STOP-1 |
