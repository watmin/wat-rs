# BRIEF — 293.K5: `extend-surface` (the LAST surface tool) — a thin wat `defmacro`

> **Executor: one sonnet LEAF.** Orchestrator drew this + the RED probe; weighs the kill forced-clean.
> Work ONLY in `wat-rs/`, NEVER worktrees. Commit nothing — leave the tree green for the orchestrator to weigh.

## The work (one paragraph)
Add `:wat::core::extend-surface` — a **wat `defmacro`, pure form-production, ZERO substrate change**. It takes a
surface keyword `:S` plus N **typeless** method forms `(m [binders] body)`, and emits one `extend-type` per **pair**
backing tier: `(extend-type :S$core-record :S ~@methods)` and `(extend-type :S$holon-record :S ~@methods)`. The user
writes **body only** — `extend-type` already fills the method's types from the surface member's sig (the 293.4e-pre.iii
capability, present on HEAD), so the macro needs **no reflection seam**. Per the K5 decision (option A, 2026-06-30) the
default rides BOTH pair tiers, so a `to-record`'d value at either tier inherits it for free.

## Why this is the whole job (grounded this session)
- The full chain `extend-surface` automates is **proven by hand** (foundation probe → 84): a source satisfies `:S`
  (data + its own method) → `to-record` lifts the DATA to a backing record → that backing record needs its OWN method
  to satisfy `:S` (projection carries data, never behavior) → a hand-written `extend-type` on the backing record
  supplies it → dispatch fires. `extend-surface` just emits those two backing-record `extend-type`s.
- `extend-type` filling types from the surface for a **typeless** `[self x]` body is LIVE (verified: monomorphic +
  generic both type-check + dispatch via full-check `cargo wat`). So the macro forwards the typeless body verbatim.
- `defsurface` / `extend-type` are **Rust special forms** (no wat sibling) → `extend-surface` is a NEW wat `defmacro`
  homed in `wat/core.wat`.

## Read in order (the rooms — grounded 2026-06-30)
1. **`wat/service.wat:66`+ (`defmacro :wat::service::defservice`)** — THE structural exemplar: a `defmacro` that takes a
   keyword + variadic forms, DERIVES keyword names from it (`keyword/to-string` → `string::concat`/`string::interpolate`
   → `keyword/from-string`, `:178-202`), and emits a `(:wat::core::do …)` of multiple forms. Copy this shape.
2. **`wat/core.wat:359-395` (the `kwargs-lower` / `defn` macro)** — the **`keyword-node`** idiom: build a keyword AST
   node from a colon-prefixed string (`:395` `kwargs-ty-node (:wat::core::keyword-node …)`). `keyword/to-string` returns
   the name WITHOUT the leading colon (`:509` prepends `":"`), so a backing node = `keyword-node` of
   `":" ++ surf-str ++ "$core-record"`. (Confirm the colon convention against `:509`; STOP-1 if it differs.)
3. **`tests/types/probe_arc293_k4_extend_type_own_aggregate.wat`** — the `extend-type` arg shape the macro emits:
   `(:wat::core::extend-type :T :S (m [binders] -> … body))`; here the binders are TYPELESS and the body is forwarded.
4. **`tests/types/probe_arc293_k5_extend_surface.{rs,wat}`** — the committed RED probe (this strike's gate). It is
   `#[ignore]`'d; **un-ignore it as the final step** and it must go GREEN (→ 84).
5. **`wat/core.wat:615` (`defmacro :wat::core::keyword/of`)** — a smaller keyword-deriving macro, for the quasiquote +
   `keyword/to-string` pattern in miniature.

## Implementation sketch (the strike path — fill it, don't invent the shape)
```clojure
;; in wat/core.wat, beside the other :wat::core:: surface/aggregate macros:
(:wat::core::defmacro :wat::core::extend-surface
  [surf <- :wat::WatAST  & methods <- :wat::core::Vector<wat::WatAST>]   ; match defservice's variadic shape
  -> :wat::WatAST
  (:wat::core::let
    [surf-str   (:wat::core::keyword/to-string surf)                      ; "k5::HasX"  (no leading colon — confirm)
     core-node  (:wat::core::keyword-node
                  (:wat::core::string::concat ":" (:wat::core::string::concat surf-str "$core-record")))
     holon-node (:wat::core::keyword-node
                  (:wat::core::string::concat ":" (:wat::core::string::concat surf-str "$holon-record")))]
    `(:wat::core::do
       (:wat::core::extend-type ~core-node  ~surf ~@methods)
       (:wat::core::extend-type ~holon-node ~surf ~@methods))))
```
The exact rest-binder type + the `~@methods` splice must match how existing variadic defmacros forward a Vector of AST
forms — copy `defservice`'s pattern verbatim where it forwards clause forms.

## Blast radius (bounded)
`wat/core.wat` ONLY — one new `defmacro`. NO Rust change. NO new special form, NO new `ArgSpec`, NO reflection seam.
If you find yourself touching `src/`, STOP — the design is wrong or a foundation claim failed; surface it.

## STOP triggers (halt + surface, never improvise)
- **STOP-1 (colon convention):** if `keyword/to-string` returns a string that ALREADY includes the leading `:`, do NOT
  also prepend `":"` (you'd get `::`). Confirm against `core.wat:509` and the `keyword-node` it feeds; adjust once.
- **STOP-2 (variadic splice):** if `~@methods` does not splice the forwarded method forms into the `extend-type` call
  as separate clause args (e.g. it nests them in a Vector), STOP — match `defservice`'s exact forwarding idiom; do not
  hand-roll an AST walk.
- **STOP-3 (the macro needs the surface's sigs):** the macro must NOT need to read the surface's method signatures — it
  forwards typeless bodies and `extend-type` fills the types. If you find the emitted `extend-type` does NOT type-check
  because the types aren't filled, STOP and surface it (it would mean the pre-iii capability regressed — do not rebuild it).
- **STOP-4 (`src/` touch):** any Rust edit means the "pure wat macro" thesis failed — STOP and report.

## The RED probe (committed, `#[ignore]`'d — this strike's gate)
`tests/types/probe_arc293_k5_extend_surface.{rs,wat}`: a Struct-floored surface `:k5::HasX [x (dbl …)]`; the source
`:k5::Pt` gets its own `dbl` (so it satisfies `:k5::HasX` and can be `to-record`'d); `extend-surface` gives the PAIR
backing records `dbl`; `:k5::demo` `to-record`s into both tiers and sums `(:k5::HasX/dbl cr) + (:k5::HasX/dbl hr)` = 84.
RED at HEAD: `extend-surface` unbound → backing records lack `dbl` → `:k5::HasX/dbl` rejects them (TypeMismatch
receiver `:k5::HasX$core-record`/`$holon-record` vs `:k5::HasX`). GREEN after K5. **Un-ignore it as the final step.**

## EXPECTATIONS (the scorecard — fixed before the strike)
| # | what | command | expected |
|---|---|---|---|
| 1 | the K5 probe goes GREEN | un-ignore `extend_surface_default_rides_both_pair_tiers`, then `cargo nextest run --release -E 'test(extend_surface_default_rides_both_pair_tiers)'` | PASS (84) |
| 2 | `extend-surface` is a pure wat macro | `git diff --stat` | `wat/core.wat` + the probe `.rs` (un-ignore) ONLY; **no `src/` change** |
| 3 | nothing else regressed | `cargo nextest run --release` | 0 failed; **91 skipped** (the K5 ignore now gone) |
| 4 | clean build | `cargo build --release` | clean |

Runtime estimate: 20–40 min (one macro; the risk is the keyword-node colon convention + the variadic splice idiom —
both copied from existing macros). Trap-door: macro hygiene on the forwarded body (surface accessors / `self` / `x`) —
they are user symbols forwarded verbatim, so no macro-introduced binding should clash; if a scope/hygiene error appears,
mirror how `defservice` forwards its op-handler bodies (it forwards user bodies into emitted forms the same way).

## You are a LEAF
Anchor `/home/watmin/work/holon/wat-rs`; `pwd` first; reject `.claude/worktrees/`. Do NOT spawn subagents. Do NOT
commit. Build incrementally; dogfood `cargo wat` on a scratch file to iterate the macro before the probe. Read every
diff. Self-verify the EXPECTATIONS. STOP + report if a STOP fires or the work needs a `src/` edit.
