# AMEND 3 — STONE 255.86: the proofs are tests

**Drawn 2026-10-03.** **Executor: grok via pulsare, working solo** (it runs the floor). Continues at `bcc914b82`.
Commit locally on `main`; **do not push**.

## Why

The cutover (`4f8f551ea`) and its cure (`54e2d948d`) are green at **6394 passed: the brief's baseline exactly.** The
brief's item 5 asked for tests, and none landed: every differential that proved a mapping ran as a `/tmp` program
(`/tmp/g1-diff/`, `/tmp/g1-arm/`), and no committed test pins a retirement refusal or the new homes. Today's proof
cannot be re-run tomorrow; a later change could quietly undo a mapping and nothing would go red.

## The work

1. **The differentials become driven tests**, in co-located `.wat` fixtures (`startup_beside` / `call_beside_value`; no
   inlined wat): for every pair in `wat-scripts/fixes/one-name-per-operation.edn`, the value the replacement returns on
   each container type the old name accepted, **pinned** (not only "old equals new": the old names are retired, so the
   test pins the value the replacement must return). Include the split cases: `into` on `(PV, V)`, `(PV, PV)`, `(V, V)`;
   `core/dissoc`/`keys`/`values` on a PersistentMap; `contains?` on a vector/set (element) and on a map (key).
2. **The retirement is tested as a table:** one test walks every row the stone added (`src/remedy/retirement.rs`
   436–478) and asserts that a program calling the retired name is refused **naming its replacement**. Derive the list
   from the table, not a hand list, so a new row is covered with no edit.
3. **The new homes run:** `wat.bytes/to-hex`, `wat.bytes/from-hex`, `wat.record/field-at`, `wat.record/same-data?`, and
   the typed constructors that replaced `List/of` and `char/of`, each with a pinned value.
4. **R-a's two remaining parents:** query the type registry (the tool that owns the fact) for `:myapp::Formattable` and
   `:wat-tests::holon::Reject`. If neither is a registered type, R-a respells them to `::`, and the member-join lint is
   corrected to decide "type" by the registry rather than by a PascalCase segment (say how). If the lint's
   PascalCase rule cannot ask the registry, **STOP** with what it would need.
5. The floor, clippy, census `--diff`; `git status` clean before the floor. The count rises by the new tests.

STOPs as in the brief. A STOP means STOP. Append to the SCORE, commit, **do not push**.
