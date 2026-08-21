# BRIEF — ②-ii: write the codemod. DRY-RUN ONLY — no corpus file is modified in this stone.

Design: `DESIGN-STONE-2-the-codemod.md`. `②-i` (`0422b67ff`) shipped the renderer this needs.
**This stone writes the migration script and proves it on a `/tmp` copy. It applies nothing.**
②-iii applies it to `wat/`; ②-iv to the rest.

**Your role: you write the text. The orchestrator builds, floors, and commits.** Do NOT run `cargo`.
✅ **`./target/release/wat` IS current and IS your instrument here** — it contains `②-i`'s renderer, so
you can run the codemod against `/tmp` copies and read real output. Use it constantly. Foreground
everything; ending your turn ends you. Do not commit, push, stash, or revert, and do not touch any
file under `wat/`, `tests/` or `wat-scripts/` other than the ONE new fix script.

## The work

Write `wat-scripts/fixes/parametrics-take-a-type-vector.wat`, a self-hosted wat-fix codemod that
rewrites every type-shaped keyword into the bracketed form, keeping the rust-ish head:

```
:wat::core::Vector<wat::core::i64>            →  (:wat::core::Vector [:wat::core::i64])
:wat::core::HashMap<wat::core::String,…i64>   →  (:wat::core::HashMap [:wat::core::String :wat::core::i64])
:wat::core::Vector<wat::core::Vector<…>>      →  nests
```

## The shape to copy

**`wat-scripts/fixes/to-faithful-clojure-rete.wat`** — it is the established pattern for a
form-level migration that carries its OWN walk. Its own comment (`:126`): *"Mirrors
`:wat::fix::fix-text-seq-edits`: threads prev-arrow? across siblings, recurses structural nodes
(resetting prev-arrow? to false, exactly as fix-text-struct-edits does)."*

⚠ **Do NOT modify `wat/fix.wat`.** Its edit trio is internal and serves the faithful-Clojure drives;
changing it puts every existing migration at risk for no gain. Reuse only its PUBLIC helpers —
`:wat::fix::fix-text-apply`, `:wat::fix::fix-text-offset-of`, `:wat::fix::structural?`,
`:wat::fix::type-shaped-keyword?`, `:wat::fix::arrow?` — and carry the walk locally.

The file/stdin harness is identical in every recorded migration; copy it from
`wat-scripts/fixes/rename-wat-record-to-core-record.wat` (`:user::migrate` / `:user::apply-each` /
`:user::main` reading one EDN vector of paths from stdin).

## ⛔⛔ TWO RULES ONLY — and the third one is a trap

`wat/fix.wat`'s `fix-text-leaf-edits` applies **three** rules. You want exactly the first two:

1. **post-arrow keyword** (`prev-arrow?` true) → a type annotation → rewrite ✅
2. **`type-shaped-keyword?`** → a parametric/tuple keyword anywhere → rewrite ✅
3. ~~`head-keyword?` → Clojure symbol~~ ⛔ **DO NOT.** That is the namespace flip, arc 300's drive,
   and it is explicitly out of scope. Step ② changes the SHAPE of the type-arg group and nothing else.

## ⛔⛔ USE `ast->source`, NEVER `write-forms` — this cost real time today

`fix-text-leaf-edits` renders its replacement with `:wat::core::write-forms`. **That is wrong for this
codemod.** `write-forms` routes through `Carriage::Display`, whose `Keyword` arm re-spells EVERY
`::`-keyword into the EDN-dotted form:

```
ast->source   (:wat::core::HashMap [:wat::core::String :wat::core::i64])   ← what you must emit
write-forms   (:wat.core/HashMap [:wat.core/String :wat.core/i64])         ← a THIRD spelling
```

Both parse, so nothing screams — the corpus would silently land in a spelling that is neither where it
is nor where it is going. **The converter is `:wat::core::keyword/to-type-form-colon`; the printer is
`:wat::core::ast->source`.** Verified live: `wat-scripts/scratch-pad/arc109-2i-colon-mode-verbatim-probe.wat`
shows both printers side by side — run it first and read the difference before you write a line.

## Prove it on `/tmp`, and prove these four things

Copy a handful of real corpus files to `/tmp`, run the codemod on the copies, and `diff`. Choose files
that between them exercise:

| # | property | why it is the one that breaks |
|---|---|---|
| 1 | a nested parametric | `Vector<Vector<i64>>` must nest, not flatten |
| 2 | a **primed** head `Peer'<I,O>` | 29 corpus sites; `'` before `<` is legal and must survive |
| 3 | a file containing `<-` `->` `<` `<=` `>` | **9,912 corpus sites must not move.** The diff must show ZERO changes to any of them |
| 4 | **IDEMPOTENCE** | run the codemod TWICE on the same copy; the second run must produce a byte-identical file |

⚠ Property 3 is the one that turns a good codemod into a catastrophe if wrong. Show it explicitly in
your report: a diff hunk count, and the confirmation that no arrow or comparison operator appears in
any hunk.

## Blast radius

ONE new file: `wat-scripts/fixes/parametrics-take-a-type-vector.wat`. Nothing else in the repo.
Your `/tmp` copies are scratch and are not part of the deliverable.

⚠ The new file lands under `wat-scripts/`, which the `every_wat_scripts_file_loads` gate parses and
type-checks on every floor. It must be valid, loadable wat — not a sketch.

## STOP triggers — each rejects; none is a fallback

1. The rewrite touches an arrow or a comparison operator. STOP — that is the catastrophic class.
2. A primed head `Peer'<I,O>` does not survive. STOP; report what it became.
3. Running twice is not idempotent. STOP; report the second run's diff.
4. You need to modify `wat/fix.wat`, or any file under `wat/`, `tests/`, `wat-scripts/fixes/*` other
   than your new one. STOP.

## Acceptance criteria

- One new loadable `wat-scripts/fixes/parametrics-take-a-type-vector.wat`.
- Demonstrated on `/tmp` copies with a `diff` for each of the four properties above.
- Emits `(:wat::core::Head [args])` — colon-quoted head, bracketed args, verbatim via `ast->source`.
- No call head converted to a symbol; no arrow touched.
- Idempotent: second run is a byte-identical no-op.
- **Nothing in the repo modified except the one new file.**
