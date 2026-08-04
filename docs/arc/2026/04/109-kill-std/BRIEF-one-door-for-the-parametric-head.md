# BRIEF — mint the one door for a type head's FQDN, then walk all 17 callers through it

Anchor at `/home/watmin/work/holon/wat-rs/`; verify with `pwd` first, and use
`git -C /home/watmin/work/holon/wat-rs` for any git read. The tree is clean at HEAD.

## The work in one paragraph

`TypeExpr::Path` stores its name **with** a leading colon; `TypeExpr::Parametric.head` stores it
**without** one, deliberately, so that two different parser paths produce a byte-identical string for
unification. Reading either correctly therefore depends on an invariant that is documented at the
parser and invisible everywhere it is used — so seventeen call sites hand-roll `format!(":{}", head)`
and one of them hand-rolls a defensive `starts_with(':')` branch. This is a purely mechanical strike:
mint two small functions, route all seventeen through them, change no behaviour whatsoever.

## Read in order

1. **`docs/arc/2026/04/109-kill-std/DESIGN-STONE-one-door-for-the-parametric-head.md`** — the whole
   spec: the exact contract, the measurement that chose that shape, and four ⛔s about what NOT to
   touch. Read it before anything else.
2. **`docs/arc/2026/04/109-kill-std/NOTE-a-parametric-head-is-bare-a-path-is-not.md`** — why the
   asymmetry exists and why "make the two variants consistent" is the wrong cure.
3. **`src/types.rs:4287`** — the comment that states the invariant outright. Its wording is the
   reason storage must not change.
4. **`src/check.rs:11178`** — the defensive `if head.starts_with(':')` branch. It is the reason
   `parametric_head_fqdn` must be **idempotent** rather than blindly prepending.

## The mint

Put both in `src/types.rs` beside `TypeExpr`:

```rust
/// The one place the bare-parametric-head invariant is written down.
/// `"wat::core::Vector"` → `":wat::core::Vector"`. Idempotent: input that already
/// carries the colon is returned unchanged.
pub(crate) fn parametric_head_fqdn(head: &str) -> String

impl TypeExpr {
    /// FQDN of this type's head — colon-prefixed, type args stripped.
    /// `None` for variants with no nameable head (Tuple, Fn, …).
    pub(crate) fn base_fqdn(&self) -> Option<String>
}
```

`base_fqdn`'s `Parametric` arm **must call** `parametric_head_fqdn` — one implementation, two doors.
Give each a doc comment that states the invariant, so the next reader meets it at the call site
instead of 4000 lines away.

## Route the callers

**Population A — 3 sites** matching *both* arms to produce one name. These collapse to
`ty.base_fqdn()`. All three are in `src/types.rs` (~`:2811`, ~`:2874`, ~`:2963`) and all three were
written on 2026-08-05; two are the `<Op>Request`/`<Op>Response` law checks and one is the ruling-A
lock's normalization.

**Population B — 14 sites** already inside a `Parametric { head, args }` arm that also need `args`.
These keep their match and swap the hand-roll for `parametric_head_fqdn(head)`. Found in
`closure_extract.rs` (2), `edn_shim.rs` (3), `types.rs` (5, including `:4863` and `:4967`),
`types/surface.rs` (1), `check.rs` (5, including the defensive branch at `:11178`), `runtime.rs` (1).

Locate them yourself rather than trusting this list to be exhaustive — the pattern is
`format!(":{}", head)` and `format!(":{head}")`. **If your count is not 17, that is STOP-1.**

## ⛔ STOPs — rejection criteria, not permission slots

- **⛔ STOP-1 — if you find more or fewer than 17 hand-roll sites, STOP** and report the list
  verbatim. The number was measured today with a scoped pattern; a different count means the pattern
  missed a shape and the orchestrator owns the re-scope.
- **⛔ STOP-2 — do NOT change how anything is stored, and do NOT touch the parser.** The bare head is
  load-bearing for unification. If you find yourself editing `parse_type_inner` or any `TypeExpr`
  construction site, you have left this stone.
- **⛔ STOP-3 — do NOT "fix" any `head == "bare::name"` comparison.** Those are correct: a bare RHS
  against a bare head. All 17 comparison forms were audited and none is colon-prefixed. Changing one
  is a behaviour change disguised as consistency.
- **⛔ STOP-4 — behaviour must not move.** This is a pure refactor. If a test changes outcome, you
  have altered semantics; STOP and report which site and which test.
- **⛔ Do not add a `_` wildcard arm on an enum scrutinee.** Doctrine.
- **⛔ Do not commit, stash, push, or touch git.** Leave the tree dirty; the orchestrator weighs.

## Verify — FOREGROUND, and block on it

```
cargo build --release
cargo nextest run --release
cargo clippy --release --all-targets
grep -rn 'format!(":{}", head)\|format!(":{head}")' src/ --include=*.rs     # expect: zero
```

Read the **Summary line**; never a piped exit code. **The floor to match exactly is
`4347 run / 4347 passed / 0 failed / 262 skipped`** — a pure refactor moves no test, in either
direction.

Report: the verbatim Summary line; every file touched; your own count of hand-roll sites before and
after; and anything you had to assume.

---

## EXPECTATIONS — written before the strike

| # | what | command | expected |
|---|---|---|---|
| 1 | the door exists and is one implementation | read `base_fqdn` | its `Parametric` arm calls `parametric_head_fqdn`, not a second `format!` |
| 2 | ★ idempotent | a unit test passing an already-prefixed head | returned unchanged, not `"::wat::…"` |
| 3 | ★ every hand-roll is gone | the grep above | **zero hits** |
| 4 | Population A really collapsed | read the 3 sites in `types.rs` | each is one `base_fqdn()` call, no surviving two-arm match for the name |
| 5 | the defensive branch is gone | `grep -n "head.starts_with(':')" src/check.rs` | zero hits — it is now one idempotent call |
| 6 | storage untouched | `git diff -- src/types.rs \| grep -c 'raw_head'` | zero — the parser's construction path is not in the diff |
| 7 | ★ behaviour unmoved | `cargo nextest run --release` Summary | `4347 / 4347 / 0 / 262`, exactly |
| 8 | clippy | `cargo clippy --release --all-targets` | clean |
| 9 | net lines | `git diff --stat` | roughly neutral-to-negative; 17 hand-rolls collapse, two small fns appear |

Rows 2, 3, 5 and 7 are re-run by the orchestrator by hand regardless of what is reported.

**Runtime prediction: 25–40 minutes.** Two build+test cycles dominate; the edit is compiler-verified.
Time-box 80.

**Trap doors, named in advance:**
1. **Prepending blindly** instead of idempotently — breaks `check.rs:11178`'s case into `"::wat::…"`.
2. **Reaching for the storage fix** — the obvious cure, and the one thing that breaks unification.
3. **Over-collapsing Population B** — those arms need `args` too; hoisting the whole match to
   `base_fqdn()` loses them. Fourteen of seventeen sites are this shape.
4. **Sweeping every `format!(":{}", x)` in the codebase** — the target is the *head* of a `TypeExpr`,
   not every colon-prepend. A pattern that cannot tell those apart is the third over-broad grep this
   week; scope it and validate it before trusting its count.
