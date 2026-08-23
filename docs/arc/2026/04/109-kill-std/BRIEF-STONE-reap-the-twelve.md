# BRIEF — reap the twelve

The first purge measured four functions and deleted them. It missed twelve siblings, because the
census was scoped from a list of names rather than from the rule
(`NOTE-the-sibling-angle-strips-my-census-missed.md`). **All twelve are now instrumented and measured.**

The tree is CLEAN, the floor is green at 4908/4908. Copy the report shape of
`SCORE-STONE-the-last-comma-lives-in-a-symbol.md`.

## The measurement — one full floor, every site probed

```
site                                          calls    type-heads found
src/runtime.rs:3127  preregister_struct_accessors_from_form      0        0   ← NEVER CALLED
src/runtime.rs:3285  preregister_enum_constructors_from_form     0        0   ← NEVER CALLED
src/runtime.rs:18971 eval_kwargs_construct (2nd)                 0        0   ← NEVER CALLED
src/runtime.rs:7492  dispatch_keyword_head_value                22        0
src/types.rs:3156    synthesize_surface_protocol            41,172        0
src/types.rs:3218    synthesize_surface_protocol            41,859        0
src/check.rs:12895   infer_aggregate_new_check             379,635        0
src/check.rs:13016   infer_kwargs_construct_check          565,691        0
src/runtime.rs:18894 eval_kwargs_construct (1st)           982,474        0
src/runtime.rs:18744 construct_aggregate                 1,024,489        0
src/check.rs:12756   canonical_ctor_callee              12,665,676        0
                                                       ──────────
                                                       15,701,018        0
```

**15.7 million calls. Zero type-heads. Three sites never called at all.** Together with the first
purge's 16.2M, that is ~32M no-op calls per floor run across the whole family.

## The 12th — `crates/wat-source-derive/src/lib.rs:73`

Not measurable by a floor run: it is a **proc macro**, so it executes at COMPILE time. Its own comment
says what it is:

> *"`:Name<I,O,A>` and the binder `:Name :- [I O A]`. **Peeling both here is what lets one caller
> string — the BASE name — address the declaration under either, so a file migrating does not silently
> take its Rust-side reader with it.**"*

A deliberate **transition shim**, accepting both spellings while the corpus migrated. The migration is
complete: the angle form is refused at the lexer, so no `.wat` file can present an angle-form
declaration name. Its angle half is unreachable — but **prove it, do not assume it.** The corpus census
(0 angle forms across 1798 files) is the evidence; state how you confirmed it.

⚠ That crate carries an earned `rune:lint(one-param-spec)` exemption because it structurally cannot
depend on the `wat` crate (`wat-macros → wat-doc → wat-source-derive` would cycle). Keep the exemption
honest: if the angle half goes, the comment justifying it must be updated to say what remains.

## What ships

1. The eleven runtime sites lose their `<`-strip; each call site uses the name directly.
2. The proc-macro shim loses its angle half, keeping the `:-` peel.
3. The `one_param_spec` rune (or a sibling) is extended to ban a bare `.find('<')` on a NAME —
   **which the previous stone could not do**, because these twelve would have failed the baseline. That
   is now the point: with them gone, the wider rule can finally be enforced.

## ⛔ What must survive — the same trap as last time

```clojure
:wat::core::<     :wat::core::>     :wat::core::>=     <-     ->
```

`src/types.rs:4688` is ③'s WALL — `if stripped.contains('<')` raising *"angle-bracket type parameters
are illegal"*. **It stays.** It is the backstop for a name that never passed the lexer, and deleting it
because it matches the grep would remove the guard that makes all of this true.

## Acceptance

| # | what | expected |
|---|---|---|
| 1★★★ | the operators still dispatch | `(:wat::core::< 1 2)` → `true`; `>`, `>=`, `<-`, `->` live |
| 2★★★ | ③'s wall still fires | `:wat::core::Vector<i64>` in source → *"angle-bracket type parameters are illegal"* |
| 3★★ | aggregates still construct | a `defrecord` + kwargs construction round-trips |
| 4★★ | a parametric `defservice` round-trips | lru-svc / hologram-svc |
| 5★★ | the twelve are gone | `find('<')` on a name survives nowhere but ③'s wall |
| 6★ | the widened rune | drawn, and POSITIVE-CONTROLLED by planting a violation |

**Rows 1 and 2 decide it.** Row 2 especially: ③'s wall matches the same grep as the twelve, and a purge
that takes it out still passes rows 1 and 3-5 while quietly making the whole campaign undoable.

## STOP triggers

- **STOP-1 — a site the census recorded as 0 calls turns out reachable.** Three sites measured zero;
  a reachable one means the census missed a path. Report it.
- **STOP-2 — a call site DEPENDED on the strip** rather than tolerating it. Report the site and both
  behaviours. (Last stone: all ~19 tolerated. Do not assume that repeats — check.)
- **STOP-3 — the widened rune cannot pass on a green tree.** Then something still strips a name and the
  census missed it: report what, rather than narrowing the rune to fit.

## Boundaries

- The eleven sites, `crates/wat-source-derive/src/lib.rs`, and the rune.
- **Do NOT touch `src/types.rs:4688`** — ③'s wall.
- **Do NOT touch `keyword/to-type-form` / `to-type-form-colon`.** Transition shim, live caller.
- **Do NOT sweep the retired spelling out of COMMENTS.** Its own stone; a blind pass erases the lines
  that RECORD the retirement.
- Do NOT commit, push, stash or amend. Keep the git index EMPTY: no `git add`, no
  `git checkout <ref> -- <path>` (it STAGES).
- Goldens: **KEEP PINNING THE SPAN** and recapture; verify each is the same call site, only moved.
- The orchestrator runs the full floor and clippy centrally.

Build with `systemd-run --user --scope -q -p MemoryMax=24G -p MemorySwapMax=0 timeout 3000 cargo build --release`.
Read exit codes DIRECTLY — never through a pipe, never after a trailing `; echo`.
`cargo wat` uses the STALE installed binary; always `./target/release/wat`.

## Your report

Rows 1 and 2 verbatim first — the operators dispatching AND ③'s wall still firing. Then rows 3-6. For
each of the twelve, whether its call site DEPENDED on the strip or merely tolerated it. How you
confirmed the proc-macro shim's angle half is unreachable. Any STOP that fired, with the arm captured
verbatim BEFORE you diagnosed it. What surprised you.
