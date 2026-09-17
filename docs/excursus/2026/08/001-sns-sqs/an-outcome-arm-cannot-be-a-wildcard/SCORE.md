# SCORE — an outcome arm cannot be a wildcard (phase 1, REPORT-ONLY)

**Struck 2026-09-16**, on `sns-sqs` from `3fd01f50a`. The builder's item **3**, sequenced after item 1
(`every-status-arm-is-named`). ⛔ **Report-only, as drawn.** No wildcard anywhere in the corpus became
an error; `src/check.rs`'s exhaustiveness verdict is unchanged (the `true` that a wildcard produces is
the same `true`), and the census collects nothing unless a reporting test switches it on.

## ⭐ The headline, in five lines

1. **The checker route WORKS on macro-generated sites.** Trap-door 3 survives, and the proof is
   airtight rather than suggestive: `wat/query/mem.wat` contains **zero** literal `_` arms yet the
   checker reports **two** in-scope `RecvOutcome` wildcards at its `defservice` line. They can only
   have come from `wat/service.wat`'s quasiquoted body. Quoted in row 1.
2. ⚠ **But span substitution means the diagnostic cannot say WHERE.** A generated wildcard reports
   the `defservice` CALL site, never `wat/service.wat:2978`. That is a **correction to EXPECTATIONS
   row 5** and it constrains what a phase-2 error message can say. Named, not glossed.
3. **The live in-scope count is 38, and 33 of them are the same idiom** (`recv'`, name `Message`,
   `_` the other five). A zero-exemption gate is viable, and the cheapest route to it is **one
   stdlib helper, not 38 hand-named arms**.
4. **Two predicted populations are ABSENT from live code.** Zero live `ServiceEvent` wildcards
   (estimate: 3) and zero live `LociDiedError` ones (estimate: 2). Shipping the absence.
5. **The whole-corpus walk is NOT in the floor**, and that is a named cost, not a tidy-up: it takes
   **762.5 s**. The floor weighs the instrument in 1.4 s; the wide numbers below come from my own
   runs, marked as such.
6. ⛔ **MY FIRST FLOOR WENT RED — two failures, both caused by this stone's own new code**, caught by
   two walls that already existed (`one_name_grammar`, `no_loose_string_assert`). Arms quoted whole,
   not re-run before capture, fixed at the source rather than runed. Floor 2 is green.

## The twelve rows

| # | what | result |
|---|---|---|
| 1 | ⭑⭑ diagnostic sees a MACRO-GENERATED site | ✅ **YES**, and provably macro-authored. Quoted below |
| 2 | ⭑⭑ census per enum | ✅ table below — **8** enums fire, 266 distinct sites corpus-wide |
| 3 | ⭑⭑ live vs tests/probes | ✅ **LIVE 38 · PROBE/TEST 228** |
| 4 | ⭑ generated vs literal | ✅ **14 span-substituted · 252 literal** corpus-wide; **6 · 32** live |
| 5 | ⭑ the known-remaining two are found | ⚠ **FOUND, but not by line number** — a CORRECTION; see below |
| 6 | ⛔ the three per-surface sites NOT flagged | ✅ **0 in scope**; 163 of 169 declined enum paths are the `::Reply` / `<Op>Response` families |
| 7 | ⛔ Option/Result/domain NOT flagged | ✅ `:my::Color` declined **on the corpus**; Option/Result cannot reach the hook at all; DRIVEN control too |
| 8 | ⭑⭑ REPORT-ONLY | ✅ three tests print and pass; no wildcard is an error; `+27` lines in `check.rs`, all inside the existing `wildcard_seen` arm |
| 9 | ⭑⭑ Floor | ⛔ **FLOOR 1 WAS RED — 2 failed, both MINE**, arms quoted whole. Fixed at source. **FLOOR 2 GREEN, 5274/5274, no `ARM.txt`.** Both reported |
| 10 | clippy + `--no-run` | ✅ clippy **0 warnings / 0 errors**; `--no-run` clean; `build --release` clean (re-run after the fix) |
| 11 | ⭑ exemptions REPORTED, not chosen | ✅ three options, a recommendation, and its cost |
| 12 | ⛔ would NOT have caught the three missing-form failures | ✅ stated plainly below |

---

## Row 1 — ⭐ TRAP-DOOR 3, and why this is a proof and not an impression

`tests/lint/outcome_wildcard_census.rs::a_macro_generated_site_is_reached`, one root, 0.45 s:

```
⭐ MACRO-GENERATED in-scope wildcard sites the CHECKER reached (root: benches/perf_arc278_fire_baseline.wat):
    wat/query/mem.wat:618  :wat::kernel::RecvOutcome  firings=2  swallowing [Closed,Stopped,TimedOut,Lost,Malformed]
        span points at: (:wat::service::defservice :wat::query::mem-store
```

⭐ **What makes this airtight rather than suggestive**, measured separately:

```
wat/query/mem.wat                literal-'_'-arm occurrences: 0
```

`wat/query/mem.wat` has **no `_` arm anywhere in its text**, and the line the checker points at is a
`defservice` head, not a `match`. So the two `RecvOutcome` wildcards it reports exist only after
expansion — they are written in `wat/service.wat`'s quasiquoted body, where the arm head is the
unquoted `~resp-sym` and **no regex can attribute them to an enum**. The checker knows the enum
(`:wat::kernel::RecvOutcome`), knows a `_` was used, and knows which five variants it swallows.

A text lint sees `(_ …)` in `service.wat` and cannot say what it is a wildcard *over*; it sees
nothing at all in `mem.wat`. That asymmetry is the whole instrument choice, and it holds.

## Row 5 — ⚠ THE CORRECTION: the two sites are found, but the instrument cannot print their line

EXPECTATIONS row 5 asks for `wat/service.wat:2978` and `:3089` to **appear**. ⛔ **They cannot, and
no fix to my code changes that** — the reason is a property of the expander, documented in this repo
before this stone existed (`tests/lint/span_substitution_justified.rs`):

> *"`kwargs-lower` rewrites `(svc/start …)` into `(svc/start$impl …)`, the emitted call carries the
> TEMPLATE's span … Substitution destroys the location at the point of substitution; nothing
> downstream recovers it."*

So every form produced by expanding `defservice` carries the **`defservice` call's** span. The
checker reaches the wildcard, types it, and reports it at `mem.wat:618` / `circuit.wat:370` / …

**What identifies them instead — three independent signatures, all measured:**

| signature | observed |
|---|---|
| the enum | `:wat::kernel::RecvOutcome`, a fixed six-variant builtin (`src/types.rs:1869`) |
| the swallowed set | `[Closed, Stopped, TimedOut, Lost, Malformed]` — exactly 5 of 6, `Message` named. Item 1 filed it as *"five behaviour decisions × two sites"* |
| **TWO per expansion** | `firings = 2 × roots` at **every** paging-capable `defservice` site: `mem.wat:618` 3536/1768 · `sqs.wat:195` 56/28 · `sns-fanout.wat:70` 22/11 · `faulting-store.wat:43` 8/4 · `circuit.wat:370` 2/1 |

And I read the source to check the signature names the right forms — `wat/service.wat:2978` and
`:3089` are both `(:wat::core::match ~r-sym ((:wat::kernel::RecvOutcome::Message ~resp-sym) …) (_ …))`,
one in the chunked-request path and one in `page-all-body`. So: **the population is found and
counted; its authoring line is not recoverable.**

⭑ **This matters for phase 2.** A hard error here would say *"`:wat::kernel::RecvOutcome` match at
`wat/query/mem.wat:618` (a `defservice` call) is missing arms for Closed, Stopped, TimedOut, Lost,
Malformed"* — correct, useful, and pointing at a file the author must not edit. Either phase 2
accepts that indirection (the enum + swallowed set is enough for a reader who knows the macro), or it
needs the expander to carry provenance, which is a **separate, larger stone** and is named here
rather than discovered later.

⚠ The `firings = 2 × roots` law has exceptions I must not hide: two probe sites
(`probe-disrupt-reaps-and-reacquires.wat:73`, `probe-parked-waiters-stop.wat:32`) report **1**
`RecvOutcome` firing at a `defservice` head. Both files **do** contain literal `_` arms (9 and 4), so
those are the author's own wildcards inside the service body, whose spans the expander substituted
the same way. Span substitution therefore separates "substituted" from "intact", **not** "macro-
authored" from "user-authored" — the second distinction needs the host file's `_` count, which is how
`mem.wat` (zero) settles row 1.

## Row 2 — the census, per enum

**Whole corpus** (`WAT_OUTCOME_CENSUS=full`, 1815 roots walked of 1870 tracked `*.wat`, 762.5 s,
test PASSED). *Sites* are distinct `file:line:col` × enum; *firings* counts re-expansion across
roots; *generated* = span substituted; *live* = span file outside `tests/ wat-tests/ docs/ benches/
wat-scripts/scratch-pad/`.

```
enum                                        sites  firings  generated   live
:probe::crash::Status                           1        1          0      0
:wat::holon::VectorDecodeOutcome                3        3          0      0
:wat::kernel::ConnectOutcome                   89       94          2      3
:wat::kernel::RecvOutcome                     163     4092         11     34
:wat::kernel::SendOutcome                       1        1          0      0
:wat::service::CallOutcome                      1        4          1      1
:wat::service::StopOutcome                      2        2          0      0
:wat::spawn::ServiceEvent                       6        6          0      0
                                   TOTAL      266     4203         14     38
```

⭐ **`:probe::crash::Status` is the `Status` arm firing on the real corpus** — see the ⚠ Status
section. `:wat::kernel::SendOutcome` = 1 site, a scratch probe: **the DESIGN's estimate of "1 live
SendOutcome (a scratch probe)" is confirmed exactly**, and it is correctly bucketed as probe, not live.

### ⚠ What the scope predicate is, and where it is WIDER than the DESIGN's list

The rule is a property, not a path list (`src/check/outcome_wildcard_census.rs::classify`): a
`:wat::`-namespaced enum whose last segment ends in `Outcome`; **or** `:wat::spawn::ServiceEvent`;
**or** any enum whose last segment is `Status` *and* whose declared variants are exactly
`Started · Stopped · Hibernated · PeersAllowed · PeersDenied · Faulted`.

I confirmed the enum list against `src/types.rs` as instructed. All **13** the DESIGN named are
covered. The suffix rule also admits **12 more** registered `*Outcome` enums the DESIGN did not list
— `ReadOutcome`, `ReadJsonOutcome`, `ReadForeignOutcome`, `ReadFrameOutcome` (×2),
`FormOutcome`, `VectorDecodeOutcome`, `CombineOutcome`, `CosineOutcome`, `DotOutcome`,
`:wat::service::Outcome`, `:wat::service::SelfOutcome`. That widening is deliberate (a new `*Outcome`
minted next month is in scope the day it is declared) and **costs nothing today**: of the twelve, only
`VectorDecodeOutcome` fires at all, 3 sites, **0 live**.

⚠ **`:wat::kernel::LociDiedError` is NOT in scope under my predicate** — it carries no `Outcome`
suffix and the DESIGN's IN list omits it — yet the DESIGN's estimate counted 2 live ones. Measured: it
is the **largest single declined population**, **16 sites**, and **0 of them are live** (the
live-scope run's declined table has no `LociDiedError` row at all). It is an error enum where an
unread variant is a swallowed failure, so whether it joins the rule is a **builder ruling I am not
taking**. Flagged, with the count.

## Rows 3 & 4 — the splits

```
LIVE 38 · PROBE/TEST 228   ||   MACRO-GENERATED 14 · LITERAL 252 · UNDETERMINED 0
```

The 38 live sites, by file — and note **all of them are in five files**:

| file | sites | of which span-substituted |
|---|---|---|
| `wat-scripts/fanout/circuit.wat` | 16 | 2 (both at `:370`, the `defservice` head) |
| `wat-scripts/topic/sns-fanout.wat` | 10 | 1 (`:70`) |
| `wat-scripts/queue/sqs.wat` | 9 | 1 (`:195`) |
| `wat-scripts/query/faulting-store.wat` | 2 | 1 (`:43`) |
| `wat/query/mem.wat` | 1 | 1 (`:618`) |
| **frozen stdlib `wat/` total** | **1** | 1 |

⭐ **Only ONE of the 38 live sites is in the frozen stdlib**, and it is the generated pair. Everything
else is in the excursus's own SNS/SQS showpiece programs.

⭐ **And the live population is one idiom, 33 times over.** Of the 34 live `RecvOutcome` sites, **33**
have `named=1 missing=[Closed,Stopped,TimedOut,Lost,Malformed]` — identical. The single exception is
`wat-scripts/queue/sqs.wat:2097` (`named=2`, `Lost` also named). Row 11 turns on this.

Authored `_` arms in live code, which is the number a gate would charge (**not** the same as the site
count, because the 5 generated `RecvOutcome` sites share ONE authoring location):

```
32  literal arms, one per site
 2  in wat/service.wat  (:2978 + :3089 — one edit fixes all 5 generated sites, and every service)
 4  in circuit.wat's own defservice body (the CallOutcome arms, span-substituted to :370)
── 
38  arms, in 6 files
```

### Coverage, stated so the numbers are not overstated

- **1815 roots of 1870.** The 55 `wat/**` files are not used as roots — a baked stdlib file loaded as
  a standalone program does not check (measured: 15 of 18 `defservice`-bearing ones error). They need
  no visit: the stdlib is type-checked with **every** root, which is why `mem.wat:618` shows
  `triggered by 1768 root(s)`.
- **111 roots did not type-check** (deliberately-bad fixtures under `tests/`, `wat-tests/`). Their
  partial contribution is kept and the count is printed. One of them
  (`tests/cli/wat_cli__freeze_time_panic.wat`) raises at FREEZE time — a panic, not an `Err` — and
  catching it is what let the walk reach the probe corpus at all.
- A variant counts as *named* only for a fully-general arm (`check.rs`'s
  `Coverage::EnumVariant { full: true }`), so the `missing` column can over-report for a narrowed arm.
  It cannot change a site count.
- The live-scope run (262 roots, 112.2 s, on the build **before** a behaviour-neutral guard hoist) and
  the full run (on the final build) agree exactly: **LIVE 38, generated 6, literal 32**. Two runs,
  two builds, same number.

## Rows 6 & 7 — the controls, which is the half that makes rows 2–4 mean anything

The census records **every** wildcarded enum match and prints the ones the predicate DECLINED. 455
declined sites across 169 enum paths:

```
128 declined enum paths end in `::Reply`            ← the per-surface reply enums (service.wat:2785)
 35 declined enum paths end in `Response`           ← the per-op response enums (service.wat:2980/:3091)
  6 everything else:
       :fanout::Seen::Verdict · :my::Color · :sa::R
       :wat::kernel::LociDiedError · :wat::sqlite::Cell · :wat::telemetry::Numeric
```

- ⛔ **Row 6 holds.** `service.wat:2785` (`~reply-variant-kw`), `:2980` (`~accepted-ctor-kw`) and
  `:3091` (`~success-ctor-kw`) are matches over per-surface generated enums, and **not one such enum
  appears in the in-scope table**. Their firing counts put it beyond doubt that these are the
  generated ones: `:wat::query::Store::Reply` **42 516** firings, `:wat::telemetry::Journal::Reply`
  **21 264**, `:wat::query::Store::ScanIndexResponse` **3 563** — counts only reachable by
  re-expanding stdlib services across 1768 roots. They are declined **by construction** (the variant
  set is generated per surface, so "name every variant" has no fixed meaning and the `_` IS the
  desync detector), not by exemption: there is no exemption mechanism in this stone.
- ⛔ **Row 7 holds, and on the corpus rather than only in a fixture.** `:my::Color` — the DESIGN
  trap-door's literal example — is declined. `Option`/`Result` **cannot** reach this hook: they are
  their own `MatchShape`s, never `MatchShape::Enum`, so the census never sees them; the driven control
  asserts that too.

### ⭐ The DRIVEN controls, and the ⚠ Status question answered both ways

Item 1 drove the corpus to zero `Status` wildcards, so reporting *"the rule can reach `Status`"* off a
unit test would have been a claim about the predicate, not about the checker. Two independent
measurements instead:

**(a) On the corpus.** `:probe::crash::Status` — 1 site, IN scope. A real
`Status`-shaped enum in a probe file, classified by SHAPE through the real checker.

**(b) Driven, with negatives.** `wat-scripts/scratch-pad/probe-outcome-wildcard-census.wat` +
`the_status_shape_is_reached_and_the_domain_controls_are_declined`:

```
:probe::census::Status                   IN  — defservice Status shape   missing=[Started,Stopped,Hibernated,PeersDenied,Faulted]
:probe::census::Color                    OUT — not a fixed registered outcome/event enum (domain, or per-surface generated)
:probe::census::Statusish::Status        OUT — named Status but NOT the defservice six-variant shape
```

The third row is the sharp one: an enum whose last path segment **is** `Status` but whose variant set
is `Up · Down` is DECLINED. **The rule keys on the shape, never on the spelling.**

⚠ **So the answer to the DESIGN's Status question is YES — reachable, by shape, at any namespace.**
That is the highest-leverage surface in the tree, and it is guarded. The cost of reaching it that way
is stated plainly: a future service whose `Status` gains a seventh variant stops matching
`STATUS_SHAPE` and silently leaves scope. The constant is pinned to `wat/service.wat`'s
`status-enum-def` with a comment saying so; **it is a coupling, not a derivation**, and I did not
build a gate that binds them. Named as residue.

## Row 8 — REPORT-ONLY, and exactly what "report-only" cost the checker

`src/check.rs`: **+27 lines, 0 deletions**, all inside the pre-existing
`MatchShape::Enum(..) => if wildcard_seen { true }` arm. The `true` is untouched — a wildcard still
BLESSES a missing arm, and every diagnostic string is byte-identical.

Collection is off behind an `AtomicBool`. The guard is the **first** thing
`record_if_enabled` does, so a normal `wat --check` pays one relaxed atomic load per wildcarded enum
match and never the variant-name allocation. That ordering is deliberate: the type-check path is the
floor's dominant cost, so a report-only feature must not tax it.

⚠ **The three new tests DO assert** — but never about a wildcard. They gate the *instrument*: that
the hook fires, that it reaches a generated site, that it declines the out-of-scope population. A
wildcard written into the corpus tomorrow reddens nothing.

⛔ **And those asserts have an EXPIRY, written into the test.** They assert a DEFECT is still present.
When the filed phase-2 stone names the five arms in `defservice`'s paging path, `generated > 0` and
the `firings == 2` assert go red **on correct work**. The comment says what to do then (repoint at
this stone's own scratch-pad fixture, or retire with the gate that supersedes it) and says explicitly:
do not re-add a corpus wildcard to keep it green.

## Row 9 — the floor. ⛔ THE FIRST ONE WAS RED, AND BOTH ARE REPORTED

### ⛔ FLOOR 1 — RED. Two failures, both MINE.

```
     Summary [ 556.493s] 5274 tests run: 5272 passed (9 slow), 2 failed, 22 skipped
```

`./scripts/floor.sh`, **`.floor/2026-09-17T02-02-00Z/`**, `[floor] exit=100`, **`ARM.txt` PRESENT**.
Not re-run. Captured whole. The two arms, named:

```
        FAIL [   0.086s] (  77/5274) wat::lint no_loose_string_assert::tests_carry_no_loose_string_assert
        FAIL [   0.412s] (  82/5274) wat::lint one_name_grammar::only_identifier_rs_parses_a_name
```

**Arm 1 — `one_name_grammar::only_identifier_rs_parses_a_name`**, panicked at
`tests/lint/one_name_grammar.rs:114:5`. Its whole offender block, verbatim:

```
    🔥🔥🔥 A SECOND NAME PARSER — 1 site(s) hand-roll one of the five name-grammar
    call-shapes (`rfind("::")`, `rsplit("::")`, `rfind('/')`, `rsplit_once('/')`,
    `strip_suffix('\'')`) OUTSIDE `crates/wat-reader/src/identifier.rs`. A name is an atom;
    structure encoded inside it must be parsed exactly ONE way, or two parsers WILL disagree
    (STONE-one-name-grammar, arc 109 — the census found 33 that already had).
    ...
    Offenders:

    src/check/outcome_wildcard_census.rs:89   rsplit("::")
```

⭐ **This is a genuine architectural finding about my own design, not a formality.** My scope
predicate takes the last `::` segment of an enum path to test the `*Outcome` suffix and the `Status`
shape — and I hand-rolled that parse. An enum path IS a wat name; the tree has exactly one door for
taking its leaf, and arc 109's census found 33 sites that had already disagreed by not using it.
**FIXED, not runed** — `classify` now calls `wat_reader::identifier::leaf(enum_path)`.

⚠ **And the fix is provably behaviour-identical, which is why every census number above still
stands.** `crates/wat-reader/src/identifier.rs:235`:

```rust
pub fn leaf(name: &str) -> &str {
    name.rsplit("::").next().unwrap_or(name)
}
```

— byte-for-byte the expression I had written inline. So the 762 s full-corpus walk, taken before this
fix, measured the same predicate. Stated rather than assumed, because a re-measured 762 s run is
otherwise the only honest alternative.

**Arm 2 — `no_loose_string_assert::tests_carry_no_loose_string_assert`**, panicked at
`tests/lint/no_loose_string_assert.rs:112:5`:

```
    🔥🔥🔥 LOOSE STRING ASSERTIONS — 2 site(s) assert a value with contains/starts_with/
    ends_with where an exact `assert_eq!` belongs. A loose check passes on reordered fields,
    malformed maps, and appended garbage.
    ...
    Offenders:

    tests/lint/outcome_wildcard_census.rs:515
    tests/lint/outcome_wildcard_census.rs:516
```

That was my trap-door-2 control, written as `!collected.iter().any(|s| s.enum_path.contains("Option")
|| ...contains("Result"))`. The lint's own rubric offers a rune for "a targeted absence over a large
output" and I could have claimed it. ⛔ **I did not, because the exact form is strictly stronger**:
the control now asserts the **exact sorted set** of enum paths the fixture's own file produces
(`[:probe::census::Color, :probe::census::Status, :probe::census::Statusish::Status]`). That proves
the `Option` match contributed nothing **and** that no fourth, unexpected enum appeared — which
`contains` could never say.

⭑ Both reds were caught by walls that already existed, on code written in the same session that was
adding a wall of its own. Worth recording as-is.

### ✅ FLOOR 2 — GREEN, and this is the floor of record

```
     Summary [ 558.546s] 5274 tests run: 5274 passed (9 slow), 22 skipped
```

`./scripts/floor.sh`, **`.floor/2026-09-17T02-14-56Z/`**, `[floor] exit=0`, **no `ARM.txt`**.
558.5 s sits alongside floor 1's 556.5 s and inside the 540.4–561.5 s same-code band the sibling
SCOREs record for this box. `5274` tests, up from the sibling's `5251`: **+23** = my 3 integration
tests + 4 unit tests in `src/check/outcome_wildcard_census.rs` + 16 others landed since. No stray
`release/wat` or `nextest` process before the run (checked: 0).

The three new tests inside it, and the four unit tests:

```
        PASS [   0.904s] ( 106/5274) wat::lint outcome_wildcard_census::a_macro_generated_site_is_reached
        PASS [   0.896s] ( 104/5274) wat::lint outcome_wildcard_census::outcome_wildcard_census_over_the_corpus
        PASS [   0.878s] ( 103/5274) wat::lint outcome_wildcard_census::the_status_shape_is_reached_and_the_domain_controls_are_declined
        PASS [   0.009s] ( 507/5274) wat check::outcome_wildcard_census::tests::a_domain_enum_is_out_even_when_it_is_called_status
        PASS [   0.009s] ( 508/5274) wat check::outcome_wildcard_census::tests::a_per_surface_reply_enum_is_out_by_construction
        PASS [   0.008s] ( 509/5274) wat check::outcome_wildcard_census::tests::the_defservice_status_shape_is_in_scope_whatever_its_namespace
        PASS [   0.009s] ( 511/5274) wat check::outcome_wildcard_census::tests::the_outcome_family_is_in_scope_by_suffix_not_by_list
```

⚠ **What floor 2 does NOT re-weigh**: the 762 s full-corpus census. It is not a floor test (see
§How to re-take these numbers), so the numbers in rows 2–4 were taken on the pre-`leaf` build. The
`leaf` body quoted above is why that is sound; it is the only `src/` change between the two, and the
test-side change touched one assertion. Nothing else moved.

## Row 10 — clippy and test compile

- `cargo clippy --release --workspace --all-targets` → **0 warnings, 0 errors** (exit 0; the count
  was taken from the captured log, not from a piped status).
- `cargo nextest run --release --no-run` → clean.
- `cargo build --release` → clean.

## Row 11 — exemptions: REPORTED, not chosen

| option | what it costs | verdict |
|---|---|---|
| **A. an in-form annotation the macro can emit and the checker can see** | a LANGUAGE change: new `match` surface syntax, a checker change, and it must survive quasiquote splicing so `defservice` can emit it. The reason then travels with the code — the only option that puts the *why* where the next reader is. Biggest build, best artefact. | viable, expensive |
| **B. an allowlist keyed by enum path + enclosing form name** | ⛔ **brittle here for a MEASURED reason, not a stylistic one.** Span substitution collapses every wildcard in one expansion onto the `defservice` head: `circuit.wat`'s **four** user-written `CallOutcome` wildcards all report at `:370`. An allowlist key at that granularity **cannot tell them apart — exempting one exempts four.** And the "enclosing form" for a generated match is the macro, so a key is either per-service (churns on every new service) or per-macro (blanket-exempts every service at once). | do not recommend |
| **C. no exemption at all** | 38 authored arms in 6 files, and each is a behaviour decision rather than a mechanical port (item 1 showed nine such arms are a whole stone). Plus permanent verbosity at every `recv'`/`connect'` call site. | ⭐ **recommended** |

⭐ **Recommendation: C, no exemption — and row 3 says why it is affordable.** The fixed enums are
exhaustible by definition, so a wildcard over one is always a latent bug; there is nothing to exempt.
The live bill is 38 arms in 6 files, **one** of which is in the frozen stdlib.

⭐ **And the cost is far below 38, because the live population is one idiom 33 times over.** Thirty-three
of the 34 live `RecvOutcome` sites are byte-for-byte the same shape — name `Message`, swallow the other
five. **One stdlib helper** that names all six arms once (`recv-or-raise`, or a `RecvOutcome`-facing
combinator) retires ~33 of the 38 without any exemption mechanism and leaves call sites *shorter* than
they are now. That reframes row 11's real cost as **1 helper + ~5 bespoke arms**, not 38 decisions —
which is the argument for C, and the thing the DESIGN could not know before the count existed.

⚠ Two costs of C I am not hiding: (i) phase 2's error message points at a `defservice` call for
generated sites (row 5), so a zero-exemption gate would occasionally name a file the author must not
edit; (ii) the `Status` shape-match is a coupling to `wat/service.wat` that nothing enforces.

## Row 12 — ⛔ what this does NOT catch, stated so it is not oversold

**This lint would have caught NONE of the three missing-form failures found this session** — a blocked
`send` that has no variant, `recv-all`'s timeout that has no form, `after`'s ring refusal that raises
instead of returning one. Those are **absent** variants, not **unread** ones. The rule guards exactly
one direction of arc 109's painted-brick doctrine — *a variant nothing READS* — and is **silent** on
the other. An enum that is missing the variant it needs passes this census with a clean bill.

It also does not catch: a narrowed arm that is exhaustive-in-practice; a wildcard over
`LociDiedError` (out of scope, 16 sites); a wildcard over a per-surface enum that has genuinely
desynced; or any wildcard in a file that does not type-check (111 roots).

## Blast radius

```
 M src/check.rs                                            +27 −0 (inside the existing wildcard arm)
?? src/check/outcome_wildcard_census.rs                    the predicate + the side-channel
?? tests/lint/outcome_wildcard_census.rs                   3 tests: census, trap-door 3, driven controls
?? wat-scripts/scratch-pad/probe-outcome-wildcard-census.wat   the driven control fixture
```

Nothing in `wat/` was touched — no `.wat` corpus edit, no codemod needed, no python or sed near a
`.wat` file. The one new `.wat` is a scratch probe in `wat-scripts/scratch-pad/`, inside the
`every_wat_scripts_file_loads` gate, verified with `wat --check` before the floor.

## How to re-take these numbers

```bash
# the floor's slice (stdlib only, 1 root, ~1.4 s for all three tests)
cargo nextest run --release -E 'binary_id(wat::lint) and test(outcome_wildcard_census)' --no-capture

# live (262 roots, 112 s) and the whole corpus (1815 roots, 762 s)
# cargo test, NOT nextest: libtest imposes no per-test deadline
WAT_OUTCOME_CENSUS=live cargo test --release --test lint -- outcome_wildcard_census_over_the_corpus --nocapture
WAT_OUTCOME_CENSUS=full cargo test --release --test lint -- outcome_wildcard_census_over_the_corpus --nocapture
```

⛔ **Why the wide walk is not in the floor**, since that is a real gap and not housekeeping: measured,
a 73-root walk takes **30.8 s** and nextest SIGTERMs it at the 30 s kill (I hit this — the test's own
output printed and the run still reported `TIMEOUT`). 1815 roots is 762 s. The per-root cost is the
stdlib re-check, the same fact `.config/nextest.toml` records as 147 ms × 98 for
`retirement_table_is_fully_reachable`. The box already carries a ~550 s scripts gate at
`priority = 100` with a 600 s kill, and the lint binary runs in that same early wave — adding a
second CPU-heavy corpus walk there risks reddening **that** gate, which is not a trade phase 1 is
allowed to make. `.config/nextest.toml` also says in its own words that raising a deadline "is not a
fix" and that the durable move is to **split per directory**. So the floor weighs the instrument and
the corpus number is taken by hand, marked as such, rather than a timeout override being added.

## Open, for the builder

1. **Does `LociDiedError` join the rule?** 16 declined sites, 0 live. Not my ruling.
2. **Can phase 2 live with `defservice`-call-site locations**, or does it need expander provenance
   (a separate, larger stone)?
3. **The `Status` shape constant is a coupling** to `wat/service.wat`'s `status-enum-def` with nothing
   binding them. A gate that ties them is a small stone; it is not in this one.
4. **`wat/service.wat:2978` / `:3089` remain OPEN**, exactly as item 1 filed them — five behaviour
   decisions × two sites in quasiquoted code. This census confirms they are the *only* generated
   in-scope wildcards the macro authors, so that stone's blast radius is 2 arms and it fixes every
   service in the tree at once.

---

## ⭑ REGRADED BY THE ORCHESTRATOR ON HIS OWN RUNS, 2026-09-17

```
floor    Summary [ 552.272s] 5274 tests run: 5274 passed (9 slow), 22 skipped
         .floor/2026-09-17T02-26-45Z/ · exit=0 · NO ARM.txt · inside the 540.4–561.5 s band
clippy   0 · nextest --no-run clean · the 7 census tests PASS in 1.352 s
```

| row | the orchestrator's own result |
|---|---|
| 1 | ✅ **re-argued independently, and it is airtight.** The instrument flags `wat/query/mem.wat:618 :wat::kernel::RecvOutcome firings=2 swallowing [Closed,Stopped,TimedOut,Lost,Malformed]`. I checked `mem.wat` with my own balanced reader: it contains **zero literal wildcard arms**. Therefore those two can only come from `wat/service.wat`'s quasiquoted body. ⭐ **The checker sees through macro expansion — the entire justification for not using a text lint.** |
| 8 | ✅ report-only: all 7 tests print and pass. |
| 9 | ✅ 552.272 s, inside the band. |
| — | ✅ the executor's **RED floor is on disk** at `.floor/2026-09-17T02-02-00Z/` with `ARM.txt`, `Summary [ 556.493s] … 2 failed`. Captured, named, fixed, re-weighed. Not re-run away. |

⭑ **The scope predicate carries its own controls, as unit tests** — `a_domain_enum_is_out_even_when_it_is_called_status`, `a_per_surface_reply_enum_is_out_by_construction`,
`the_defservice_status_shape_is_in_scope_whatever_its_namespace`,
`the_outcome_family_is_in_scope_by_suffix_not_by_list`. A census is only as good as its predicate, and
this one can be shown to decline the right things, not merely to accept them.

### ⛔ TWO CORRECTIONS TO THE ORCHESTRATOR'S OWN DESIGN, both accepted

1. **Row 5 was impossible as I wrote it.** I demanded `service.wat:2978`/`:3089` appear *by name*.
   **Macro expansion SUBSTITUTES SPANS** (`tests/lint/span_substitution_justified.rs`), so a generated
   wildcard reports at the `defservice` call site. The executor counted them by three independent
   signatures instead — enum identity, the exact five swallowed variants, and `firings = 2 × roots` at
   every paging-capable expansion (mem 3536/1768 · sqs 56/28 · sns 22/11 · faulting-store 8/4 ·
   circuit 2/1). ⭑ **This constrains phase 2**: an ERROR cannot point at the offending line, only at
   the expansion site. Carrying provenance through expansion is its own stone, and nobody knew that
   before this census.
2. **Two of my estimates were wrong and are reported ABSENT rather than reconciled.** Zero live
   `ServiceEvent` wildcards (I estimated 3) and zero live `LociDiedError` (I estimated 2 — 16 sites
   exist, all probe, and out of scope under the chosen predicate, which the executor flagged as *a
   builder ruling it declined to take*). My "1 live `SendOutcome` scratch probe" is confirmed exactly.

### The census, and why a zero-exemption gate is viable

```
LIVE 38 · PROBE/TEST 228   ‖   GENERATED 14 · LITERAL 252   (266 sites, 4203 firings)
live: 6 generated / 32 literal, in only 5 files — and only ONE is in the frozen stdlib
declined: 455 sites / 169 enum paths — 128 ::Reply, 35 Response (the per-surface families)
```

⭐ **Exemption recommendation ACCEPTED: option C, no exemption mechanism at all.** Two measured
reasons, neither of them taste:

- **Option B (per-site exemptions) is brittle by measurement**: `circuit.wat`'s four user-written
  `CallOutcome` wildcards **all report at the same span**, so exempting one would exempt four.
- **The bill is far below 38 arms**: **33 of the 34 live `RecvOutcome` sites are the identical
  idiom**, so one stdlib helper retires ~33 of them. Real cost ≈ 1 helper + ~5 bespoke arms.

### Declared limits, accepted

- **The whole-corpus walk is NOT in the floor** (762.5 s; a 73-root walk already hit nextest's 30 s
  kill). Floor-resident: the instrument plus its controls at **1.4 s**. The wide numbers are the
  executor's own runs and are marked as such — **not** floor-guaranteed.
- `STATUS_SHAPE` is an **unenforced coupling** to `wat/service.wat`'s six-variant Status. If that enum
  gains a variant, the predicate silently stops matching. Named, not fixed.

### What this stone deliberately does NOT do

It is **report-only**. Nothing is a hard error. And — stated in the DESIGN and gated by row 12 — it
would have caught **none** of the three missing-form failures this session found (a blocked `send`
with no variant, `recv-all`'s formless timeout, `after`'s raising ring refusal). It guards *unread*
variants; those are *absent* ones.
