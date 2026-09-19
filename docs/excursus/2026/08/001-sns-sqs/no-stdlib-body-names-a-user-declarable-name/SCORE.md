# SCORE — no stdlib body names a user-declarable name

**SCORED.** Executor: grok, 2026-09-19, branch `sns-sqs`, HEAD `e130a3e1e` (DRAWN). Did not commit.

The stone's number is **zero**. The gate landed. No `wat/` repair, no `src/` behaviour change.

---

## Row 1 — census by channel (AST, never grep)

Instrument: `startup_bare()` → `sym.functions_iter()`, `FunctionBody::Wat` bodies, the same
`quote_boundary` / `is_unquote_escape` the walker uses. User-declarable =
`n.contains("::") && !is_reserved_prefix(n)` (the complement of `Privilege::User`).

Non-vacuity, so a zero is not "the walk never ran":

| premise | measured |
|---|---:|
| functions in the bare world | `fns > 1000` (same bound as the two-name test) |
| `FunctionBody::Wat` bodies | **2177** |
| evaluated-position `Keyword` leaves (all names, including `:wat::`) | **34811** |
| evaluated-position `Symbol` leaves | **20966** (unnamespaced binders; none user-declarable) |

| channel | unique user-declarable names | occurrences |
|---|---|---:|
| own path | *(none)* | 0 |
| signature (`param_types` / `ret_type` / `rest_param_type`) | *(none)* | 0 |
| **evaluated body leaf** | ***(none)*** | **0** |
| quoted body leaf | `:user::main`, `:user::spawn::service-locus` | 9 + 10 = **19** |

**The stone's number is the evaluated-position count: 0.**

Quoted sites (all `Keyword`; file:line name in enclosing fn):

| file:line | name | enclosing fn |
|---|---|---|
| `wat/spawn.wat:613` | `:user::spawn::service-locus` | `:wat::spawn::ProcessOpts/launch` |
| `wat/telemetry/span.wat:58` | both | `:wat::telemetry::span::service-forms` |
| `wat/cache.wat:383` | both | `:wat::cache::hologram-svc::service-forms` |
| `wat/kernel/services/stdio.wat:104` | both | `:wat::kernel::stdin-svc::service-forms` |
| `wat/kernel/services/stdio.wat:69` | both | `:wat::kernel::stderr-svc::service-forms` |
| `wat/telemetry/journal.wat:92` | both | `:wat::telemetry::journal::service-forms` |
| `wat/query/sqlite-store.wat:372` | both | `:wat::query::sqlite-store::service-forms` |
| `wat/query/mem.wat:618` | both | `:wat::query::mem-store::service-forms` |
| `wat/cache.wat:195` | both | `:wat::cache::lru-svc::service-forms` |
| `wat/kernel/services/stdio.wat:43` | both | `:wat::kernel::stdout-svc::service-forms` |

Nine `:user::main` (every `…::service-forms`) and ten `:user::spawn::service-locus` (the same nine
plus `ProcessOpts/launch`). Matches the mention stone's 19, now split by channel.

**Keyword vs Symbol.** The census counts both. The restriction walker looks up only `Keyword`
(`check.rs:1677`); Symbols are counted so a namespaced symbol-ref the normalizer missed cannot
hide. Evaluated user-declarable: 0 Keyword, 0 Symbol. Quoted user-declarable: 19 Keyword, 0 Symbol.

---

## Row 2 — the gate, on the property

`no_stdlib_body_names_a_user_declarable_name_in_evaluated_position` in
`tests/function/probe_tier_b_stdlib_verdict_is_not_bake_fixed.rs`.

A violation panics with **file:line, channel, name, enclosing fn, leaf kind**, and what it means:
a user declaration of that name can change a stdlib verdict through `check:restricted-call` (8c).
Not a path list.

**Reuses `quote_boundary` / `is_unquote_escape`.** Not a fifth copy. Visibility widening named in
row 6.

The walk mirrors the walker's four-way split exactly (`check.rs:1701`): `AllData` and `Quasiquote`
are data; `Match` / `MatchesSubject` / `MakeRule` / `Ordinary` stay live (exhaustive match, not a
`_`). Unquote / unquote-splicing inside a quasiquote template resume the evaluated walk
(`walk_qq_template`, same shape as `walk_restricted_quasiquote_template`). Nested quasiquote stays
template (still scanned for escapes). `quote` / `forms` / `literal` have no escape.

Quoted occurrence counts are pinned (9 / 10) so a twentieth quoted mention is a RED, not a silent
surface growth.

---

## Row 3 — ⭐ the fresh 8c control

The inverted witness stays a control for the quoted-mention *repair*, not for the sweep. Two new
tests, together, distinguish *"the sweep ran and found nothing"* from *"the sweep did not run."*

| test | what it observes | if `src/check.rs:750` is deleted |
|---|---|---|
| `the_restricted_call_sweep_still_walks_every_wat_body` | `include_str!` of `check.rs`: the `P_CHECK_RESTRICTED` window still contains `functions_iter()`, `FunctionBody::Wat`, and `walk_for_restricted_call(body, name, …)` | **RED** — `P_CHECK_RESTRICTED` absent |
| `the_restricted_call_phase_records_a_hit_on_a_real_freeze` (own file, own process: `force_mode` is a `OnceLock`) | armed boot census, `8c   check:restricted-call(ALL fns)` hits ≥ 1 after `startup_bare` | **RED** — the report row stays at 0 hits |

**Would this pair stay green if `:750` were deleted? No.**

Emptying the closure but keeping the `phase(P_CHECK_RESTRICTED, \|\| {})` wrapper: source window
loses the three needles → RED; census hits stay 1. Both halves are load-bearing.

⚠ Remaining gap, named: a skip-stdlib filter *inside* the loop that left those three needles in
the 900-byte window would keep the source pin green and the census at 1 hit. Deletion of the
phase is the demand; a later C that filters by reserved prefix inside the still-present loop is
a different shape and would need its own pin.

---

## Row 4 — non-vacuity, per test

| test | premise |
|---|---|
| two-name surface (existing) | `fns > 1000` — cited and unchanged |
| evaluated-position gate | `fns > 1000`, `wat_bodies > 1000`, `eval_keyword_leaves > 1000` |
| 8c source pin | `P_CHECK_RESTRICTED` found in `check.rs` |
| 8c census hits | `force_mode(Phases)` succeeded (process isolation), `fns > 1000`, hits ≥ 1 |
| inverted witness | fixture still declares `:user::main` |

---

## Row 5 — the stale justification

`walk_names` at the old `:215` now says what is true: the raw-leaf walk over-approximates on
purpose for **path, signature, and quoted** channels (none of path/signature is exempted by
`quote_boundary`; a quoted name is still a deliberate surface). `walk_for_restricted_call`
**does** know the difference now. The position-aware census is the model of that walker.

**The two-name assertion still stands** — union of all four channels is still exactly
`:user::main` and `:user::spawn::service-locus`.

---

## Row 6 — scope wall

No `src/` behaviour change. Nothing elided. No `.wat` edit.

Visibility widening, named:

| item | was | now | why |
|---|---|---|---|
| `resolve::boundary::Boundary` | `pub(crate)` | `pub` | census must `match` the six variants |
| `quote_boundary` | `pub(crate)` | `pub` | reuse the classifier; a fifth copy fails row 2 |
| `is_unquote_escape` | `pub(crate)` | `pub` | unquote in a template IS evaluated |
| re-export from `resolve/mod.rs` and `lib.rs` | — | `pub use` | integration tests are a separate crate |

Match arms of `quote_boundary` / `is_unquote_escape` are byte-identical to HEAD. `is_where_form`
stays `pub(crate)` — the restriction walker does not consult it (`MakeRule` stays live).

---

## Row 7 — floor

Clippy `--release --workspace --all-targets -- -D warnings`: `Finished` in 17.20s, no
`error`/`warning` lines.

Floor, this strike:

```
     Summary [ 268.987s] 5309 tests run: 5309 passed, 22 skipped
```

`.floor/2026-09-19T01-25-25Z/` — exit **0**, **no `ARM.txt`**. Count **5309** = 5306 (mention/A)
plus the three new tests (gate, source pin, census hits).

An earlier floor on a rustfmt accident (formatting `lib.rs` reformatted the crate, shifted
`freeze.rs:1547→1559` and knocked a same-line rune off `sort'`) is captured at
`.floor/2026-09-19T01-13-46Z/` and was **reverted** before this run. That ARM is evidence of the
accident, not of this stone.

---

## What would make this stone wrong — checked

- **Walk never descended / `FunctionBody::Wat` matched nothing.** 2177 Wat bodies, 34811
  evaluated Keyword leaves. Row 4 exists for this; both bounds held.
- **`quote_boundary` exempts more than the walker.** Census matches `AllData` / `Quasiquote` /
  `Match` / `MatchesSubject` / `MakeRule` / `Ordinary` exhaustively. Only the first two are data.
  Same four-way split as `check.rs:1701`.
- **Unquote escape.** `walk_qq_template` resumes `walk_eval` on `is_unquote_escape` arguments.
  Exempting whole templates is unrepresentable in this walk.
- **Keyword vs Symbol.** Both counted, as above.

---

## `:restricted-to` recount (reading, not a gate)

DESIGN said one wat site (`wat/spawn.wat:380`) plus five `#[restricted_to]` in Rust.

Live `{:restricted-to …}` metadata-maps in `wat/`:

| site | binding |
|---|---|
| `wat/spawn.wat:380` | `spawn-program'` (the one DESIGN named) |
| `wat/kernel/services/stdio.wat:362` | `write-fd-raw` |
| `wat/kernel/services/stdio.wat:376` | `flood-stdout-raw` |
| `wat/kernel/services/stdio.wat:384` | `str-double` |

**Four**, not one — DESIGN undercounted the three stdio gates.

`#[restricted_to]` on live Rust fns, still **five**: `src/io.rs:1343` (`IOWriter/from-fd`),
`:1383` (`IOReader/from-fd`), `src/runtime.rs:26001` (`close`), `src/kernel/spawn.rs:500`
(`spawn-thread`), `:586` (`spawn-process`).

---

## Porcelain

```
src/lib.rs
src/resolve/boundary.rs
src/resolve/mod.rs
tests/function/probe_tier_b_stdlib_verdict_is_not_bake_fixed.rs
tests/function/probe_tier_b_restricted_sweep_still_runs.rs   (new)
```

plus this SCORE. `wat/` untouched. `core.wat` still 2152.

---

## What this gives C

Evaluated-position user-declarable leaves in stdlib bodies are **zero, by corpus, held by a
gate**. 8c is still not closed *by construction* (the door is keyed by any name — step A's
verdict). C can elide 8c only behind this gate plus the sweep control; a third unquoted
user-declarable name reddens the gate and reopens the door. The quoted two names remain a
deliberate surface on the path/signature/quoted channels, which no boundary exempts for path
and signature, and which the two-name assertion still pins for quoted.

---

## ⭑ REGRADED BY THE ORCHESTRATOR, 2026-09-19 — accepted, all seven rows

### ⭐ Row 3's central claim is now MEASURED, not argued

The SCORE argues *"would this pair stay green if `:750` were deleted? No."* That is testable, so I
tested it: deleted the whole `phase(P_CHECK_RESTRICTED, …)` block from `src/check.rs`, rebuilt
release, ran the six probes, reverted (`git checkout src/check.rs`, `P_CHECK_RESTRICTED` back to 1).

| test | sweep deleted |
|---|---|
| `the_restricted_call_sweep_still_walks_every_wat_body` | **FAIL** |
| `the_restricted_call_phase_records_a_hit_on_a_real_freeze` | **FAIL** — census row reads `8c   check:restricted-call(ALL fns)   0.00   0.00   0` |
| `a_user_restriction_on_user_main_no_longer_reddens_stdlib_bodies` | PASS |
| `the_user_declarable_surface_inside_stdlib_bodies_is_exactly_two` | PASS |
| `no_stdlib_body_names_a_user_declarable_name_in_evaluated_position` | PASS |

⭐ **Three green tests sail through a total elision of the sweep; the new pair is the only thing
that catches it.** That is precisely the failure the predecessor predicted in writing, and the
demand it filed against itself is discharged — by measurement, not by argument.

### What I checked myself

| claim | check |
|---|---|
| `src/` is visibility-only | ✅ diff is `pub(crate)`→`pub` on `Boundary` / `quote_boundary` / `is_unquote_escape` plus re-exports. **Match arms byte-identical.** No behaviour change. |
| the walk mirrors the walker | ✅ **exhaustive** six-variant match, no `_` arm; only `AllData` + `Quasiquote` are data; `is_unquote_escape` resumes the evaluated walk. It cannot manufacture a zero by over-exempting. |
| the census is not vacuous | ✅ 2177 Wat bodies, 34811 evaluated Keyword leaves, `fns > 1000` — all three bounds asserted in-test. |
| `wat/service.wat:3734` missing from the site table | ✅ **not a hole** — that `:user::main` is in the `defservice` **macro** body; the nine `…::service-forms` it generates are the functions, and macros are not in `functions_iter()`. |
| the six probes | ✅ all PASS (0.533 s). |
| floor artifact | ✅ read directly, not taken on report: `.floor/2026-09-19T01-25-25Z/` — **no `ARM.txt`**, `Summary [ 268.987s] 5309 tests run: 5309 passed, 22 skipped`. 5309 + 22 = 5331 reconciles with my own `6 run + 5325 skipped`. |
| clippy | ✅ re-run whole, counted over the **entire** file rather than a window: **0 errors, 0 warnings**. |
| no rustfmt reformat lurking | ✅ diffstat 470 insertions / 13 deletions over 4 files. |

### ⛔ MY DESIGN WAS WRONG, AND THE EXECUTOR CAUGHT IT

DESIGN said `:restricted-to` has **one** wat site. It has **four** (`wat/spawn.wat:380` plus
`wat/kernel/services/stdio.wat:362`/`:376`/`:384`). Cause: **`wat/*.wat` does not recurse** — it
globs **27** files of a **55**-file corpus, missing all of `wat/kernel/`, `wat/telemetry/`,
`wat/query/`. The same bad glob produced the "~270 non-`:wat::` FQDN tokens" ceiling, which was
therefore not even a valid ceiling. It did no damage only because DESIGN forbade quoting it as a
census and demanded the AST — but the rule was right for a reason I had not actually satisfied.

### ⚠ A brittleness in row 3's first half, named for the record

The captured red at `.floor/2026-09-19T01-13-46Z/` (`ARM.txt` kept, 3 failed) was a **rustfmt
accident with the sweep fully intact** — and one of its three arms was the new source pin. So:

> **`the_restricted_call_sweep_still_walks_every_wat_body` has a false-RED mode on reformatting.**

That is the safe direction for a control (a false RED costs a look; a false GREEN ships an
elision), and the census-hits half is immune to it. But a future `cargo fmt` **will** redden it,
and the temptation will be to widen the window or delete the needles. ⛔ **Do not.** Re-derive the
needles against the reformatted source instead.

The other two arms were the same mechanical cause, and one is worth filing on its own:
`probe_supervisor_select_lost` pins `src/freeze.rs:1547` **inside an EDN golden stack frame**, so
any edit above that line in `freeze.rs` reddens it. Pre-existing, not this stone's business, and
not a licence to dismiss it when it fires.

### The state this hands C

Evaluated-position user-declarable leaves in stdlib bodies: **zero, by corpus, held by a gate that
names file/line/name/enclosing-fn on violation.** 8c remains not-closed *by construction* (step A's
verdict stands — the door is keyed by any name), so C elides **8b + 8d(ALL-fns) + 8f = 121.58 ms**
and leaves 8c running at 3.07 ms, now with a live control proving it runs.
