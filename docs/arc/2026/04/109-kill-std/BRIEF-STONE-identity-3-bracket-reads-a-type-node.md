# BRIEF — identity 3: `bracket.wat` must read a type NODE, not string-surgery its name

⚠ **This brief covers ONE of the three "one-offs" B-3 grouped as stone 3.** Grounding them showed
they are three unrelated mechanisms with three different destinations, and **two of the three are
UNRULED** — see the tail of this file. Only the `bracket.wat` one has a settled destination, so only
it is briefed.

## The work

`wat/bracket.wat:512-518` derives an `Address` type from a `Peer` type by **substring surgery on a
keyword's rendered name**:

```clojure
c-ty   (:wat::core::nth arg-ch 2)                    ;; the 1st param's TYPE node
c-nm   (:wat::core::ast-name c-ty)                   ;; ":wat::kernel::Peer<S,R>"
addr   (:wat::core::string::join "Address"
         (:wat::core::string::split c-nm "Peer"))    ;; ":wat::kernel::Address<S,R>"
addr-b (:wat::core::string::subs addr 1 (:wat::core::string::length addr))
```

`ast-name` requires a Symbol, Keyword or StringLit. Once a caller writes the `:-` spelling —
`(:wat::kernel::Peer :- [S R])` — `c-ty` is a **List**, `ast-name` raises, and the macro dies at a
site that has nothing to do with the caller's mistake.

**Teach it to read both node shapes**: keep the `Peer`→`Address` name transform, but apply it to the
type's HEAD and preserve the arguments, whatever spelling the node wears.

## Why this one and not the other two

`bracket.wat` **consumes** a type the caller wrote. Its destination is settled: read what is there.
The other two **emit** type names, and what they should emit is not yet ruled.

## What "done" looks like

1. A caller passing `[p <- :wat::kernel::Peer<S,R> …]` still works — the angle spelling is untouched.
2. ★ A caller passing `[p <- (:wat::kernel::Peer :- [S R]) …]` works, deriving the same `Address`.
3. The derived `Address` carries the SAME arguments as the `Peer` it came from. A version that
   derives the base name and drops `<S,R>` would pass a `--check` and break the dial at runtime —
   prove the args survive, do not assume it.
4. `arity == 6` (the DIAL path) is the one that reaches this code. Exercise it, not just the macro's
   other arms.

## Boundaries

- Do NOT run `scripts/floor.sh` or a full `cargo nextest` — the orchestrator measures centrally.
  ⚠ A scoped run is not the floor: on a recent stone `binary_id(wat::services)` was 128/128 green
  while the floor was red by six.
- Do NOT commit, push, stash, revert or amend. Leave everything in the working tree.
- Touch `wat/bracket.wat` only.
- Do NOT change `wat/core.wat`'s `{b}::Kwargs{p}` / `:{b}$impl{p}`, and do NOT change
  `wat/fix.wat`'s replacement text. Both are unruled; see below.
- Write no new base-extraction helper — `ast-kind`, `ast->children` and the existing string verbs
  are enough to branch on the node shape.

## Your own checks

`cargo build --bin wat`, then `target/debug/wat --check` and RUN files under
`wat-scripts/scratch-pad/`. Row 3 needs the derived type observed, not merely accepted — a
`macroexpand` of the bracket form shows what was derived. Prefix long commands with
`systemd-run --user --scope -q -p MemoryMax=16G -p MemorySwapMax=0 timeout 900`.
Diagnostics go to **stderr** — judge by exit code AND empty output, never grep alone.

Delete any scratch `.wat` that must fail; `tests/lint/wat_scripts_fixes_load.rs` type-checks
everything under `wat-scripts/`.

## STOP triggers — ship nothing further and report

- **STOP-1.** If row 3 fails — the args are dropped from the derived `Address` — STOP and report.
  Deriving the right base with the wrong arguments is worse than raising, because it type-checks.
- **STOP-2.** If reading the node shape requires a verb `bracket.wat` cannot call at its position in
  the load order (it predates `string.wat`; the existing code says so at :516), STOP and report which
  verb and where it loads.
- **STOP-3.** If the `Peer`→`Address` transform turns out to have other callers or other source
  spellings than the one at :514, STOP and report them. One site is what this brief assumes.

## ⛔ THE OTHER TWO ONE-OFFS ARE UNRULED — do not touch them

**`wat/core.wat`'s `{b}::Kwargs{p}` (:835) and `:{b}$impl{p}` (:949).** Both are DECLARATION NAMES
that `defn` EMITS with the angle spelling. α ruled that declarators *accept* `name :- [T …]`; nothing
has ruled that generated code should *emit* it. This is the same open question as `defservice`'s
DECL-NAME bindings, which stones 2b and 2c deliberately left alone.

**`wat/fix.wat:502`'s replacement text** (`":wat::core::Vector<wat::WatAST>"`, inside the live
`argspec-type-edits-walk`, called at :514 and :558). This is a codemod helper that WRITES a type
name into source files. Whether a codemod emits the destination grammar — or stays frozen as the
historical migration it performed — is a ruling nobody has made.
