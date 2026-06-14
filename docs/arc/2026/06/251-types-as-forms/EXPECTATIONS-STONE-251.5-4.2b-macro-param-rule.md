# EXPECTATIONS — Stone 251.5-4.2b: `fix-macro-param-types`

Scorecard fixed BEFORE the strike. The Inquisitor scores against its OWN re-run, reads the diff,
credits nothing the disk doesn't show.

## Scorecard

| # | what | command | expected |
|---|---|---|---|
| 1 | the gate: defmacro types rewritten + comment byte-identical + defn UNTOUCHED | `cargo test --release -p wat --test probe_arc251_fix_macro_param_types` | `1 passed` |
| 2 | the fix-text engine still intact (reuse didn't break it) | `cargo test --release -p wat --test probe_arc251_fix_text_comment_faithful` | `2 passed` |
| 3 | lib baseline | `cargo test --release -p wat --lib -- --test-threads=1` | `915 / 36` (zero new) |
| 4 | nursery baseline | `cargo test --release -p wat --test nursery -- --test-threads=1` | `895 / 4` (zero new) |
| 5 | compiles | `cargo test --release --workspace --no-run` | exit 0 |
| 6 | pure-wat, engine reused | `git -C . diff --stat` | `wat/fix.wat` only (the new rule; NO change to fix-text-apply/fix-source) |

## Inquisitor's own additional weigh

- **Read the diff**: confirm `fix-macro-param-types` REUSES `fix-text-apply` + `fix-text-offset-of`
  (does not re-implement the splice — the "one engine" invariant), and that the new edits are
  REPLACEMENTS (canonical type string), not deletions.
- **defmacro-scoping, hard**: the probe's `defn :user::f` types must be byte-identical in the output.
  This is the load-bearing scoping proof — a rule that clobbers real `defn`/`fn` types is a reject.
- **Idempotence (in-session)**: run the rule on its own output → byte-identical (an already-canonical
  defmacro yields zero edits).

## Runtime prediction

15–25 min. The splice spine is reused; the new work is the defmacro-detection + the position-aware
argspec walk (`prev-arrow?` + `after-amp?`) emitting replacement edits + the rettype edit. The 6-vs-7
item shape + the rest-param `&` tracking are the fiddly bits.

## Trap-doors named

- **Scope creep into defn/fn** — the rule must fire ONLY inside `:wat::core::defmacro` forms; a
  position-walk that doesn't gate on the defmacro head would rewrite real types (STOP-1).
- **Rest-param type** — `& rest <- :T` → `:wat::core::Vector<wat::WatAST>`, NOT `:wat::WatAST`
  (it binds a Vec of forms); requires the `after-amp?` tracker (STOP-3).
- **6 vs 7 item** — a metadata-map at index 2 shifts the argvec to index 3; detect via
  `(ast-kind ch[2]) == "vector"` vs `"map"`.
- **char vs byte** — reuse `fix-text-offset-of` (already char-based); stay in char space.
- **Replacement extent** — the edit replaces the TYPE keyword token only (its `ast-name` char-len);
  the `<-`/`->` arrows and names stay.

## Out of scope (affirmatively cut — NOT deferred silently)

- **The corpus RUN** (applying `fix-macro-param-types` over the ~16 real macro files) — the NEXT step
  once the rule is proven; this stone is the rule + its fixture proof only.
- **The ENFORCE validator** (queue item b) — lands after the corpus conforms.
- **The 251 clojure cutover** (fix-source's rules over the corpus) — later; this rule is
  rust-scheme-internal and orthogonal.
