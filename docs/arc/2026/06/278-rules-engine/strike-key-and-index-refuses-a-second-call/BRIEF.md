# BRIEF — make the first-keying door refuse a second keying

Read `DESIGN.md` first, especially the severity note: **this is hygiene on an unreachable branch**,
not a live defect, and the SCORE must say so. One `debug_assert`, one corrected doc, one
debug-gated test carrying `rune:excusare(no-falsifier)`.

## Read in order

1. `src/rete/kernel/session.rs:253-272` — `keys()` and `key_and_index`. The `or_insert` is the
   silent branch; the doc sentence *"A later call does not replace keys"* is what changes.
2. `src/rete/kernel/session.rs:274-…` — `writer()`, A1's cure. It returns `None` until keyed, and it
   is the sanctioned post-keying route your assert message points callers at.
3. `src/rete/kernel/fire/mod.rs:875-896` — caller 1. Note **both** guards: `match keys(join_id)`
   reuses the stored list, and `if !is_keyed(join_id)` gates the call, with `else` going to
   `writer()`.
4. `src/rete/kernel/fire/pass/hash_join.rs:114-145` and `:279-284` — caller 2. `first_keying =
   !left_idx.is_keyed(…)`, and the `else` arm reads the stored list back.
5. `docs/CONVENTIONS.md`'s `rune:excusare` table (added last strike) — the `no-falsifier` row's
   decisive test is what your test's rune reason must answer.

## What ships

**The guard**, first statement in `key_and_index`:

```rust
debug_assert!(
    !self.keys.contains_key(&join_id),
    "key_and_index is the FIRST-keying door and join {join_id} is already keyed — \
     add tokens through `writer()`"
);
```

**The doc**, rewritten to say what the code means: this is the first-keying door; both callers gate
on `is_keyed` so a second call is a caller bug; `writer()` is the post-keying route.

**The test** — `#[cfg(debug_assertions)]`, `#[should_panic]`, calling `key_and_index` twice on one
`join_id`, carrying:

```rust
// rune:excusare(no-falsifier) — <what you tried on the release floor, and why it cannot work>
```

## Mutation proof

Run the test under **debug** (`cargo nextest run` without `--release`, filtered to that test) and
quote the panic. Then state plainly that the release floor cannot run it and why — that is the
rune's reason, not a gap.

**Do not** reach for `assert!` to make it floor-provable. Rejected in DESIGN: paying release cost so
a test can pass is the tail wagging the dog.

## Blast radius

`src/rete/kernel/session.rs` — one `debug_assert`, one doc comment, one debug-gated test.
**No release behaviour. No caller changes.**

## Floor arithmetic

The test is `#[cfg(debug_assertions)]`, so under `--release` it does not exist: **5470 run / 22
skipped, unchanged.** Any movement is STOP-2.

## STOP triggers

1. The test does not panic under a debug run → STOP; the guard is not wired.
2. Any floor number moves → STOP; `debug_assert` is release-free.
3. `grep -rn "key_and_index(" src/` finds a third caller → STOP and report; the severity assessment
   is wrong and this is a live defect.
4. You reach for `assert!` → STOP.

## Prior result to copy for shape

`../strike-seenset-one-door/SCORE.md` — a hygiene strike that stated its severity honestly and took
a STOP rather than shipping a guard whose cost it could not justify.
