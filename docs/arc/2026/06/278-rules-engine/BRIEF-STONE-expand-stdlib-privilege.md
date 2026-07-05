# BRIEF — substrate stone: `expand_all` stdlib registration privilege (unblocks baked `:wat::` defservices)

> **Executor: one sonnet SHADOWDANCER.** Orchestrator drew this against a RED probe already on disk. Work ONLY in
> `/home/watmin/work/holon/wat-rs/` (`pwd` first; any `.claude/worktrees/` path is illegal — re-anchor, use `git -C`).
> `cargo nextest run` (NEVER `cargo test`). **Commit NOTHING** — leave the tree for the orchestrator to weigh.

## The RED probe (already in the tree — do NOT remove it; your fix turns it green)

`wat/query/mem.wat` (a `:wat::query::mem-store'` **defservice**) is baked into `src/stdlib.rs`, and
`tests/rete/probe_arc278_query_contract` currently **FAILS at startup**:

```
#wat.macro/ReservedPrefix "cannot declare macro :wat::query::mem-store'/start — reserved prefix (:wat::, :rust::)"
  :name ":wat::query::mem-store'/start"
```

Confirmed by `cargo nextest run --release -E 'test(query_contract)'` → **1 failed** (ReservedPrefix). This is the exact
gap. When your fix lands, stdlib loads, and it goes green.

## The gap (grounded — the mechanism)

A `defservice` is a **macro-generating-macro**: expanding it EMITS a companion defmacro (`…/start`). Expansion-born
defmacros are registered inside `expand_all` at `src/macros/expand.rs:42` and `:117` via `registry.register(def)?` — the
reserved-prefix-**CHECKED** path. `expand_all` is called identically for stdlib (`src/freeze/env.rs:108`) and user code
(`env.rs:114`), so it can't tell them apart, and a `:wat::`-named companion born during **stdlib** expansion hits
`ReservedPrefix` and aborts stdlib load — breaking every program.

The **literal** top-level path already solves this: `register_stdlib_defmacros` (`src/macros/parse.rs:29-43`, called at
`env.rs:91`) registers baked defmacros via `registry.register_stdlib(def)?` — the **PRIVILEGED** path that bypasses the
gate. So `MacroRegistry` already has both methods: `register` (checked) and `register_stdlib` (privileged). Expansion-born
registrations just never use the privileged one.

## The work (one paragraph)

Give `expand_all` the same privilege distinction the literal path has: thread a stdlib-privilege flag into `expand_all`,
and at its two defmacro-registration sites call `registry.register_stdlib(def)?` when expanding **stdlib**, else the
checked `registry.register(def)?`. Wire the callers so stdlib expansion is privileged and user expansion is NOT. Nothing
else changes — you are extending an existing privilege from literal defmacros to expansion-born ones, **stdlib-only**.

## The exact change

1. **`src/macros/expand.rs`** — `expand_all` gains a privilege parameter (a `bool stdlib`, or reuse
   `crate::types::RegistrationPrivilege` if that reads cleaner). At the two `registry.register(def)?` sites
   (currently ~`:42` and ~`:117`), select: `if stdlib { registry.register_stdlib(def)? } else { registry.register(def)? }`.
   **Thread the flag through every recursive / nested expansion call** inside `expand_all`/`expand_form` that can reach a
   registration site (a companion born inside a nested `do` must inherit the same privilege) — grep for all `register(`
   call sites reachable from `expand_all` and cover each.
2. **`src/freeze/env.rs`** — `env.rs:108` (the stdlib `expand_all`) passes privilege = **stdlib/true**; `env.rs:114` (the
   user `expand_all`) passes **user/false**.
3. **Do NOT touch** `MacroRegistry::register`'s reserved-prefix check itself — the fix is *call-site selection* between
   the two existing methods, not weakening the check.
4. **Wire a MemStore round-trip `deftest'`** proving the real baked satisfier works: construct a `:wat::query::MemStore`,
   `put` a batch of `StoredRow`s, `scan` a page, keyset-paginate the next page, `put`+`scan-index` a GSI — assert the rows
   + cursors. (`wat/query/mem.wat` is already baked; this proves the defservice satisfier is live under `:wat::query::`.)

## Read in order (the rooms)

1. `src/freeze/env.rs:85-120` — the two `expand_all` calls (`:108` stdlib, `:114` user) + `register_stdlib_defmacros` at `:91`.
2. `src/macros/parse.rs:25-45` — `register_stdlib_defmacros`: the **pattern to mirror** (it calls `register_stdlib`).
3. `src/macros/expand.rs:35-125` — `expand_all` + `expand_form`, the two `registry.register(def)?` sites, and the
   nested-`do` recursion (`is_do_containing_defmacro`) — every reachable register site must honor the flag.
4. `src/macros.rs` (or wherever `MacroRegistry` lives) — confirm `register` (checked) vs `register_stdlib` (privileged)
   signatures; use the existing `register_stdlib`, don't invent a third.

## STOP triggers (rejection criteria — the user gate is SACRED)

- **STOP-USER-GATE (critical):** user code must STAY fully gated. A **user** source (not stdlib) that defines or *uses* a
  macro-generating-macro producing a `:wat::`-named macro must STILL halt with `ReservedPrefix`. If your change privileges
  the user `expand_all` (`env.rs:114`) even slightly, STOP — the privilege is **stdlib-only**. Prove it with a test (below).
- **STOP-CHECK-WEAKEN:** if the only way you find to fix it is editing `register`'s reserved-prefix check (rather than
  choosing `register_stdlib` at the call site), STOP and report — that's a different, riskier change the orchestrator must weigh.
- **STOP-RECURSION:** if a register site reachable from `expand_all` can't receive the flag cleanly (some path doesn't
  thread the registry/flag), STOP and report the path — do NOT leave a site on the wrong method.

## The gate (EXPECTATIONS)

| what | command | expected |
|---|---|---|
| the RED probe goes GREEN (stdlib loads with mem.wat baked) | `cargo nextest run --release -E 'test(query_contract)'` | **1 passed** |
| the real MemStore round-trips | `cargo nextest run --release -E 'test(mem_store)'` (or the deftest's runner) | passed |
| **user gate intact** — a user `:wat::` macro-gen-macro still halts | a new test: user source with a `:wat::`-named defservice/kwargs-defn → `startup` errors with `ReservedPrefix` | errors as expected |
| whole floor | `cargo nextest run --release` | `0 failed` (modulo the known `no_inlined_wat_in_tests` reminder) |

Runtime ~30-45 min (macro-layer change → release rebuild). Trap-door: a nested-`do`/recursive expansion register site you
miss will leave a companion on the checked path — grep every `register(` reachable from `expand_all` and thread the flag to all.

## Blast radius (bounded)

`src/macros/expand.rs` (the `expand_all` signature + the 2 register sites + recursion threading) · `src/freeze/env.rs`
(the 2 `expand_all` call sites) · a MemStore round-trip `deftest'` + a user-gate-intact test. **No change to
`register`'s reserved check.** `wat/query.wat` + `wat/query/mem.wat` + `src/stdlib.rs` (mem.wat entry) stay as they are.
