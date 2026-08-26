# DESIGN — the numerics get their homes: `:wat::core::i64::+` → `:wat::i64::+`

DRAWN 2026-08-25 against `1d65ef115`. Builder's ruling, this session.

## The ruling

> *"numerics... they get a renamed.... `:wat::core::i64::+` => `:wat::i64::+` and so on... just like
> string got moved from `:wat::core::string::join` => `:wat::string::join`. its going to be a large
> refactor - we do not fear them."*
>
> *"core's defclause for arith will go to `wat.core/+` => `[wat.i64/+ wat.f64/+ wat.u8/+]` and so
> on...... our u8 support is basically nonexistent, we will add full support for all rust primitive
> numerics once we have the pattern set."*
>
> *"we are grinding towards flipping from our current rust-ish syntax to our future
> `:wat::i64::+` => `wat.i64/+`."*

## ★ THIS SUPERSEDES A RECORDED RULING — say so, do not silently diverge

`RULING-the-numerics-rehome-is-a-SPLIT-type-and-ops-part.md` (2026-08-19) sends the OPS to
`wat.core.i64/+` — which, in today's colon spelling, is **where they already are**. That half is
**superseded**: the ops now go to `:wat::i64::` / `:wat::f64::`, top-level, adjacent to
`:wat::string::`.

**Its TYPE half stands, untouched and load-bearing:** `:wat::core::i64` (the type, **7,952 live
sites**) heads for arc **251**'s `wat.type/`, not here. The trailing `::` is the entire
discrimination between the two populations, and it is the trap that ruling already caught.

## Why the destination is `:wat::i64::` and not something shorter or deeper

The surface is grinding toward Clojure-faithful tags. `:wat::i64::+` renders as **`wat.i64/+`** — a
two-segment namespace and a name. `:wat::core::i64::+` renders as `wat.core.i64/+`, which buries the
type one level too deep. **Choosing the home now makes the future flip a re-spelling instead of a
second migration.**

## What this stone ENABLES, and deliberately does not build

**Arc 256 — generic defclause — is BANKED, and its STUB says it is *enabled by 255*.** Its thesis is
exactly the builder's `wat.core/+ => [wat.i64/+ wat.f64/+ wat.u8/+]`: *"N hand-written Rust
`infer_<op>` fns collapse into ONE generic-clause inference rule + N wat declarations. Inference
stays Rust (the engine); the KNOWLEDGE moves to wat."*

A defclause's clauses need **addressable, registered, per-type names**. Today the per-type numerics
are unregistered and buried in `core::`'s junk drawer — which is *why* 256 cannot strike.
**This stone is the enablement, not the defclause.** `:wat::core::+` is NOT retired here; it stays
the polymorphic entry and becomes a defclause under 256.

**u8 and the rest of the Rust primitive numerics are out of scope and that is the point** — this
stone sets the pattern with i64 + f64 so the remainder land cheaply.
Measured: `:wat::core::u8` has **157 live sites** (all the TYPE — Bytes and friends);
`:wat::u8::` has **0**. There are no u8 ops to move.

## The measurement

36 operations, and my independent census reproduces the 2026-08-19 ruling's counts exactly:

```
i64 (17)  + - * /  < <= > >= = not=  mod quot rem  to-bigint to-f64 to-rational to-string
f64 (19)  + - * /  < <= > >= = not=  abs clamp max max-of min min-of round to-i64 to-string
```

Live corpus, `docs/` excluded — **history does not move**:

```
:wat::core::i64::            1850
:wat::core::f64::             331
:wat::rete::core::i64::       485      the rete DSL clone — a restricted clone of wat's
:wat::rete::core::f64::        60      language, mirrored the same way string was
                             ─────
                              2726 sites · 4 prefixes
```

Partition control: `:wat::core::i64` total (type+ops) **9,802** − ops **1,850** = **7,952** TYPE
sites that must NOT move. The two sub-counts partition the whole; that is the non-vacuity check.

## ⛔ THE FINDING THAT SHAPES THE WHOLE STONE

**The string promotion — this stone's own prior art — shipped with ZERO retirement rows, and the
dead spelling still type-checks today.** Proven this session with a positive and a negative control:

```
:wat::core::string::length              a REAL name that MOVED
    --check EXIT=0        run EXIT=1 (UnknownFunction)
:wat::core::string::utterly-invented    a name that NEVER existed
    --check EXIT=0
```

`src/resolve/walk.rs:268` — `if is_reserved_prefix(head) { return true }` — accepts **any**
`:wat::`-prefixed name: real, retired, or invented. Its comment excuses this with *"leaf-level
validation is the type checker's concern"*, and **the type checker does not do it.** The excuse
names a layer that never took the responsibility. This is the hole arc 255 exists to annihilate.

**Consequence for THIS migration:** rename 2,726 sites with nothing else, and every site the codemod
misses **still type-checks**, failing only at runtime on a path that may never execute. The
migration's correctness would be unverifiable by the only tool that could verify it. We would repeat
the string move's mistake at four times the scale.

So the stone is three, in sequence, each green on its own.

---

## STONE A — the homes exist (both spellings live)

`src/intrinsic/i64.rs` + `src/intrinsic/f64.rs`, mirroring `src/intrinsic/string.rs` (19 registered
ops, the proven shape). 36 ops registered as `#[wat_intrinsic(":wat::i64::+")]`.

**Old spellings keep working.** Nothing in the corpus moves yet, so the tree stays green throughout.

## STONE B — the corpus moves

A **rules codemod**, `wat-scripts/fixes/rename-numerics-to-their-homes.wat`, copied in shape from
`rename-core-string-to-string.wat`. Four full-name prefix renames, each `::`-terminated.

⛔ Its header records the trap, and it is not optional: **`wat/fix.wat`'s `rename-keyword-prefix` is
a silent no-op for an open (`::`-terminated) namespace prefix** — the parked
`BLOCKED-rename-core-string-to-string.wat` in `scratch-pad/` is the evidence. **The rules form is
the one that works**, and it has two entry points — `wat --grep` (finder, counts Matches without
writing) and `wat` (applier). Dry-run on a `/tmp` copy and `diff` before applying.

Plus the **~640 Rust-side string literals** naming these ops.

## STONE C — the old names die, and `:wat::` stops being blind

1. **36 retirement rows** (`src/remedy/retirement.rs`) — the uuid/regex precedent, the one the string
   move skipped. `:wat::core::i64::+` then fails **at check time with a remedy naming
   `:wat::i64::+`**, instead of silently.
2. **Narrow the blanket-accept.** `is_resolvable_call_head` stops blanket-accepting the promoted
   namespaces and requires `sym.get(head).is_some()` or a registry hit.

   ⚠ **The membership test is sym-OR-registry, not registry alone.** `wat/*.wat` defines verbs in
   these namespaces too, and those are legitimate; an instrument that counts only
   `#[wat_intrinsic]` registrations conflates *"no such name"* with *"defined in wat, not Rust"*.
   I built that wrong instrument while drawing this and caught it before quoting it.

   **The method is empirical, not a census: drop the blanket-accept for the promoted namespaces and
   read the screams.** My censuses have been wrong repeatedly this arc; a wall imposed once is worth
   five of them. `[[feedback_impose_the_check_and_read_the_screams]]`

3. **★ THE GATE IS ALREADY WRITTEN — DO NOT INVENT ONE.** A prior self committed the disconfirming
   probe and disarmed it with the unlock condition attached. **We have circled back.**

   ```
   tests/wat_lang/probe_undefined_builtin_resolves.rs
     :17  wrong_operator_leaf_is_a_check_error      (:wat::core::i64::+'2 1 2)
     :31  bogus_leaf_under_known_namespace_...      (:wat::core::Bogus ...)
          #[ignore = "RED-at-HEAD: checker rejection of undefined builtins
                      (arc-255 builtin-registry) not yet built;
                      unlock when we circle back to arc 255"]
   ```

   Its module doc names the mechanism this stone must build, in the prior self's own words:
   *"resolve checks leaf membership against the dispatchable-builtin source of truth and rejects it
   at check time."* And the first test is **literally a renamed-away i64 operator** — the exact case
   Stone C manufactures when `:wat::core::i64::+` retires. **Stone C is done when those two ignores
   are deleted and the tests pass.** That acceptance is inherited, not invented, and it outranks any
   gate I would have written.

   A sibling is banked against the other hole found this session:
   `tests/types/probe_diag_typealias_leniency.rs:16` — *"undeclared field-type keywords are accepted
   LENIENTLY today… un-ignore when 255 makes them check errors"* — which is exactly the
   `:wat::core::NotARealType` acceptance a rider surfaced on 2026-08-25. Two findings I took for new
   already had gates waiting.

4. **And still break a door.** After the ignores come off and the tests pass, remove the narrowing on
   purpose and confirm they go RED again. A gate that survives removal of the door it guards is a
   claim. `NISI FRANGAS, NIHIL PROBAS.`

5. **The old gate, for reference —** A test proving `:wat::core::i64::+` now fails at CHECK
   time with a remedy, and that an invented name under a promoted namespace fails too. Then remove
   the narrowing on purpose and confirm the gate goes green — a gate that survives removal of the
   door it guards is a claim. `NISI FRANGAS, NIHIL PROBAS.`

## The four questions

- **Obvious?** YES — `wat.i64/+` is where a reader looks for i64's `+`. `wat.core.i64/+` is not.
- **Simple?** YES — one prefix rename per family; the generic entry is untouched.
- **Honest?** YES, and only because of Stone C. Stones A+B alone ship a rename the checker cannot
  verify — which is precisely what the string move did.
- **Good UX?** YES — a wrong per-type numeric name becomes a check-time error with a remedy, where
  today it is silence followed by a runtime death.

## Out of scope, affirmatively cut

- **The `wat.core/+` defclause** — arc 256, banked, enabled by this stone. Named, not scheduled.
- **u8 / u16 / u32 / i8 / … the rest of the Rust primitive numerics** — the pattern first.
- **The TYPE half** (`:wat::core::i64` → `wat.type/i64`, 7,952 sites) — arc 251's namespace.
- **The tag-syntax flip** (`:wat::i64::+` → `wat.i64/+`) — the destination this naming is chosen for,
  not this stone's work.
