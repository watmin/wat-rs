# BRIEF — STONE 255.67: cutover 2 of 7 — the corpus spells its hard primitives `wat.type/…`

**Drawn 2026-09-27 against `main` @ `4d5f238a5`.** **Executor: a Sonnet subagent** (grok's credits are out). A strike:
a recorded codemod, the corpus it converts, small `src/` support, and tests. Commit locally on `main` (as many commits
as the work needs, each with `git add -- <paths>`, never `-A`); **do not push**. Your final message is your report: say
what landed, the gates' results verbatim, and anything you stopped on.

## Where this sits

`WEIGH-STONE-255.65-size-the-cutover.md` sequences the Clojure cutover in seven green stones. Stone 1 (255.66) made
**position decide**: `(wat.type/Vector :- [T])` is the type in a type position and the empty vector elsewhere, and a
`wat.type/` head dispatches through the existing denotation door to the old key (`constructor_head_key`,
`src/types.rs:141`; `type_denotation`, `src/edn/render.rs:~3693`). **The temporary door (T-door) is ruled:** in this
stone the internal key stays `:wat::core::…`, and stones 4 and 5 delete the door.

## The ruling (builder, 2026-09-27; `FINDING-the-shape-of-a-declared-signature.md` § "`wat.type/` is closed")

`wat.type/` holds exactly **24** hard primitives:
`i64 f64 u8 bigint rational char String bool keyword nil Value Never Fn Record Struct Vector HashMap HashSet List Tuple
PersistentVector PersistentMap Bytes AST`. Today each is `:wat::core::<name>`, except `AST`, which is `:wat::WatAST`.

**This stone converts only those 24 names, and only where they are types or type constructors.** Everything else stays
as it is now: function and form heads (`:wat::core::defn`, `:wat::core::let`, … are stone 5), `:wat::core::Uuid`
(stone 3 moves it to `wat.uuid/UUID`), `:wat::time::Instant`/`Duration`, `:wat::holon::HolonAST`/`Record`, and declared
types (`:wat::core::Option`, `Result`, `Span`, `Pos`, `Error`, `EvalError`, the read outcomes, `Orderable`,
`Equatable`).

## The work

1. **A recorded codemod**, `wat-scripts/fixes/types-to-wat-type.wat`, on the wat-fix framework (`wat/fix.wat`,
   `:wat::fix::fix-text`, comment-faithful span edits; copy the shape of an existing `wat-scripts/fixes/*.wat`, and read
   `wat/fix.wat`'s header, including the STASH-DANCE / BOOTSTRAP note). A keyword `:wat::core::<one of the 24>` or
   `:wat::WatAST` becomes the symbol `wat.type/<name>` (`wat.type/AST` for `WatAST`) **when it is a type**:
   - after `<-` or `->`; after `:-` where `:-` annotates a binder or a return;
   - inside a type bracket `:- [ ... ]`, and inside a fn-type bracket `[A B :-> R]`;
   - as the head of a `(Head :- [...])` form, **in any position** (in a type position it is the type; elsewhere it is
     the constructor, and stone 1 made both dispatch);
   - in a `typealias` body; in `extend-type`'s child and target; a `derive`'s type arguments; a record, struct or
     newtype field type; an enum variant's payload type; a surface feature's types; a bound `[T :< X]`.

   **Not** when it is a keyword **value** (data: a map key, an argument to `type-of`, `keyword-node`, `from-string`, a
   quoted example compared as data). If you cannot tell whether an occurrence is a type or a value, do not convert it:
   list it (STOP-2 below if there are many).
2. **The bootstrap** (255.63/.64 measured it): the codemod lives in `wat-scripts/fixes/`, but it runs on the stdlib
   framework (`wat/fix.wat` is `include_str!`'d). Build the release binary **before** converting the stdlib; convert
   `wat/**/*.wat`; rebuild; confirm the converted stdlib loads (`./target/release/wat --check` of a trivial file);
   then convert every other tracked `*.wat` / `*.wat.bad`. **Exclude the recorded codemods themselves**
   (`wat-scripts/fixes/**`: a tool is never its own input). Dry-run on a `/tmp` copy of a few files and `diff` first.
   Listing every path, apply: `printf '["p1" "p2" …]\n' | ./target/release/wat ./wat-scripts/fixes/types-to-wat-type.wat`.
   The corpus is ~2,300 files; a full conversion took ~45 minutes in earlier measurements.
3. **Resolution for the new spellings.** `wat.type/<name>` must resolve for all 24, through the existing denotation door
   (`wat.type/X` → `:wat::core::X`; `wat.type/AST` → `:wat::WatAST` exists since 255.66). Measure each of the 24 with
   a probe. `:wat::core::Tuple` is a callable but not a `TypeEnv` member (255.65 §4): make `wat.type/Tuple` resolve in
   type position. `:wat::core::Never` is deliberately unregistered (test `stone_255b_never_is_deliberately_unregistered`,
   `src/types.rs`): if the corpus has no `:wat::core::Never` type position, leave it; if it does, report it (do not
   register it).
4. **Rust doc directives.** The intrinsics' `@arg` / `@ret` type tokens (in `src/**/*.rs` and `crates/**/*.rs`) are
   types too, and the same 24 names get the same new spelling there. The doc grammar requires a type token to start
   with `:`, `(` or `[` (`crates/wat-doc/src/lib.rs:641`, `:693`, and the two special-form twins near `:1501`/`:1544`).
   Let it also accept a **namespaced** symbol (`wat.type/i64`), and keep refusing a bare one like `Bytes` (test
   `bare_symbol_without_colon_is_still_refused_by_the_colon_rule` keeps its meaning). Earlier layers built this cure
   as `docs/arc/2026/06/255-builtin-registry/probes-255.64/src.diff` (the `type_token_shape_ok` hunk); reuse it. Convert
   only type tokens; leave `@example` forms for stone 5.
5. **Tests.** A Rust test that drives a small wat fixture written in the new spelling for each of the 24 (a parameter
   typed with it; for the containers, the constructor in value position), plus the codemod's own replay fixture
   (copy how an existing recorded codemod is tested; `wat-scripts/fixes/replay/` if that is the convention).

## Gates (run all; quote the Summary lines verbatim)

| what | how | expected |
|---|---|---|
| release floor | `scripts/floor.sh`, then read the Summary line in `.floor/<stamp>/` | all passed. The count against 6200 at `ae97d8092`, plus your new tests |
| clippy | `cargo clippy --release --all-targets -- -D warnings` | rc 0 |
| census | `scripts/replay/census.sh` **before** converting (pre), then `scripts/replay/census.sh --diff` after | no new rc flips (the three carried `fix_source_local_rules__contract-0{6a,6b,7}` goldens stay as they are) |
| delta | `scripts/replay/delta.sh` | RECOVERY 0; report NEW |
| idempotent | re-run the codemod over the converted corpus | 0 changes |

## STOP triggers (checked against the work list: none fires on a site the list orders converted)

- **STOP-1:** a converted file's `--check` result changes (a census flip), and the cause is not a spelling the rulings
  cover. Quote the error, name the file and site, and STOP.
- **STOP-2:** more than a handful of occurrences cannot be classed as type vs value. List them with file:line and STOP.
- **STOP-3:** the floor is red. **Do not re-run it** (there is no known flake; a re-run destroys the evidence). Copy the
  failing test's whole block from `.floor/<stamp>/` into your report verbatim, name the arm, and STOP.
- A STOP means STOP: report, do not work around it.

## Doctrine

- `holon/CLAUDE.md` binds you: the codemod doctrine (no hand-edits or sed/python for `.wat` migrations), the floor via
  `scripts/floor.sh`, no known flake, scratch `.wat` in `wat-scripts/scratch-pad/`.
- Capture `rc=$?` on the **next** statement; never `$?` inside a string containing `$(…)`.
- **If this brief contradicts the code, the code wins. Say so plainly.**
- Write `SCORE-STONE-255.67-cutover-2-types.md` beside this brief (what you did, counts of converted sites by position,
  the gates verbatim, anything listed or stopped). Commit it. **Do not push.**
