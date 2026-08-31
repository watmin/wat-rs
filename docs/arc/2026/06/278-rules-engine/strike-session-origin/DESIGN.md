# DESIGN-STONE — the session ceiling's zero point belongs to the session

> **Origin (2026-08-30/31).** Vigilia Class A4, found INDEPENDENTLY by two wards from opposite
> directions — `secare` hunting shared state, `sequi` hunting state threading. That convergence is
> why it leads Class A.

## Why

`alloc_counter::SESSION_ORIGIN` is **one `Cell<Option<usize>>` per THREAD**. `mark_session_origin`
sets it **unconditionally** — no save, no restore, no already-marked arm — and it is called from
`arm-session`, which every `compile-all` reaches (`arm.rs:1205`). So a second session on the thread
**rebases the zero point**, and everything the first had already staged stops being charged to it.

`session_bytes()` then reads `thread_bytes() - origin_B`. Two consequences, the second worse:

1. The first session gets a **fresh budget** it did not earn.
2. Once `thread_bytes()` falls below `origin_B` — any drop on the thread — `saturating_sub` floors
   the reading at **0, and the session has no ceiling at all for the rest of its life.**

**The module states the assumption honestly** (*"one session per thread at a time… if that ever
stops being true, this is the line that moves to a Session field"*). It was already false when
written: `arm_lease.rs:141` holds two live sessions on one thread in a **green** test, and
sequential `compile-all` is the ordinary shape, not a corner.

## The measurement — driven, and its sensitivity explains itself

Both arms stage the **same 16,000 facts into one session**, in two rounds of 8,000. The only
difference is one unrelated `compile-all` between the rounds. Ceiling 4 MB, HEAD `74e7f2dd7`:

```
"control" "REFUSED"      ← the ceiling is live at this workload
"probe"   "NO-BREACH"    ← identical workload, admitted
```

Swept, and the sweep is corroboration rather than tuning:

| ceiling | control | probe | |
|---|---|---|---|
| 4,000,000 | REFUSED at staged 8477 | **NO-BREACH** | the differential |
| 2,500,000 | 4898 | 4896 | both refuse |
| 1,500,000 | 2512 | 2510 | both refuse |

At the lower ceilings the breach lands in **round one**, before the rebase can forgive anything —
so the arms agree. The differential appears exactly when the rebase falls between the rounds,
which is what the mechanism predicts. A probe whose sensitivity is explained by the defect is
worth more than one that merely fails.

## The algorithm

Key the origin the way `ARM_TABLE` already keys its entries — by the session's **network
identity** (`arm.rs`'s `network_identity`, a PMap allocation id). A thread-local
`FxHashMap<u64, usize>` of origins replaces the single `Cell`; `session_ceiling_breach` and
`check_insert_ceiling` take the identity of the session they are judging; `mark_session_origin`
records against that identity instead of clobbering one slot.

The "unmarked marks itself" answer stays, and stays honest — it just becomes per session.

## ★ THE ONE CONTRACT DECISION

**The fix must not claim more than it delivers, and the module's own doc already names the bound:**

> *"the move would fix only the re-basing, never the cross-charging: a thread-local counter cannot
> separate two sessions sharing a thread, wherever the origin is stored"*

That is **true and must survive this strike.** After the fix, session A's reading still includes
whatever session B allocated on the same thread — so A **over-counts** and refuses **early**. The
module already rules that direction safe (*"it over-counts in one direction, and that is the safe
one… refuses slightly EARLY, never late"*).

So the strike converts **an unsafe silent failure — no ceiling at all — into a safe conservative
one.** The doc must say exactly that. **Replacing an honest bound with a false claim of precision
would be the worse outcome**, green floor or not: the current comment's honesty is the only reason
this defect was tractable, and the same sentence that named the hole must now name the residue.

## Blast radius — verified on the disk, not claimed

```
src/alloc_counter.rs:118,133,158      SESSION_ORIGIN, mark_session_origin, session_bytes
src/rete/kernel/arm.rs:1205           the one mark site
src/rete/kernel/session.rs:1404,1420  session_ceiling_breach, check_insert_ceiling
src/rete/kernel/fire/delta.rs:682     the fixpoint door
src/rete/kernel/insert.rs:194,235     the two insert doors
```

Six files. **Wider than every previous strike in this chain** — that is the cost of a zero point
that lives in the wrong place, and it is the reason the fix is a type change rather than a patch.

## Out of scope — AFFIRMATIVELY CUT

- **Cross-charging.** Not a defect to fix here; it is the residue the ★ decision requires be
  *stated*. Separating two sessions' allocations needs a per-session allocator, not a per-session
  origin.
- **A7's O(N²) import.** Different failure (cost, not correctness) at the same door.
- **A5's `arm.rs:1190` prose.** *"`compile-all` is the one door EVERY rule passes"* is false at
  import and at hand-assembled Sessions — and this strike makes it **more** wrong by adding a
  registration that import will also skip. Sequenced next, deliberately, for that reason.

⚠ **A7's OTHER half lands here whether we like it or not:** `import_export` never calls
`mark_session_origin`, so an imported session self-marks on first sight. With per-session origins
that is still the honest answer, but the import door should register explicitly. Take it in this
strike **only if it costs one call**; if it costs more, STOP and surface it.
