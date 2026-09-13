# BRIEF 2a1 — the declaration door: expand, register, return, with no startup

Anchor: `/home/john/work/holon/wat-rs`. Verify with `pwd`. **Never use worktrees.** Do not touch
`~/work/holon/` (the frozen root).

## ⛔ TREE STATE

```
branch   replay/grok-rete   <-- YOU ARE HERE
HEAD     (the commit that lands this brief)   floor 5411/5411 · clippy 0 (2b verification)
```

## Why (the builder: "we build it")

The replay's codemods must learn the field names of the types a file declares, in declaration
order, exactly as the substrate registers them. That covers macros, acronyms, and generated types:
`defsurface`'s `Op`/`Reply`, `sift-rules-defsvc`'s enum. The work itself is ordinary: expand the
program's declarations with the stdlib macros, register their types into a COPY of the stdlib
registry, and read them back. The Rust side does exactly that in the test helper `stdlib_loaded()` +
`check()` (`src/check.rs:23436`). **Wat cannot:**
- the reflection verbs (`type-of`, `is-type?`, `field-names-of`) query only the running world;
- the only door is `eval-with-defs!`, a REPL turn. It runs two full startups per call, rebuilds stdlib
  each time (`src/freeze/env.rs:107`), and checks every body. That costs ~0.46 s per question, and a
  stale body poisons it.

Measured (`FINDINGS-composition.md` findings 9–12):
- the exact wat-side workaround (batching plus bisection) caps at **2.4×** per commit (99 s → 42 s
  on five files);
- the whole-corpus check still takes hours unsharded.

The four questions were all YES, argued in the main chat.

## D1 — the probe, FIRST, inside the crate

A `#[cfg(test)]` unit test beside `stdlib_loaded()` (`src/check.rs:23436`) that performs the door's
operation, answering two unknowns before any verb exists:
1. **Does registration survive a stale body?** Use the declarations of
   `bootstrap/era/probe-L/w25/tests/services/probe_arc278_sift_rules.wat` and
   `…/tests/comms/probe_arc293_W2f_process_dials_thread.wat` (step-26 input, where a `defservice`
   body poisons `eval-with-defs!`). Every enum each file declares must register, with its field names
   in declaration order.
2. **How long does one call take** on sift_rules' forms?

Mirror `build_env`'s USER half on copies of the stdlib half: `register_defmacros`, the acronym
pre-registration (`preregister_acronyms`, step 4), `expand_all`, `register_types`.
`src/freeze/env.rs:124-194` is where each happens. Stop before definitions and before
`check_program`.

## D2 — the stdlib snapshot

The door works on copies of the stdlib half: macros after `env.rs:125`, types after `:194`, and the
symbols expansion needs (the helper passes the stdlib `SymbolTable`). `TypeEnv`, `MacroRegistry` and
`SymbolTable` are all `Clone`.
- **Built at most once per process.** Its construction must not run inside a lock that stdlib
  expansion can reach: `src/runtime.rs:10294` records exactly that deadlock, a `OnceLock` whose
  initialiser re-entered itself through stdlib macro expansion.
- If you capture it during startup, measure the delta on `wat --check` of an empty file (today
  ~218 ms) and report it.
- If you build it lazily, prove with a test that no stdlib expansion path reaches the verb.

## D3 — the verb

A pure verb, with no `!` (the builder may rename it), e.g. `:wat::runtime::declared-types`:
- takes one program's forms, `(:wat::core::Vector :- [:wat::WatAST])`;
- returns an outcome: the types those forms added to the copy, each as the same `TypeInfo` that
  `type-of` returns;
- reuse `eval_type_of`'s construction (`src/reflect/verbs.rs:1489`: `type_kind_value`,
  `type_params_of`, `type_body_value`), never a second one;
- a declaration that cannot register comes back as a structured refusal naming that form and its
  cause, never a panic, and never a silent drop;
- each call starts from a FRESH copy, so one program's declarations never reach another's (finding
  5's cross-file leak is the reason);
- document it like its siblings (`@arg`/`@ret`/`@example`, as `type-of` does).

## D4 — tests (each case asserts field names in declaration order)

1. a plain `defenum` and `defrecord`;
2. `defsurface`: its generated `Op`/`Reply` variants;
3. `:wat::string::declare-acronyms` (the `CreateWebACL` spelling;
   `tests/macros/probe_arc265_acronym_registry_svc.wat`);
4. a `sift-rules-defsvc`-generated enum;
5. a `defservice` whose `:impls` body is stale (a positional constructor): its types still register;
6. two calls declaring the SAME type name with different fields: each sees only its own;
7. a malformed declaration: a structured refusal naming it;
8. **the oracle, from the other side:** for each of cases 1–4 that loads normally, the verb's answer
   equals `type-of` in a world where that source was started with `startup_from_source`.

## ⛔ STOP triggers — rejections; report the verbatim evidence

- **STOP-1:** registration needs bodies checked, i.e. D1's poisoned files do not register without
  `check_program`.
- **STOP-2:** the snapshot cannot be built without a lock stdlib expansion can re-enter.
- **STOP-3:** the floor is red. Paste the whole block verbatim, and do not re-run.

## Out of this stone, affirmatively

- The codemods asking the door (match-arm, positional-ctor, variant-separator, via one door in
  `wat/fix.wat`) is stone **2a2**. Its bar is already measured: the exact batched ladder's outputs
  (`bootstrap/era/probe-Q/run4.sh`: 0 files differ, UNRESOLVED 15, 0 constructors lost).
- `is-type?`/`type-of` disagreeing on `:wat::core::Vector` is its own main defect, recorded in FINDINGS.

## Tier

Commit on green: `scripts/floor.sh` (read the Summary) and
`cargo clippy --release --all-targets -- -D warnings` at 0. **Do not push.** Yield with `SCORE-2a1.md`.
