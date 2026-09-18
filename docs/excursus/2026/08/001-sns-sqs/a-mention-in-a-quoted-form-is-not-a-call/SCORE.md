# SCORE — a mention in a quoted form is not a call

**Struck 2026-09-18**, branch `sns-sqs`, from HEAD `4db14e8c3`. Nothing committed; the tree is left
dirty for the orchestrator.

---

## ⭑⭑ ROW 1 — ⭐ PHASE 1: ARC 198'S PURPOSE, AND THE CENSUS

### Arc 198's purpose, in its own words

From `docs/arc/2026/05/198-defn-restricted/DESIGN-STONE-a-restriction-governs-mention-not-head-position.md`,
quoting the builder and then deriving the rule:

> *"the purpose of the whitelist is to restrict who can call the thing being defined"*
>
> To call a thing you must first **name** it. Therefore:
>
> ### A restricted FQDN may not be NAMED by a function outside its whitelist — in any position.

What it was defending against, measured live in that stone and **proven executing**:

```clojure
;; ACCEPTED before 198 — the same restricted fn, named one line earlier
(:wat::core::defn :user::sneaky [] -> :wat::core::String
  (:wat::core::let [f :wat::kernel::str-double]
    (f "AB" 3)))
;; => --check EXIT=0. run EXIT=0. THE KERNEL FN EXECUTED FROM :user::
```

Three instances of one root: **value-position mention** (the general escape, above), the
**constructor trampoline** (`kwargs-construct` puts the real callee at `items[1]`, where a
head-only walker never looks), and a **written safety claim that was never attacked**
(`stdio.wat:358` reasoned about the *authoring* surface and was silent about the *reference*
surface — *"you never need a passing call site"*).

And the sharper statement, from the companion
`NOTE-restricted-call-fires-on-mention-not-call.md` (2026-08-28), which is the part that had to
survive:

> ★ **THE LOAD-BEARING PROPERTY IS DECIDABILITY, NOT POSITION.** The mention rule is not a coarse
> approximation of the real check that a smarter check could refine. It is *the only syntactically
> decidable form of the check*, and it is syntactic precisely because a value can be rebound.

That NOTE also records `apply` as the live laundering vector — `(:wat::core::apply
:wat::kernel::spawn-thread […])`, the verb in **argument** position — and notes the premise has
**strengthened**, not expired: arc 255's O-iv sweep widened `apply` to 81+ ALGEBRA doors.

### ⭐ Why the narrowing does not touch any of it

Every escape 198 closed is a name **this program resolves**: a `let` binding, an `apply` argument,
a trampoline's `items[1]`, a map value, a collection literal. The narrowing exempts exactly one
thing — a `WatAST::Keyword` sitting in **quoted data**, which this program never resolves at all.
It is source text handed to a *different program*, resolved at a *different time*, by a *different
caller*. Arc 198's width is over **position**; this is a distinction of **world**. And it costs no
decidability: `quote` / `forms` / `literal` / `quasiquote` heads are as syntactically visible as a
call head — the same property that let option B survive where option C died.

### ⭑⭑ THE CENSUS — form-aware, never text, and it comes back CLEAN

**Instrument 1 — `wat --grep` over the form tree** (`wat-scripts/scratch-pad/restricted-name-mention-census.wat`,
kept; it loads under `every_wat_scripts_file_loads`). The nine restricted bindings in the frozen
world — four `{:restricted-to …}` in `wat/` (`write-fd-raw`, `flood-stdout-raw`, `str-double`,
`spawn-program`) and five `#[restricted_to]` in `src/` (`IOWriter/from-fd`, `IOReader/from-fd`,
`close`, `spawn-thread`, `spawn-process`) — across **all 1773 `.wat` files** in `wat/`,
`wat-scripts/` and `tests/`:

```
rc::head     18   head-position calls
rc::nonhead   6
```

All six non-head mentions read, one by one:

| site | what it is |
|---|---|
| `wat/kernel/services/stdio.wat:361, 375, 383` | the **declaration sites** — the name at `items[1]` of its own `defn` |
| `wat/spawn.wat:362` | the **declaration site** of the `spawn-program` defclause |
| `wat-scripts/scratch-pad/255-stone-p5-b-restricted-yields-render.wat:32, 33` | `(:wat::core::def :user::thread-doc (:wat::core::render-doc :wat::kernel::spawn-thread))` — a **live value-position mention at top level**, the canary arc 255 P5-b kept on purpose |

⭐ **Not one of the six is inside a quoted form.** The two real non-call mentions are exactly the
kind 198 closed, they are *not* quoted, and the narrowing leaves them alone — the canary still
reads as it did. (It survives for an unrelated reason it documents itself: a top-level form has no
enclosing fn, so the check is skipped rather than failed. That is a **different** open hole, named
in the canary and in the NOTE; it is not this stone's, and this stone does not widen it.)

**Instrument 2 — the walker counting its own delta.** Because a name census is only as good as its
name list (and `tests/` fixtures declare restrictions on names of their own — `:my::kernel::…`,
`:test::restricted-target`, `:probe::guarded`, and 87 `:restricted-to` occurrences in all), a
temporary census was compiled into `walk_for_restricted_call`'s call site: run the **pre-narrowing
arc-198 walk** alongside the new one and record every error the narrowing removes. Non-vacuity
control first — the witness program reports **9** suppressions under the same cache settings the
sweep uses.

- **Whole `.wat` corpus** (1773 files, each `--check`ed): the census file was **never created**.
  **Zero** suppressions. ⚠ That sweep ran on the tree *before* this stone added its own four
  fixtures; the floor sweep below covers them.
- **Whole floor** (5306 tests, inline Rust-string fixtures included): 30 raw lines, **21 distinct**
  (the repeats are separate test processes checking the same fixture), across **12** enclosing fns —
  and every one is a witness or a fixture written *by this stone*:

  | callee | distinct suppressions | where |
  |---|---|---|
  | `:user::main` | 9 | the nine `…::service-forms` bodies, all inside `(:wat::core::forms …)` |
  | `:user::spawn::service-locus` | 10 | those nine **plus** `:wat::spawn::ProcessOpts/launch` |
  | `:my::kernel::restricted-fn` | 2 | `tests/kernel/wat_arc198_quoted_mention_ok_templates.wat` — this stone's own new fixture |

# ⭐ NO SITE RELIES ON A NON-CALL MENTION FIRING FROM QUOTED DATA. NOT ONE.

The instrument was then deleted; `git diff` carries no trace of it.

---

## ⭑⭑ ROW 2 — THE WITNESS DIES. BOTH OF THEM.

```
$ ./target/release/wat …/base.wat              "hi"   exit 0      (non-vacuity)
$ ./target/release/wat …/w1_user_main.wat      "hi"   exit 0      ⭐ WITNESS 1
$ ./target/release/wat …/w2_service_locus.wat  "hi"   exit 0      ⭐ WITNESS 2
```

Witness 1 is `(:wat::core::defn :user::main {:restricted-to [:my::]} [] -> :wat::core::nil
(:wat::kernel::println "hi"))` — verbatim the DESIGN's program. Witness 2 adds
`(:wat::core::defn :user::spawn::service-locus {:restricted-to [:my::]} [] -> :wat::core::i64 1)`.

The delta census names exactly what stopped firing, and the numbers match the Tier B SCORE's
measurement to the unit: **9** for witness 1 across `wat/cache.wat` ×2,
`wat/kernel/services/stdio.wat` ×3, `wat/query/mem.wat`, `wat/query/sqlite-store.wat`,
`wat/telemetry/journal.wat`, `wat/telemetry/span.wat` — six files; **10** for witness 2, those nine
plus `:wat::spawn::ProcessOpts/launch` (`wat/spawn.wat:613`).

Gated: `tests/kernel/wat_arc198_def_restricted.rs` —
`quoted_mention_users_own_restricted_main_does_not_redden_the_stdlib` and
`…_service_locus_…`.

---

## ⭑⭑ ROW 3 — A REAL VIOLATION STILL FIRES. DRIVEN, THREE SHAPES.

`:wat::kernel::str-double` is whitelisted `[:wat::kernel:: :wat::test::]`; the caller is `:user::`.

| shape | source | `DefRestrictedCallerNotAllowed` | exit |
|---|---|---|---|
| head-position call | `(:wat::kernel::str-double "x" 2)` | **1** | 1 |
| ⭐ `let` alias (198's own witness) | `(:wat::core::let [f :wat::kernel::str-double] (f "AB" 3))` | **1** | 1 |
| ⭐ `apply` laundering (the NOTE's vector) | `(:wat::core::apply :wat::kernel::str-double ["AB" 3])` | **1** | 1 |

Verbatim, from the `let`-alias run:

```
#wat.check/DefRestrictedCallerNotAllowed {:message "`:wat::kernel::str-double` has a restricted
caller whitelist [:wat::kernel:: :wat::test::]; the enclosing fn `:user::sneaky` does not match any
entry …" :callee ":wat::kernel::str-double" :enclosing-fn ":user::sneaky"
:prefixes [":wat::kernel::" ":wat::test::"]}
```

The pre-existing `def_restricted_*` family (6 tests, including
`def_restricted_value_position_alias_denied`) is green and untouched.

---

## ⭑⭑ ROW 4 — ⭐ THE BOUNDARY: AN UNQUOTED MENTION INSIDE A QUASIQUOTE STILL FIRES

This is the whole implementation, and the place a plausible fix goes wrong. Five programs, one
character of difference between the first two:

| program | fires? |
|---|---|
| `` `(:wat::core::do ~(:wat::core::apply :wat::kernel::str-double ["AB" 3])) `` | ⭐ **YES — 1** |
| `` `(:wat::core::do (:wat::kernel::str-double "x" 2)) `` | no — 0 |
| `` `(:wat::core::do [~(:wat::core::apply …)]) `` (escape inside a **bracketed** form) | ⭐ **YES — 1** |
| `` `(:wat::core::do ~@(:wat::core::apply …)) `` (`unquote-splicing`, the other escape head) | ⭐ **YES — 1** |
| `` `(:wat::core::do ~(:wat::core::quote (:wat::kernel::str-double "x" 2))) `` | no — 0 |
| `(:wat::core::quote (:wat::kernel::str-double "x" 2))` | no — 0 |
| `(:wat::core::forms (:wat::core::defn :user::main … (:wat::kernel::str-double "x" 2)))` | no — 0 |

The last row of the escape set is the composition test: an escape resumes the **full** walk, which
re-enters the quote boundary, so a quote *inside* an unquote is data again — correct in both
directions.

Gated: `quoted_mention_unquote_escape_inside_a_quasiquote_still_fires` (whose fixture puts the
restricted FQDN in **argument** position inside the escape, so it is arc 198's laundering shape,
not a head call) and `quoted_mention_template_text_is_not_a_mention_by_the_enclosing_fn`.

### How it is implemented, and why not by hand

`walk_for_restricted_call` consults **`resolve::boundary::quote_boundary`** — the module whose own
doc says it exists because *"the chains **drifted**"* when each pass answered "what is the
argument-evaluation shape of this head?" with its own `if`-chain. The walker becomes its **fourth**
consumer, not a fifth hand-rolled copy:

- `Boundary::AllData` (`quote` / `forms` / `literal`) — walk `items[0]` (the boundary head is this
  form's own live call head), return. No child is code and there is no escape out.
- `Boundary::Quasiquote` — walk `items[0]`, then descend the template through
  `walk_restricted_quasiquote_template`, which resumes the full walk **only** under an
  `is_unquote_escape` head. That helper is deliberately the same shape as
  `resolve::quote::check_quasiquote_template` (the 4th quasiquote descent in the tree, after
  `resolve::quote`, `resolve::normalize` and `closure_extract`), including its reading of a nested
  quasiquote, so the two cannot disagree about what a nested template means.
- `MatchesSubject` / `Match` / `MakeRule` / `Ordinary` — **unchanged, deliberately.** Their "data"
  regions are DSL argument shapes resolved in *this* program, not child-program source. Narrowing
  them would be narrowing past the witness (trap-door 1). Named in row 11.

---

## ⭑ ROW 5 — THE STALE COMMENT IS CORRECTED, AND SO IS THE WALKER'S DOC

`check.rs:722` said:

> *"For every fn body, walk every call site; if the call **head** names a binding with a
> `:restricted-to` key…"*

— the **pre-arc-198** rule, still sitting above code that had fired on every mention for a month.
A reader who trusted it could not have predicted a single one of the nine stdlib errors. It now
states the rule the code means (mention, in any position, **except** quoted data), and carries a
dated line saying what it used to say and why that mattered — so the correction is legible rather
than invisible.

The walker's own doc gains a `⛔ QUOTED DATA IS NOT A MENTION` section: the witness and its count,
why arc 198's decidability property is untouched, and — with the two-line `~` contrast — that
exempting a whole quasiquote would reopen 198 through `~`.

---

## ⭑⭑ ROW 6 — FLOOR

```
     Summary [ 257.579s] 5306 tests run: 5306 passed, 22 skipped
```

`.floor/2026-09-18T23-39-00Z/` · exit=0 · **no `ARM.txt`** (the script writes one only on red).

### ⛔ THE FIRST FLOOR WAS RED, AND IT IS REPORTED, NOT ROUNDED

The census floor (`.floor/2026-09-18T23-29-30Z/`, ARM captured, **not re-run**):

```
     Summary [ 261.361s] 5306 tests run: 5305 passed, 1 failed, 22 skipped
        FAIL [   0.602s] (2490/5306) wat::function
          probe_tier_b_stdlib_verdict_is_not_bake_fixed::a_user_restriction_on_user_main_reddens_stdlib_bodies
```

The arm: `tests/function/probe_tier_b_stdlib_verdict_is_not_bake_fixed.rs:79:10` —
`.expect_err("a ':restricted-to' on ':user::main' must still redden the stdlib bodies that quote
that name — if this passes, a check sweep stopped running")`. It is the Tier B refutation's own
witness, committed at `8ca49e502`, and it **asserts the defect this stone repairs**. It was
inverted, not silenced:

- renamed `a_user_restriction_on_user_main_no_longer_reddens_stdlib_bodies`; it now asserts the
  world freezes, plus a non-vacuity check that the fixture still declares `:user::main` (so the
  gate cannot pass on an empty premise);
- its doc records verbatim what it asserted until today — nine errors, six files — because that
  measurement was correct;
- ⚠ **and it records what the inversion COSTS, out loud.** It passed before because
  `check:restricted-call(ALL fns)` *fired*; it passes now because nothing remains for it to fire
  on, and it would keep passing if that sweep were **elided entirely**. It is no longer a control
  on the sweep. A future Tier B must derive a fresh one, and the module doc says so.
- The fixture keeps its `_neg_restricted.wat.bad` path so Tier B's SCORE stays followable.

`the_user_declarable_surface_inside_stdlib_bodies_is_exactly_two` was **not** touched and is green:
it counts raw leaves, quoted or not, on purpose.

---

## ⭑ ROW 7 — CLIPPY + `--no-run`

```
$ cargo clippy --release --workspace --all-targets
    Finished `release` profile [optimized] target(s) in 16.73s
$ cargo clippy --release --workspace --all-targets 2>&1 | grep -cE '^(warning|error)'
0
```

**0 warnings, 0 errors** — the number read from the output, not assumed. `cargo nextest run
--release --no-run`: `Finished` in 1m 34s, no diagnostics.

---

## ⭑ ROW 8 — CIRCUIT, BYTE-IDENTICAL

```
"timeout=yes;discarded=yes;redial=Connected;retry-on=fresh"
"n=2000;m=4;j=3;total=8000;distinct=8000;dup=0;workers=12;empty=1;seen-recorded=8000;seen-skipped=0;…"
… pub-exh-last=none … bp-delay=0 …
```

`distinct=8000;dup=0` ✅ · `timeout=yes;discarded=yes;redial=Connected;retry-on=fresh` ✅ ·
`pub-exh-last=none` ✅ · `bp-delay=0` ✅. exit 0. Process table verified quiet first; run through
`scripts/capped.sh --limit 8g` under `timeout -k`.

---

## ⭑ ROW 9 — IS THE EIGHTH DOOR CLOSED? **YES, ON TODAY'S CORPUS — BY MEASUREMENT, NOT BY CONSTRUCTION**

The question asked, and only this one: *can `CheckEnv::get_binding_metadata` still be reached for a
stdlib body by any user write?*

`get_binding_metadata` has exactly **one** live read in the tree — `check.rs:1679`, inside
`walk_for_restricted_call` (`src/check/env.rs:418` is the definition; the third hit is a doc
mention). So the door's reachability is entirely this walker's walk.

Two measurements compose:

1. **The surface is two.** `the_user_declarable_surface_inside_stdlib_bodies_is_exactly_two` — a
   closure over every fn in the frozen bare world, green today — reports the complete set of names
   reachable from a stdlib body that a user program could legally declare:
   `:user::main`, `:user::spawn::service-locus`.
2. **Both are wholly inside quoted data.** The delta census names **every** occurrence of those two
   names from a stdlib body: **19** — `:user::main` in 9 bodies, `:user::spawn::service-locus` in
   10 — across **10 distinct stdlib functions**, and the narrowing suppressed **all 19**. None
   survives in a live position.

⇒ After this change the walker performs **no `get_binding_metadata` lookup on a user-writable name
while walking a stdlib body**. Every name it does look up from a stdlib body is `:wat::`-prefixed,
which `resolve::gate` refuses to a `Privilege::User` program — so the door is still *called*, but
its result cannot differ between two user programs.

⚠ **State the bound honestly.** This is closed by a **corpus fact**, not by construction: a third
user-declarable name appearing *unquoted* in a stdlib body would reopen it, and the wall against
that is the surface test's pinned list — which would go red, but only after someone wrote it.
Proving the closure is Tier B's job; this row reports the observation and nothing more.

---

## ⛔ ROW 10 — OUT-OF-SCOPE ITEMS, UNTOUCHED

`git diff --numstat` is the whole answer: `src/check.rs` **+114/-3** in five hunks (the call-site
comment, the walker's doc, the boundary branch, the new template helper) and two test files
(**+58/-34**, **+64/-0**). No other `src/` file is in the diff.

| named out of scope | state |
|---|---|
| the blame-inversion diagnostic (blame the DECLARER, not the mentioner) | untouched — after the fix there is nothing left to re-blame *here*, but a genuine violation still names the mentioner. Its own stone. |
| the rendezvous contradiction (`:user::main` is what the kernel invokes) | untouched — and ⭑ **still the one message nobody prints**: the witness now compiles clean, so the user is told nothing at all about a `:restricted-to` on their entry point. Worth noting the fix *moved* this defect from "nine wrong errors" to "silence". Its own stone. |
| Tier B | not re-attempted. The eighth door is closed (row 9); the closure is not proved, and the inverted witness's cost to a future Tier B is written into that test's doc. |
| `runtime.rs:2112`, `check.rs:6068`, `env.rs:456`, `.config/nextest.toml` | untouched — `src/runtime.rs` and `src/check/env.rs` are not in the diff at all, and `check.rs`'s hunks are at 722/729/1637/1692/1737. |

---

## ⭑ ROW 11 — WHAT I COULD NOT SETTLE

1. ⭐ **Whether `Boundary::MakeRule`'s quoted `:when` / `:then` vectors should be exempt too.** They
   are the one remaining boundary whose data region is genuinely *quoted*, and by the argument in
   row 1 a restricted name in a rete rule's `:when` vector is arguably resolved by the rule engine
   rather than by the enclosing fn. I left it firing because the witness does not reach it and
   trap-door 1 forbids narrowing past the witness. **What would settle it:** a program that puts a
   restricted FQDN in a `make-rule` `:when` vector and asks whether the rete lowering ever *calls*
   it from the enclosing fn's frame. Zero corpus sites exercise it today (census instrument 2 found
   none), so it is unforced either way.
2. **Whether `binding_metadata` is written at all for a scalar `def`.** Tier B's SCORE recorded a
   measured asymmetry — `(:wat::core::def :user::spawn::service-locus {:restricted-to [:my::]} 1)`
   is *green* where the `defn` form was red — and named it uninvestigated. This stone does not
   change it and did not investigate it either; it is orthogonal to quoting. **What would settle
   it:** a probe on the `register_defines` path for a non-fn-shaped `def`.
3. ⚠ **The top-level-mention hole is still open and is now the only non-call escape left in the
   corpus.** `wat-scripts/scratch-pad/255-stone-p5-b-restricted-yields-render.wat` launders
   `:wat::kernel::spawn-thread` through a top-level `def` — the check is *skipped* (no enclosing
   fn), not failed. The census found it; the narrowing neither widens nor closes it. It deserves
   its own stone, and the canary file already says so in its own header.
4. **The `.bad` suffix on a fixture that must now freeze green.** `_neg_restricted.wat.bad` is kept
   under its old name so Tier B's SCORE stays followable by path, and the test doc says the suffix
   is historical. That is a name describing behaviour the file no longer has — the exact defect
   class row 5 is about — traded knowingly against breaking a sibling document's citations.

## ⭐ WHAT SURPRISED ME

**The narrowing needed no new language fact.** `resolve::boundary` already encodes "which heads
capture their arguments as data", exists *specifically* because three passes had drifted on that
question, and its `is_unquote_escape` already encodes the `~` boundary for three descents. The
walker was the pass that never asked. The fix is the walker joining an existing conversation — and
the reason the `~` trap-door was cheap to avoid is that somebody had already paid for it.

---

## ⭑ REGRADED BY THE ORCHESTRATOR ON HIS OWN RUNS, 2026-09-18

```
floor    Summary [ 256.008s] 5306 tests run: 5306 passed, 22 skipped
         .floor/2026-09-18T23-48-48Z/ · exit=0 · NO ARM.txt
clippy   0  ← read from the output
```

| driven control | my own result |
|---|---|
| the witness dies | ✅ `{:restricted-to [:my::]}` on `:user::main` → `"hi"`, exit 0 (was 9 errors / 6 stdlib files) |
| a real violation still fires | ✅ head call from a disallowed namespace → **1** error |
| **`apply` laundering still fires** | ✅ **1** error — arc 198's actual escape, intact |
| the 51 restriction tests | ✅ all pass, including `quoted_mention_unquote_escape_inside_a_quasiquote…`, `quoted_mention_template_text_is_not_a_mention…`, `def_restricted_value_position_alias_denied` |

### ⭐ WHY THIS DOES NOT REOPEN ARC 198 — the argument is the executor's and it is right

Every escape 198 closed is a name **this program resolves**: the value-position alias, the
`kwargs-construct` trampoline, `apply` laundering. The narrowing exempts only names in **quoted data**,
which this program **never resolves**. ★ **A distinction of WORLD, not of position** — and quote heads
are as syntactically visible as call heads, so 198's stated load-bearing property (*"THE LOAD-BEARING
PROPERTY IS DECIDABILITY, NOT POSITION"*) is untouched.

Implementation routes through **`resolve::boundary::quote_boundary`** — the module that exists *because*
these chains drifted before — making the walker its **fourth consumer** rather than a fifth hand-rolled
descent. `Match`/`MatchesSubject`/`MakeRule` keep 198's width deliberately.

### ⛔ MY OWN CONTROL WAS INVALID TWICE, AND I NEARLY REPORTED A HOLE THAT ISN'T THERE

Attempting row 4 myself:

1. **bare `nil` main** → refused with `UselessMain` *before the check ran*. Both my quoted and unquoted
   fixtures returned "0 errors" **vacuously**.
2. rebuilt with a real body → the unquoted fixture still returned 0, because **the macro is never
   invoked**, so the unquote is never reached.

⛔ **I was one sentence away from writing "the `~` hole is open."** The executor's control is real, is
named `wat_arc198_quoted_mention_bad_unquote_escape.wat`, and passes. ★ **Fourth invalid fixture of mine
today** — `noop.wat`, `ls -S` on the cache, a previous build's fingerprint, and now this. Every one
produced a clean-looking number that meant *my instrument missed*.

### Its first floor was RED, and the red is the most interesting thing in the stone

`.floor/2026-09-18T23-29-30Z/`: `5305 passed, 1 failed`. The arm was
`probe_tier_b_stdlib_verdict_is_not_bake_fixed.rs:79` — **the test that ASSERTS the defect this stone
repairs.** It inverted the test, kept the nine-errors-six-files measurement verbatim in its doc, and
then wrote out loud that ⛔ **it is no longer a control on the sweep**: it would now pass even if
`check:restricted-call` were elided entirely, **so a future Tier B must derive a fresh control.** That is
a test quietly ceasing to test what its name claims — caught and named rather than left to rot.

### The eighth door: closed by MEASUREMENT, not by construction — and it says so

One live read (`check.rs:1679`); the surface is **two names**; all **19** of their occurrences across 10
stdlib bodies are quoted and all 19 suppressed; every remaining lookup from a stdlib body is
`:wat::`-prefixed, which `resolve::gate` refuses to a user. ⚠ **A third *unquoted* user-declarable name
would reopen it.** That is the honest ceiling on what Tier B may now claim, and the prize returns to the
full **124.65 ms** rather than the 121.58 ms partial.

### Left open and named

**The top-level-mention hole** — arc 255's canary launders `spawn-thread` through a top-level `def`,
where the check is **skipped, not failed**. It is now *the only non-call escape in the corpus*, and it is
its own stone. Also: `Boundary::MakeRule`'s quoted `:when`/`:then` vectors (zero corpus sites, left
firing per trap-door 1); whether a scalar `def` writes `binding_metadata` at all; and the
`_neg_restricted.wat.bad` filename that now names behaviour its fixture no longer has, **kept knowingly**
so Tier B's SCORE stays followable by path.
