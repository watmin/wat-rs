# BRIEF — Stone 251.5-4.2b: `fix-macro-param-types` — the first fix-wat RULE riding the engine

**Executor:** Shadowdancer (sonnet). **Anchor:** `/home/watmin/work/holon/wat-rs/` (verify `pwd`;
operate ONLY here; `git -C /home/watmin/work/holon/wat-rs`; ignore any `.claude/worktrees/` path).
The RED probe is on disk + verified RED (`tests/probe_arc251_fix_macro_param_types.rs` —
`UnknownFunction :wat::fix::fix-macro-param-types`). Do NOT commit — the Inquisitor weighs.

## The work in one paragraph

Add `:wat::fix::fix-macro-param-types (src) -> migrated-src` to `wat/fix.wat` — the first real
migration RULE, riding the just-shipped `fix-text` engine. A defmacro param/return is annotated with
a type the macro engine DISCARDS; the only honest type is `:wat::WatAST` (a macro arg is always a
form). This rule rewrites, COMMENT-FAITHFULLY, **defmacro forms ONLY**:
- each FIXED param's type   → `:wat::WatAST`
- the REST param's type     → `:wat::core::Vector<wat::WatAST>`
- the RETURN type           → `:wat::WatAST`
It REUSES the engine's spine — `fix-text-apply` (the right-to-left span splice) + `fix-text-offset-of`
+ `fix-text-line-start` (already standalone in fix.wat). Only the EDIT-COLLECTION is new. It must NOT
touch `defn`/`fn` type annotations (those are real types) — defmacro-scoped.

## The algorithm

```
fix-macro-param-types(src):
  lines = (string::split src "\n")
  tree  = (read-string src)
  forms = (ast->children tree)
  edits = macro-param-edits(forms, lines)       ;; NEW collector (below)
  (fix-text-apply src (reverse edits))          ;; REUSE the engine's splice

macro-param-edits(forms, lines):                 ;; map over top-level forms, defmacro → edits, else []
  for each form:
    if (= (ast-kind form) "list") AND (first children) is the keyword :wat::core::defmacro:
       ch = (ast->children form)                  ;; [defmacro-kw, name, (meta?), argvec, ->, rettype, body]
       ;; 6-item: argvec at index 2; 7-item (metadata map at index 2): argvec at index 3
       argvec  = ch[2] if (ast-kind ch[2]) == "vector" else ch[3]
       rettype = the node right AFTER the `->` symbol in ch
       emit: argspec-type-edits(argvec, lines)  ++  one replacement edit for rettype → ":wat::WatAST"
    else: no edits

argspec-type-edits(argvec, lines):               ;; position-aware walk of the argvec children
  walk left-to-right tracking prev-arrow? (prev token was the `<-` symbol) and after-amp? (a `&` seen):
    when prev-arrow? AND (ast-kind tok) == "keyword"  → it is a TYPE SLOT, emit a replacement edit:
        { off: (fix-text-offset-of (ast-span tok) lines),
          old-len: (string::length (ast-name tok)),
          new-text: (if after-amp? ":wat::core::Vector<wat::WatAST>" ":wat::WatAST") }
    set after-amp? when tok is the `&` symbol.
```
Edit shape = `Tuple(off, old-len, new-text)` : `:(i64,i64,String)` — IDENTICAL to `fix-text`'s, so
`fix-text-apply` consumes it directly. Replacement (not deletion): new-text is the canonical type
string. Comments + formatting survive because you splice the original text.

## Read in order (the rooms)

1. `wat/fix.wat:285` (`fix-text-apply`) + `:148` (`fix-text-offset-of`) + `:130` (`fix-text-line-start`)
   — REUSE these verbatim. + `fix-text` (`:310`) for the driver shape (split → read-string →
   collect → reverse → apply).
2. `tests/probe_arc251_fix_macro_param_types.rs` — the gate (defmacro types rewritten + comment
   byte-identical + the sibling `defn`'s real types UNTOUCHED). Make it GREEN.
3. `src/macros/parse.rs:108-109` — the defmacro shape (6-item `[head name argvec -> rettype body]`;
   7-item `[head name meta argvec -> rettype body]`, metadata MAP at index 2).
4. `wat/fix.wat:63` (`arrow?`) — detect the `<-`/`->` annotation arrow symbols (reuse).

## Blast radius

`wat/fix.wat` (the new `fix-macro-param-types` + its `macro-param-edits`/`argspec-type-edits`
helpers). NO Rust change. NO change to `fix-text`/`fix-source`/`fix-text-apply` (reuse them). NO
change to the probe.

## STOP triggers (rejection — ship nothing, report)

1. **STOP-1:** the rule touches a `defn`/`fn` type annotation (the probe's `:user::f` types change) —
   it MUST be defmacro-scoped; report rather than ship a version that clobbers real types.
2. **STOP-2:** a needed reuse (`fix-text-apply` / `fix-text-offset-of`) isn't callable as a standalone
   — report; do NOT duplicate the splice machinery (one engine).
3. **STOP-3:** the rest-param type can't be distinguished from a fixed param (the `&` tracking) —
   report; do not emit `:wat::WatAST` for the rest (it binds a Vector<wat::WatAST>, a different type).

## The gate (report each exact line; do NOT commit)

```
cargo test --release -p wat --test probe_arc251_fix_macro_param_types        # 1 passed
cargo test --release -p wat --test probe_arc251_fix_text_comment_faithful    # 2 passed (engine intact)
cargo test --release -p wat --lib -- --test-threads=1                        # 915 / 36 (zero new)
cargo test --release -p wat --test nursery -- --test-threads=1               # 895 / 4 (zero new)
cargo test --release --workspace --no-run                                    # compiles
```
Run `cargo test` PLAINLY. Stale rust-analyzer diagnostics may contradict a clean build — trust your build.

## Prior comparable (copy the shape)

`fix-text` (the engine you ride — `wat/fix.wat`, shipped `a528147d`) + `fix-seq`/`fix-text-seq-edits`
(the position-aware `prev-arrow?` walk you mirror, adding `after-amp?`).
