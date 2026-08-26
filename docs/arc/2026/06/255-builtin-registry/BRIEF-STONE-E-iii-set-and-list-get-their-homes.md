# STONE E-iii — set + list get their homes: `:wat::hashset::` and `:wat::linkedlist::`

DRAWN + BRIEFED 2026-08-26 against `0d303f780`.
PRIOR ART — **read E-ii's commit message first** (`0d303f780`); its two-renderings finding is why
this brief has a census step the earlier ones lacked. Chain: A-i · A-ii · B-i · B-ii · C · D ·
E-i `110335bd5` · E-ii `0d303f780`.

## The move — and BOTH families take a MARKED name

```
:wat::core::HashSet/*  ->  :wat::hashset::*    conj · contains? · empty? · length         (4)
:wat::core::List/*     ->  :wat::linkedlist::*      conj · contains? · empty? · get · length   (5)
```

**Neither may take the unmarked name, and this is measured, not taste:**

```
HashSet  Arc<HashSet<Value>>                        copy-on-write
List     Arc<std::collections::LinkedList<Value>>   copy-on-write   (value.rs:340)
```

Both are the **copy-on-write** flavor — the same side of the axis as `HashMap` and `Vector`, not the
structurally-shared side (`rpds`) that `PersistentMap`/`PersistentVector` sit on. The builder has
ruled that persistent list and set are coming (*"we'll just make the collections be backed by
persistent structs (including the ones we don't have now list.. set...)"*). **So `:wat::set::` and
`:wat::list::` must both stay FREE for the flavors that will become the default.** Squatting either
guarantees a second migration of the very family that ends up unmarked.

`linkedlist` names what it is (SPELLED OUT: the house pattern is `hash`+`map`, `hash`+`set`,
so it is `linked`+`list`; an elided `llist` communicates only to a reader who already knows what the
first `l` stands for, which fails Obvious before anything else is weighed) — a `LinkedList` — the same way `hashset` names a `HashSet`. Both marked
spellings are provisional; when the builder rules the marker, changing them is ONE prefix rename,
which is the entire point of keeping the unmarked slot empty.

⚠ **`:wat::core::List` — the bare CONSTRUCTOR — does NOT move.** `src/intrinsic/list.rs` registers it
already; it is the type's constructor, arc 251's territory. Only the slash-verbs move, and the
trailing `/` is the whole discrimination. **STOP-3.**

## The ground — measured at HEAD, and TWO of these differ from E-i/E-ii

```
colon-form files          23   (9 .rs · 14 .wat)
dotted-form files          0   ← CHECKED. E-ii's red came from a golden holding `:wat.core/Vector/length`
wat/core.wat               0   ← NO BOOTSTRAP HAZARD. E-i and E-ii both had one; this stone does not.
negative fixtures at risk  0   ← the Stone C pre-check, run and clear
rete/vocabulary.rs         4      rete/expr_ir.rs   1      macros/eval.rs   4
```

## ★ CENSUS BOTH RENDERINGS — the finding E-ii paid for

The same name has two spellings and a census in one is blind to the other:

```
:wat::core::List/conj     the COLON form
:wat.core/List/conj       the DOTTED form — what EDN goldens hold
```

E-ii's floor red was a golden holding the dotted form, invisible to every census in that stone —
**and the sweep afterwards found E-i had shipped past the same rendering.** For this stone the dotted
count is **0 today**, which I verified. **Re-run both after the work**; the arc migrates toward the
dotted form, so this population grows:

```bash
git grep -lE ':wat::core::(HashSet|List)[:/]' -- ':!docs' | sed 's/.*\.//' | sort | uniq -c
git grep -lE ':wat\.core/(HashSet|List)/'     -- ':!docs' | sed 's/.*\.//' | sort | uniq -c
```

## ★★ THE CONSUMER I HAVE NOW MISSED TWICE — it is in this brief

`src/rete/expr_ir.rs`'s `OpExec::of` matches `row.core_name` **literally** — a fast-path opcode
compiler, entirely separate from the naming-rule table. E-i's brief missed it; E-i's commit message
named it as missed; **I left it out of E-ii's brief anyway.** One arm here.

And the rete naming invariant fires as always — `rete_name == core_name` with `::rete::` spliced
after `:wat::`, enforced by a test, 4 rows. `RETE_MODULES` will need `:wat::rete::hashset::` and
`:wat::rete::linkedlist::`. Expect `tests/rete/datamancer.rete.edn` to need regeneration (its `:abi`
hashes every `rete_name`); regenerate via its documented command and diff to confirm only `:abi`
changed. **Do NOT add a `NAMING_RULE_EXCEPTIONS` entry — STOP-1.**

⚠ `macros/eval.rs`'s `is_pure_total` F5 allow-list has 4 entries. In E-ii the bootstrap itself used a
migrating verb inside a `defmacro` body, and omitting dual admission during the migration window made
**every program** fail to load. `wat/core.wat` is clear here, but check `wat/*.wat` before assuming.

## Your role

cwd `/home/john/work/holon/wat-rs`; `pwd` first. **Ending your turn ENDS you** — every command
FOREGROUND. **No sub-agents. No git worktrees.**

⛔ **DO NOT RUN `git stash` IN ANY FORM.** Not `push`, not `-u`, not `pop`, not `list`. This is a rule
about the COMMAND, not about a particular entry: `stash@{0}` holds irreplaceable work, and a LIFO
push/pop *looks* safe right up until one mistake destroys it. If you want a before/after comparison,
copy files to `/tmp` and diff there. (E-ii's rider pushed and popped its own stash, reported it
honestly, and no harm came — but the rule it read said "never touch `stash@{0}`", which a correct
LIFO argument slips past. This wording does not.)

Also: do not commit, push, revert, or `git checkout`.

You may run `cargo build --release`, `cargo build --release --all-targets`,
`./target/release/wat --check|--grep <f>`, `./target/release/wat <f>`, and targeted
`cargo nextest run --release -E '<filter>'`. **Not** the floor, **not** clippy.
⚠ **`cargo test` is not a diagnostic** — it runs N tests in one process, which
`src/host/test_runner.rs:48-55` documents as unsupported. Use `cargo nextest -E` instead.

**Structure:** `src/intrinsic/hashset.rs` + `src/intrinsic/linkedlist.rs`, thin `#[wat_intrinsic]` shims
copying `src/intrinsic/vector.rs` (E-ii's, closest prior art). Algorithms stay in `src/collection/`.
Non-pure-det verbs need `@example-norun`; these are all pure∧det, so `@example` is right — but
confirm rather than assume.

**Corpus by wat-fix RULES codemod** — slash-form, so copy
`wat-scripts/fixes/rename-core-vectors-to-their-homes.wat`. Population from **wat-grep, not text**.
Dry-run on /tmp, read the diff, apply, prove idempotence.

**Verbs come from the DISPATCH TABLE, not corpus usage.** E-ii's brief listed 5 and 6 because I
censused what `.wat` calls; the truth was 6 and 7 because `empty?` has zero per-type call sites. A
verb that exists but is never called still needs a home, or it strands at retirement.

## STOP triggers — each rejects

1. **STOP-1 — the pairing gate needs a `NAMING_RULE_EXCEPTIONS` entry.**
2. **STOP-2 — a retirement row does not fire.** Prove the check-time error; name the door.
3. **STOP-3 — you would move `:wat::core::List`**, the bare constructor. Only slash-verbs move.
4. **STOP-4 — a room's line number does not hold.** Written against `0d303f780`.

## Acceptance — every row measures a MECHANISM, every bar is satisfiable

```bash
# 1. all 9 verbs RUN under the new spellings — a scratch-pad probe asserting a result for each.
./target/release/wat wat-scripts/scratch-pad/<probe>.wat; echo "EXIT=$?"        # 0

# 2. old spellings are CHECK errors naming their replacements (one per family).
./target/release/wat --check /tmp/old-set.wat;  echo "EXIT=$?"                  # non-zero + remedy
./target/release/wat --check /tmp/old-list.wat; echo "EXIT=$?"                  # non-zero + remedy

# 3. the pairing gate.
cargo nextest run --release -E 'test(naming_rule_tests)'

# 4. bootstrap loads, via an ORDINARY program (never `wat --check wat/core.wat` — unsatisfiable).
./target/release/wat --check /tmp/trivial.wat; echo "EXIT=$?"                   # 0
cargo nextest run --release -E 'test(every_wat_scripts_file_loads_on_the_current_runtime)'

# 5. BOTH renderings, all extensions, AFTER the work — classify every survivor.
# 6. the bare constructor did not move — boundary-anchored, before AND after, prove equality.
git grep -oE ':wat::core::List(::|/)?' -- ':!docs' | grep -vE '(::|/)$' | wc -l

cargo build --release && cargo build --release --all-targets
```

## Report back with

- Each command's actual output, naming the command that produced each number.
- **Both renderings' post-work census**, with every survivor classified.
- Everything the rete invariant forced — rows, `RETE_MODULES`, any regenerated artifact and its diff.
- Which door produced the retirement error.
- The wat-grep population vs any text count, and the difference explained.
- The cascade's waterfall, per phase.
- Anything the brief got wrong. What you did NOT do, and why.
