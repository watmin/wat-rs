# BRIEF — STONE ⑤-F+G: THE BLANKET DIES

**Read `DESIGN-STONE-5fg-the-blanket-dies.md` first.** Arc 255's founding sentence. Everything ahead
of it has landed; the corpus is already clean. **No new declarations. Seven expectations move, and
two of them are the proof.**

## ① THE DELETION — `src/resolve/walk.rs`, `is_resolvable_call_head`

```rust
-  if is_reserved_prefix(head) {
-      return true;
-  }
+  if crate::intrinsic::registry().contains(head) {
+      return true;
+  }
+  if head.starts_with(":rust::") {
+      return true;
+  }
```

Delete the now-unused `use super::reserved::is_reserved_prefix;` import.

Three properties, each load-bearing — the design has the measurements:

- **No prefix is consulted for `:wat::*`.** Every registry name is reserved-prefixed, so gating on
  the prefix would NARROW the blanket, not delete it, and leave a prefix list in front of the
  registry.
- **The rung FALLS THROUGH on a miss.** It must not `return false`. Keeping the early return while
  swapping the value severs `sym.get`, unit variants, macros and surface methods below it — measured
  at 600 of 845 files refusing instead of 97.
- **`:rust::*` defers to its own authority.** A rust path is legal iff a `use!` declaration covers
  it, and `check_form` asks `UseDeclarations::covers` on the very next lines. This predicate has no
  `use_decls`; answering here would duplicate the question and answer it worse.

★ The full diff is preserved at commit **`d79597b3a`** (reverted, kept). Use it; do not retype it.

## ② TWO UNIT TESTS WHOSE SUBJECT IS THE BLANKET — `src/resolve/mod.rs:177` and `:186`

Their comment is the blanket's own specification:

> ```rust
> // These aren't implemented yet but shouldn't fail resolution —
> // they're under reserved prefixes that the spec carves out.
> ```

⛔ **Do not "fix" these by swapping in names that happen to resolve.** Rewrite them to assert the
rule that now holds — **a reserved-prefix name resolves IFF the registry knows it** — which is this
arc's founding sentence as a unit test. Measured, in the BARE environment these tests use:

```
:wat::kernel::send             registered intrinsic           → resolves
:wat::config::dim-count        registered intrinsic + scheme  → resolves
:wat::holon::Subtract          a stdlib MACRO — resolves at FULL load via `macros`, NOT in a
                               bare env. Its corpus file wat-tests/holon/Subtract.wat --checks
                               CLEAN with the deletion.
:wat::config::set-dim-count!   REAL but unregistered — a live arm at src/config.rs:441. It never
                               reaches the resolver in practice: `collect_entry_file`
                               (freeze.rs:1218, step 2) consumes leading config setters BEFORE
                               resolve at step 7, which is why the corpus census is ZERO for it.
```

So each test needs a POSITIVE (a registered name resolves) and a NEGATIVE (an unregistered
reserved-prefix name does not). Name them for what they now measure. ⛔ Do NOT declare
`set-dim-count!` — the config family is its own stone and the floor does not need it.

## ③ TWO REMEDY TESTS — `tests/diagnostics/probe_arc241_stone10_remedy.rs` c04, c08

c04 feeds `:wat::core::xyzzy` and expects `"<startup succeeded — no error to display>"`. **The
blanket made a nonsense name type-check clean and the test froze that as its expectation.** It is
now refused with no "did you mean", which is still exactly what the test claims.

Update the expected values to the new output. ⛔ **STOP if the new error carries a remedy** — c04's
and c08's claim is that a DISTANT unknown gets no suggestion, and a remedy appearing would be a
finding about the threshold, not something to assert away.

## ④ ONE NEGATIVE FIXTURE — `tests/services/probe_arc209_c0b3bc_post_spawn.rs:78`

`ProcessLaunch/bogus-field` was caught by the CHECKER; `resolve` now catches it first. Refused
either way, one pass earlier. Update the expected EDN to the `UnresolvedReferences` it now produces.

## ⑤ THE TWO RATCHETS — and this is the stone's PROOF, not maintenance

`tests/resolve/probe_arc255_the_blanket_hides_a_phantom_head.rs`, planted in stone ④ pinning
PRE-deletion behaviour and verified non-vacuous by applying this very deletion:

```
bogus_rete_head_type_checks_today_but_raises_unknown_function_at_runtime
dot_spelling_reaches_the_keyword_accessor_fallthrough_today
```

Both assert `--check` exits **0**. Both must now assert **1**, with re-captured `.edn` goldens.

★★ **Rewrite each row's name and doc to state the POST-blanket truth**, and record in the file
header that the flip happened and what it proves: `:wat::rete::f64::>X` — a bogus head that has
type-checked clean for as long as it has existed — now fails, and the dot spelling
`(:wat::core::Option.Some {…})` is refused at resolve instead of silently yielding
`#wat.core/Option.None {}`. That silent wrong answer is why the seam ordered the blanket's death
BEFORE the dot flip, and its disappearance is the evidence the ordering was right.

## Blast radius

`src/resolve/walk.rs` · `src/resolve/mod.rs` (test module) ·
`tests/diagnostics/probe_arc241_stone10_remedy.rs` (+ any goldens) ·
`tests/services/probe_arc209_c0b3bc_post_spawn.rs` (+ golden) ·
`tests/resolve/probe_arc255_the_blanket_hides_a_phantom_head.rs` (+ its two `.edn` goldens).
**No new declarations. No `.wat` corpus edits. No `src/intrinsic/` changes.**

## Acceptance

```
cargo nextest run --release -E 'test(resolve::tests)'                                  all pass
cargo nextest run --release -E 'test(probe_arc241_stone10_remedy)'                     all pass
cargo nextest run --release -E 'test(probe_arc209_c0b3bc_post_spawn)'                  all pass
cargo nextest run --release -E 'test(probe_arc255_the_blanket_hides_a_phantom_head)'   all pass
cargo nextest run --release -E 'test(probe_arc255_the_reserved_prefix_wall)'           all 7 pass
target/release/wat --check tests/resolve/probe_arc255_the_blanket_hides_a_phantom_head__bogus_rete_head.wat
        → EXIT 1  (was 0 — the blanket is dead)
```

## STOP triggers — each is a REJECTION. Ship nothing; report.

**STOP-1.** If you re-add a prefix acceptance for `:wat::*` in any form — STOP. `:rust::*` defers to
`use!`; nothing else gets one.

**STOP-2.** If you touch `src/resolve/registration.rs` or anything the seven
`probe_arc255_the_reserved_prefix_wall_is_not_the_blanket` rows cover — STOP. That is the WALL —
userland may not DEFINE under `:wat::*`/`:rust::*` — a DIFFERENT consumer of the same predicate, and
it stays. Those seven rows must remain green; they were measured green both with the blanket and
with it deleted.

**STOP-3.** If a remedy assertion would have to be weakened — STOP with the verbatim output.

**STOP-4.** If any test outside the five files above goes red — STOP and report it. The corpus census
was ZERO dependents; a sixth file means the census missed something.

**STOP-5.** If you find yourself declaring a verb to make a test pass — STOP. Every declaration this
deletion needed has already landed.

## Tier

You edit and report. Run the six acceptance commands, foreground, and nothing else. **Do NOT run
`scripts/floor.sh` or `cargo clippy`** — the orchestrator runs those centrally, once, on a quiescent
tree, and this stone's central weigh is the whole point. **Do NOT commit.**

Work in `/home/john/work/holon/wat-rs`; verify with `pwd` first. Any path containing
`.claude/worktrees/` is harness state and must not be operated on.
