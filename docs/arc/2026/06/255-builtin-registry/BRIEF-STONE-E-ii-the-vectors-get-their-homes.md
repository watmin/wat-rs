# STONE E-ii — the vectors get their homes: `:wat::vector::` and `:wat::vec::`

DRAWN + BRIEFED 2026-08-26 against `1b7a32bd3`.
PRIOR ART — **read E-i's commit message first** (`110335bd5`); this brief exists because of what it
cost. Chain: A-i `b2d10158f` · A-ii `1333e90d0` · B-i `ae2330bc1` · B-ii `870d59898` ·
C `11b85591e` · D `1a3f6a703` · E-i `110335bd5`.

## The move

```
:wat::core::PersistentVector/*  ->  :wat::vector::*    215 sites   ← UNMARKED
:wat::core::Vector/*            ->  :wat::vec::*        71 sites   ← marked
PersistentVector verbs: concat · conj · contains? · get · length
Vector verbs:           concat · conj · contains? · get · length · extend
```

⚠ **`extend` exists only on `Vector`.** The two verb sets are NOT identical here (unlike the maps).
Do not assume symmetry; register what each family actually has.

**Both flavors survive** — the builder ruled they co-exist. Persistent takes the UNMARKED name so
that when the backend swap lands it **never moves again**; only `:wat::vec::` moves, once, as a
prefix rename. `:wat::set::` and `:wat::list::` remain unclaimed (E-iii).

## ⛔⛔ THE THING E-i's BRIEF OMITTED, AND IT COST THE MOST

`src/rete/vocabulary.rs` enforces an invariant, tested:

```rust
rete_name == core_name.replacen(":wat::", ":wat::rete::", 1)     // "one rule, no exceptions"
```

**I discovered this in B-ii, wrote a brief around it, and then left it out of E-i's.** It rippled
into `RETE_MODULES`, two corpus consumers, and a checked-in compiled artifact whose baked ABI hash
had to be regenerated. **It fires again here — measured: `vocabulary.rs` has 12 vector occurrences,
`expr_ir.rs` has 4.** So:

1. Each vector row's `core_name` moves to the new spelling; `rete_name` **follows by the rule**, and
   the gate proves the pairing. **Do NOT add a `NAMING_RULE_EXCEPTIONS` entry** — that is silencing
   the instrument. **STOP-1.**
2. `RETE_MODULES` will need the new prefixes admitted (`:wat::rete::vector::`, `:wat::rete::vec::`).
   It already carries `:wat::rete::map::` from E-i — same move, one line each.
3. Expect a checked-in artifact to need regeneration (E-i: `tests/rete/datamancer.rete.edn`, whose
   `:abi` hashes every `rete_name`). Regenerate via its documented command; diff to confirm **only**
   the `:abi` line changed.

### ⚠ AND THE ROWS ARE NOT ALL VERBS

`vocabulary.rs` carries rows whose `core_name` is the **bare TYPE** — `":wat::core::PersistentVector"`
and `":wat::core::Vector"` at `:757` and `:766`, no slash. **Those do NOT move**: the bare type is arc
251's `wat.type/`, and the trailing `/` is the entire discrimination. Move the slash-verb rows only.

## The ground — measured at HEAD

```
.wat 82 files      .rs 12 files      bootstrap: wat/core.wat has 1 occurrence
negative fixtures in executable position: NONE
   (the only .bad hit is docs/arc/2026/05/130-…/substrate.wat.bad — history, out of scope)
```

Two extensions only — but **re-run the all-extension census at the END**; a count taken before the
work is not a count taken after, and `.jsonl` was invisible to a two-extension bar in Stone C while
`.edn` was a false positive of a loose pattern in E-i's.

## Phase order — `wat/core.wat` again

```
PHASE 1   register.  BOTH SPELLINGS LIVE.  Nothing moves.
PHASE 2   corpus moves by codemod.  Both still work.
PHASE 3   retire.  Delete the old machinery.
```

Verify an **ORDINARY program** still `--check`s clean at each phase boundary. ⚠ Do **not** use
`wat --check wat/core.wat` — it fails at baseline for an unrelated reason (`include_str!` + `Stdlib`
privilege; as an entry file it parses twice and collides with itself). Stone D shipped that as an
acceptance row and it was **unsatisfiable by construction**.

## Your role

cwd `/home/john/work/holon/wat-rs`; `pwd` first. **Ending your turn ENDS you** — every command
FOREGROUND. **No sub-agents. No git worktrees.** Do not commit, push, stash, revert, or
`git checkout`; `git stash@{0}` is untouchable.

You may run `cargo build --release`, `cargo build --release --all-targets`,
`./target/release/wat --check|--grep <f>`, `./target/release/wat <f>`, and single named tests.
**Not** the floor, **not** clippy.

⚠ **`cargo test` is not a diagnostic here.** The floor is `cargo nextest`. A red from `cargo test`
is information about `cargo test` — it runs N tests in one process, which
`src/host/test_runner.rs:48-55` documents as unsupported. If you reach for it, use
`cargo nextest run --release -E '<filter>'` instead.

**Structure:** registry homes `src/intrinsic/vector.rs` + `src/intrinsic/vec.rs` — thin
`#[wat_intrinsic]` shims with `///` preambles, copying `src/intrinsic/map.rs` (E-i's, closest prior
art). Algorithms stay in `src/collection/`. ⚠ Non-pure-deterministic verbs need `@example-norun`,
not `@example` — a gate enforces it.

**Corpus by wat-fix RULES codemod** — these are **slash-form**, so copy the slash rule from
`wat-scripts/fixes/rename-core-maps-to-their-homes.wat` (E-i's, exact shape). Population from
**wat-grep, not text**. Dry-run on /tmp, read the diff, then apply, then prove idempotence.

## STOP triggers — each rejects

1. **STOP-1 — the pairing gate needs a `NAMING_RULE_EXCEPTIONS` entry.** Report the row; the gate is
   the invariant, not the obstacle.
2. **STOP-2 — a retirement row does not fire.** Prove the check-time error and name the door.
3. **STOP-3 — you would move a bare-TYPE row** (`":wat::core::Vector"`, no slash). You would not.
4. **STOP-4 — a room's line number does not hold.** Written against `1b7a32bd3`.

## Acceptance — every row measures a MECHANISM, every bar is satisfiable

```bash
# 1. the new names RUN — a scratch-pad probe asserting a result for every verb of BOTH families
#    (5 + 6 = 11), under the new spellings.
./target/release/wat wat-scripts/scratch-pad/<probe>.wat; echo "EXIT=$?"     # 0

# 2. old spellings are CHECK errors naming their replacements.
./target/release/wat --check /tmp/old-vec.wat; echo "EXIT=$?"                # non-zero + remedy

# 3. the pairing gate — the invariant that makes the rete half correct.
cargo nextest run --release -E 'test(naming_rule_tests)'

# 4. bootstrap loads, via an ORDINARY program.
./target/release/wat --check /tmp/trivial.wat; echo "EXIT=$?"                # 0
cargo nextest run --release -E 'test(every_wat_scripts_file_loads_on_the_current_runtime)'

# 5. all-extension census AFTER the work; classify every survivor.
git grep -lE ':wat::core::(PersistentVector|Vector)[:/]' -- ':!docs' | sed 's/.*\.//' | sort | uniq -c

# 6. the bare TYPE did not move — boundary-anchored, before AND after, prove equality.
git grep -oE ':wat::core::(PersistentVector|Vector)(::|/)?' -- ':!docs' | grep -vE '(::|/)$' | wc -l

cargo build --release && cargo build --release --all-targets
```

## Report back with

- Each command's actual output, naming the command that produced each number.
- **Everything the rete invariant forced** — rows, `RETE_MODULES`, any regenerated artifact and its diff.
- Which door produced the retirement error.
- The wat-grep population vs any text count, and the difference explained.
- The /tmp dry-run: files, hunks, confirmation every hunk is a prefix rewrite.
- The cascade's waterfall, per phase.
- Anything the brief got wrong. What you did NOT do, and why.
