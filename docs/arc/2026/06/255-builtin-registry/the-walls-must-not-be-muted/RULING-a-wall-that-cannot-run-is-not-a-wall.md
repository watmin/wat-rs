# RULING — a wall that cannot run is not a wall. Unmute them before the registry resumes.

> **Builder, 2026-09-04:** *"wutttttt........ we need to attack this too... we must ensure that our
> docs are actually correct"* — on learning the floor has never compiled a single doctest.
>
> And the governing sentence: *"we resume the registry onslaught after we've forced our hands on
> these matters... **our mitigations and walls must not be muted**."*

## THE DISTINCTION THIS EFFORT TURNS ON — muted by RULING vs muted by ACCIDENT

```
the floor's "17 skipped"  =  5 default-filter excludes  +  12 #[ignore] attributes  =  17
                             fully accounted for; every one carries a written reason

doctests                  =  NOT among the 17. They are not SKIPPED, they are ABSENT.
                             No config excludes them. No one decided. No one noticed.
```

`.config/nextest.toml` is a **model** of the first kind. Every exclusion is measured and argued —
the five `default-filter` names each carry three different stated reasons, the deadline raises
carry the numbers that forced them, and the `retries = 1 → 0` strike is recorded verbatim: *"the
second run passing DESTROYS the only evidence the first run produced… A red is a red; the arm is
the finding."* That file is not the problem. **It is the standard the rest must meet.**

★ **A muting with a written, measured reason is a RULING. A muting nobody chose is a HOLE.**
This effort does not object to exclusion. It objects to exclusion that never had to argue.

## ⛔ FINDING 1 — THE DOCTEST HOLE, with proof

The floor is `cargo nextest run --release`. It runs **zero** doctests. That is not inferred from
documentation; it is proven by the tree's own contents:

```
src/function/parse.rs:934     /// ```
                              /// (:wat::core::extend-type :t::Robot :t::Greeter
                              ///   (greet [self loudness] "beep"))     ← WAT, in a BARE fence
```

A bare fence is Rust to rustdoc. That block cannot compile as Rust. **The floor is green.**
Therefore nothing compiles it. QED, from the disk rather than from a manual.

And the cost is not hypothetical:

```
src/lib.rs:487                /// ```
                              /// use wat::eval_algebra_source;
                              /// use holon::{ScalarEncoder, VectorManager};   ← REAL Rust
```

**A public API example that has never once been compiled.** It may be lying to users at this
moment and no instrument in the repository can tell. That is the same shape as every defect this
campaign has pulled out: a claim nothing asks about.

**Measured census of fenced blocks in Rust doc comments:**

```
bare 64  ·  text 42  ·  rust 3  ·  scheme 2  ·  ignore 2  ·  compile 2  ·  no 1
```

The 64 bare ones are the population: each is either real Rust that has never been verified, or
non-Rust that is only harmless because the gate is absent. **Both are defects, and they need
opposite fixes** — one wants to run, the other wants an honest tag.

⚠ `cargo doc` is a second unexercised surface. Nothing in the floor invokes it, so rustdoc's own
lints have never spoken here either. Not yet measured.

## ⬜ FINDING 2 — the 12 `#[ignore]`s are claims with dates on them

Twelve `#[ignore]` attributes across nine files. Most carry a named follow-up in their doc header —
*"committed `#[ignore]`'d (RED at HEAD, keeps the floor green); the S7 strike un-ignores it"*,
*"`#[ignore]` until then"*, *"un-ignored by sonnet post-stone"*. That pattern is **legitimate and
deliberate**: a disconfirming probe committed red, so the floor stays honest while the strike is
drawn.

But an exemption must earn its standing **as it ages**, not only when offered. Every one of those
headers names a stone. **The census nobody has run: which of those stones have already landed?**
An `#[ignore]` whose follow-up shipped is a test that stopped asking its question and a wall that
went quiet without anyone deciding it should.

## THE ORDER — and it is not negotiable, because of what this campaign already learned

```
1  MEASURE   cargo test --doc  → how many are red today?          (quiescent tree, first act)
2  FIX       the real Rust ones. TAG the non-Rust ones (text/wat/edn) so the exclusion is
             HONEST rather than accidental.
3  ARM       doctests join the floor — AT ZERO.
4  CENSUS    the 12 #[ignore]s against their own named follow-ups. Un-ignore what shipped;
             re-argue what did not.
5  THEN      the doc-comment migration (`#wat.doc/…` tagged EDN), under a gate that can see it.
```

★ **Step 3 before step 5 is the load-bearing ordering.** `tests/lint/no_rc_use.rs` states the rule
this campaign keeps proving: *"a lint raised at zero is a wall, a lint raised at 1306 is a
campaign."* Migrating ~5,400 doc-comment lines to a new fenced form **while nothing checks fences**
would be the largest documentation change in this repository's history performed under no gate at
all. Arm first, then move.

## WHAT THIS EFFORT IS NOT

It is not an objection to `#[ignore]`, to `default-filter`, or to any exclusion. Those are rulings,
and this repository's are unusually well argued. It is the claim that **an exclusion must be
DECIDED**, and that a gate which was never wired is not an exclusion at all — it is a wall with
nothing behind it, and the registry onslaught resumes only after our own instruments answer for
themselves.
