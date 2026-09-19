# BRIEF — measure whether the combinator-inner path is rare

Read `DESIGN.md` first. **No hoist ships.** Two tripwires, one floor run, two numbers, and a verdict
on a rune that has never been re-weighed.

## Read in order

1. `src/rete/kernel/fire/mod.rs:2137-2140` — the `Some`/`None` split inside `any_seeded_keyed`.
   `Some` reuses §5's hoisted `Arc`; `None` re-derives through `ensure_gather`.
2. `src/rete/kernel/fire/mod.rs:483` — caller 1, passes the hoisted `join_keys`. Your **positive
   control**: if this counts zero, the probe failed.
3. `src/rete/kernel/fire/mod.rs:559` — caller 2, the inner Leaf under `CondDriver::Exists`, passes
   `None`. The site in question.
4. `src/rete/kernel/fire/mod.rs:494-496` — **the rune**. *"n tokens with combinator inners is the
   rare path."* This sentence is what you are measuring. Do not edit it yet.
5. `src/rete/kernel/tests/rank_and_instrument.rs:1096-1104` — the stability gate, already covering
   `exists-of-and`. Read it so the SCORE can say the hoist's precondition is gated if the numbers
   call for one.

## The instrument

A file-append tripwire at each caller, exactly the shape validated earlier this session
(`../strike-census-E-reachability/`):

```rust
// TRIPWIRE — transient, arc 278 combinator-rarity probe. REVERTED before commit.
use std::io::Write;
if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true)
    .open("/tmp/arc278-combinator-rarity.log")
{
    let _ = writeln!(f, "hoisted");   // and "re-derives" at the other site
}
```

Not a census counter: `census_count` is only visible inside a `with_count_census` window and
nextest runs a process per test, so it cannot aggregate across the suite.

## Procedure

```
rm -f /tmp/arc278-combinator-rarity.log
# add both tripwires
./scripts/floor.sh                 # must stay GREEN
sort /tmp/arc278-combinator-rarity.log | uniq -c
git checkout -- src/rete/kernel/fire/mod.rs
git diff --quiet src/rete/kernel/fire/mod.rs && echo CLEAN
```

Read the floor's **Summary line** from `.floor/latest/clean.log`, never a piped exit code.

## What to write

Both counts and their ratio. Then the verdict:

- **`re-derives` well under `hoisted`** → the rune holds. Say so, and propose the one-line dated
  amendment to the rune recording the measurement (do not apply it in this strike unless the SCORE
  is where it lands — say which you did).
- **comparable or larger** → the rune has rotted; the hoist is warranted and becomes its own strike.
  Do not hoist here.

## Blast radius

`src/rete/kernel/fire/mod.rs` — transiently for the two tripwires, then **nothing**, or at most the
rune's one dated sentence. No hoist, no test, no gate.

## STOP triggers

1. Both counts zero → STOP; the probe did not fire.
2. `hoisted` zero while `re-derives` is not → STOP; no baseline.
3. Floor RED with the tripwires in → STOP; they must be behaviour-neutral.
4. You are hoisting → STOP.

## Prior result to copy for shape

`../strike-census-E-reachability/SCORE.md` — a measurement strike that reported a number, refused to
over-claim from it, and left the code alone.
