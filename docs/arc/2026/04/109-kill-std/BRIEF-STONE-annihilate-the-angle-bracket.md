# BRIEF — annihilate the angle bracket

**`:-` is the parameterization operator and there is no other.** Angle brackets stop meaning anything
as type syntax: not a declaration name, not a type reference, not a method-member name, not a
call-site type application. You will remove the PERMISSION at both lexer doors, migrate the 26 files
that then fall outside the language, and leave the comparison operators and arrows exactly as they are.

Read `DESIGN-STONE-annihilate-the-angle-bracket.md` first — it carries the measurements this brief
rests on. Copy the shape of `SCORE-STONE-the-last-comma-lives-in-a-symbol.md` for your report.

## ⛔ THE ORDER IS FORCED — measured, and getting it wrong costs the whole flight

The recorded codemod reads its input through `read-string`. **Once the wall is up it cannot read a
file containing an angle form** — it dies with `UnknownFunction: :wat::edn::ForeignRecord does not
implement surface method 'message'`. ③'s wall was at the type PARSER and the codemod carried its own
renderer to dodge it; **this wall is at the LEXER** and no renderer dodges that.

```
STEP 1   tree is already wall-DOWN     run the codemod for classes A + B
STEP 2   hand-fix                      classes C, D, E
STEP 3   apply the wall                git apply STONE-annihilate-the-angle-bracket.wall.patch
STEP 4   rebuild, verify acceptance    the table below
```

No stash, no revert, no branch dance — the tree starts wall-down because the wall is not committed.

## STEP 1 — the codemod, classes A and B only

`wat-scripts/fixes/angle-brackets-to-binder.wat` is the recorded migration (R21). It is CORRECT for:

- **A — declaration name**: `defn :test::make-3tuple<T>` → `defn :test::make-3tuple :- [T]`
- **B — type reference**: `xs <- :wat::core::Vector<t::New>` → `xs <- (:wat::core::Vector :- [:t::New])`

⛔ **It is WRONG for class D (call-site type application)** and you must not let it near one. Measured:

```clojure
(:wat::test::assert-eq<:wat::core::i64> …)  →  ((:wat::test::assert-eq :- [::wat::core::i64]) …)
(:test::make-3tuple<wat::core::bool> true)  →  ((:test::make-3tuple :- [:wat::core::bool]) true)
                                               ArityMismatch: expected 1 argument(s); got 2
```

Two defects: a doubled `::` on an already-colonned arg, and a *reference form standing where a
callable head goes* — a form is not a name. **Correct the codemod so a call-site type application is
DELETED rather than rendered** (it already carries a DECL-NAME-vs-REFERENCE role split in
`declarator-head-keyword?`; this is a third role), then re-run it. If you judge the role split too
invasive for this stone, FENCE it instead — make the codemod refuse a call-head site loudly — and say
so in your report. What you may NOT do is leave it silently emitting a form that does not compile:
that is a recorded migration re-introducing an illegal shape on every future run.

**Dry-run on `/tmp` copies and `diff` before touching the corpus.** R21 requires it.

## STEP 2 — the hand-fixed classes

**C — method-member name (3 sites).** The door already takes the binder; it shipped last stone.

```
tests/types/probe_arc293_4e_pre_ii_generic_surface_method.wat:15    (make<T> …  → (make :- [T] …
tests/types/probe_arc293_4e_pre_iii_extend_impl_inherits_types.wat:13
wat-scripts/probes/arc-170/probe-locus1-generic-surface-method.wat:9
```

**D — call-site type application: DELETE IT. There is no replacement spelling.**

```
tests/kernel/probe_arc259_started_at_boot.wat:6     (:wat::test::assert-eq<:wat::core::i64>  → (:wat::test::assert-eq
tests/program/probe_arc259_cpu_count.wat:13         same
tests/program/probe_arc259_env_peer_kind.wat:15     (:wat::test::assert-eq<:wat::program::PeerKind>
wat-tests/core/generic-tuple-turbofish.wat:21       (:test::make-3tuple<wat::core::bool> true) → (:test::make-3tuple true)
```

This is proven, not assumed — the same fixture with the declaration on the binder and the turbofish
removed `--check`s clean, runs, and returns `"hello"`. Inference already does the work;
`canonical_callable_name` was stripping the suffix at every lookup, which is what made it decoration.

**E — the 13 `.wat.bad` negative fixtures.** Two kinds, and the difference decides the treatment:

- **(i) the angle form is INCIDENTAL** — the fixture's real subject is something else. Migrate the
  angle form and leave the subject alone. This is exactly the arc 232 call from last stone.
  Most of the 13, including `probe_arc241_stone5_c06`, `probe_arc249_*`, `probe_arc251_*_fact01`,
  `probe_arc278_call_context_two_param_public_arm`, `typed_if_match_bare_symbol_variant`,
  `wat_arc170_slice_1e_user_main_nil_slice2_4arg`, `probe_arc241_stone17_defmacro_canonical_c02`.
- **(ii) the angle LEXER *is* the subject** — these tested that the permission WORKED:
  `tests/types/probe_arc214_lexer_primed_generic_head_primed_space.wat.bad`,
  `tests/types/probe_arc214_lexer_primed_generic_head_unprimed_space.wat.bad`,
  `tests/wat_lang/wat_arc072_letstar_parametric_whitespace.wat.bad`.
  **Their subject no longer exists.** Re-point them: they now assert the angle head is REFUSED by the
  new wall. A negative control that CAN be kept MUST be kept — do not delete them.
  `tests/types/probe_arc232_generic_method_type_application.wat.bad:13` is kind (ii) as well: its
  subject is the callable turbofish, which this stone refuses at the reader instead of at the comma.

Each `.wat.bad` has an owning `.rs` asserting a specific message. When you change what a fixture
triggers, that assertion moves with it. **Assert the MECHANISM, not the whole diagnostic** — that is
what made last stone's red legible instead of silent.

⛔ **LEAVE `docs/arc/2026/05/130-cache-services-pair-by-index/complected-2026-05-02/{substrate,test}.wat`
alone.** Archived snapshots inside `docs/`, not loaded corpus.

## STEP 3 — the wall

```bash
git apply docs/arc/2026/04/109-kill-std/STONE-annihilate-the-angle-bracket.wall.patch
```

Four hunks in `crates/wat-reader/src/lexer.rs`: a `LexErrorKind::AngleTypeHeadInName` variant, its
`Display` arm, and the two `angle_depth += 1` permissions replaced by the refusal. The patch is the
exact text that produced every measurement in the DESIGN — apply it rather than retyping it.

## STEP 4 — acceptance, measured under the wall

| # | form | expected |
|---|---|---|
| 1★★ | `:wat::core::Vector<wat::core::i64>` | ⛔ lex error naming `:-` |
| 2★★ | `(make<T> [x] -> :T)` | ⛔ refused |
| 3★★ | `(:my::helper<wat::core::i64> 1)` | ⛔ refused |
| 4★★ | `:wat::core::HashMap'<wat::core::i64>` | ⛔ refused — arc 214's primed head |
| 5★★ | `a<b` | ⛔ refused — **the narrowing; state it in your report** |
| 6★★★ | `:wat::core::<` · `:wat::core::>=` · `<-` · `->` | ✅ still lex |
| 7★★★ | `(:wat::core::Vector :- [:i64] 1, 2, 3)` | ✅ `[1 2 3]` — the comma dual |
| 8★★ | `:wat::kernel::Peer'` · `foo/bar` | ✅ still lex |
| 9 | corpus | every `.wat`/`.wat.bad` outside `docs/arc` reads clean under the wall |
| 10 | acceptance criteria | `target/release/wat --check <file>` per file; a scoped `nextest -E` on the tests you touched |

**Rows 6 and 7 decide it.** Rows 1–5 go green for a lexer that refuses `<`, or every symbol, or
everything — all of which destroy the language. **Only the operators and the comma dual surviving**
prove you removed the type-head permission and nothing else. This is the same pairing that made the
comma strike meaningful, and it is the row a careless wall fails.

## STOP triggers

- **STOP-1 — a file outside the 28 goes red.** The census was taken by imposing this exact wall over
  all 1798 files. A 29th means the wall does more than the DESIGN says. Report the file and the form.
- **STOP-2 — an operator or arrow stops lexing** (row 6). The wall is wrong; report the exact form.
- **STOP-3 — a class-D deletion changes a value or a type.** The claim is that a call-site type
  application is inert. If deleting one changes behaviour, that claim is false for that site — stop
  and report it with the before/after.
- **STOP-4 — a `.wat.bad` of kind (ii) has no honest re-pointing.** Report it rather than deleting the
  control.

## Boundaries

- `crates/wat-reader/src/lexer.rs` (via the patch), the 26 non-archived corpus files, their owning
  `.rs` tests, and `wat-scripts/fixes/angle-brackets-to-binder.wat`.
- **Do NOT touch `src/types.rs:4631`** — ③'s type-parser wall stays; it is the backstop for names
  MINTED at expand time, which never pass through the lexer.
- **Do NOT delete the downstream machinery** (`canonical_callable_name`, `split_type_params`,
  `split_name_and_type_params`, `split_method_name_type_params`, the `find('<')` splits). That is the
  sibling purge stone and it needs a green floor first to say what is genuinely dead.
- Do NOT commit, push, stash or amend. Keep the index EMPTY: no `git add`, no
  `git checkout <ref> -- <path>` (it STAGES).
- The orchestrator runs the floor and clippy centrally. Run cheap targeted checks —
  `target/release/wat --check <file>` (~0.2s) and a scoped `nextest -E` — not the floor.

Prefix long commands with `systemd-run --user --scope -q -p MemoryMax=24G -p MemorySwapMax=0 timeout 3000`.
Read exit codes DIRECTLY — never through a pipe, never after a trailing `; echo`.

## Your report

Rows 1–8 verbatim, in one run, rows 6 and 7 alongside the refusals. What you did with the codemod's
class-D arm (corrected or fenced) and why. The kind-(i)/kind-(ii) split you made across the 13
`.wat.bad` files, and the re-pointing you gave each kind-(ii). Any STOP that fired, with the arm
captured verbatim before you diagnosed it. What surprised you.
