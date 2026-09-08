# SCORE (orchestrator) — STONE P-1: RELAND. The wall is RIGHT 22 times of 27.

**Independent re-run, not the rider's report.** Floor run centrally per FM 18/19.

```
FLOOR EXIT=100     Summary [ 191.889s] 5252 tests run: 5225 passed, 27 failed, 18 skipped
```

⛔ **A RED IS A RED.** Not re-run. The roster below is from the kept log
(`.floor/latest/raw.log`), ANSI-stripped, complete — not a window.

⚠ **My own first read was truncated.** I ran the floor with `| tail -6` and reported "6+
failures." It was 27. The kept log had all of them; my pager did not.
`[[feedback_a_truncating_pager_makes_absence_unfalsifiable]]` — same day, same hands, again.

## Every one of the 27 is the new wall. No unrelated red.

```
26 × src/freeze.rs:1162:9   call_beside_value: fixture … failed to freeze: #wat.type/UnknownNamedType
 1 × tests/program/wat_arc170_slice_1e_user_main_nil.rs:108:5
```

Verbatim, the cluster of 16:

```
thread 'list::list_conj_prepends' (3364326) panicked at src/freeze.rs:1162:9:
call_beside_value: fixture beside "/home/john/work/holon/wat-rs/tests/collection/list.rs" failed
to freeze: #wat.type/UnknownNamedType {:message "annotation names unknown type :wat::core::Int —
not a declared type, not a type variable, and not a builtin" :location #wat.core/Span {:file
"src/check.rs" :line 15256 :col 13 :end #wat.core/Option.None {}} :causes [] :path ":wat::core::Int"}
```

## ★★★ Six names, and they split into two populations

**TRUE CATCHES — 3 names, 22 failures. The wall works.**

```
:wat::core::Int          16   0 declarations anywhere. A PHANTOM. The real name is i64.
:wat::core::Keyword       5   0 declarations. A PHANTOM (the type is `keyword`).
:wat::kernel::ExitCode    1   0 declarations — and arc 170's DESIGN §482 reads
                              "Nil IS the exit code (no ExitCode type — superseded 2026-05-10)".
                              A fixture has been annotating a type RETIRED FOUR MONTHS AGO.
```

These are exactly the class P-1 was drawn to catch, and nothing in the substrate could see them
before this stone. **The stone's thesis is proven by its own floor.**

**FALSE POSITIVES — 3 names, 5 failures. All ONE cause: a partial set.**

```
:rust::test::Greeting     3   a `use!`d FFI type. Lives in UseDeclarations
                              (src/rust_deps/mod.rs:290 `contains`), validated against
                              RustDepsRegistry::has_type. resolve/walk.rs:112 already enforces
                              per-program coverage against it — for CALL HEADS. Annotations
                              were never wired to the same store.
:t::Marker                1   a DERIVE MARKER — `(:wat::core::derive :t::A :t::Marker)`.
:wat::spawn::Spawned      1   a DERIVE MARKER — `(derive :wat::kernel::Thread :wat::spawn::Spawned)`.
                              Markers appear in `subtype_edges` (types.rs:542) as VALUES,
                              never as `types` keys.
```

## ⛔ THE BRIEF IS THE UPSTREAM DEFECT

I specified the union as `TypeEnv::contains ∨ is_builtin_primitive` and called it *"the membership
union arc 296 Stone Q established."* **It is not the union. It is two of at least four stores.**

```
TypeEnv.types · builtin_names   asked      (via TypeEnv::contains)
is_builtin_primitive            asked
UseDeclarations                 NOT ASKED  ← :rust::* FFI imports
subtype_edges (derive markers)  NOT ASKED  ← :t::Marker, :wat::spawn::Spawned
```

★ This is `[[feedback_a_name_checked_against_a_partial_set]]` in its **fifth and sixth positions —
surfaced by the cure written for the fourth.** Stone Q's `is-type?` carries the same blindness:
it will answer `false` for a properly `use!`d rust type and for every derive marker.

## The rider: right diagnosis, wrong rung

It found `:rust::sqlite::Connection`, diagnosed it correctly in writing — *"a TypeEnv hole … not a
phantom"* — disclosed the skip in BOTH the SCORE and the CENSUS, and held all four STOPs. What it
reached for was `if is_reserved_prefix(name) { continue; }` — **arc 255's founding defect, imported
verbatim, twice, into the campaign whose disease is blankets.** Convention rung, not the fix.

I imposed the check with both `continue`s removed and read the screams:

```
838 of 845 corpus files refuse, on exactly TWO names — both :rust::sqlite::*
```

Two names, not a class. The skip was silencing a **false** report, and the honest fix is to widen
the question, not to exempt the population.

## ⚠ AND THE CENSUS COULD NOT SEE THE PHANTOMS

The census reported `0 refusing / 0 distinct names` over 845 `.wat` files — and it is **honest for
the population it could see.** Every real phantom lives in a fixture beside a RUST test file,
reached through `call_beside_value`, which is not a `.wat` file under `wat/`, `wat-scripts/` or
`wat-tests/`. **The census's population excluded the only place the defect lives.** The floor found
in one run what 845 files could not.
`[[feedback_state_what_the_instrument_can_see_before_quoting_it]]`

⚠ My own census had a second defect: my first pattern grepped `"path":"…"` (JSON) against EDN output
that renders `:path ":…"`, and returned a confident 0. I was auditing the rider for a vacuous green
with a vacuous green. Corrected before use.

## Disposition

**RELAND**, on the tree as it stands — nothing reverted. The wall stays; the QUESTION widens, and
the two `continue`s die. `RELAND-1` brief follows.

## Rows

| # | expected | actual |
|---|---|---|
| 1 | phantom param+return EXIT=1 naming the path | ✓ |
| 2 | phantom field EXIT=1 naming the path | ✓ |
| 3 | bound type param EXIT=0 | ✓ |
| 4 | bare Uppercase, no binder, EXIT=0 | ✓ |
| 5 | `:i64` stays `BareLegacyPrimitive` | ✓ — the named diagnostic survived |
| 6 | declared type EXIT=0 | ✓ |
| 7 | builtins + instantiated generic EXIT=0 | ✓ |
| 8 | 7 passed, 0 skipped | ✓ |
| 9 | walk shared, not re-rolled | ✓ `walk_type_expr` is the recursion; both are visitors |
| 10 | census deduped | ✓ produced — ⛔ but blind to the population that mattered |
| — | **FLOOR** | ⛔ **EXIT=100, 27 failed** — not a row the rider was asked to run, and the one that decided this |

★ Rows 1–10 all passed. **The stone passed every bar I wrote and is still a reland** — because the
bars I wrote could not see the store I did not know about. An acceptance row is only as good as the
census behind it. `[[feedback_an_acceptance_row_is_a_pin_unless_it_derives_its_bar]]`
