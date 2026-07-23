# NOTE — mass-normalize the `__recv` / `__cause` (and kin) double-underscore hygiene-leak in hand-authored `.wat`

**Surfaced:** 2026-07-23, during arc-278 self-scheduling item-(c) (the Failure-record thread). The builder,
reading a recv'-wall `match`, asked *"what's with the `__prefix` stuff… what could possibly be colliding
here to warrant it?… I thought we had Racket's hygiene stuff and we didn't need that?"* — and he is right.

## The finding (grounded)

`__recv` (254 occurrences) and `__cause` (243), plus scattered `__work` / `__pool-work` / `__runner` /
`__start` / `__acc` / `__internal`, appear across **~75 `.wat` files**. The two dominant ones are the
fingerprint of the **recv'-wall codemod** — `wat-scripts/fixes/wrap-client-method-match-in-recvoutcome.wat`
(+ `unwrap-recvoutcome-false-positive.wat`) — which mechanically wrapped every `recv'` site into a `match`
and **baked a defensive `__`-prefixed binding name into the SOURCE.**

The double underscore is **neither hygiene nor needed** here:

- **wat already has Racket sets-of-scopes hygiene** (`src/scope/mod.rs`; `walk_template` scope-tags
  macro-template symbols; `fresh-symbol`; tested `probe_macro_hygiene_capture.rs`). A **macro-introduced**
  binding is capture-safe with a plain name — no manual `__` required. Hygiene is what makes the guard
  unnecessary *for macros*.
- But `__recv` / `__cause` are **not macro output** — they are literal text a codemod wrote into source.
  Hygiene is an expansion-time mechanism; it never touches codemod-written source. And it isn't needed
  either: a `match`-arm binding is **already lexically isolated to its arm body** — the only real risk a
  blind rewrite faced was shadowing a same-named binding *inside the wrapped expression*, which it
  "solved" with `__` instead of being a hygienic macro or checking per-site.

So the `__` is a codemod habit importing a guard that hygiene (for macros) or plain lexical scoping (for
source) already provides. It fails `intueri` — the name reads "machine-minted," not "the received message."

## The cleanup (deferred to a codemod strike)

- **Target: HAND-AUTHORED / codemod-baked `.wat` source only.** Normalize `__recv` → `recv`, `__cause` →
  `cause`, and the kin, where the binding is an ordinary lexically-scoped `let`/`match` binding.
- **Corpus-wide → a wat-fix codemod, NEVER hand-edits or python/sed** (the standing doctrine). Copy a
  recorded `wat-scripts/fixes/*.wat` migration as the shape; dry-run on a `/tmp` copy + `diff`; apply to
  every path; commit as the recorded migration. Idempotent.
- **Correct the generator.** `wrap-client-method-match-in-recvoutcome.wat` should stop emitting `__` on
  future runs (or be retired if it was a one-time migration), so the leak cannot regrow.

## The BOUNDARY — do NOT touch these (the `__` is legitimate there)

- **Rust-side AST codegen** that hand-builds `WatAST` via `Identifier::bare` — `__peer`/`__req`/`__op`/
  `__send`/`__r`/`__m` (`runtime.rs:5432+`), `__acc__` (`rete/kernel.rs:1416`), `__arc133_tuple_anchor_N`
  (`check.rs:2912`). This path bypasses the macro hygiene layer, so it collision-proofs manually against
  unknown user identifiers at the call site. That guard is real. (Open question, its own follow: should
  this route through the `fresh-symbol` facility instead of a hardcoded `__` prefix? — separate from this
  source-normalization note.)
- Any codemod/macro that *generates* a `recv'`-wall `match` for arbitrary caller sites: a blind generator
  cannot see each site's scope, so it must keep a collision-proof name (or be made hygienic). The
  normalization is for the SOURCE the last migration already baked, not for the generators.

## Why arc 109

The `recv'`/`Peer'` read surface is the reactor's; this is source hygiene in the reactor/`Peer'`-unification
family (`[[NOTE-tier-head-peer-unification-cleanup]]`), tracked alongside the other 109 deferred cleanups.
Not on the arc-278 critical path — logged here so it is a bounded arc item, not vague future-prose
(`exigere`), and picked up as its own codemod strike.
