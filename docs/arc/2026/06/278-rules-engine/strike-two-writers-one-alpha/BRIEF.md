# BRIEF — drive whether two writers of `wm.alpha[aid]` can collide, then make the answer permanent

Two paths write the same alpha entry in one seed pass; one appends, the other replaces. **Act one is
a drive, not a cure.** Establish whether any constructible program makes one `aid` receive both — and
report honestly if you cannot.

## Read in order

1. `src/rete/kernel/fire/pass/alpha.rs:57-100` — the seed loop's three-way split: the `_ =>` arm
   (non-Aggregate or `Nature::Struct`) → `alpha_activate_fact`; the `packed` + class-in-`leaf_aids`
   case → deferred into `class_ids`; everything else → `alpha_activate_fact`.
2. `src/rete/kernel/fire/pass/alpha.rs:114-133` — the batch loop. `wm.alpha.insert(aid, …)` at
   `:130` **replaces** the whole `Arc<Vec<Element>>`.
3. `src/rete/kernel/fire/delta.rs:96-105` — writer 1, inside `alpha_activate_fact`:
   `Arc::make_mut(cx.wm.alpha.entry(aid).or_default())` then push.
4. `src/rete/kernel/session.rs:309` — `pack_i64_row`. Returns `None` unless **every field is
   `Value::i64`** — a runtime-value test. Read `:256`'s doc beside it, which calls it a property of
   the ***declared*** fields. **That gap is the whole question.**
5. `src/rete/kernel/fire/delta.rs:118-170` — the existing `predicted` vs `actual` occupancy
   differential, and `leaf_occ_armed()`. Note `record_leaf_occ_diff` is `#[cfg(test)]` and
   `with_leaf_occ_diff` has **one** call site: `rank_and_instrument.rs:626`.

Also read `docs/arc/2026/06/278-rules-engine/strike-two-writers-one-alpha/DESIGN.md`, and
`VIGILIA-2026-08-30-WORK-LIST.md`'s **D2** block — the precedent for closing a shape finding as a
bounded negative.

## Act one — the drive

Construct a program in which one `aid` receives **both** a push (writer 1) and a replace (writer 2)
in the same pass. The mechanism you are trying to trigger: **packability varying between facts of the
same class**, so some facts of a class join the batch and others fall to `alpha_activate_fact`.

Angles worth trying, and say which you tried:
- a declared field whose runtime value is not `Value::i64` in some instances
- a class with more than `I64_ROW_CAP` (8) fields versus one with fewer, mapping to a shared `aid`
- one `aid` reachable under two different classes in `leaf_aids`
- `Nature::Struct` versus `Record` facts feeding the same alpha node

**Arm the existing differential** (`with_leaf_occ_diff`) while you drive — it already computes
`extra`/`missing` for exactly this invariant, and a non-empty `extra` is the collision, observed.

## Act two — make the answer permanent

- **If you find a trigger:** report it before curing. It changes the strike, and the cure is a
  different conversation.
- **If you cannot:** the finding is *latent*. Add the assertion the DESIGN pins — writer 2 refuses to
  replace an entry writer 1 has already touched this pass — so the latent case cannot go live in
  silence. Argue your choice of `assert!` vs `debug_assert!` on cost, and mutation-prove it.

## Blast radius

`src/rete/kernel/fire/pass/alpha.rs`, `src/rete/kernel/fire/delta.rs`, one gate. This IS the fire
path, so measure any hot-loop cost you add and say what it is.

## STOP triggers

1. **If you find a trigger, STOP and report before curing.**
2. **If the assertion you add fires on the existing floor**, STOP — that is a live collision found by
   accident, and it is the finding.
3. **If arming `with_leaf_occ_diff` more widely is needed to see anything**, stop and report; that is
   the census's own question.
4. **⛔ Do not reap either writer.** "I could not construct a trigger" is not "there is no trigger" —
   D2's ruling, and it governs here.

## Mutation proofs

- For the assertion: **construct the collision artificially** (force one fact of a batchable class to
  fail packing) and confirm it fires. An assertion never observed firing is not proven.
- Confirm the floor is green with it in, and say whether it is `debug`-only.

## What to report

- Every angle you tried for the drive, and its result — **especially the ones that failed**. A
  bounded negative is a real deliverable here and its value is entirely in what was attempted.
- Whether `extra` was ever non-empty, and under what program.
- The assertion, its cost, and its mutation.
- Scoped nextest Summary lines including `binary_id(wat::lint)`.
- **Anywhere this brief was thin or wrong.** Eight riders have run on this arc; every one found a
  real defect in the brief, including five false claims of mine. Be blunt.

Do not commit.
