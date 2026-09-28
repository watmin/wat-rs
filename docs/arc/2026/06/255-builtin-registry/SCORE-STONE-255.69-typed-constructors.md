# SCORE — STONE 255.69: every constructor names its type — the type-driven codemod, and types build their own values

**Executor: a Sonnet subagent.** Commits `c9d1106be` (part 1 — run-time dispatch), `55cf63db7`
(the codemod tool), `6e376fee9` (part 2 — the corpus conversion), `3d8f1a8cc` (the floor's own
two fixes — see § "The floor went RED" below), this SCORE.

## Part 1 — types build their own values, at run time too (`c9d1106be`)

**The gap** (measured, SCORE-STONE-255.68 § "The two scalar constructors"): `(wat.type/u8 65)` and
`(wat.type/char "a")` type-checked but died `#wat.runtime/UnknownFunction {:message "unknown
function: :wat::type::u8"}` at run time. `constructor_head_key` (`src/types.rs`) redirected a
`wat.type/`-spelled head to its registered `:wat::core::…` intrinsic only for the seven CONTAINER
names — a hand-listed match arm — even though `canonical_type_key` (the door it already called)
denotes EVERY member of the 24-name closed set (`WAT_TYPE_HARD_PRIMITIVES`) identically, container
or scalar.

**The fix.** Generalized the match to a registry-membership check on the door's own output:

```rust
pub(crate) fn constructor_head_key(head: &str) -> std::borrow::Cow<'_, str> {
    let denoted = canonical_type_key(head);
    if denoted != head && crate::intrinsic::registry().contains(&denoted) {
        std::borrow::Cow::Owned(denoted)
    } else {
        std::borrow::Cow::Borrowed(head)
    }
}
```

No list of scalar names — `denoted != head` is only ever true for a `wat.type/`/`:wat::type::`
spelling `canonical_type_key` actually touches (any name outside the 24-set, or a non-`wat.type`
head, passes through as `denoted == head` and the branch is a no-op), and the registry-membership
guard is what makes the generalization safe: a hard primitive with no registered constructor
FUNCTION (`nil`/`bool`/`Never`/`Fn`/… are literals, not callables) still falls through to `head`
unchanged, so the caller's existing dispatch chain names the site's OWN original spelling in its
`UnknownFunction` diagnostic rather than a denoted key nothing backs.

**Tests** (`tests/function/probe_stone255_69_wat_type_scalar_dispatch.{wat,wat.bad,rs}`):

- `u8_constructs_its_own_value` — `(wat.type/u8 65)` runs, yields `Value::u8(65)`.
- `char_constructs_its_own_value` — `(wat.type/char "a")` runs, yields `Value::wat__core__Char('a')`.
- `collection_head_still_dispatches` — regression guard: `(wat.type/PersistentVector 1 2 3)` still
  runs, length 3 (the five collection heads already worked before this stone; the fix must not
  disturb them).
- `unregistered_wat_type_head_still_refuses` — `(wat.type/Nope 1)` (a name declared nowhere) still
  fails to CHECK (`UnresolvedReference` naming `Nope`) — the generalization does not widen
  acceptance beyond the checker's own `is_known_type` door.

**Gates** (this run, alone, foreground): `cargo build --release` clean; `cargo clippy --release
--all-targets -- -D warnings` rc 0; `cargo test --release --test function stone255_69` —
`test result: ok. 4 passed; 0 failed`.

## Part 2 — the codemod and the corpus conversion (`55cf63db7`, `6e376fee9`)

### The tool

**Census** — re-derived from scratch (not reused from 255.68's uncommitted throwaway binary), as a
committed, durable `wat-scripts/scratch-pad/255-69-census-walk.wat`: an AST walk (`read-string` →
`ast->children`/`ast-kind`/`ast-name`/`ast-span`) over every corpus file, reporting a `List` node
whose head is one of the five constructor names (either spelling) where `items[1]` is not the `:-`
binder. **Result: 1804 sites**, per-head `PersistentVector 1339, Tuple 237, PersistentMap 122, List
96, Vector 10` — **byte-identical** to SCORE-STONE-255.68's own count and per-head table, an
independent cross-check from a fresh walk.

**Type table** — a Rust program (`scratch/stone25569_desugar.rs`, NOT committed per the brief: "an
input, not a committed artifact"), because the checker's DISPLAY text for a recorded type uses a
notation `read-string` refuses to parse back (the compact tuple form `:(A,B)` — confirmed
empirically: `#wat.parse/Lex {:message "comma inside keyword body retired… For a tuple type use the
`:-` binder form"}`). The tool:

1. Runs `WAT_CHECK_TYPES=1 wat --check <file>` over the same 2545-file scope SCORE-STONE-255.68
   used (`git ls-files 'wat/**/*.wat' '*.wat' '*.wat.bad'`, excluding `wat-scripts/fixes/**` and
   `*.wat.golden`), unions the `TYPE`/`UNRESOLVED` lines by `(file, line, col, tag, ty, wide)`.
2. Joins the 1804 census sites to that union by exact `(file, line, col)`. **1663 distinct
   positions matched** (0 with more than one distinct record — no `MULTI`), **141 not matched**
   (not-checked). Of the matched, **33 are `UNRESOLVED`-tagged**, **1630 are `TYPE`-tagged**
   (concrete + generic). Every one of these four numbers, and the FULL not-checked/unresolved/
   generic site lists, is **byte-identical** to SCORE-STONE-255.68's own (`diff` against this
   SCORE's committed lists: identical) — a full, independent re-derivation, not a copy.
3. **Bridges** the `TYPE`-tagged rows' recorded-type TEXT into parseable wat source — a NOTATION
   fix only, never a type-name decision: `:(A,B,…)` (top-level or nested, empty or 1-tuple-with-
   trailing-comma) → `(:wat::core::Tuple :- [A B …])`, recursively; a bare (colon-stripped)
   namespaced leaf — `format_type_inner`'s convention once inside a tuple's own element list — has
   its leading colon restored, UNCONDITIONALLY (confirmed empirically that this is needed for BOTH
   a namespaced type AND a rigid type-parameter var — see STOP-2 below).
4. Classifies each bridged site (see STOP-2/the exclusions below) and emits, for the CODEMOD-able
   set: `paths.json` (the file list, the codemod's stdin) and `table.edn` (the codemod's
   `.typed-constructors-table.edn` side-channel — file → "line:col" → bridged type text; EDN map
   syntax, not JSON's `key: value`, confirmed empirically that the colon separator fails the wat
   reader: `"invalid keyword: empty"`).

### The codemod, `wat-scripts/fixes/typed-constructors.wat`

Reads `paths` from stdin (the SAME single-line protocol every recorded `wat-scripts/fixes/*.wat`
driver uses — `scripts/replay/{census,delta}.sh` already know how to drive it) and the type table
from the `.typed-constructors-table.edn` side-channel (never a file this codemod ships or a path
baked into a committed artifact). For each file: walks its AST (the same hit rule as the census,
with `types-to-wat-type.wat`'s genuineness guard — `source-matches-name?` — on the call's own head);
for a hit whose `(line, col)` is in the table, replaces the head's OWN SPAN (nothing else) with
`<converted-head> :- [<converted-args>]`, span-faithfully (`wat/fix.wat`'s
`fix-text-offset-of`/`fix-text-apply`) — the empty `(:wat::core::PersistentVector)` becomes
`(wat.type/PersistentVector :- [wat.type/i64])`; `(:wat::core::PersistentVector a b)` becomes
`(wat.type/PersistentVector :- [wat.type/i64] a b)`.

Every type-NAME spelling decision runs through the SAME door 255.67's `types-to-wat-type.wat` used
(`:wat::keyword::to-type-form`, gated on `t2wt::target-names` copied VERBATIM — the identical
24-name closed set), applied to (a) the call's own head, directly, and (b) each keyword LEAF inside
the table's bridged type fragment, via its own span-faithful point-edits (never a whole-subtree
`write-forms` round-trip — see STOP-1's first finding below for why that matters). A keyword not in
the 24-name set is left byte-identical — no colon-stripping, no re-rendering.

## The two STOP-1s this stone found, and how each resolved

**STOP-1 finding #1 — `write-forms` on an unconverted Keyword re-normalizes it.** The codemod's
first draft rebuilt the whole type-fragment AST (`with-children` + `write-forms` on the converted
tree). Dry-running against `wat/gen.wat:952:25`'s recorded type `(:wat::core::PersistentVector :-
[(:wat::gen::Gen :- [:B])])` produced `(wat.type/PersistentVector - [(:wat.gen/Gen - [B])])` —
`:wat::gen::Gen` (not one of the 24; must stay untouched per item 3, "every other type as it is
spelled today") became `:wat.gen/Gen`, and the `:-` marker itself lost its colon. `write-forms`
does not byte-preserve an unconverted Keyword — it renders it in a NORMALIZED dotted form.
**Fixed**: span-faithful point-edits only, exactly `types-to-wat-type.wat`'s own mechanism —
collect an edit for each ELIGIBLE keyword leaf (walking only the args-vector subtree, never the
fragment's head/marker, so `:-` is never visited at all), rebase each edit's offset to the
args-vector's own slice, `fix-text-apply` that slice alone. Re-verified against the same site,
`wat-tests/core/generic-tuple-nongeneric-baseline.wat`, and a dozen more hand cases (nested
Tuple-of-Tuple, HashMap 2-slot, a non-`::` var) before the full corpus pass; the full corpus
census-diff (below) is the final proof.

**STOP-1 finding #2 — a rigid type-parameter var is not uniformly colon-prefixed in the checker's
own storage.** `wat/gen.wat:983:31`/`wat/seq.wat:133:36` recorded `:T` (WITH the colon — `format_type`
rendered a `TypeExpr::Path` whose own stored string already carried it), but
`tests/collection/…generic-tuple-infer.wat`'s call-site inference recorded a BARE `T` (no colon) at
the identical kind of slot — an inconsistency in the checker's own `TypeExpr::Path` storage, not
this tool's to resolve. `parse_bracket_type_keyword` (`src/check.rs`, arc 109) requires EVERY slot
in a constructor's own `:-` bracket to be a `WatAST::Keyword` — a bare `Symbol` fails
("bracketed type must be a type keyword"), confirmed in total isolation for both spellings.
**Fixed**: the bridge restores a leading colon on EVERY bare leaf token, unconditionally (not just
namespaced ones) — item 3's "write the definition's own parameter (`T`)" means literally `:T`.

## A structural limitation this stone's OWN target grammar cannot express — found, not worked around

Dry-running the codemod over the full 375-file, 1630-site draw (before either exclusion below) and
`--check`-comparing every standalone-checkable file against its pre-conversion self surfaced **31
files flipping `rc 0 → rc≠0`** — a genuine STOP-1 by the brief's own definition ("a converted site's
file changes its `--check` result"). Traced to exactly three, disjoint, fully-diagnosed mechanisms;
**none hand-patched** — each resolved by EXCLUDING the site from this stone's table (the same
disposition already given to unresolved/not-checked sites), converting only what the checker's own
constructor-binder grammar today accepts:

1. **35 sites — a compound (nested) element type.** `parse_bracket_type_keyword` (`src/check.rs`)
   requires every slot in a constructor's own `:-` bracket to be a BARE `WatAST::Keyword` — never a
   nested `(Head :- […])` form. Confirmed in complete isolation, both empty and with value args:
   `(wat.type/PersistentVector :- [(wat.type/PersistentVector :- [wat.type/i64])])` →
   `#wat.check/MalformedForm {:reason "bracketed type must be a type keyword"}`. The brief's own
   target shape is genuinely inexpressible here — not a bug in this codemod, a property of the
   constructor-binder grammar today (this is where the angle-bracket generic spelling would have
   gone, and it is fully retired: `:wat::gen::Gen<B>` lexes as an error, "angle-bracket type
   parameters are illegal in a name"). Per head: PersistentVector 10, PersistentMap 6, Tuple 19.
   Full list below.
2. **74 sites — every `List`-headed constructor.** `:wat::core::List`'s OWN registered inference,
   `infer_linked_list_constructor` (`src/check.rs` ~16486), predates the `:-` binder convention
   entirely and was never updated to peel it — unlike `infer_list_constructor`
   (Vector/PersistentVector's arm), whose OWN comment says it "now peels its own `:- [T]`
   param-spec." `infer_linked_list_constructor` unifies EVERY arg (the `:-` marker, the args
   vector, and the values alike) against one fresh element-type variable positionally. Confirmed in
   complete isolation: `(wat.type/List :- [wat.type/i64] 1 2 3)` ALONE fails to check —
   `":wat::core::List: parameter #2 expects :wat::core::keyword; got (:wat::core::Vector :- […])"`
   (param #2 is the `[wat.type/i64]` args-vector, being checked as though it were an ordinary
   value). **All 74 `List` concrete sites are left untouched** — the brief names `List` among the
   seven convertible heads, but the checker does not support the shape for it today; the code wins.
3. **1 site — `wat-tests/gen.wat:354:3`.** `:wat-tests::gen::sbases` is defined TWICE in this file
   (354 and 837) with byte-IDENTICAL bodies; the checker tolerates an identical-body redefinition
   silently (the ORIGINAL file checks clean — confirmed: `wat --check wat-tests/gen.wat`, rc 0, no
   output) but line 837 never received its own type record (independently confirmed already in the
   141 not-checked set). Converting 354 alone makes the two bodies textually DIVERGE, and the SAME
   redefinition that was silently tolerated when identical becomes a genuine
   `#wat.check/DefRedefForbidden` (confirmed: the dry-run copy's `--check` flips `rc 0 → 1` on
   exactly this, nothing else in the file). Excluded so both twins stay byte-identical.

**None of this was hand-patched.** Each mechanism was isolated to a minimal repro BEFORE any
corpus-scale decision, the exclusion is a TABLE-BUILDING decision (never a corpus edit), and the
final applied conversion's own full-corpus census-diff (below) shows zero flips of any kind.

## Coverage — the count converted by head against the 1630 target

| head | concrete+generic (255.68) | **converted** | compound (excluded) | List (excluded) | duplicate-defn (excluded) |
|---|---:|---:|---:|---:|---:|
| PersistentVector | 1297 (1294 + 3 generic) | **1286** | 10 | — | 1 |
| Tuple | 164 | **145** | 19 | — | — |
| PersistentMap | 95 | **89** | 6 | — | — |
| List | 74 | **0** | — | 74 | — |
| Vector | 0 | **0** | — | — | — |
| **total** | **1630** | **1520** | **35** | **74** | **1** |

`1520 + 35 + 74 + 1 = 1630`. **1520 of 1630 (93.3%) converted**; the other 110 (6.7%) are left
alone for a NAMED, DIAGNOSED reason each — none silently dropped.

### The 33 unresolved + 141 not-checked sites (SCORE-STONE-255.68, re-verified byte-identical)

Independently re-derived this stone (§ "The tool" above) and `diff`ed against SCORE-STONE-255.68's
own full lists: **identical**, position for position. Not repeated here; see that SCORE's §§ "The
33 `unresolved` sites" / "The 141 `not-checked` sites" for the full per-site list. These are left
untouched — the wall (a later stone) will see them as its heretics, unchanged by this stone.

### The 3 generic sites (SCORE-STONE-255.68) — outcome this stone

- PersistentVector `wat/gen.wat:952:25` — **excluded** (compound: its element type is
  `(:wat::gen::Gen :- [B])`, itself a nested Parametric).
- PersistentVector `wat/gen.wat:983:31` — **converted**: `(wat.type/PersistentVector :- [:T])`.
- PersistentVector `wat/seq.wat:133:36` — **converted**: `(wat.type/PersistentVector :- [:T])`.

### The 35 compound-element sites (full list — left alone, § "A structural limitation" #1)

- PersistentMap `wat-scripts/scratch-pad/probe-zero-magnitude-reachable.wat:72:15`
- PersistentMap `wat-scripts/scratch-pad/probe-zero-magnitude-reachable.wat:77:11`
- PersistentMap `wat/rete/acc.wat:156:5`
- PersistentMap `wat/rete/oracle/fire.wat:162:34`
- PersistentMap `wat/rete/oracle/fire.wat:164:34`
- PersistentMap `wat/rete/oracle/fire.wat:167:34`
- PersistentVector `wat-scripts/perf/grid/where-collection.wat:291:7`
- PersistentVector `wat-tests/gen-patterns.wat:164:22`
- PersistentVector `wat-tests/gen.wat:135:43`
- PersistentVector `wat-tests/gen.wat:315:43`
- PersistentVector `wat-tests/gen.wat:343:46`
- PersistentVector `wat-tests/gen.wat:707:33`
- PersistentVector `wat/gen.wat:952:25`
- PersistentVector `wat/rete/oracle/pass.wat:651:20`
- PersistentVector `wat/rete/oracle/pass.wat:767:29`
- PersistentVector `wat/rete/oracle/pass.wat:86:14`
- Tuple `tests/collection/probe_arc216_stone7_tuple_roundtrip.wat:31:12`
- Tuple `tests/collection/probe_arc216_stone7_tuple_roundtrip.wat:40:9`
- Tuple `tests/collection/probe_arc216_stone7_tuple_roundtrip.wat:49:9`
- Tuple `tests/resolve/probe_arc255_the_type_position_has_its_own_authority__control_the_four_names.wat:6:3`
- Tuple `tests/services/probe_arc255_24_defservice_declares_what_it_emits.wat:103:3`
- Tuple `tests/types/ord_tuple_recursion_deep.wat:4:5`
- Tuple `tests/types/ord_tuple_recursion_deep.wat:5:5`
- Tuple `tests/types/probe_arc255_53_tuple_member.wat:13:13`
- Tuple `wat-scripts/probes/arc-170/w3-n-dial-runner.wat:49:23`
- Tuple `wat-scripts/scratch-pad/arc109-tuple-bracket-reader.wat:27:3`
- Tuple `wat-scripts/scratch-pad/arc109-tuple-bracket-reader.wat:50:3`
- Tuple `wat-scripts/scratch-pad/census-join-scope-where.wat:140:63`
- Tuple `wat-scripts/scratch-pad/census-join-scope-where.wat:142:63`
- Tuple `wat/fix.wat:1715:19`
- Tuple `wat/fix.wat:1721:13`
- Tuple `wat/kernel/services/stdio.wat:138:3`
- Tuple `wat/lint.wat:435:34`
- Tuple `wat/lint.wat:440:36`
- Tuple `wat/lint.wat:447:28`

### The 74 `List`-headed sites (left alone, § "A structural limitation" #2)

Every one of `tests/collection/list.wat` (14 sites), `tests/collection/probe_collection_transform_ops.wat`,
`tests/collection/probe_seq_container_parity.wat`, `tests/collection/probe_seq_container_registry.wat`,
`tests/function/probe_arc255_67_wat_type_container_defclause_dispatch.wat`,
`tests/types/probe_arc255_20_a_name_nothing_declares_registry_row_checks.wat`,
`tests/types/probe_arc255_66_position.wat`,
`tests/types/probe_arc255_22_an_edge_declares_its_type_parameters_seqable_Elem.wat`,
`tests/types/probe_arc255_67_cutover_types.wat`,
`tests/types/probe_stone_118_b2c_surface_arm_never_dispatches.wat`,
`tests/types/probe_stone118_3b_seqable_parametric_satisfaction.wat`,
`tests/value/probe_stone_D_join_over_seqable.wat`,
`tests/types/probe_first_bare_accessors_first_list.wat`,
`wat-scripts/scratch-pad/255-p6c-w6-direct-calls.wat`,
`wat-scripts/scratch-pad/255-stone-o-iv-d-direct-calls.wat`,
`wat-scripts/scratch-pad/255-stone-o-iv-b-collections-sweep-direct.wat`,
`wat-scripts/scratch-pad/probe-118B2-one-clause-lazy-producer.wat`,
`wat-scripts/scratch-pad/probe-118B1-seqable.wat`,
`wat-scripts/scratch-pad/probe-118B2-rider-verification.wat`,
`wat-scripts/scratch-pad/probe-brief-get-is-total-by-fallback.wat`,
`wat-scripts/scratch-pad/probe-brief-first-nth-to-string.wat`,
`wat-scripts/scratch-pad/probe-hashset-and-linkedlist-homes.wat`,
`wat-scripts/scratch-pad/probe-seqable-to-stream-native-check.wat`,
`wat-tests/core/core-foldl-spec.wat`, `wat-tests/core/core-nth-differential.wat`,
`wat-tests/core/core-nth.wat`, `wat-tests/core/core-seq-walkers.wat`,
`wat-tests/core/core-seqable.wat`, `wat-tests/holon/list-round-trip.wat` — every `List`-headed
census site in each. (Exact `(file, line, col)` per site: `wat-scripts/scratch/
stone25569_desugar.rs`'s `unsupported` output, reproducible from the recorded method above; not
re-pasted at 74-line length here — every one of these files' `List` sites is 100% of that file's
`List` census hits, confirmed by construction: the exclusion is head-keyed, not site-keyed.)

### The 1 duplicate-defn exclusion

- PersistentVector `wat-tests/gen.wat:354:3` (twin of the not-checked `:837:3`).

## The floor went RED once — both arms found, diagnosed, fixed, re-verified individually, THEN the floor re-run (`3d8f1a8cc`)

Per doctrine: not re-run on the red, captured whole, both arms named below verbatim from
`.floor/2026-09-28T20-58-05Z/ARM.txt`, then fixed and surfaced — only then re-run.

**First floor, at `6e376fee9`** — `.floor/2026-09-28T20-58-05Z/`:

```
Summary [ 385.574s] 6219 tests run: 6217 passed (24 slow), 2 failed, 24 skipped
FAIL [   0.022s] (1645/6219) wat rete::kernel::tests::where_tree_branch_differential::where_tree_branch_agrees_with_the_reference_filter
FAIL [   2.114s] (2257/6219) wat::cli every_recorded_migration_replays::every_recorded_migration_is_fixtured_or_runed
```

**Arm 1 — `every_recorded_migration_is_fixtured_or_runed`** (verbatim, `ARM.txt:679-698`):

```
stdout ───

    running 1 test
    test every_recorded_migration_replays::every_recorded_migration_is_fixtured_or_runed ... FAILED

    failures:

    failures:
        every_recorded_migration_replays::every_recorded_migration_is_fixtured_or_runed

    test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 86 filtered out; finished in 2.11s

  stderr ───

    thread 'every_recorded_migration_replays::every_recorded_migration_is_fixtured_or_runed' (3976400) panicked at /home/john/work/holon/wat-rs/tests/cli/every_recorded_migration_replays.rs:654:5:
    recorded-migration coverage failed (1):
    typed-constructors: no fixture, no rune:replay(unreadable-preimage)
    note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

**Mechanism**: `wat-scripts/fixes/typed-constructors.wat` is a new recorded migration; the gate
requires every one to carry either a `wat-scripts/fixes/replay/<stem>/{before.pre,after.post}`
fixture or a `;; rune:replay(unreadable-preimage) — <reason>` header line. Item 5 of the brief
("Tests: the codemod's replay fixture (before/after)") was only half done — item 1's run-time
rows were written, the codemod's own replay-gate requirement was missed. **A genuine fixture is
not possible here**: the codemod's `:user::main` unconditionally reads
`.typed-constructors-table.edn` (the type table side-channel), which is deliberately never
committed (item 2's own rule) and is absent in the replay harness's working directory
(`current_dir(manifest())`, no such file) — the read fails before a single site is even
considered, so no `before.pre` is a readable preimage this mechanism could drive; committing one
would mean committing its (explicitly non-committed) type-table input too. **Fix**: added the
`rune:replay(unreadable-preimage)` header line with that reason (`3d8f1a8cc`) — the same
exemption `angle-brackets-to-binder.wat`/`parametrics-take-a-type-vector.wat` already carry for
an analogous "no fixed preimage exists" case. Re-verified alone:
`cargo test --release --test cli every_recorded_migration_is_fixtured_or_runed` →
`test result: ok. 1 passed; 0 failed`.

**Arm 2 — `where_tree_branch_agrees_with_the_reference_filter`** (verbatim, `ARM.txt:40-59`):

```
stdout ───

    running 1 test
    test rete::kernel::tests::where_tree_branch_differential::where_tree_branch_agrees_with_the_reference_filter ... FAILED

    failures:

    failures:
        rete::kernel::tests::where_tree_branch_differential::where_tree_branch_agrees_with_the_reference_filter

    test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1367 filtered out; finished in 0.01s

  stderr ───

    thread 'rete::kernel::tests::where_tree_branch_differential::where_tree_branch_agrees_with_the_reference_filter' (3970551) panicked at src/rete/kernel/tests/where_tree_branch_differential.rs:509:5:
    assertion `left == right` failed: the non-uniform `where-*` axes must match NON_UNIFORM exactly — a new axis, a deleted one, or one that changed its row-driver shape all land here
      left: ["where-accum-from-left", "where-accum-group", "where-accum-lead", "where-accum-lead-cascade", "where-accum-where", "where-accum-where-chain", "where-boolean", "where-collection", "where-control", "where-exists", "where-fact-bind", "where-inline-computed", "where-inline-keyword", "where-join-left", "where-join-order", "where-multivar", "where-nested-combinators", "where-nesting", "where-not-and", "where-not-and-bound", "where-not-and-not", "where-not-bound", "where-not-derived-in-query", "where-not-fact", "where-not-not", "where-not-or", "where-not-where", "where-not-windy", "where-numeric", "where-or-and", "where-or-conditions", "where-or-inline", "where-query-compat", "where-query-params", "where-record", "where-shapes", "where-string", "where-test-chain"]
     right: ["where-accum-from-left", "where-accum-group", "where-accum-lead", "where-accum-lead-cascade", "where-accum-where", "where-accum-where-chain", "where-exists", "where-fact-bind", "where-join-left", "where-nested-combinators", "where-not-and", "where-not-and-bound", "where-not-and-not", "where-not-bound", "where-not-derived-in-query", "where-not-fact", "where-not-not", "where-not-or", "where-not-where", "where-not-windy", "where-or-and", "where-or-conditions", "where-or-inline", "where-query-compat", "where-query-params", "where-test-chain"]
    note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

**Mechanism**: `classify()` (`src/rete/kernel/tests/where_tree_branch_differential.rs`) decides
"uniform" vs "non-uniform" for every `wat-scripts/perf/grid/where-*.wat` axis by finding a
**hardcoded raw-FILE-TEXT marker**, `"::seed (:wat::core::match (:wat::rete::compile-all rules
(:wat::core::PersistentVector (:"`, inside each file's `:NS::seed` definition. This stone's own
corpus conversion correctly rewrote every one of those 38 files' untyped
`(:wat::core::PersistentVector (:NS::q-Hit))` call to `(wat.type/PersistentVector :- […]
(:NS::q-Hit))` — exactly what the brief asks for — so the marker stopped matching and 11 axes
(`where-boolean`, `where-collection`, `where-control`, `where-inline-computed`,
`where-inline-keyword`, `where-join-order`, `where-multivar`, `where-nesting`, `where-numeric`,
`where-record`, `where-shapes`, `where-string`) were misclassified non-uniform (`left`, the
COMPUTED list, grew past `right`, the hardcoded `NON_UNIFORM` expected list). This function
already carries a **dual-spelling fallback for exactly this class of drift**, for a DIFFERENT
marker (its own row-count check) — its own comment: *"this walk keys off the raw FILE TEXT, not
a parsed AST, so it is one more site that compared by spelling instead of denoting first."* Code
wins over the brief's naive expectation that only OLD tests would need no change — but the fix is
squarely in the PRE-EXISTING precedent this same function already set for the same problem class,
not a new exemption. **Fix**: added the identical dual-spelling handling for the `MID` marker
(`3d8f1a8cc`) — every axis's `q-Hit` denotes `:wat::rete::Query` uniformly (confirmed corpus-wide
via `grep`, not assumed), so the second spelling is a fixed string, not a wildcard. The paren
nesting after the head is unchanged (the type vector sits BEFORE the `(:NS::q-Hit)` argument, not
around it), so the `tail` check needed no change. Re-verified alone:
`cargo test --release --lib rete::kernel::tests::where_tree_branch_differential` →
`test result: ok. 1 passed; 0 failed`.

Neither fix touched the corpus or the codemod's OWN transformation — both are test-infrastructure
corrections for tests that key off raw text instead of the checker's own denotation, the same
class of fragility `where_tree_branch_differential.rs`'s own prior comment already named once.
`cargo clippy --release --all-targets -- -D warnings` re-verified clean after both fixes (rc 0).

**Second floor, at `3d8f1a8cc`** — `.floor/2026-09-28T21-11-30Z/`:

```
Summary [ 380.813s] 6219 tests run: 6219 passed (21 slow), 24 skipped
```

`exit=0`. All green.

## Gates

| what | how | result |
|---|---|---|
| `cargo build --release` | after part 1 and after each corpus pass | clean each time |
| stdlib loads | `wat --check`/`wat` a trivial program after the stdlib-only pass rebuild | `(:wat::core::PersistentVector 1 2 3)` checks and runs |
| clippy | `cargo clippy --release --all-targets -- -D warnings` | rc 0 |
| part 1 tests | `cargo test --release --test function stone255_69` | `4 passed; 0 failed` |
| census (pre) | `scripts/replay/census.sh` before any corpus edit | `.census/2026-09-28T20-52-44Z.txt`, 2266 files |
| census (post) | `scripts/replay/census.sh` after the full corpus conversion | `.census/2026-09-28T20-55-15Z.txt`, 2266 files |
| census --diff | `scripts/replay/census.sh --diff PRE POST` | `census-diff: no STOP-8`, rc 0 |
| delta | `scripts/replay/delta.sh --list <362 files> --codemod wat-scripts/fixes/typed-constructors.wat` | `ORIG-CLEAN 347/362  CONV-CLEAN 347/362  NEW 0  RECOVERY 0`, rc 0 (run against the pre-conversion content, restored via `git checkout <part-1 commit> -- <list>` then `git checkout HEAD --` after, to test the codemod's own transformation rather than a no-op on already-converted files) |
| idempotent | the codemod re-run over all 362 converted files | `sha256sum` before/after identical — 0 changes |
| coverage | 1520 / 1630 converted, 110 excluded (35 + 74 + 1), each named | stated above, arithmetic closes |
| release floor (1st) | `scripts/floor.sh`, once, alone, foreground, at `6e376fee9` | RED — 2 failed (both diagnosed, fixed, individually re-verified — § "The floor went RED" above) |
| release floor (2nd) | `scripts/floor.sh`, once, alone, foreground, at `3d8f1a8cc` (after both fixes) | `Summary [ 380.813s] 6219 tests run: 6219 passed (21 slow), 24 skipped`, exit=0 |

## Files

- `src/types.rs` — part 1, `constructor_head_key` generalized. Committed `c9d1106be`.
- `tests/function/probe_stone255_69_wat_type_scalar_dispatch.{wat,wat.bad,rs}` — part 1 tests.
  Committed `c9d1106be`.
- `wat-scripts/fixes/typed-constructors.wat` — the recorded codemod. Committed `55cf63db7`.
- `wat-scripts/scratch-pad/255-69-census-walk.wat` — the report-only census tool the type table's
  own census reused. Committed `55cf63db7`.
- 362 corpus files (15 stdlib, 347 rest) — the conversion. Committed `6e376fee9`.
- `wat-scripts/fixes/typed-constructors.wat` (the `rune:replay(unreadable-preimage)` header
  line) and `src/rete/kernel/tests/where_tree_branch_differential.rs` (the `MID` dual-spelling
  fix) — the floor's own two fixes. Committed `3d8f1a8cc`.
- `scratch/stone25569_desugar.rs` — the type-table builder (Rust; chosen over a `.wat` script
  because the checker's compact-tuple DISPLAY notation is not `read-string`-parseable, so a
  text-notation bridge has to run before any wat-side AST walk can see the fragment at all).
  **Not committed** — an input, per the brief; kept in the session scratchpad
  (`/tmp/claude-1000/…/scratchpad/stone25569_desugar.rs`), reproducible from this SCORE's method.
- `.typed-constructors-table.edn` — the codemod's side-channel input, regenerated fresh per
  invocation from `scratch/stone25569_desugar.rs`'s output. **Not committed** (untracked; removed
  from the working tree after each drive).
