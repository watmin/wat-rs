# BRIEF — STONE 255.68: the checker's type record on `main`, and measure every untyped constructor call

**Drawn 2026-09-28 against `main` @ `468fbf16c`.** **Executor: a Sonnet subagent** (grok's credits are out). A strike
**plus a measurement**: part 1 lands (additive `src/` + tests), part 2 lands only its SCORE. Commit locally on `main`
(`git add -- <paths>`, never `-A`); **do not push**. Your final message is your report.

## The rulings (builder, 2026-09-28)

- **Every collection constructor has one shape: `(wat.type/X :- [T…] items…)`.** `Vector` and `PersistentVector` alike
  (and `HashMap`/`PersistentMap`, `HashSet`, `List`, `Tuple`). **An untyped constructor call is illegal**; a wall will
  enforce it and the heretics self-identify. (Later, collections move to rpds-backed `wat.type/vec`/`map`/`set` in the
  same shape; not this stone.)
- **A type constructs its own values:** `(wat.type/u8 65)` makes a `u8` (the reader defaults integers to `i64`), and
  `(wat.type/char "a")` a `char`.
- **The ~1,800 untyped collection calls get their `T` from the checker, not from the text** (option TD): a type-driven
  codemod writes the element type the checker inferred. This stone builds the instrument and measures; the codemod is
  the next stone.

## Measured by the orchestrator (the residue after 255.67; `WEIGH-STONE-255.67-cutover-2-types.md`)

Code only, excluding `wat-scripts/fixes/`, comments and strings: bare `(:wat::core::PersistentVector …)` 1,334 (179+ are
the empty `(:wat::core::PersistentVector)`), `(:wat::core::Tuple …)` 230, `(:wat::core::PersistentMap …)` 122 (often
the empty seed of an `assoc` chain), `(:wat::core::List …)` 109, `(:wat::core::Vector …)` 6; scalar constructors
`(:wat::core::u8 65)` 57 and `(:wat::core::char "a")` 17. (An earlier count of `keyword`/`bool`/`Record`/`i64` "calls"
was a false positive: `keyword-node`, `Record/same-data?`, and similar. Re-derive the list; do not trust these numbers.)

## Part 1 — the type record, written fresh on `main`

`origin/the-little-wat` has `bbfac2ee8`, *"STONE(the-little-wat/type-record): the checker can say what type it gave
every node"*: under `WAT_CHECK_TYPES=1`, `wat --check <file>` also prints the type inference assigned to every node, by
position, with each inference root's **final** substitution applied. Unset, nothing new runs.

- **Read it, do not cherry-pick it:** `git show bbfac2ee8` (and its SCORE/brief on that branch, if any), read-only. Do
  not merge, check out or cherry-pick that branch. The builder's precedent (255.55): write it fresh against `main`,
  keeping the **same names and the same env variable and output shape**, so the eventual merge reconciles like with
  like.
- **Strictly additive:** unset, `--check` is byte-for-byte what it is today. Prove that with a test.
- Its own caveat (F-196): it cannot type inside a function body spelled with namespaced symbols, because of
  normalization order. The corpus still spells heads with keywords, so it works for this window. Record whether that
  holds on `main`.
- Tests: a driven row that runs `wat --check` with `WAT_CHECK_TYPES=1` on a small fixture and asserts the recorded
  types of named nodes (an empty `(:wat::core::PersistentVector)` passed to a `(Vector :- [i64])`-typed parameter
  records `i64` as its element).

## Part 2 — measure every untyped constructor call (SCORE only)

With the type record, for **every** untyped collection constructor call in the corpus (`wat/`, and every tracked
`*.wat` / `*.wat.bad` except `wat-scripts/fixes/**` and `*.wat.golden`), record:

1. the head, the file:line, and the element type(s) the checker inferred at that node;
2. its class: **concrete** (a closed type: the codemod can write `(wat.type/X :- [that] …)`); **generic** (a type
   parameter of the enclosing definition, e.g. `T`: the codemod writes `:- [T]`); **unresolved** (still a variable:
   a person must decide); **not checked** (the file does not check, or F-196 hides it);
3. counts per head and per class, and the full **unresolved** and **not checked** lists.

Also measure the two scalar constructors: how `(:wat::core::u8 65)` and `(:wat::core::char "a")` are registered and
dispatched today (intrinsic? a special arm?), and what making them `wat.type/u8` / `wat.type/char` constructors would
touch. Do not change them.

## Gates

| what | how | expected |
|---|---|---|
| release floor | `scripts/floor.sh`; read the Summary in `.floor/<stamp>/` | all passed; the count against 6213 at `ed5d7749b`, plus your new tests |
| clippy | `cargo clippy --release --all-targets -- -D warnings` | rc 0 |
| additive | `wat --check` output with the variable unset is unchanged | a test proves it |

## STOP triggers (checked against the work list: none fires on a site it orders changed)

- **STOP-1:** porting the type record needs a design change beyond what `bbfac2ee8` does on its branch (`main` has
  diverged too far to keep its names and output shape). Report the divergence and STOP.
- **STOP-2:** the floor is red. Do not re-run it. Copy the failing block verbatim from `.floor/<stamp>/`, name the arm,
  and STOP.
- A STOP means STOP.

## Doctrine

`holon/CLAUDE.md` binds you (codemods for `.wat`; the floor via `scripts/floor.sh`; no known flake). Capture `rc=$?` on
the next statement. Never wait with `pgrep -f`. If this brief contradicts the code, the code wins: say so. Write
`SCORE-STONE-255.68-the-type-record-and-untyped-ctors.md` beside this brief, commit it, **do not push**.
