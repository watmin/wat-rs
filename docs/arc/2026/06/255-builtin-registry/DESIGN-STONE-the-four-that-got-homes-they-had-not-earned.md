# DESIGN — STONE: the four that got homes they had not earned

> Home #4 phase 2 (`56eb6ab3a`) killed `string_ops.rs` by giving every family it held a file. Four of
> those families got a **home** without getting a **right name**. The builder caught it:
> *"i don't know if i fully agree to them… this was more than i asked for."*

## The ruling

> *"i wanted to move uuid to `:wat::uuid::Uuid` for class and `:wat::uuid::*` for fns"* ·
> *"`/of` are meant to die as the ctor for a type is just itself invoked on its argument"* ·
> *"i say these are our next targets — `:wat::regex::*` and `src/regex/*.rs` feels fine?… we can grow
> it as we go"*

## ★ ALL FOUR ARE ONE CLASS — a name migration where the HANDLER DOES NOT CHANGE

That is the finding, and it is what makes this one stone instead of four. Every target already has a
working implementation; what is wrong is only what it is called.

| # | today | target | verbs | corpus sites |
|---|---|---|---|---|
| 1 | `:wat::core::Uuid/*` | `:wat::uuid::*` (+ `:wat::uuid::Uuid` the type) | 7 | **101** |
| 2 | `:wat::core::regex::matches?` | `:wat::regex::matches?` | 1 | 13 |
| 3 | `:wat::core::List/of` | `:wat::core::List` | 1 | 62 |
| 4 | `:wat::core::char/of` | `:wat::core::char` | 1 | 17 |

**193 sites, 10 verbs, zero behaviour change.** Re-register each intrinsic under its new name,
codemod the corpus, delete the old registration — the mechanism stone E proved, now with a working
rules-based codemod (`wat-scripts/fixes/rename-core-string-to-string.wat` is the shape).

## Why `/of` is FINISHING a migration, not starting one

This is not taste. **Every other collection type is already its own constructor**, measured:

```
(:wat::core::PersistentVector 1 2 3)   → #wat.core/PersistentVector [1 2 3]
(:wat::core::HashSet …)                → #{"a" "b"}
(:wat::core::Vector …) (:wat::core::Tuple …) (:wat::core::HashMap …) (:wat::core::PersistentMap …)

(:wat::core::List 1 2 3)               → UnknownFunction     ← the holdout
(:wat::core::char "x")                 → UnknownFunction     ← the holdout
```

Each working constructor is a thin match arm delegating to `crate::collection::eval::eval_*_ctor`.
`:wat::core::List` **already exists as a TYPE** (`types.rs`, `check.rs`, and
`runtime.rs:9266`'s `declared_type_name`) — it simply has no constructor arm. The body it would call
already exists: `eval_list_of`, now at `src/intrinsic/list.rs:33`.

So `List/of → List` is not new machinery. It is registering an existing handler under the name the
language already uses for every one of its siblings. `keyword/of` is already a kept gravestone;
these are the last two.

★ **And the type-name/head-position question answers itself** — `PersistentVector` is simultaneously
a type in annotation position and a constructor in head position, and has been for the whole corpus.
`List` and `char` inherit that, unchanged.

## ⚠ TWO CASINGS FOR ONE THING — resolve it here, since we are touching it

`src/` holds **both** `:wat::core::Char/of` and `:wat::core::char/of`. That is the casing question
from `109/NOTE-the-type-names-go-short-and-lowercase.md` showing up as an actual duplicate rather
than a preference. Whichever survives, only one should — and this stone is the moment it costs
nothing to settle.

## The file homes

```
src/uuid/       the namespace home; src/intrinsic/uuid.rs stays the registry home   (E's two-home split)
src/regex/      builder-approved: "src/regex/*.rs feels fine — we can grow it as we go"
```

`List` and `char` are single verbs and stay where phase 2 put them. ⚠ **One open question worth a
sentence, not a stone:** every other collection constructor's body lives in `crate::collection::eval`.
`List`'s lives in `src/intrinsic/list.rs`. Consistency argues for `collection::eval`; the registry
argues for where it is. Name the choice in the brief rather than letting a rider guess.

## ACCEPTANCE

1. **Each of the 10 verbs answers `metadata-of` under its NEW name, and the old name is
   `UnknownFunction`.** Both directions per verb — a rename that leaves the old name working is a
   bridge, and R9 is about bridges nobody demolishes.
2. **`(:wat::core::List 1 2 3)` and `(:wat::core::char "x")` evaluate** — the holdout closes.
3. **193 corpus sites migrated by codemod**, dry-run diffed byte-level first. No hand edits (R21).
4. **Idempotent AS A QUERY** — re-run the finder, get 0. The property the rules-based mechanism has
   and the char-walk never did.
5. **The doctest count is STILL 5.** Ten verbs change names; their `@example` lines change with
   them. If the count rises, an example is naming a verb that no longer exists.
6. **Only one of `Char`/`char` remains.**
7. Floor green accounted BY NAME; clippy 0.

## OUT OF SCOPE — affirmatively cut

- **A regex ENGINE.** `matches?` is one predicate with 13 call sites. `src/regex/` is a home to grow
  into, not a promise to fill this stone.
- **`:wat::core::Uuid` the TYPE's own spelling beyond the namespace move.** `:wat::uuid::Uuid` is the
  target; whether it later becomes `wat.type/uuid` belongs to the type-names note, not here.
- **The other 33 loose files at `src/` root.** This stone is four families that a carve relocated
  without asking. The rest of that population is its own arc.

---

# ⛔ GROUNDED 2026-08-25 — WHAT THE DRAW GOT WRONG

Everything above this line was drawn from `grep -c`. Every number in it is wrong, and one of the
acceptance rows was already satisfied. This section is the derived replacement; where the two
disagree, **this one measured, that one expected**
(`[[feedback_an_acceptance_row_is_a_pin_unless_it_derives_its_bar]]`).

The instrument is committed: **`wat-scripts/scratch-pad/probe-four-homes-census.wat`** — the FINDER
HALF of the migration, four rules over `wat/grep.wat`'s fact base, run over all 1567 tracked `.wat`
files. It is the FM 2-bis empirical probe: it proves the composition and hands back the count in the
only unit the idempotence claim can be stated in — **keyword-leaf occurrences**, not lines, not
prose, not string literals.

## The census, derived — and it is TWICE what the draw claimed

```
                            DRAWN   .wat leaves   .rs occurrences
:wat::core::char/of            17            67                33   ← the draw was off by 50
:wat::core::List/of            62            62                24
:wat::core::Uuid/nil            —            48                12
:wat::core::Uuid/v4             —            19                17
:wat::core::regex::matches?    13            13                 8
:wat::core::Uuid/v5             —            12                 9
:wat::core::Uuid/from-string    —            10                 8
:wat::core::Uuid/to-string      —             6                 7
:wat::core::Uuid/version        —             1                 6
:wat::core::Uuid/rfc4122-…?     —             1                 6
                            ─────   ───────────   ───────────────
                              193           239               130
```

Plus 5 `.wat` comment lines, 1 `.wat.bad` fixture, 1 CLI golden, 17 user-doc lines, and 4
`Char/of` gravestones. **~397 sites, not 193.**

## ★ THE TWO MECHANISMS ARE DISJOINT, AND THE SPLIT IS STRUCTURAL

- **`.wat` → the codemod.** 239 keyword leaves. Measured: **zero** genuine string-literal
  occurrences of these names anywhere in the corpus, so the kind guard costs nothing here and is
  kept anyway — it is what makes the count honest as well as the rewrite safe (stone E's rider
  found that defect the hard way).
- **`.rs` → by hand.** 134 occurrences across 22 files. **The codemod cannot reach them and must
  not**: in Rust every one of these names lives inside a `"…"` string literal, and the kind guard
  excludes string literals *by construction* — the guard that prevents corrupting a literal's quotes
  is the same guard that makes `.rs` unreachable. (`rename-core-string-to-string.wat`'s header
  usage line says `git ls-files '*.wat' '*.rs'`; run against a `.rs` file it returns zero matches,
  silently. That line is itself a pin. Do not copy it.)

## ⛔ FOUR DOORS THE DRAW NEVER NAMED — three of them EMIT the name

The draw's door table was a grep over `src/`. It missed `tests/**/*.rs` entirely and it missed the
whole emitter class — sites that do not *call* the verb, they *construct a call to it*:

| door | what it does | why it is invisible to a call-site census |
|---|---|---|
| `crates/wat-reader/src/parser.rs:406` | the **`\c` char literal desugars to `(:wat::core::char/of "x")` at PARSE TIME** | no corpus file contains the call; every `\c` in the corpus becomes one |
| `src/runtime.rs:21373` | `to-wat` renders a char back as `(:wat::core::char/of "c")` | it is the *output* side of the same round-trip |
| `src/closure_extract.rs:1994 / 2005 / 2015` | portable-value encoding emits `Uuid/from-string`, `char/of`, `List/of` calls | ditto — a wire format, not a call site |
| `src/rete/purity.rs:2213` | a **frozen alphabetical NAME list** (a ratchet) | it is data, not a call |

★ These four are why this stone cannot be verified by "the old name greps to zero." The `\c` literal
is the sharpest: rename `char/of` without `parser.rs`, and every `\c` in the corpus starts emitting
a call to a name that no longer exists — **and not one `.wat` file changes.** R9's tenth door, one
arc later, in a different disguise.

## ⚠ ROW 6 WAS ALREADY SATISFIED — there is no `Char`/`char` duplicate

The draw said *"`src/` holds both `:wat::core::Char/of` and `:wat::core::char/of`."* On disk:

```
src/closure_extract.rs:2001   // Stone 242.1 — renamed from :wat::core::Char/of to …   ← KEEP (FM 14 bucket C)
src/runtime.rs:21369          // Stone 242.1 — renamed from …                          ← KEEP
src/check.rs:17682            // Stone 242.1 — renamed from …                          ← KEEP
wat-tests/holon/char-round-trip.wat:3   ;; …the `(:wat::core::Char/of "x")` constructor. ← UPDATE (bucket B)
```

Three are **retirement-record comments** — the artifact FM 14 says to keep. `Char/of` was killed by
stone 242.1; only its gravestones remain, and my grep counted them as live code. The real work is
one stale prose line. Row 6 is rewritten accordingly.

## ⚠ THE Uuid **TYPE** SPLITS TO PHASE 2 — because bundling it falsifies the headline

`:wat::core::Uuid` the type: **20 corpus sites, 11 `src/` doors** — `types.rs` (the opaque list),
`edn_shim.rs:2235` (the `#uuid` EDN round-trip), `runtime.rs:9257` (`type_name`, i.e. what
`(:wat::core::type v)` **returns**), `value.rs:1451` (the gate), `check.rs:1534/17596`.

Renaming it changes an **observable shape** — the reflection answer and the EDN tag. This stone's
whole claim is *zero behaviour change*; bundling the type would make that claim false. So:

- **Phase 1 (this brief): the ten VERBS.** Handler untouched, name only.
- **Phase 2 (named here, not deferred): `:wat::core::Uuid` → `:wat::uuid::Uuid`.** Its verification
  is a different instrument — EDN round-trip, `type` reflection, goldens — and it lands against a
  `:wat::uuid::` namespace phase 1 has already settled. Stepping-stone test: YES.

`:wat::` is reserved at the root (`src/resolve/reserved.rs:14`), so `:wat::uuid::` and
`:wat::regex::` are substrate-owned the moment they are written. There is no namespace registry to
update — which is why stone E's `:wat::string::` needed none.

## ⚠ NO NEW EMPTY DIRECTORIES

The draw proposed `src/uuid/` and `src/regex/`. Neither has anything to hold — `intrinsic/regex.rs`
is 60 self-contained lines and `intrinsic/uuid.rs` is all handlers. E's two-home split existed
because `string_ops.rs` had *helpers* to rehome; these do not. **Creating a home with nothing in it
is precisely the error this stone is named for**, one level up. `src/regex/` and `src/uuid/` get cut
when they hold something. The builder's *"`src/regex/*.rs` feels fine… we can grow it as we go"*
is read as approval of the direction, and the growing is the trigger.

## ACCEPTANCE — replaces the seven rows above

1. **Each of the 10 verbs answers `metadata-of` under its NEW name, and the OLD name is
   `UnknownFunction`.** Both directions per verb. A rename that leaves the old name working is a
   bridge (R9).
2. **`(:wat::core::List 1 2 3)` and `(:wat::core::char "x")` evaluate.** Both are `UnknownFunction`
   at HEAD — verified this session on a freshly-built binary, not assumed.
3. **`\c` still round-trips.** `(:wat::kernel::println \x)` → `\x`, and the `to-wat` render is
   `(:wat::core::char "c")`. This is the parser/emitter triple; it is the row that catches the
   invisible door.
4. **239 `.wat` keyword leaves migrated by codemod**, dry-run on a `/tmp` copy and diffed first.
   No hand-edited `.wat` (R21).
5. **Idempotent AS A QUERY** — re-run the finder over the whole corpus, get **0**. The property the
   rules mechanism has and a char-walk never did.
6. **The doctest failure count is STILL 5** — the bar is derived, not expected: run
   `cargo nextest run --release --run-ignored all -E 'test(verify_examples_reports_no_failures)'`
   and read `left: N` from the panic. **N == 5 at HEAD, measured this session.** The ten `@example`
   lines rename with their verbs; if N rises, an example names a verb that no longer exists.
7. **`Char/of` survives only as its three retirement comments**; `wat-tests/holon/char-round-trip.wat:3`
   names `char/of`.
8. Floor green **accounted BY NAME**; clippy 0 under `-D warnings`.
