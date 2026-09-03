# BRIEF — STONE 1b-i: the 28 rete Alias rows enter the registry

DESIGN: `docs/arc/2026/06/255-builtin-registry/DESIGN-STONE-1b-i-the-alias-surface-and-why-1b-is-not-one-stone.md`

## The work, in one paragraph

`src/rete/vocabulary.rs`'s `RETE_OPS` table holds 35 `OpClass::Alias` rows. One
(`:wat::rete::i64::>`) is already in the intrinsic registry as arc 255 Stone 2a's witness; six
point at `:wat::core::=`/`not=`, which have no registry row yet and are out of scope. **Add the
remaining 28 to the intrinsic registry as doc-only `@alias` rows**, then update the three ledger
constants the ratchets will name for you. An alias declares a name and a target and nothing
else: no handler, no `role = eval`, no `role = check`, and **none of the five axes** — the
registry derives those from the target at fold time (Stone 2a-b), and declaring one is a
`DocError::AliasDeclaresAxis` compile error.

## Read in order

1. **`src/intrinsic/special/rete_i64_gt_alias.rs`** — the template. Copy its `///` block shape
   exactly: `@added`, `@alias`, `@arg`×N, `@ret`, `@example`, then
   `#[wat_special_form(":<fqdn>")] pub(crate) struct <Name>;`. Its module header also explains
   why `OpClass::Fallback` rows may never be aliased — that is the boundary of this stone.
2. **`src/rete/vocabulary.rs`** — the source of truth for all 28. Each row gives you
   `rete_name` (the FQDN you register), `core_name` (your `@alias` target), `params` (your
   `@arg` types, in order) and `ret` (your `@ret` type). **Read each row; do not transcribe from
   this brief** — the brief lists line numbers so you can find them, not types you should trust:

   ```
    294 i64::<        306 i64::>=      318 i64::<=      335 f64::>       344 f64::<
    353 f64::>=       362 f64::<=      443 core::not    623 string::concat
    632 string::starts-with?           641 string::ends-with?
    650 string::contains?              664 string::empty?
    673 string::length  682 string::trim  691 string::to-lowercase
    700 i64::to-f64   722 vector::length  811 vector::contains?
    847 map::contains-key?             1006 i64::=      1015 i64::not=
   1024 f64::=       1033 f64::not=   1242 holon::presence?
   1261 i64::to-string  1270 f64::to-string  1279 core::bool::to-string
   ```
   Three of them carry `type_params` and parametric `ParamType`s (`vector::length`,
   `vector::contains?`, `map::contains-key?`) — those are the ones whose `@arg` spelling needs
   care, and the gate in step 5 is what tells you when you have it right.
3. **`src/intrinsic/special/mod.rs`** — add one `pub(crate) mod` line, alphabetically placed.
4. **`src/intrinsic/mod.rs`** — the three ledger constants you will edit:
   `FROZEN_CHECKER_DEBT_LEDGER` @923 · `REGISTRY_MEMBERSHIP_GAP_A` @1405 ·
   `REGISTRY_MEMBERSHIP_GAP_B` @1719.
5. **`src/intrinsic/mod.rs:2254`, `doc_arg_ret_types_match_checker_scheme`** — read it before
   you author the types. All 28 of these names already have a `TypeScheme` in `CheckEnv`
   (`register_builtins` builds one from each `RETE_OPS` row), so this gate **actively compares
   your `@arg` and `@ret` strings against it** and reds with both spellings side by side. It is
   your teacher for the parametric rows: author, run, read the message, correct.

## Implementation sketch

One new file, `src/intrinsic/special/rete_alias.rs`, holding all 28 structs grouped by family
(`i64` · `f64` · `string` · `vector` · `map` · `core` · `holon`) with a short `//` section
comment per family. One module doc header naming the DESIGN and stating the contract in a
sentence. Each row:

```rust
/// Alias for `:wat::i64::<` — "this name means that name." Calling
/// `(:wat::rete::i64::< a b)` dispatches through the registry's `alias_of` field
/// straight to `:wat::i64::<`; no separate implementation exists at this name.
///
/// @added 1.0.0
/// @alias :wat::i64::<
/// @arg a :wat::core::i64 the left operand
/// @arg b :wat::core::i64 the right operand
/// @ret :wat::core::bool whether `a` is strictly less than `b` — the target's answer, unchanged
/// @example (:wat::rete::i64::< 1 2) #=> true
#[wat_special_form(":wat::rete::i64::<")]
pub(crate) struct ReteI64Lt;
```

Struct names: `Rete` + the family + the verb, CamelCase (`ReteI64Lt`, `ReteStringTrim`,
`ReteMapContainsKey`). Every `@example` must be a real call that produces the stated value.

## Blast radius

`src/intrinsic/special/rete_alias.rs` (new) · `src/intrinsic/special/mod.rs` (one line) ·
`src/intrinsic/mod.rs` (three ledger constants only). **`src/rete/vocabulary.rs` is READ-ONLY
for this stone** — `RETE_OPS` keeps all 74 rows exactly as they are; this stone makes the
registry able to answer, it does not make any consumer stop asking `RETE_OPS`.

## STOP triggers — halt and report, do not improvise

- **STOP-1.** A row you are about to write is not `class: OpClass::Alias` in `RETE_OPS`. Report
  which and stop. `Fallback` in particular: aliasing one makes its 4-arg `:undefined` form
  unreachable and breaks live rete tests.
- **STOP-2.** A `@alias` target has no registry row (the floor says `DANGLING @alias`). The six
  `:wat::core::=`/`not=` rows are the known cases and are out of scope; if a SEVENTH appears,
  report it — the DESIGN's census would then be wrong and I need to know.
- **STOP-3.** `FROZEN_CHECKER_DEBT_LEDGER` gains any name. The DESIGN predicts DEBT stays at
  exactly 95; a rise means a row is registered that has no `CheckEnv` scheme, i.e. something
  that is not one of these 28. Report the names.
- **STOP-4.** A test outside the three ledger ratchets goes red. Capture the failing test's
  entire stdout+stderr block verbatim from `.floor/latest/raw.log`, name the exact assertion
  that fired, and report — before re-running anything.
- **STOP-5.** You find yourself wanting to declare `@Purity`/`@Determinism`/`@Totality`/
  `@ExpandTime`/`@Category` on any row. Stop: an alias inherits all five. If the compiler
  demands one, the row is not being parsed as an alias and I need to know why.

## Verification, in this order

```bash
cargo build --release 2>&1 | tail -20
./scripts/floor.sh > /dev/null 2>&1; echo "EXIT=$?"
grep -E "^\s+Summary" .floor/latest/raw.log | tail -2
cargo clippy --all-targets -- -D warnings 2>&1 | tail -20
```

Expect the ledger ratchets to red on the first full run — that is them naming your edits.
Their messages tell you exactly which constant to add to or delete from. Apply, re-run, repeat
until the Summary is clean. Read the Summary line, never a piped exit code.

## Acceptance — derived, not expected

```
registry rows     490 → 518      (+28)
GAP_A              88 → 60       (−28; all 28 are on it)
GAP_B             106 → 78       (−28; all 28 are on it)
DEBT               95 → 95       ⬅ UNCHANGED. The sharpest row: any mis-transcribed
                                    @arg/@ret surfaces here by name.
KNOWN_UNREVIEWED   20 → 20       an alias declares no Totality
floor        5127/5127 → 5155/5155,  0 FAIL
clippy                    0 under `-D warnings --all-targets`
```

## Working rules

Everything foreground. No sub-agents. No worktrees, no `git stash`, no `git revert`, no commit
and no push — leave the tree dirty and report; the orchestrator commits. If something is
genuinely undecidable from the disk, **"I cannot tell" is a correct and welcome outcome** —
report what you measured and what remains open rather than choosing a plausible guess.

A prior comparable result to copy for shape: `BRIEF-STONE-2a-the-alias-field.md` and the row it
produced, `src/intrinsic/special/rete_i64_gt_alias.rs`.
