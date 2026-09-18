# DESIGN — D3: the door's own structural claim is false, and the gate cannot see it

> Drawn 2026-09-05 at HEAD `c7b4ce30d`. Source: vigilia 2026-09-05 D3 (`struere` L1-1, mutation-proved
> by `experiri`). **Every line below verified on disk at THIS HEAD.**

## The claim, and its four counter-examples

`src/rete/kernel/fire/pass/mod.rs:24-28`:

> ⛔ **AND THE CENSUS MOVES WITH THE DURABLE WRITE, NOT BESIDE IT.** … It was previously a SECOND
> statement next to the push at each of thirteen sites … **Here they are one act, so a future site
> cannot push without counting or count without pushing.**

That is a claim about *structure*. It is false. Four sites still open-code the act:

| site | shape |
|---|---|
| `pass/mod.rs:151-158` (`left_activate_join`) | **124 lines below the claim, in the file that makes it.** A near-exact duplicate of `record_tokens`, differing only in `extend` vs `reserve` + `extend_from_slice` |
| `fire/mod.rs:2089-2093` (proven-reuse arm) | **byte-for-byte `record_token`'s body**, inlined |
| `fire/mod.rs:2099-2103` (eval arm, tree branch) | same |
| `fire/mod.rs:2123-2127` (eval arm, fallback branch) | same |

## ★ And the gate built to catch this is proven blind

`experiri` mutation-proved it, same 100-test gate set both ways:

- drop `beta_written` at an **open-coded** site → `100 tests run: 100 passed`. **Invisible.**
- drop it **inside `record_token`** → **RED**, `fanout_cost.rs:100`: *"recorded no beta writes — the
  instrument is not armed"*, with `node 1: written 0, read 2001`.

The gate works — for sites the helper covers. The three census worlds (`fanout`, `cascade`, `tri`)
contain **no `:where`**, and all four open-coded sites live behind a filter parent, so no world in
that gate can reach them.

**So today: a false structural claim, and an instrument that cannot falsify it.** That is the exact
pair this arc keeps finding, and `session.rs:236` — the D2 cure — **cites `record_token` by name as
its precedent**, so the precedent is weaker than the citation says.

## The one contract decision, pinned

**Climb to the top rung: make the bypass a COMPILE ERROR, not a convention.**

The check rung (replace four call sites) makes the claim true *today* and leaves a fifth site free to
appear — which is precisely the argument `session.rs:224-231` already ruled on: *"⛔ THE CURE IS
STRUCTURAL, NOT CONVENTIONAL. Bumping the counter at the two bypass sites would have cured today's
two writers and left a third free to appear."*

**The mutating surface is small enough to close.** Direct `.beta` touches: 27, of which the mutations
outside the doors are the three in `fire/mod.rs` and `.beta.clear()` in `delta.rs` (the legitimate
round reset). Everything else is `.get` / `.len` / tests.

So: `beta` becomes **private** on `WorkingMemory` (`session.rs:440`), its only mutating doors are
`record_token` / `record_tokens`, plus an explicit round-reset door for `delta.rs` and shared
accessors for the readers. Same shape as `JoinRightIndex` (`session.rs:242`) and `JoinLeftIndex`,
which this arc has now landed twice.

## Scope

**IN:** the four call sites, the private field + doors, the compile-error proof, floor GREEN.

**OUT, affirmatively cut:**
- **Adding a `:where` to a census world.** It would let the *existing* gate see these sites — but if
  the bypass cannot compile, the gate does not need to. Name it in the SCORE as still-open coverage;
  do not do it here.
- `d_beta`'s own shape, the filter path's cost, F2, the CLASS A remnants.

## ⛔ Cure and gate ship together, floor green at the end.
