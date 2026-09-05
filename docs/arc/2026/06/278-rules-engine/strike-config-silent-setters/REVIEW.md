# REVIEW — Ω4: STOP-1 was right, and MY contract decision was wrong

> Weighed against my own reading of the disk.

## The STOP was correct, and the discipline around it was correct

`tests/wat_lang/wat_arc157_def.wat:25` is **not** an entry file carrying a stray setter. It is a
fixture asserting a designed capability, and the nine reds are correct behaviour breaking.

`src/check.rs:722-728` processes a redef setter **inline, mid-program, deliberately**:

> *"Arc 157 slice 1a-ii — if the form is a set-redef! setter, update `env.redef_allowed` in-line so
> subsequent def forms see the new flag (single-pass program-order semantics)."*

Toggling redef partway through a file is the whole point. **A config setter is legitimately a body
form**, and my `ends_with('!')` discriminator outlawed it.

**That is my error, in the DESIGN, not the rider's.** I pinned a discriminator without checking
whether any config setter is legal mid-body. One family is, by design. Capturing the floor,
naming all nine arms, refusing to re-run, and refusing to patch the fixture were all correct — and
patching that fixture would have deleted a real feature to make my rule true.

## The correct discriminator already exists in the tree

`src/special_forms.rs` **already draws this exact line**, and `lookup_special_form(name)` (`:65`) is
its one query door:

| head | registered? | meaning |
|---|---|---|
| `:wat::config::set-redef!` (`:128`), `set-eval-redef!` (`:133`) | **yes** | legal FORMS — valid anywhere in the body |
| `set-dim-count!`, `set-capacity-mode!`, `set-global-seed!`, `rete::set-max-fire-rounds!` | **no** | entry-file-only setters |

**Revised rule — replaces the DESIGN's `ends_with('!')`:**

> In the remainder, a `:wat::config::…!` head that is **NOT** a registered special form is
> misplaced. If its leaf is a valid `set-` → `SetterAfterNonSetter`. Otherwise → `UnknownSetter`.

Checked against all four cases:

| form in the remainder | registered? | verdict |
|---|---|---|
| `set-eval-redef! true` (the fixture) | yes | **legal** — the nine reds go green |
| `set-redef! true` | yes | **legal** |
| `set-dim-count! 4096` after a body form | no | `SetterAfterNonSetter` — Ω4b still cured |
| `setmax-fire-rounds! 5` (typo) | no | `UnknownSetter` — Ω4a still cured |
| `(:wat::config::dim-count)` accessor | n/a — no `!` | untouched |

**Why the registry and not a new allowlist:** an allowlist in `config.rs` beside the checker's own
inline handling would be one rule encoded twice with nothing forcing agreement — CLASS A, the
defect this whole vigilia is about, and the one we cured in `JoinLeftIndex` two commits ago. The
registry is already the single door; use it.

## What to change

1. In the remainder scan, gate on `crate::special_forms::lookup_special_form(head).is_none()`
   **before** classifying. A registered head is not our business.
2. Everything else stands — `UnknownSetter` / `SetterAfterNonSetter`, the spans, removing the dead
   in-loop guard, the four gates.
3. **Add a fifth gate row**: a `set-eval-redef!` in the body must remain legal. That row is the
   regression guard for exactly the mistake my DESIGN made, and without it the next tightening of
   this rule re-breaks arc 157 silently.
4. Re-run the floor **at final state**. Expect `wat_arc157_def`'s nine to be green.

## Rows I accept as they stand

1★ Ω4a cured (`UnknownSetter`, span on the typo, exit 3) · 2★ Ω4b cured (`SetterAfterNonSetter`,
span on the setter, exit 3) · 3★ variant reachable from one construction site · 4★ accessors legal,
control prints `4096` · 5 one-name-grammar via `identifier::leaf`, no `rsplit` · 7 clippy rc=0
under `-D warnings` · 8 blast radius: `src/config.rs` only, zero lines in `resolve/` or `check.rs`.

Row 6 is the only one outstanding and it is mine to have caused.
