# STONE HOME-12 — the AST surface gets a registry home

DRAWN 2026-08-27 against `04345f5d9`.
**PRIOR ART:** `git log -1 04345f5d9` (HOME-11 — the same shape, same file, and the degrade-and-restore
row this stone repeats) and `src/intrinsic/edn.rs` (written by it — **your shape**).

## The move — TEN verbs, and every one is a producer

```
accessors     ast-kind · ast-name · ast-span · ast-end-span · ast->children · ast->source
constructors  symbol-node · keyword-node
parse/gensym  read-string · fresh-symbol
                          ->  src/intrinsic/ast.rs
```

**Names are NOT changed.** They stay `:wat::core::ast-kind` etc. This is pure re-registration like
HOME-8/10/11 — **no codemod, no RetirementEntry rows, no `.wat` corpus file. STOP-4.**

⚠ **A rename to `:wat::ast::*` is the eventual clojure-ified form and is NOT this stone.** Measured:
**1,571 corpus sites** outside `docs/` — larger than every codemod this arc has run (Stone F was 433).
It is a builder ruling with a real price tag, and folding it in here would smuggle a 1,571-site
migration under a registration stone. **STOP-6.**

## ⛔ THE ONE CONTRACT DECISION — ALL TEN ARE PRODUCERS

Measured: every one of the ten stamps `Provenance::RuntimeBuilt` in its body (one code site each,
all in `src/edn/render.rs`). They mint AST values and record which verb made them.

**So every handler must return `Result<TrackedValue, EvalBreak>`, not a bare `Value`.** Stone G
(`38f51c9fc`) made this expressible; before it, a registry handler could not carry provenance at all.
Returning a bare `Value` degrades the stamp to `SymbolBound` **silently, with every test green** —
that is what Stone E-iv did to four keyword verbs and what took a whole stone to reverse.

This is a stronger version of HOME-11's risk: there, 3 of 13 were producers. **Here it is 10 of 10.**
There is no "and the rest are plain shims" half to fall back on. **STOP-1.**

## ⛔ SIX NEIGHBOURS THAT LOOK LIKE THEY BELONG AND DO NOT

`macroexpand · macroexpand-1 · quasiquote · struct->form · forms · ann-form` are AST-shaped and sit
in the same dispatch region, but they are **registered SPECIAL FORMS** — `src/special_forms.rs`'s
`REGISTRY: HashMap<String, SpecialFormDef>` holds each with an arity signature (`:221`, `:234`,
`:235`, `:247`, `:248`, `:142`). A special form is a different contract (`#[wat_special_form]`,
`src/intrinsic/special/`), not an intrinsic. **Do not carve them. STOP-3.**

`show-source · render-doc · type-params-used-in · type-equal?` already live in
`src/intrinsic/reflect.rs` — doc/type reflection, a sibling domain. Leave them.

## ★ A FILE-DOMAIN FINDING — REPORT IT, DO NOT ACT ON IT

The ten handlers live in **`src/edn/render.rs`** (5,016 lines), which holds 10 AST handlers beside 7
EDN handlers. **There is no `src/ast/`.** AST accessors filed inside the EDN renderer is a
misfiling, and the fix is a FILE-domain carve — a different deliverable from this one, as HOME-11's
commit records ("home" has meant two things and only the registry kind serves arc 255).

**This stone does not move them.** Register from where they are. If the split looks obvious while
you are in there, say so in the report; do not do it. **STOP-5.**

## Rooms — verified against `04345f5d9`

```
src/runtime.rs               the 10 ":wat::core::…" => arms (dispatch_keyword_head + _value)
src/edn/render.rs            all 10 handlers + their RuntimeBuilt sites
src/intrinsic/edn.rs         ★ THE SHAPE — HOME-11's producer/non-producer split, one commit old
src/intrinsic/keyword.rs     the other producer home, re-stamped by Stone G
src/special_forms.rs:142,221,234,235,247,248   the six that are NOT yours
src/intrinsic/mod.rs         `mod ast;`
src/macros/eval.rs           is_pure_total — MEASURE. `read-string`/`fresh-symbol`/`ast-*` are
                             exactly the verbs a macro body calls at expand time.
src/rete/purity.rs           the ledger scans a union of arms AND #[wat_intrinsic] names; a pure
                             re-registration may need zero edits (HOME-10/11 both measured that).
```

## STOP triggers — each REJECTS

1. **STOP-1 — a producer handler returns a bare `Value`.** All ten are producers. Silent and green.
2. **STOP-2 — you would change a verb's behaviour.** Registration only.
3. **STOP-3 — you would carve one of the six special forms.** Different contract.
4. **STOP-4 — codemod, RetirementEntry row, or `.wat` corpus edit.** Nothing is renamed.
5. **STOP-5 — you would move handlers out of `src/edn/render.rs`.** File carve ≠ this stone.
6. **STOP-6 — you would rename to `:wat::ast::*`.** 1,571 sites; a builder ruling, not a rider's.

## Acceptance

```bash
# 0. ★ THE HOME EXISTS.
ls src/intrinsic/ast.rs
grep -c '#\[wat_intrinsic(' src/intrinsic/ast.rs                       # 10
grep -cE '":wat::core::(ast-|ast->|symbol-node|keyword-node|read-string|fresh-symbol)[^"]*"\s*=>' src/runtime.rs   # 0

# 1. ★ PROVENANCE SURVIVES — degrade-and-restore, per verb-family. A green test proves nothing.
#    Show a value from `:wat::core::symbol-node` rendering RuntimeBuilt{producer …}; break ONE
#    handler to a bare Value; show it fall to SymbolBound; restore; show it back. Paste all three.

# 2. every verb still RUNS, same answers — a scratch-pad probe asserting each of the ten.
# 3. cargo build --release --all-targets   (and note: a clean build is NOT a clean clippy)
```

## Report back with

Row 0 verbatim. **Row 1's three outcomes.** Confirmation that all ten return `TrackedValue`, and the
evidence per verb. What `is_pure_total` needed. Whether the `src/ast/` file split looks warranted —
as a REPORT, not an action. Anything this brief got wrong; what you did NOT do, and why.
