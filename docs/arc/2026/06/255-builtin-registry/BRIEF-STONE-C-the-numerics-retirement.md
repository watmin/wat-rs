# STONE C — the numerics retirement: the old spellings die

DRAWN + BRIEFED 2026-08-26 against `870d59898`.
DESIGN: `DESIGN-STONE-the-numerics-get-their-homes.md`. A-i `b2d10158f` · A-ii `1333e90d0` ·
B-i `ae2330bc1` · B-ii `870d59898`.

## ⛔ THE DESIGN SAID STONE C ALSO NARROWS THE BLANKET-ACCEPT. IT DOES NOT — MEASURED.

I imposed the narrowing as a throwaway probe against this exact tree — `is_resolvable_call_head`
default-denying under `:wat::` unless `sym`, the registry, a unit variant, or a macro knows the head
— built it, and ran the floor:

```
     Summary  5059 tests run: 2520 passed, 2539 FAILED, 19 skipped
```

**Half the floor.** A wholesale default-deny on `:wat::` is not a stone; it is the arc's endgame, and
it needs the registry far more complete than 189 names. The probe is reverted; the number is the
deliverable.

**Consequence — and it corrects my own amendment of yesterday:** the inherited gate
`tests/wat_lang/probe_undefined_builtin_resolves.rs` (two `#[ignore]`s, *"unlock when we circle back
to arc 255"*) **CANNOT be un-ignored by this stone.** Its first test is
`(:wat::core::i64::+'2 1 2)` — a *bogus leaf under a real namespace* — and only the resolver
narrowing rejects that. I wrote "Stone C is done when those two ignores are deleted." That was
wrong. **Leave them ignored, and do not touch that file.**

## What this stone IS

The old per-type numeric spellings stop working. `:wat::core::i64::+` becomes a **check-time error
naming its replacement**, instead of a name that silently resolves and dies at runtime.

```
36 retirement rows          :wat::core::{i64,f64}::*  ->  :wat::{i64,f64}::*
delete the old machinery    both spellings have lived since Stone A; the old half now goes
```

## The rooms — measured at `870d59898`

```
.rs   554 occurrences        .wat  170 occurrences
  src/runtime.rs        248     the two dual-spelling probes         40   ← retire their old halves
  src/check.rs           52     wat-scripts/fixes/…per-type-spelling 17   ← another codemod's RULE
  src/rete/purity.rs     48     wat-tests/holon/eval-coincident      14      literals — DO NOT TOUCH
  src/macros/eval.rs     36     wat-tests/core/core-threading         6
  src/rete/expr_ir.rs    26     wat-scripts/grep/core-numerics-ops    6   ← my census's own rule
  src/intrinsic/{f64,i64}.rs 38
  src/resolve/mod.rs     13
  src/freeze.rs          10
```

**Classify every one before deleting it.** Four buckets, and only the first is deletable:
- **A — live machinery for the old spelling** (dispatch arms, the old halves of the four lists,
  scheme registrations): DELETE.
- **B — prose describing the old spelling as current**: UPDATE to the new one.
- **C — history** (a comment recording the retirement, an arc reference): KEEP. `docs/arc/**` never moves.
- **D — another program's DATA** (a codemod's rule literals, a grep's `starts-with?` argument):
  **KEEP.** Rewriting `rete-where-per-type-spelling.wat`'s 17 literals would repoint a recorded
  migration at a name it was never written to find.

## The five things that retire WITH the old names

1. **`fold_numeric_home`** (`src/runtime.rs`, above `dispatch_substrate_impl`) — its entire purpose
   was the transition. Its two callers (`dispatch_substrate_impl`, `expr_ir.rs`'s `OpExec::of`) fold
   the NEW spelling onto the OLD; when the old is gone, the arms are rewritten to the new spelling
   and the fold is deleted. **Read its doc comment** — it lives outside `dispatch_substrate_impl`
   because the rete purity gate censuses that body for `":wat::…"` literals.
2. **The `register_builtins` derivation** (`src/check.rs`) — it aliases new→old. When the old
   schemes go, the new spelling's scheme must be registered DIRECTLY and the loop deleted.
   ⚠ `max-of`/`min-of` already have their own variadic scheme registered beside it; keep that.
3. **The old halves of four hand-lists** — `is_pure_total` (macros/eval.rs), `pure_det` and `total`
   (rete/purity.rs), the `check.rs` scheme arrays. Each currently carries both spellings.
4. **The two dual-spelling probes** in `wat-scripts/scratch-pad/` — they exist to prove BOTH work.
   Rewrite each to assert the surviving spelling only; do not delete them (the loader gate walks
   that directory and they are the live proof the verbs work at all).
5. **The converters' hardcoded names** — `eval_f64_round` (`const OP: &str = ":wat::core::f64::round"`),
   `eval_f64_clamp`, `eval_f64_to_i64`, and i64's converters. Today they name the OLD spelling in
   their errors, which was harmless while both lived. **After this stone they name a name that no
   longer exists.** Make `op` a parameter, as the arithmetic ops already do.

## Your role

cwd `/home/john/work/holon/wat-rs`; run `pwd` first. **Ending your turn ENDS you** — every command
FOREGROUND, blocking. **You may not spawn sub-agents.** Do not commit, push, stash, revert, or
`git checkout`; `git stash@{0}` must never be touched.

You may run `cargo build --release`, `cargo build --release --all-targets`,
`./target/release/wat --check|--grep <f>`, `./target/release/wat <f>`, and single named tests.
**Not** the floor, **not** clippy.

**EXPECT A LARGE CASCADE.** Deleting a spelling that 554 `.rs` sites mention will go red widely. The
fail-count is the progress meter; each error names the next site; watch it waterfall. Do not stash,
do not revert, do not panic at the count.

## STOP triggers — each rejects

1. **STOP-1 — a retirement row does not fire.** Adding a row is not the same as the checker
   consulting it. Prove a retired spelling produces a CHECK-time error naming its replacement, and
   say which door produced it. (A prior stone shipped 14 rows that were inert.)
2. **STOP-2 — you need to edit `tests/wat_lang/probe_undefined_builtin_resolves.rs`.** Its unlock
   condition is the resolver narrowing, which this stone does not do. Leave the ignores.
3. **STOP-3 — you need to rewrite another program's rule literals** (bucket D) to reach a count.
   Report the count instead; the number is wrong, not the record.
4. **STOP-4 — a room's line number does not hold.** Written against `870d59898`.

## Acceptance — every row derives its bar

```bash
# 1. the old spelling is a CHECK error naming its replacement — the point of the stone.
printf '(:wat::core::defn :user::main [] -> :wat::core::nil\n  (:wat::core::let [_a (:wat::kernel::println (:wat::core::i64::+ 2 3))] nil))\n' > /tmp/c.wat
./target/release/wat --check /tmp/c.wat; echo "EXIT=$?"     # non-zero, remedy names :wat::i64::+

# 2. the new spelling still works.
printf '(:wat::core::defn :user::main [] -> :wat::core::nil\n  (:wat::core::let [_a (:wat::kernel::println (:wat::i64::+ 2 3))] nil))\n' > /tmp/n.wat
./target/release/wat /tmp/n.wat; echo "EXIT=$?"             # 0, prints 5

# 3. the fold is gone. BAR: 0.
git grep -c fold_numeric_home -- src/ | wc -l

# 4. what remains of the old spelling, and every one classified B/C/D in your report.
git grep -cE ':wat::(rete::)?core::(i64|f64)::' -- '*.rs' '*.wat' ':!docs'

# 5. the corpus still loads.
cargo test --release --test lint every_wat_scripts_file_loads_on_the_current_runtime
cargo build --release && cargo build --release --all-targets
```

## Report back with

- Each command's actual output, naming the command that produced each number.
- **Which door produced the retirement error**, and the diagnostic's full text.
- **Your four-bucket classification** of every surviving occurrence, with `file:line` — this is the
  row I will read most closely, because bucket D is where a careless count corrupts a recorded migration.
- The cascade's waterfall.
- Anything the brief got wrong. What you did NOT do, and why.
