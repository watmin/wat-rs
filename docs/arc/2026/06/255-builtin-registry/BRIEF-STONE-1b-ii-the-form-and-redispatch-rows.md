# BRIEF — STONE 1b-ii: the 6 Form + 2 Redispatch rete rows enter the registry

DESIGN: `docs/arc/2026/06/255-builtin-registry/DESIGN-STONE-1b-ii-the-form-and-redispatch-rows-have-no-teacher.md`

## The work, in one paragraph

Eight more `RETE_OPS` rows become `@alias` rows in the intrinsic registry — the six
`OpClass::Form` rows and the two `OpClass::Redispatch` rows whose `core_name` is already
registered. Same contract as the 29 that landed in Stone 1b-i (`src/intrinsic/special/
rete_alias.rs`): each is a doc-only unit struct declaring a name and a target and nothing else —
no handler, no `role = eval`, no `role = check`, and **none of the five closed-domain axes**,
which the registry derives from the target at fold time. Add them to the existing
`rete_alias.rs`, then update the ledger constants the ratchets name for you.

## ⛔ READ THIS BEFORE YOU AUTHOR A SINGLE TYPE — it is the whole stone

Stone 1b-i's brief told its rider to transcribe `@arg`/`@ret` from each row's own `params`/`ret`
in `RETE_OPS`. **That instruction is correct for 1b-i's rows and WRONG for all eight of yours.**

Every one of your eight rows reads `params: &[]` and `ret: ParamType::Bool`. Both fields are
**dead** for these two classes — `ReteOp`'s own field docs say so: *"`Alias`/`Fallback` only …
Empty for `Form`/`Redispatch`"* and *"unused for `Form`/`Redispatch`."* The `Bool` is what the
field was initialised to, not a claim about the verb. It is provably wrong for at least two of
yours: `:wat::core::List` returns a `List`, and `:wat::core::fn` returns a function.

★★★ **The only honest source of `@arg`/`@ret` for these eight is the TARGET's own registry
row.** Open it, read its `@arg` lines and its `@ret` line, and carry them across — including
whether it uses a rest arg (`name…`), because that is what makes the registry record the alias
as variadic rather than `Exact(N)`.

⚠ And know what is NOT protecting you. `doc_arg_ret_types_match_checker_scheme`
(`src/intrinsic/mod.rs:2254`) begins `match check_env.get(entry.name) { None => continue }`. In
1b-i every row had a scheme, so that gate caught and named every type mistake. **Your eight have
no scheme, so it skips them and verifies nothing.** No gate will catch an invented type here.
Copy from the target; do not reason one out.

## Your eight rows, and where each target's shape lives

| rete name (register this) | `@alias` target | read the target's row HERE |
|---|---|---|
| `:wat::rete::core::and` | `:wat::core::and` | `src/intrinsic/special/and_form.rs:48` |
| `:wat::rete::core::or` | `:wat::core::or` | `src/intrinsic/special/or_form.rs:48` |
| `:wat::rete::core::if` | `:wat::core::if` | `src/intrinsic/special/control_flow.rs:36` |
| `:wat::rete::core::let` | `:wat::core::let` | `src/intrinsic/special/binding.rs:31` |
| `:wat::rete::core::match` | `:wat::core::match` | `src/intrinsic/special/match_form.rs:41` |
| `:wat::rete::core::fn` | `:wat::core::fn` | `src/intrinsic/special/fn_form.rs:48` |
| `:wat::rete::core::List` | `:wat::core::List` | `src/intrinsic/list.rs:36` |
| `:wat::rete::holon::coincident?` | `:wat::holon::coincident?` | `src/intrinsic/holon/atom.rs:2331` |

Those `file:line`s are the `#[wat_…]` attribute; the `///` block you need sits directly above
it. `atom.rs` holds many verbs — read the row at that line, not its neighbours.

Confirm each row's class in `src/rete/vocabulary.rs` before writing it (lines 284, 460, 486,
495, 522, 546, 784, 1235 — verify, do not trust): every one must read `class: OpClass::Form` or
`class: OpClass::Redispatch`.

## Implementation sketch

Extend `src/intrinsic/special/rete_alias.rs` with two new sections after the existing families,
matching the file's own comment style:

```rust
// ─── core (Form — lazy / short-circuiting, mirrored by re-dispatch) ───────────────────────────
// ─── core · holon (Redispatch) ───────────────────────────────────────────────────────────────
```

Each row copies the shape already in that file — a short prose line, then `@added`, `@alias`,
the target's `@arg` lines, the target's `@ret`, and an `@example` that is a real call producing
the stated value. Struct names continue the file's convention: `ReteCoreAnd`, `ReteCoreIf`,
`ReteCoreList`, `ReteHolonCoincident`, and so on.

Also update the file's module header: it currently says "29" rows and "None of the 29 below is
`Fallback`". Make it 37, and keep the `Fallback` prohibition intact — it is load-bearing prose
that moved into that header when the witness file was folded away.

## Blast radius

`src/intrinsic/special/rete_alias.rs` (extend) · `src/intrinsic/mod.rs` (ledger constants only).
**`src/rete/vocabulary.rs` is READ-ONLY** — all 74 rows stay exactly as they are. No new module,
no change to `special/mod.rs`, no consumer rewired, nothing deleted.

## STOP triggers — halt and report, do not improvise

- **STOP-1.** A row is not `OpClass::Form` or `OpClass::Redispatch` in `RETE_OPS`. Report which.
- **STOP-2.** You cannot find a target's `@arg`/`@ret` at the `file:line` in the table above, or
  the target's row does not carry them. **Do not infer the type from the verb's name, from
  `RETE_OPS`, or from how the verb is called elsewhere** — report it and stop. This is the one
  place in the stone where a guess would ship unchecked.
- **STOP-3.** `FROZEN_CHECKER_DEBT_LEDGER` grows by anything other than exactly 8. The DESIGN
  derives +8 (all eight lack a `CheckEnv` scheme). A different number means a different
  population is being registered.
- **STOP-4.** A test outside the ledger ratchets goes red. Capture that test's entire stdout and
  stderr block verbatim from `.floor/latest/raw.log`, name the exact assertion that fired, and
  report — before re-running anything.
- **STOP-5.** You want to declare `@Purity`/`@Determinism`/`@Totality`/`@ExpandTime`/
  `@Category`. Stop: an alias inherits all five, and declaring one is a compile error.

## Verification, in this order

```bash
cargo build --release 2>&1 | tail -20
./scripts/floor.sh > /dev/null 2>&1; echo "EXIT=$?"
grep -E "^\s+Summary" .floor/latest/raw.log | tail -2
cargo clippy --all-targets -- -D warnings 2>&1 | tail -20
```

Expect the ledger ratchets to red on the first full run — that is them naming your edits. Two
kinds this time: GAP_B will report STALE names to **delete**, and DEBT will report NEW names to
**add**. Apply exactly what they name, re-run, repeat until the Summary is clean. Read the
Summary line, never a piped exit code.

## Acceptance — derived, not estimated

```
registry rows      515 → 523     +8 attribute sites
GAP_A               60 → 60      unchanged — none of the 8 is on it
GAP_B               78 → 71      7 of the 8 are on it; :wat::rete::core::List is on neither
                                 gap ledger, so it drains nothing and only pays DEBT
DEBT                95 → 103     ⬅ +8, all eight. The honest cost of this stone.
KNOWN_UNREVIEWED    20 → 20
floor        5127/5127 → 5127/5127   registering a row mints no `#[test]` fn
clippy                    0
```

Count registry rows with the pattern ANCHORED to the attribute site, never a bare substring —
a loose `wat_intrinsic("` search picks up prose placeholders (`<fqdn>`, `…`, `:wat::holon::…`)
and over-reports:

```bash
grep -rhoP '^\s*#\[wat_(special_form|intrinsic)\("\K[^"]+' src/ --include=*.rs | sort -u | wc -l
```

## Working rules

Everything foreground. You may not spawn sub-agents. No worktrees, no `git stash`, no
`git revert`, no commit, no push — leave the tree dirty and report; the orchestrator commits.
**"I cannot tell" is a correct and welcome outcome** — given that no gate checks your types,
reporting an uncertainty is worth far more here than a plausible guess.

Shape to copy: `src/intrinsic/special/rete_alias.rs` as it stands, and
`BRIEF-STONE-1b-i-the-alias-surface.md` with its DESIGN.
