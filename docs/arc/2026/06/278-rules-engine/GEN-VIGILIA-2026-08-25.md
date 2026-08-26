# The vigilia against `wat/gen.wat` — 2026-08-25

> 18 inward wards cast in parallel against the finite-generator library on the day it was
> declared feature-complete and promoted into the stdlib. **17 reported; `excusare` had not
> returned when the session closed. `circumspicere` was never cast** — it goes last by doctrine,
> and it is the one aimed at what the whole guard walked past. **Cast it first when work resumes.**
>
> Nothing below is fixed. This is the audit; the fixes are the next session's work.

## Why this document exists

Before the cast, `wat/gen.wat` had 19 laws, every one mutation-proven, all green, plus a
feature-completeness check done by *expressing* every remaining item in the QuickCheck surface.
On that evidence I reported it clean and promoted it.

**Every finding below is mine, from that same day.** The guard is not reporting old rot.

---

## THE PATTERN — read this before the list

Look at where the defects cluster:

- `card` and `at` can disagree — two fields of one struct, built independently
- `one-of`/`bind` trust an unvalidated sum of other generators' cardinalities
- the `record` macro re-splices a generator expression the caller supplied
- `lift2` and the `coords` path encode the same radix twice
- a law pins only the case where the right answer equals the do-nothing answer

**Every one is at a SEAM between two things built separately and tested separately.** Nineteen
laws, each proving one piece in isolation, and not one crossed a seam. That is the same class
that produced the `check`-has-no-witness and `shrink`-doesn't-compose gaps found two rounds
earlier — and those were found because the builder asked a question, not because I audited.

**The generalisable lesson: a law per component proves the components. It says nothing about the
paths between them, and that is where this library's defects live — all of them.**

---

## L1 — CAN COMPUTE A WRONG ANSWER

Each reproduced by hand against the live binary before crediting.

### A · A negative `card` sails through the emptiness guard into a vacuous pass
*wards: sequi, conformare, struere (independently)*

```
(:wat::gen::ints 5 2)                        ->  card -3
(:wat::gen::check <that> <always-fails>)     ->  Checked(points -3, violations 0)
```

`check`'s guard at `wat/gen.wat:150` is `(= card 0)`; `-3` is not `0`. `(range 0 -3)` is empty
(`src/collection/transform.rs:125`), the fold never runs, and a clean pass is reported over a
negative denominator.

**This refutes, in its own function, the claim I wrote there:** *"'0 violations' can no longer be
reported without knowing what it was 0 out of. The wrong reading has no form."* It has a form.
The only consumer already carries the hand-written guard that proves it —
`wat-tests/gen.wat:406`'s `(assert-true (> pts 0))` is the discipline the type was supposed to
make unnecessary.

Producers with no guard: `ints` (`:64`, no `lo <= hi`), `take` (`:413`, clamps high only),
`card-of` (`:96`, bare product fold), `vector-of` with negative `n` (yields card 1, not empty).

### B · A negative `card` SILENTLY EATS POINTS from a real space
*ward: struere*

Worse than A, because nothing is even reported as odd:

```
one-of [ (ints 5 3) card -2 ,  (ints 100 103) card 3 ]   ->  card 1
at(0)  ->  102
```

`good` has three real points (100, 101, 102). One is enumerated; **two vanish**. The dispatch
subtracts each branch's card, so subtracting a *negative* card ADVANCES the offset. `bind` has
the identical shape (`:560-563`, `:575-578`). No raise, no `EmptySpace`, no signal.

### C · `lift2` and the `record`/`coords` path DISAGREE
*ward: solvere*

Two hand-written encodings of mixed radix. Card 6, at index 6:

```
lift2   ->  Pair{a:0, b:12}
record  ->  Pair{a:0, b:10}
```

Out of contract, but they disagree, and `ints`' `at` has no bounds check so neither refuses.
**L10 was written as the tripwire for exactly this drift** (`wat-tests/gen.wat:167-169`: *"if
these two ever disagree, one of the two construction paths has drifted"*) — and drives indices
0..5, stopping one short of where the divergence begins.

Which survives: `coords`. `lift2`/`lift3` should be expressed over it.

### D · A gate that cannot go red
*wards: complectens, vocare (independently)*

**Mutation-proven by me:** replacing `shrink-index`'s whole body with `k` — a shrinker that
shrinks nothing — **passes `test-shrink-index`**.

The law (`wat-tests/gen.wat:440-442`) picks `k=5`, the one index in that space where the correct
answer equals the do-nothing answer. It asserts the negative half (does not wrongly lower) and
never the search, which is the entire function. Its sibling `law-shrink` pins
`[3 4 5 6] -> [1 0 2 0]` and *does* kill the identity — the shape it should have had.

Related: `held`'s `(> pts 0)` is **constant-true** — every deftest passes a literal positive
card — and `points` is the SUT echoing its own `Gen/card` back. Mutate `check` to report any
wrong *positive* count, or to enumerate `(range 1 card)`, and all 23 laws stay green.

### E · The containment claim is FALSE — a `Gen` crosses the wire and arrives dead
*ward: secare*

`wat/gen.wat:42-45` asserts *"The checker names this itself if you try — it is a good error."*
It does not. `is_pure_type`'s `Parametric` arm (`src/check.rs:13782`) never consults the
`TypeEnv`, so bare `Gen` is impure but `(Gen :- [T])` reads as pure — **and `Gen` has no bare
spelling.** A `Gen` in a `defrecord` loads clean and encodes as:

```
:at #wat.core/fn nil        (src/edn_shim.rs:4000)
```

`card` arrives honest, `at` arrives dead, nothing in the value says so. **This is a substrate
finding, not a gen.wat one** — but gen.wat is the file that certifies the gate holds.

### F · `record` re-evaluates its generator arguments PER POINT, not twice
*ward: struere*

**Measured by me, same 800-point space:** inline **1577 ms** vs let-bound **30 ms** — **52x**.

`wat/gen.wat:311-313` says *"each generator expression is emitted TWICE"*. The `card` copy is
evaluated once; the `at` copy is spliced into the fmap lambda's **body** (`:338`, `:342-344`) and
re-runs per generated point. The cost model is wrong by a factor of the point count, and the
comment documents the footgun instead of removing it — while the macro already carries
`fresh-symbol`, which is exactly what binding each generator once would need.

---

## L1 — FALSE CLAIMS IN SHIPPED PROSE

### G · "No native i64 mod/rem" — false since 2026-07-05
*wards: probare, cernere (independently, with the full lineage)*

`wat/gen.wat:53` justifies `digit` and `shift` existing at all. But `:wat::core::i64::{mod,rem,quot}`
ship (`wat/core.wat:497,501`; `src/runtime.rs:5928,5941`), landed in `720303f46` **seven weeks
before** gen.wat was promoted.

`cernere` traced the lineage: true when written in `wat-scripts/perf/grid/node-share.wat:71`
(2026-07-03, two days before the ops shipped) → copied to a scratch probe → and by 2026-08-01 a
sibling in the **same directory** (`where-collection.wat:74`) had already corrected it. I
inherited the retired claim into the stdlib, citing "the grid axes already use" — the very files
where the correction lived.

### H · The header cites two files I deleted the same day
*wards: purgare, intueri, probare, cernere, conferre, struere — six independently*

`wat/gen.wat:8-9` cites `wat-scripts/fuzz/gen-selftest.wat` and `tests/lint/gen_lib_laws.rs` as
the gate for the promotion evidence. Both were deleted by `6b87ba6fd` — my own commit, whose
message says so. Repeated at `docs/GENERATIVE-TESTING.md:193` and `src/stdlib.rs:85`.

**The load-bearing honesty claim of the file points at nothing.**

### I · Four disagreeing law counts, none correct
*wards: probare, cohaerere, conferre, intueri*

`wat/gen.wat:6` says 18/319 · `GENERATIVE-TESTING.md:8,192` say 19/325 · `:377` says 21 ·
`src/stdlib.rs:85` says 18/319. **Actual: 23 deftests, 337 check-driven points.**

This is the hand-maintained-total failure that `wat-tests/gen.wat`'s own header says `deftest`
was adopted to eliminate — reproduced in the prose instead.

### J · The doc teaches a contract the library refuses
*wards: cohaerere, conferre, intueri*

`GENERATIVE-TESTING.md:358` — *"### `gen-check` REFUSES an empty generator"*, `:366` — *"now
**raises** on an empty space"*. The code returns `CheckOutcome::EmptySpace` and its own comment
says *"NEVER RAISES"*. The doc corrects itself 110 lines later at `:486`, leaving both standing.
A reader who stops at §358 writes a raise-handler for an API that returns a value.

### K · Six user-facing error strings name verbs that do not exist
*wards: intueri, cernere, conformare*

`wat/gen.wat:208,229,262,267,338,393` — `"gen-elements: …"`, `"gen-nth: …"` etc., against the
file's own header at `:16-18` declaring the `gen-` prefix retired. `:582` (`"bind: …"`) is the
single conforming site — the diagnostic surface is stale AND internally inconsistent, one message
against six. A reader who greps the name in a raise finds nothing.

### L · The "ONE ordered list" lists three shipped items as open
*wards: exigere, cohaerere, conferre, intueri*

`GENERATIVE-TESTING.md:255-259` lists sampling, shrinking and bounded collections as pending.
All three ship, with laws. The list's own preamble says *"There is one list. It is this one."* —
it survived the fork it warns about and died of staleness instead. `:467` and `:502-504` assert
`gen-vector` unbuilt in the strongest terms while `:420` uses it in a **verified** row.

---

## L2 — real, smaller

- **`coords-scattered` has ZERO consumers anywhere** (purgare, vocare, cernere, probare). Its own
  law bypasses it and tests `reverse-index` directly. Delete `reverse-index` from its `at` and
  L17 stays green — sampling degrades to a sequential prefix silently. Its only documentation is
  a comment (`:402`) written in the retired `gen-` names.
- **11 verbs + 3 structs reachable only from their own law** (purgare) — ~200 of 618 lines.
- **`one-of` and `bind` are one dispatch fold written twice** (solvere); `shrink-dim` and
  `shrink-index` share one descent fold, comment and all. `nth`/`nth-str` are one generic
  function at two types.
- **`bind` caches cardinalities but discards the generators**, re-running `f` per lookup
  (temperare, struere, sequi). Measured at **29-35% of the rete fuzzer's generator time**.
  temperare warns the obvious `bind = one-of` simplification is a *regression* when `f` is cheap
  — keep both vectors.
- **Nested `bind` multiplies**: depth-3 nested bind vs `coords` on identical 10,000-point spaces
  — 18.1s vs 0.73s, ~25x (struere).
- **Four laws re-derive expected values with the implementation's own arithmetic** (vocare,
  complectens) — L9, L11, L12, L20. L11's comment says the second digit is *"the easiest place
  for the radix wiring to be wrong"*, then writes that wiring out as its oracle.
- **`Coord` is unnamed** (perspicere) — 23 spellings of `PV<i64>`; `Bases` is the same type, so
  `reverse-index` accepts a coordinate where bases belong and type-checks clean. Parametric
  `typealias` is available (`wat/kernel/channel.wat:42`). Watch the name: `wat/core.wat:1096`
  already generates `<fqdn>::Coords`.
- **`EmptySpace` is nullary** (conformare) — eight distinct producers collapse to one dataless
  token, so the caller told to "make a ruling" has nothing to rule on. The only arm in existence
  is `(assert-true false)`.
- **`prop` is `[T :-> i64]` summed into `violations`** (conformare, struere, sequi) — permits
  `violations > points`, and negative weights let a witness coexist with a zero count.
- **7 `Option/expect` sites, not 6** (conformare); 6 are caller-reachable. `Gen/at` is typed
  TOTAL, so a domain violation has nowhere to *return* — the raises are the type's fault, not
  discipline's.

## LEAVE / N-A — recorded so the absence is falsifiable

- **`partire`: LEAVE.** The three tempting cuts (driver, sampling, shrinking) are *exactly* the
  three operations the design collapses into one. Splitting re-separates the machinery whose
  fusion is the file's thesis. No sub-region has an independent test surface. The stdlib
  tolerates far larger single-concern files.
- **`secare`: no parallel boundary.** Zero parallel primitives; every driver is a `foldl` over a
  range threading an immutable accumulator. In-locus thread sharing is genuinely safe and
  compiler-proven (`Arc<EnvCell>`, immutable persistent captures, `Send` required by
  `src/kernel/spawn.rs:693`) — gen.wat never says so. The honest sentence is a **locus**
  distinction, not a checker promise.

---

## Ordering for the fix, when work resumes

1. **Cast `circumspicere`** — never cast, and it is aimed at what the guard missed.
2. **Negative `card`** (A + B) — one guard at construction fixes both; `one-of`/`bind` then need
   no change. A smart constructor on `Gen` beats a `<= 0` test at the consumer.
3. **The laws that cannot fail** (D) — before any other fix, or a fix cannot be verified.
4. **`record`'s per-point re-evaluation** (F) — 52x, and the macro already has `fresh-symbol`.
5. **`lift2`/`lift3` onto `coords`** (C) — collapses L10 from tripwire to redundancy.
6. Then the prose sweep (G-L) as one codemod, and the `Coord`/`Bases` aliases.

**Do not** fix the prose first. It is the largest count and the smallest consequence, and doing
it first would produce a clean-looking file with every live defect intact.

---

## `excusare` — returned after the summary above was written

**2 struck, 3 weakening, 11 HOLD.** It re-ran every number rather than reading the comment.

**Struck:** the promotion-standing block (`wat/gen.wat:6-9`, same as finding H — it is the block
that answers *"why does this get to be stdlib"*, written in the present tense, citing two deleted
files); and `wat-tests/gen.wat:3-5`'s *"every law below is driven by `check`"* — five of 23 are
bare `assert-eq` bypassing `held` entirely, so they never touch this suite's vacuity defence.

**Weakening:**
- **The ratchet pins the numerator and not the denominator.** `bad = 120` is asserted; the 1260
  and the 66/54 family split are prose. A change removing 66 divergent `f=3` shapes and adding 66
  elsewhere reads green. This is the same failure `CheckOutcome` refuses by design — the gate was
  *handed* the point count and does not use it. **excusare re-ran the census and all three numbers
  are exactly right today: TOTALCARD 1260, f=3 → 66, f=7 → 54, every other f → 0.**
- **The `frequency` cut's second half is wrong for the shape it sits beside.** "Cardinality is the
  weight" holds for enumeration, but the only sampler (`coords-scattered`) is defined over
  `coords` bases only — there is no scattered order for a `bind`/`one-of` space, and this file's
  own space is `such-that ∘ bind`. A `take` prefix of it walks branch 0 to exhaustion, so
  cardinality-as-weight does nothing to a short prefix. The enumeration half survives and carries
  the verdict; the sampled-prefix half must be narrowed to coords-shaped spaces.
- Timing figures drifted: the harness attributes **12.71s**, so `~7ms/case` is nearer `~10ms` and
  `~300x` nearer `~435x`. Warrant untouched at 4.7x headroom.

**Explicitly upheld** (worth keeping, because a ward confirming a reason is as useful as one
striking it): the overflow-needs-no-guard exemption re-verified today with the exact bases; the
`index-assoc` deferral (still open at `COLLECTION-CAPABILITIES.md:41`); the late-bound-`where`
cut; the no-`println`-in-a-deftest reason (corroborated structurally — every stdio test uses
`deftest-hermetic`); the `time-limit "60s"` (the sanctioned escape hatch, in the words of the
constant that defines the budget); and the ratchet's *warrant* — target named, open, and in
reach, with three `#[ignore]`d probes asserting the CORRECT behaviour so the fix makes them pass.

---

## ⚠ OUT OF SCOPE, AND POSSIBLY THE MOST IMPORTANT THING HERE

`excusare` surfaced this rather than diagnosing it, correctly:

> `cargo test --test kernel` in **debug** is **569 failed / 16 passed**, every failure the same
> arm: `panicked at src/types.rs:598:9: builtin leaf :wat::core::Option already registered as a
> structured TypeDef`. Release is clean for the same tests (569 passed).

**Confirmed present:** `src/types.rs:598` is a `debug_assert!` in `register_builtin_leaf`.
**And `scripts/floor.sh` runs `cargo nextest run --release`** — so the floor this session has
called green all day *cannot see this*, by construction.

`wat-rs/CLAUDE.md` is explicit that *"only in debug" is the same dismissal wearing a compiler
flag*, and that a `debug_assert!` panic is a real failure.

### VERIFIED 2026-08-26 — real, diagnosed, and NOT OURS TO FIX

Measured at `c43473e38`, `cargo nextest run --test kernel` (debug): **16 passed, 569 failed**,
every one `src/types.rs:598:9`. `cargo nextest run --lib` (debug): **667 passed, 490 failed**,
same arm. Release green for both.

**Cause.** `:wat::core::Option` and `:wat::core::Result` are written into BOTH of `TypeEnv`'s two
stores, which are meant to be disjoint: as `TypeDef::Enum` at `src/types.rs:1231` (the 2026-08-05
"Option and Result ARE ENUMS" block, into `types`), then AGAIN as structureless leaves by the
`BARE_CONTAINER_HEADS` loop at `:2746`, into `builtin_names`. `register_builtin_leaf`'s first
`debug_assert!` exists precisely to keep them disjoint, so it fires — on every `TypeEnv`
construction. Only 2 of the 7 heads collide; the other five have no `TypeDef`.

**Age — it is NOT gen's doing.** Both halves are present in `src/types.rs` at `10599eb36`
(2026-08-22, arc 255 "THE DOOR tells the truth for the first time"), **216 commits before this
stamp**. A gen ward merely tripped over it.

**A one-line guard fixes it** — in the loop, `if env.get(&name).is_some() { continue; }`, derived
from the registry rather than a hardcoded `Option`/`Result` skip-list. Measured with it applied:
debug kernel goes **571 passed / 13 failed / 1 timed out**. Release behaviour is byte-identical —
`contains()` already answered `true` for both names through `types`, so the second registration
bought no lookup, only the collision.

**NOT APPLIED. Builder's ruling, 2026-08-26: out of scope and reverted.** This is arc 255 ground,
not rete and not gen, and the builder has a pending task to change how `Option`/`Result`/enums
behave — which lands on exactly these lines. The tree is unmodified. Whoever takes that task
should take this with it.

**And the class finding, which outlives the fix:** the tree holds **13 `debug_assert!`** and
BOTH gates run release — `scripts/floor.sh:96` and `.github/workflows/ci.yml:96`. **No gate has
ever exercised one.** Note before anyone proposes "add a debug run to the floor": with the guard
applied, debug still failed 13 + 1, every one at exactly the 5000ms `deftest` budget or nextest's
30s ceiling — budget exhaustion in an unoptimized build, not defects. A debug gate needs those
budgets scaled first, or it is a red gate on day one.
