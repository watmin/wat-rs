# WEIGH — STONE 255.24 (C-b2): defservice declares the type parameters it emits — ACCEPTED

**Executor commit `9d6926b03`.** Weighed by the orchestrator against disk on 2026-09-24.

## Re-measured

| row | measured | result |
|---|---|---|
| tree · floor | `git status`; `.floor/2026-09-24T18-31-27Z/clean.log` | clean; `6058 tests run: 6058 passed (9 slow), 22 skipped`; all 3 `arc255_24` tests PASS |
| ⛔ an undeclared letter in a `defn` signature | `(:wat::core::defn :probe::g [x <- :K] -> :K x)`, `--check` | **rc=0**: `K` is declared nowhere and checks clean |
| an unused `defn` type parameter | `(defn :probe::f :- [K T] [x <- K] -> K x)` | rc=0 |

Taken from the report without re-running:

- clippy 0;
- census `no STOP-8` (215, identical files);
- delta NEW 2 / RECOVERY 0;
- ledger 215;
- the first floor went red on 4 `peers_bijection` goldens whose `wat/service.wat` line numbers moved 16
  lines. They were recaptured, the diff is `:line` only, and the whole floor was re-run.

## What landed

- **One helper, `decl-binder`, in the macro.** It uses `type-params-used-in` over each emitted
  signature, so every generic emitted `defn` declares **exactly** the letters its signature uses:
  - `serve`, `extract-addr`, `stop`, `hibernate`, `grant`, `revoke`, `start$impl` and `resume$impl` get
    `[K V T]` (or `[T]` for a monomorphic service; `[T Xt]` when the service binds its own `T`);
  - `init` and the other admin defns get `[K V]`.
- Type declarations and the extend-types already declared theirs.
- **The census is a pinned value** (`arc255_24_every_emitted_defn_declares_its_letters`). It fails on the
  pre-stone macro, and that failure is the only row that flips across the stone.
- **Left binderless, each named:**
  - the child `:user::main` (C-b4);
  - `::service-forms` (its letters are quoted AST, not types);
  - the per-locus copies, whose **bodies** still name a free `T` in `Locus/launch :- [… (Status :- [K V T])]`
    (step 3 retires those copies).

## Findings

- ⛔ **The checker enforces no `defn` binder.** A signature letter that is never declared checks clean,
  and so does a declared letter that is never used. This stone's "exactly the letters it uses" is
  enforced by the macro helper, **not by the language**. That is the spelling root at the `defn` level,
  and it belongs to **C-c** (a type parameter is known from its declaration). C-c's wall should refuse
  both.
- **The brief was wrong about the kwargs witness.** The `$impl` where the transport is lost is the
  **kwargs macro's**, which already declares `:- [T0 T2]` (255.21). It stays open, identical before and
  after (`GrantHandles :- [_]`, `coord … _`), at the D2 arm. That is **C-b5**.
- The start$impl transport rows hold, but do not flip. The spelled `T` already generalised before the
  stone. They pin that the declared binder preserves the behaviour.
- The executor reports running one bare `python3 -c 1` (a no-op) against CLAUDE.md's venv rule. All real
  Python used `run_with_venv.sh`. **Recorded.**
