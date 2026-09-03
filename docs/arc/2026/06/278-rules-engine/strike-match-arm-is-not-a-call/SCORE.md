# SCORE — D5, weighed against the orchestrator's own re-run

> Re-run at `d10ae67c4` + the rider's tree. **The cure lands. The finding is that my three mutations
> could not reach the design decision my own DESIGN emphasised — the rider added a fourth and it
> reddens.**

## The scorecard

| # | required | result, MY re-run |
|---|---|---|
| 1 | ★ a legal `match` compiles in `:then` | ✅ `experiri-then-match.wat` → `"loaded"` |
| 2 | ★ the two spellings agree | ✅ bare and wrapped both compile, byte-identical fired values |
| 3 | `where` fence unchanged | ✅ `experiri-when-match.wat` → `"loaded"` |
| 4 | a constructor in an arm BODY still validated | ✅ **proven by mutation 2, driven by me** |
| 5 | no phantom insert in the diagnostic | ✅ |
| 6 | the repro becomes a gate | ✅ 5 tests + 4 fixtures + a `.wat.bad` golden |
| 7 | the rune retired with its reason | ✅ `grep -c 'rune:lint('` → **0** |
| 8 | `let`/`fn`/`cond` untouched | ✅ no branch added |
| 9 | engine untouched | ✅ `git diff --stat -- src/rete/kernel/fire/` empty |
| 10 | lints | ✅ green |
| 11 | clippy | ✅ rc=0 |
| — | floor | ✅ **`5332 tests run: 5332 passed, 21 skipped`**, exit=0 (5327 + 5) |

## ⭐ A — MUTATION 2 IS THE SEPARATOR, AND I DROVE IT

The named failure-even-if-green was *a fix that stops walking `match` forms altogether*. Skipping the
arm body as well as the pattern (`.skip(1).take(0)`):

```
FAIL (1/5) a_misspelled_constructor_in_a_match_arm_body_is_still_refused
  ...if this program RAN, the cure stopped walking match forms instead of skipping their
  patterns, and four error kinds are now dark inside every arm body
Summary [0.838s] 5 tests run: 4 passed, 1 failed
```

**Exactly one red; the other four green.** That is the proof rows 1, 3 and 5 cannot catch this
failure mode — which matters because this arc has already shipped that shape once
(`strike-nested-wall`: the same walker orphaned by a lowering, four error kinds unreachable, every
gate green).

## ⛔⛔ B — MY THREE MUTATIONS COULD NOT REACH MY OWN DESIGN DECISION

DESIGN pinned the head resolution as the thing to get right. **All three mutations I specified pass
with `head == ":wat::rete::core::match"`** — the naive key that leaves the defect intact for the core
spelling. The rider noticed and added a fourth. Driven by me:

```
FAIL (1/5) the_core_spelling_is_refused_by_the_fence_not_by_a_phantom_arity_error
Summary 5 tests run: 4 passed, 1 failed
```

**A scorecard that emphasises a decision and gates every part of the change except that decision is a
scorecard with a hole in exactly the place it was pointing.** The cure uses
`resolve_core_name(head) == ":wat::core::match"`; without mutation 4 that indirection was unproven.

## ⛔ C — AND MY QUESTION WAS THE WRONG ONE

The brief said *"measure which spelling(s) actually reach the walker."* **Both do.** But only the rete
spelling is *legal* in a `:then` — `wat/rete/compile.wat`'s then-item fence refuses
`:wat::core::match` downstream with *"is not a rete primitive"*. Read literally, my STOP-2 (*"if you
cannot drive a spelling you are about to add a branch for, stop"*) would have pushed toward the
rete-only key that mutation 4 reddens.

The right question is **not "can the program succeed" but "is the walker's behaviour on it
observable"** — for the core spelling the cure changes *which wall refuses and what it says*: at HEAD
it died at freeze inventing an insert of an enum variant; now it dies at the fence naming the head.

## ⛔ D — MY READ-LIST CITED THE USELESS SIBLING

I sent the rider to `clause.rs:260`. That is inside `expr_is_provably_boolean`, which reads
`items[2..]` and each arm's `.last()` — a *different* traversal. The shape the cure actually needed is
**`purity.rs:1310`**, verified by me:

```rust
WatAST::List(items, list_span) if matches!(items.first(), Some(WatAST::Keyword(k, _))
    if crate::rete::vocabulary::resolve_core_name(k) == ":wat::core::match") => {
    let scrut = items.get(1)...; classify_expr(scrut, axes, sym, seen)?;
    let arms = items.get(2..)...
```

Line-for-line the contract, **including the `resolve_core_name` guard** — the very decision my
mutations failed to gate. The working reference was in-tree and my read-list pointed one file over.
Third strike running where the read-list is my weak part.

## ⭐ E — A RETIREMENT THAT UN-RETIRED ITSELF, CAUGHT BY THE RIDER

Its first header rewrite *quoted* the retired marker inside backticks. `declaration_on()` matches
`rune:lint(` **anywhere on a line**, so the quotation silently re-exempted the file from the load
check — a retirement that undoes itself by describing itself. The header now forbids the literal
appearing in that file at all, and `grep -c` returns 0. Disclosed, not discovered.

## ⚠ F — A DOCTRINE GAP, NOT A DEVIATION

`wat-rs/CLAUDE.md` sends scratch `.wat` to `wat-scripts/scratch-pad/`, but
`every_wat_scripts_file_loads` reddens on anything that refuses — and reconnaissance for a
*refusal* defect is exactly a must-fail program. Two of the rider's four probes could not have lived
there. It used the session scratchpad and deleted them. **That is a real hole in the repo's own
doctrine**, recorded as **D9** rather than waved through.

## Per-arm status

| arm | status |
|---|---|
| `:wat::rete::core::match` in `:then` | **proven** — mutation 1 reddens it |
| `:wat::core::match` in `:then` | **proven** — mutation 4 reddens it; never a legal program (fence refuses downstream) |
| arm BODY recursion | **proven** — mutation 2, driven by me |
| scrutinee recursion | **reachable but NOT driven** — the rider flagged this itself rather than claiming a proof. One call, contract-mandated, and unwalking it would be silent |
| `let` / `fn` / `cond` | **not reachable, no branch added** — measured in the DESIGN's enumeration |
