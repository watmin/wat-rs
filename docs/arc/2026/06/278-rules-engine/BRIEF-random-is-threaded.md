# BRIEF — random is threaded

Add `:wat::rand::next` and `:wat::rand::below` — a threaded, pure, **deterministic** PRNG with `i64`
state. wat currently has no randomness interface at all, and chaos cannot be admissible here without
a reproducible one.

## Read in order

1. **`DESIGN-STONE-random-is-threaded.md`** — the contract decision (threaded, never ambient) and
   why that shape is the one that passes wat's determinism axis.
2. **`src/rete/purity.rs:10`, `:244` (`intrinsic_meta`), `:1802-1803`** — the two-axis precedent.
   `uuid::v4` is `Pure` and **not** `Deterministic`. **Yours must be both**, and the mirror of those
   two assertions is your row 1.
3. **`src/intrinsic/vector.rs:181-196`** — `#[wat_intrinsic(":wat::vector::set")]` with its doc
   comment and `@example`. **This is the shape to copy**; it is perf-3's own addition and it is
   already through the gates.
4. **`src/check.rs:17153`** — `register_builtins`. **`SCORE-perf-3-indexed-vector-update.md` records
   that a new verb registered as an intrinsic but missing from `register_builtins` /
   `intrinsic_meta` turned the floor red.** Do not rediscover that.

## The sketch

Load-bearing: the state threads through, both axes classify, no new type. Illustrative: the algorithm.

```
(:wat::rand::next  state)   -> (Tuple :- [:wat::core::i64 :wat::core::i64])
(:wat::rand::below state n) -> (Tuple :- [:wat::core::i64 :wat::core::i64])
```

SplitMix64: advance the state by a fixed odd increment, then mix with two xor-shift-multiply rounds.
`below` must avoid modulo bias — reject-and-redraw, or a widening multiply.

## Blast radius

`src/intrinsic/` (a new module or an existing one), `src/check.rs` (`register_builtins`),
`src/rete/purity.rs` (`intrinsic_meta`), and a test. **This is the first `src/` change in this line
of work** — every stone since the durable topic has held `wat/` and `src/` empty, and this one
cannot.

**No `.wat` corpus change.** Nothing calls it yet; that is the chaos stone.

## STOP triggers

1. **If you find yourself adding ambient state — STOP.** A global RNG is non-Deterministic and
   unreplayable, and it is the whole thing this design rejects.
2. **If `Axis::Deterministic` will not classify — STOP and surface it.** That is the design's central
   claim; if the analysis disagrees, the shape is wrong and I want to know before it ships.
3. **If `below` has modulo bias — STOP.** A biased chaos schedule is a lying instrument.
4. **If any `.wat` file needs to change — STOP.** Nothing should call this yet.

## Floor

`./scripts/floor.sh`. **Read the Summary line, never a piped exit code.** A red is a red — capture,
name the arm, do not re-run.

Write `SCORE-random-is-threaded.md` when done. Graded by re-running.
