# AMEND — STONE 255.67: K1, canonicalize at parse; then finish the stone

**Drawn 2026-09-28.** **Executor: a fresh Sonnet subagent**, continuing the first agent's work. The brief
`BRIEF-STONE-255.67-cutover-2-types.md` stands, as amended here. Commit locally on `main`; **do not push**.

## Where the stone is (read these first)

- **Committed** (`029ae9f2c`, `0ace00868`, `a0efa4ae3`): the codemod `wat-scripts/fixes/types-to-wat-type.wat` (a single
  left-to-right walk, five position rules A–E, text from `:wat::keyword::to-type-form`), its replay fixture, the
  `wat.type/AST` rendering fix, the doc grammar accepting a namespaced symbol, and `tests/types/probe_arc255_67_cutover_types.*`.
- **Uncommitted in the working tree** (the first agent was cut off before committing): the **converted stdlib**
  (`wat/**/*.wat`, 63 of 65 files) and three fixes, each with a regression test:
  1. `src/types.rs`: `extend-type`/`derive`'s Symbol-child edge key was built un-denoted;
  2. `src/check.rs` `check_legacy_user_main_signature`: accepted only a Keyword return type;
  3. `src/function/subsume.rs` `value_matches_type_by_name`: raw compare of an un-denoted container head.
- **The floor on that working tree was RED**, and it was **not re-run**: `.floor/2026-09-28T05-09-49Z`,
  `6213 tests run: 6110 passed (23 slow), 103 failed, 24 skipped`. The orchestrator classed it:
  - ~170 hits: `:wat::rete::insert-all: parameter #2 expects (:wat::core::PersistentVector :- [:wat::core::Record]); got
    (:wat::core::PersistentVector :- [:woi::Reading])`: a record is no longer admitted as the `Record` root inside a
    container, now that the stdlib spells that root `wat.type/Record`;
  - 38: *"type `:wat::core::Vector` does not implement surface method `seq` — expected a `defn :wat::core::Vector/seq`
    but none is registered"* (also `PersistentVector`, `List`): `wat/seq.wat`'s `extend-type` now names
    `(wat.type/Vector :- [T])`, so the method key and the lookup key differ in spelling;
  - a few: `(Vector :- [:T])` where `(Vector :- [String])` is expected, and `concat` expecting `String` got `:T`. **Cause
    unknown**; it may be a codemod mis-conversion.

**Every one of the five known bugs is one class:** a key or a comparison built from one spelling of a type and checked
against the other. Under the temporary door, a parsed type can carry either `:wat::core::X` or `:wat::type::X`.

## The ruling (builder, 2026-09-28): K1

**Canonicalize at parse, now, to the old key.** Wherever a type spelling enters the system, `wat.type/X` (and the
retired `:wat::type::X` keyword) becomes the **old canonical key** `:wat::core::X`, and `wat.type/AST` becomes
`:wat::WatAST`. After this, **no parsed type, registered edge key, method key or constructor dispatch key carries a
second spelling**, so no comparison can see one. This is still T-door: the internal key stays old until stone 4 flips
what that one key *is*. The 31 `type_denotation` compares stop mattering (leave them; stone 4 deletes them).

## The work

1. **Commit the working tree as it is** (the converted stdlib and the three fixes with their tests), in one commit whose
   message records the red floor `.floor/2026-09-28T05-09-49Z` (103 failed) as the state it lands at, and that K1
   follows. Then implement K1 in the next commit(s).
2. **K1: one function** (for example `canonical_type_key`) that maps a type spelling to its one key, used by **every**
   place a type spelling enters: `parse_type_node`'s symbol and keyword arms, `parse_type_form`'s head and arguments,
   `parse_type_expr` / `parse_type_expr_from_source`, `extend-type`/`derive`'s child and target (edges **and** the method
   keys `register_extend_type_methods` builds), the member-method rekey (`freeze::env::rekey_type_member_functions`),
   and the constructor dispatch key (`constructor_head_key`, `src/types.rs:141`). Report every site, and say which of
   the first agent's three fixes K1 makes redundant (keep them; they are harmless).
3. **Floor.** Run `scripts/floor.sh` on the K1 tree. If the `insert-all` and `seq` classes are gone, K1 worked. If the
   `:T` rows remain, find their cause (read the converted file against its original; check whether the codemod
   converted something that is a type variable or a value).
4. **Finish the stone as the brief says:** convert the rest of the corpus (re-derive the list:
   `git ls-files '*.wat' '*.wat.bad' | grep -v '^wat-scripts/fixes/' | grep -v '^wat/'`, excluding the 9 unreadable
   `.wat.bad` fixtures the first agent listed:
   `docs/arc/2026/05/130-cache-services-pair-by-index/complected-2026-05-02/{substrate,test}.wat.bad`,
   `tests/cli/wat_grep__malformed.wat.bad`, `tests/types/probe_arc214_lexer_primed_generic_head_{primed,unprimed}_space.wat.bad`,
   `tests/types/probe_arc232_generic_method_type_application.wat.bad`, `tests/types/struct_destructure_non_symbol.wat.bad`,
   `tests/value/wat_arc220_char_supplementary_plane.wat.bad`, `tests/wat_lang/wat_arc072_letstar_parametric_whitespace.wat.bad`);
   rebuild; idempotence (a re-run changes 0 files); census `--diff` against the pre-census
   `.census/2026-09-28T04-23-04Z.txt`; `scripts/replay/delta.sh`; clippy; the final floor.
5. **Write `SCORE-STONE-255.67-cutover-2-types.md`**, covering both agents' work: the codemod and its rules, the five
   bugs of one class and K1, both floors (the red one quoted verbatim, with its arms), and every gate verbatim.

## STOP triggers (checked against the work list: none fires on a site it orders changed)

- **STOP-1:** after K1, a floor red that is **not** a two-spellings comparison, and not the `:T` rows being chased in
  step 3. Do not re-run. Copy the failing block verbatim from `.floor/<stamp>/`, name the arm, and STOP.
- **STOP-2:** the `:T` rows turn out to be the codemod converting something that is not one of the 24 hard primitives in
  a type position. Report the rule and the sites, and STOP. Do not patch the corpus by hand.
- **STOP-3:** a census flip after converting the rest of the corpus that is not a spelling K1 covers. Quote it and STOP.
- A STOP means STOP: report, do not work around it. A red floor is never re-run to make it green.

## Doctrine

`holon/CLAUDE.md` binds you (codemods for `.wat`, the floor via `scripts/floor.sh`, no known flake). Capture `rc=$?`
on the next statement. Do not wait on a process with `pgrep -f`, which matches its own command line; wait for the
command itself, or for a file the command writes. If this amendment contradicts the code, the code wins: say so.
Commit with `git add -- <paths>`, never `-A`. **Do not push.**
