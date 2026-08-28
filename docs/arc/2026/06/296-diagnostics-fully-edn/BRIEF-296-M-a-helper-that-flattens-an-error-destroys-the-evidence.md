# STONE 296-M — a helper that flattens an error destroys the evidence

DRAWN 2026-08-27 against `4b49f3c5c`.
**PRIOR ART — read first:** `git show 4ed3c711e` (Stone L Phase 1: `assert_startup_error!`, which
this stone makes usable across 71 more helpers) and `git log -1 4b49f3c5c` (Phase 2, incl. the
`no_inlined_edn` rune-scope trap and the read-vs-measure failure).

**Builder's ruling, 2026-08-27:** *"we do not leave known flaws - fix it."* Stone L's orchestrator
proposed fixing only the 13 live-exposure helpers and leaving 52 latent. That was refused. **All of
them.**

## The defect

```rust
fn eval_probe(defn: &str, call: &str) -> Result<Value, String> {
    let w = startup_from_source(...).map_err(|e| format!("startup (type-check): {e:?}"))?;
    let ast = wat::parse_one!(call).map_err(|e| format!("parse: {e:?}"))?;
    eval_in_frozen(&ast, &w, &Environment::new()).map_err(|e| format!("eval: {e:?}"))
}
```

**The discriminant is destroyed before any assertion can see it.** On a positive assertion that is
harmless. On a negative one it is fatal: the type is `String`, so there is nothing left to name, and
Stone L's `assert_startup_error!` cannot reach it. The only moves left to an author are bypass the
helper or take a `rune:lint(bare-is-err)` exemption — so **this shape manufactures exemption
pressure whose real cause is a signature.** Do not eliminate the failure; eliminate the situation
that produces it.

## ⛔ THE ONE CONTRACT DECISION — no new type; `StartupError` IS the union

These helpers do not flatten out of laziness. They chain **three different error types** through one
`?` — `StartupError`, `ParseError`, `RuntimeError` — and `String` is the only type all three share.
That is a missing union type, and **it is already on disk**:

```
StartupError = Parse(ParseError) | Config | Load | Macro(MacroError) | Type(TypeError)
             | Resolve(ResolveError) | Check(CheckErrors) | Validator | Runtime(Box<RuntimeError>)
             | Stdlib | SigmaFn | …
```

So every helper returns **`Result<T, StartupError>`**, and each step wraps into its REAL variant
instead of a `format!`:

```rust
fn eval_probe(defn: &str, call: &str) -> Result<Value, StartupError> {
    let w = startup_from_source(...)?;                                     // already StartupError
    let ast = wat::parse_one!(call).map_err(StartupError::Parse)?;
    eval_in_frozen(&ast, &w, &Environment::new())
        .map_err(|e| StartupError::Runtime(Box::new(e)))
        .map(|tv| tv.value_owned())
}
```

**Mint no new error type.** `StartupError`'s `Debug` is already EDN (296 Stone B), so `.expect(msg)`
at a call site prints strictly MORE than the old `format!` string did. **STOP-1** is minting a type.

## The population — 71 helpers across 65 files

```
types 19 · resolve 18 · rete 15 · reflection 5 · macros 5 · program 4 · collection 3
function 1 · diagnostics 1
```

Only ~10 have their `Err` arm inspected today; the rest `.expect(...)`. **That is not a reason to
skip them** — the builder ruled on the class, and a positive-only helper is a trap the next negative
test falls into. Migrate all 71.

## Phase order — mirrors Stone L, and for the same reason

```
PHASE 1   nothing to build. assert_startup_error! and StartupError both already exist.
PHASE 2   migrate 71 helpers -> 0.  Fanned by directory.
PHASE 3   the wall: tests/lint/no_error_flattening_helper.rs, landing on an ALREADY-ZERO tree.
```

Phase 3 bans `fn … -> Result<_, String>` whose body `map_err`s a typed error into a `format!`, in
`tests/`. It lands last so it never shows a red on main. **STOP-4** is landing it on a non-zero tree.

## Rooms

```
src/freeze.rs  `pub enum StartupError`     the union; read its variants before wrapping
src/lib.rs     `assert_startup_error!`     Stone L Phase 1 — what these helpers become reachable by
tests/rete/probe_arc278_seq1b_list_hofs.rs `eval_probe` — the canonical three-error-type chain
tests/function/stone18a.rs:34              `try_startup` — the one the builder pointed at
tests/lint/no_loose_string_assert.rs       the lint shape for Phase 3
```

## Method

A call site that only wanted the string still gets one: `.map_err(|e| format!("{e:?}"))` **at the
call site**, where it is visible, rather than baked into the helper where it is invisible. Prefer
letting `.expect()` print the typed Debug — it is EDN and strictly better.

Where a helper's `Ok` type or arity must change, keep the change inside your directory. If a helper
is `pub(super)`/`pub(crate)` and crosses directories, that is **STOP-3** — report it, do not reach.

## Your role

cwd `/home/john/work/holon/wat-rs`; `pwd` first. **Ending your turn ENDS you** — every command
FOREGROUND. **No sub-agents. No worktrees.** Do not commit, push, revert, `git checkout`, or run
`git stash` in any form.

**Do not run cargo.** Sibling riders are editing this workspace concurrently; the build is a serial
resource and the orchestrator compiles and weighs centrally. Ground with
`./target/release/wat --check` (no build lock) and by reading the error enums' source. Your guards
and signatures are compile-checked centrally — be precise, and mirror `4ed3c711e`'s worked shapes.

## STOP triggers — each REJECTS

1. **STOP-1 — you would mint a new error type or an enum wrapper.** `StartupError` is the union.
2. **STOP-2 — a helper whose steps genuinely cannot map into any `StartupError` variant.** Report
   the helper, the step, and the error type. Do not force it and do not fall back to `String`.
3. **STOP-3 — a helper visible outside your directory.** Report it; the orchestrator sequences it.
4. **STOP-4 — you would write the Phase 3 lint.** Not this phase.
5. **STOP-5 — you would change a helper's behaviour.** Signatures and error plumbing only; if a
   test's assertion must change to compile, that is fine, but no test's MEANING may change.

## Acceptance

```bash
# 1. your directories report zero flattening helpers (the committed census, extended).
python3 docs/arc/2026/06/296-diagnostics-fully-edn/PROBE-296-M-flattening-helper-census.py

# 2. no new error type was minted.
git diff --stat -- src/            # must be EMPTY

# 3. the loose-assert and bare-is-err positions did not regress: no `.contains()` escapes added.
grep -rn 'rune:lint' <your directories>     # list every exemption you took, with its reason
```

## Report back with

Every helper migrated: its file, its old and new signature, and which `StartupError` variant each
step wraps into. Every STOP finding in full. Every call site whose `.expect` message changed, and
what it prints now. Anything this brief got wrong. What you did NOT do, and why.
