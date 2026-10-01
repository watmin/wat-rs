;; wat-scripts/fixes/typed-constructors.wat — arc 255, STONE 255.69.
;; SCOPE: corpus
;; rune:replay(unreadable-preimage) — this codemod's transformation is a function of TWO
;;   inputs, not one: the file's own text, AND a per-site type table sourced externally from
;;   `WAT_CHECK_TYPES=1 wat --check` (item 2's own rule: "the table is an input, not a
;;   committed artifact"), supplied at run time via the `.typed-constructors-table.edn`
;;   side-channel this file's `:user::main` reads. `every_recorded_migration_replays.rs`'s
;;   harness runs a stem's own codemod with `current_dir(manifest())` and NO such file present
;;   (it is deliberately never committed) — `:user::main` would fail on the read before a
;;   single site could even be considered, so no `before.pre`/`after.post` pair is a readable
;;   preimage this replay mechanism can drive: committing one would mean committing its
;;   (explicitly non-committed) type-table input too. Item 1's run-time behavior is covered
;;   instead by `tests/function/probe_stone255_69_wat_type_scalar_dispatch.rs`; this codemod's
;;   OWN correctness is covered by the corpus-scale evidence in this stone's SCORE (independent
;;   census re-derivation, dry-run before/after `--check` parity, `scripts/replay/{census,
;;   delta}.sh`, and idempotence over all 362 converted files) — a genuine before/after fixture
;;   pair, not a canned one this harness could exercise.
;; Self-hosted fix-wat codemod: no hand-editing of .wat files — use the tool.
;;
;; BRIEF: docs/arc/2026/06/255-builtin-registry/BRIEF-STONE-255.69-typed-constructors.md
;; MEASURED BY: docs/arc/2026/06/255-builtin-registry/{WEIGH,SCORE}-STONE-255.68-*.md — 1,804
;;   untyped constructor calls, joined to the checker's own type record by exact file:line:col;
;;   1,627 concrete + 3 generic = 1,630 codemod-able, 33 unresolved + 141 not-checked left alone.
;;
;; Converts every UNTYPED constructor call this stone's type table has a concrete or generic
;; type for — `(:wat::core::PersistentVector a b)` (or the already-`wat.type/`-headed spelling)
;; → `(wat.type/PersistentVector :- [<elem>] a b)`, same for `Vector`/`List`/`HashSet`
;; (`:- [<elem>]`), `HashMap`/`PersistentMap` (`:- [<key> <val>]`), `Tuple` (`:- [<slot>…]`,
;; the compact `:(A,B)` recorded form desugared upstream — see below). A hit is a List node
;; whose head is one of the five constructor names (either spelling) where `items[1]` is not
;; already the `:-` binder — empty calls included.
;;
;; ══ WHERE THE TYPE COMES FROM (the table, an INPUT — never committed) ═══════════════════════
;; The type is NOT looked up here: it is supplied, per (file, line, col) of the call's own List
;; span, by a table built OUTSIDE this file from `WAT_CHECK_TYPES=1 wat --check` (item 2 of the
;; brief) — TD, the checker decides the element type, this codemod only SPELLS and SPLICES it.
;; The table arrives as ONE combined stdin datum, `{"paths": […], "table": {file: {"L:C": ty}}}`
;; (`:c69::drive`'s shape below) — never as a file this codemod reads by path, so the table stays
;; an input, not a committed artifact.
;;
;; The table's `ty` text is the checker's own recorded type, already bridged from its DISPLAY
;; notation into parseable wat source by a text-only preprocessing step (no type-name judgment):
;; the compact tuple form `:(A,B)` (confirmed NOT reader-parseable — `read-string` on it refuses
;; with "comma inside keyword body retired... use the `:-` binder form") is rewritten to
;; `(:wat::core::Tuple :- [A B])`, recursively (a tuple can nest inside another type or inside
;; another tuple, where `format_type_inner`'s own colon-STRIPPED convention additionally loses
;; the leading colon this bridge restores on every namespaced leaf). This is a NOTATION bridge
;; only — it never decides whether a name is one of the 24 hard primitives; every actual
;; type-NAME decision below runs through the same door 255.67's codemod used.
;;
;; ══ SPELLING (item 3's rule; NOT string surgery) ══════════════════════════════════════════════
;; Every keyword LEAF inside the table's (bridged, now-parseable) type fragment is walked
;; unconditionally — the whole fragment IS type syntax, so unlike `types-to-wat-type.wat`'s
;; corpus-wide walk (which must first decide "is this position a type"), no position-context
;; threading is needed here:
;;   - one of the 24 hard primitives (`:t2wt::target-names`, copied VERBATIM from
;;     `types-to-wat-type.wat` — the SAME closed set, so this codemod can never drift from what
;;     that accepted one calls "the same head") → `(:wat::keyword::to-type-form leaf)`, the ONE
;;     canonical door, exactly as stone 255.67 used it.
;;   - a bare keyword with no `::` in its body (a rigid type-parameter var recorded AS a
;;     colon-keyword by the checker's renderer, e.g. `:T`) → the colon is stripped and it is
;;     re-emitted as a bare SYMBOL (`T`) — item 3's "for a generic site, write the definition's
;;     own parameter."
;;   - every other keyword (a non-primitive namespaced type, `:wat::gen::Gen`) → left EXACTLY as
;;     recorded — "every other type as it is spelled today."
;; The call's own HEAD (in the CORPUS file being edited, not the table) is converted the same
;; way, through the same door, directly: a `:wat::core::X` keyword head becomes the
;; `to-type-form`-produced `wat.type/X` symbol; a head already spelled `wat.type/X` (post-255.67
;; corpus sites) is left unchanged.
;;
;; ══ THE EDIT ═══════════════════════════════════════════════════════════════════════════════
;; One point edit per hit: the call's own HEAD SPAN (its own token, nothing else) is replaced
;; with `<converted-head> :- [<converted-args>]` — `(:wat::core::PersistentVector a b)` becomes
;; `(wat.type/PersistentVector :- [wat.type/i64] a b)`; the empty
;; `(:wat::core::PersistentVector)` becomes `(wat.type/PersistentVector :- [wat.type/i64])`.
;; Span-faithful via `wat/fix.wat`'s `fix-text-offset-of` / `fix-text-apply` (old-text is the
;; head's own `ast-name`, matched byte-for-byte before the replace commits — `source-matches-
;; name?` skips a reader-synthesized head rather than mis-editing it, mirroring
;; `types-to-wat-type.wat`'s class-B guard).
;;
;; A site the table has NO entry for (unresolved — the checker's own type stayed a variable — or
;; not-checked — inside `quote`/a macro template/a `forms` literal/a file that doesn't freeze) is
;; left completely untouched: the wall enforcing "every constructor call names its type" is a
;; LATER stone, and these are its heretics (listed in this stone's SCORE).
;;
;; Usage — dry-run on a /tmp copy first, diff, THEN apply. The driver reads ONE combined JSON/EDN
;; datum from stdin: `{"paths": ["pathA", …], "table": {"pathA": {"12:5": "(:wat::core::… )"}}}`
;; — `paths` lists EVERY file THIS invocation should convert (a subset of the table's own keys is
;; fine; an entry in `table` for a file not in `paths` is simply unused, which is how the stdlib-
;; first / rest-second two-pass drive of one build shares a single table).
;;
;; Idempotent: a converted site's head is already `wat.type/X` with `items[1]` already `:-`, so
;; `is-hit?` no longer matches it — a re-run finds nothing left to convert for that site.

;; ── the 24 hard primitives (Stone 255.67's closed set) — copied verbatim, not re-derived, so
;;    this codemod can never name a different "same head" than the accepted 255.67 one.
(:wat::core::defn :t2wt::target-names [] -> (wat.type/HashSet :- [wat.type/String])
  (wat.type/HashSet :- [wat.type/String]
    ":wat::core::i64" ":wat::core::f64" ":wat::core::u8" ":wat::core::bigint"
    ":wat::core::rational" ":wat::core::char" ":wat::core::String" ":wat::core::bool"
    ":wat::core::keyword" ":wat::core::nil" ":wat::core::Value" ":wat::core::Never"
    ":wat::core::Fn" ":wat::core::Record" ":wat::core::Struct" ":wat::core::Vector"
    ":wat::core::HashMap" ":wat::core::HashSet" ":wat::core::List" ":wat::core::Tuple"
    ":wat::core::PersistentVector" ":wat::core::PersistentMap" ":wat::core::Bytes"
    ":wat::WatAST"))

(:wat::core::defn :t2wt::target-name? [name <- wat.type/String] -> wat.type/bool
  (:wat::core::contains? (:t2wt::target-names) name))

;; the five constructor call heads, either spelling.
(:wat::core::defn :c69::head-names [] -> (wat.type/HashSet :- [wat.type/String])
  (wat.type/HashSet :- [wat.type/String]
    ":wat::core::PersistentVector" "wat.type/PersistentVector"
    ":wat::core::Tuple" "wat.type/Tuple"
    ":wat::core::PersistentMap" "wat.type/PersistentMap"
    ":wat::core::List" "wat.type/List"
    ":wat::core::Vector" "wat.type/Vector"))

(:wat::core::defn :c69::item-name [n <- wat.type/AST] -> wat.type/String
  (:wat::core::let [k (:wat::core::ast-kind n)]
    (:wat::core::if (:wat::core::or (:wat::core::= k "keyword") (:wat::core::= k "symbol"))
      (:wat::core::ast-name n)
      "")))

;; is-hit? — a genuine untyped constructor-call List (SCORE-STONE-255.68's census rule).
(:wat::core::defn :c69::is-hit? [node <- wat.type/AST] -> wat.type/bool
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind node) "list")
    (:wat::core::let [ch (:wat::core::ast->children node)]
      (:wat::core::if (:wat::core::empty? ch)
        false
        (:wat::core::let [head  (:wat::core::first ch)
                          hname (:c69::item-name head)]
          (:wat::core::if (:wat::core::contains? (:c69::head-names) hname)
            (:wat::core::if (:wat::core::< (:wat::core::length ch) 2)
              true
              (:wat::core::not (:wat::core::= (:c69::item-name (:wat::core::nth ch 1)) ":-")))
            false))))
    false))

;; table-key — "L:C" for the call node's OWN span (its `(`) — how the table (built from the
;; checker's type record, which notes the same call-expression's own span) is keyed.
(:wat::core::defn :c69::table-key [node <- wat.type/AST] -> wat.type/String
  (:wat::string::concat
    (:wat::core::str (:wat::fix::span-line node))
    (:wat::string::concat ":" (:wat::core::str (:wat::fix::span-col node)))))

;; leaf-edit — the point edit for ONE keyword leaf found while walking an args fragment, or no
;; edit at all for a keyword that needs no conversion. item 3's spelling rule for "every other
;; type": left BYTE-IDENTICAL via a span-faithful edit (`wat/fix.wat`'s `fix-text-offset-of`/
;; `fix-text-apply`, exactly `types-to-wat-type.wat`'s own mechanism) — NEVER round-tripped
;; through `write-forms`, which renders an unchanged keyword in its own normalized dotted
;; spelling (`:wat.gen/Gen`), not the corpus's literal `::`-colon text (confirmed empirically
;; dry-running against `wat/gen.wat`'s `952:25` site before this fix). This "no edit" arm is
;; also what correctly leaves a rigid type-parameter var (`:T`, recorded by the checker AS a
;; colon-keyword — Case 4 of `type_expr_to_clojure_form`'s doc: "a type-var was never
;; colon-spelled in any spelling") untouched: `(wat.type/PersistentVector :- [:T] …)` checks and
;; runs (confirmed empirically); a BARE symbol `T` in this position does NOT — `#wat.check/
;; MalformedForm {:reason "bracketed type must be a type keyword"}`, `parse_bracket_type_keyword`
;; (`src/check.rs`) requires a `WatAST::Keyword`, never a `Symbol`, in a constructor's own `:-`
;; bracket — so item 3's "write the definition's own parameter" means literally `:T`, not `T`.
;; The SAME "no edit" arm is what leaves `:-`/`:->` alone too — neither is one of the 24, so
;; there is nothing left for a separate marker guard to do.
(:wat::core::defn :c69::leaf-edit
  [node  <- wat.type/AST
   lines <- (wat.type/Vector :- [wat.type/String])]
  -> (wat.type/Vector :- [(wat.type/Tuple :- [wat.type/i64 wat.type/String wat.type/String])])
  (:wat::core::let [name (:wat::core::ast-name node)]
    (:wat::core::if (:t2wt::target-name? name)
      (:c69::point-edit node (:wat::core::write-forms (:wat::keyword::to-type-form node)) lines)
      (:wat::fix::empty-edits))))

(:wat::core::defn :c69::point-edit
  [node     <- wat.type/AST
   new-text <- wat.type/String
   lines    <- (wat.type/Vector :- [wat.type/String])]
  -> (wat.type/Vector :- [(wat.type/Tuple :- [wat.type/i64 wat.type/String wat.type/String])])
  (:wat::core::let [off (:wat::fix::fix-text-offset-of (:wat::core::ast-span node) lines)]
    (wat.type/Vector :- [(wat.type/Tuple :- [wat.type/i64 wat.type/String wat.type/String])]
      (wat.type/Tuple :- [wat.type/i64 wat.type/String wat.type/String] off (:wat::core::ast-name node) new-text))))

;; collect-edits / collect-edits-seq — every keyword leaf under a type-fragment node is a type
;; position (no marker/position threading needed, unlike `types-to-wat-type.wat`'s corpus-wide
;; walk — the WHOLE fragment IS type syntax); `:-` and the fragment's own HEAD are never visited
;; (the caller starts this walk at the args-VECTOR node only, never the fragment's head/marker).
(:wat::core::defn :c69::collect-edits
  [node  <- wat.type/AST
   lines <- (wat.type/Vector :- [wat.type/String])]
  -> (wat.type/Vector :- [(wat.type/Tuple :- [wat.type/i64 wat.type/String wat.type/String])])
  (:wat::core::if (:wat::fix::structural? node)
    (:c69::collect-edits-seq (:wat::core::ast->children node) lines)
    (:wat::core::if (:wat::core::= (:wat::core::ast-kind node) "keyword")
      (:c69::leaf-edit node lines)
      (:wat::fix::empty-edits))))

(:wat::core::defn :c69::collect-edits-seq
  [items <- (wat.type/Vector :- [wat.type/AST])
   lines <- (wat.type/Vector :- [wat.type/String])]
  -> (wat.type/Vector :- [(wat.type/Tuple :- [wat.type/i64 wat.type/String wat.type/String])])
  (:wat::core::if (:wat::core::empty? items)
    (:wat::fix::empty-edits)
    (:wat::core::concat
      (:c69::collect-edits (:wat::core::first items) lines)
      (:c69::collect-edits-seq (:wat::core::rest items) lines))))

;; rebase-edit — an edit's offset, given ABSOLUTE to the fragment text, rebased relative to the
;; args-vector's OWN start offset (so it can be applied to the SLICED args substring alone).
(:wat::core::defn :c69::rebase-edit
  [edit  <- (wat.type/Tuple :- [wat.type/i64 wat.type/String wat.type/String])
   base  <- wat.type/i64]
  -> (wat.type/Tuple :- [wat.type/i64 wat.type/String wat.type/String])
  (wat.type/Tuple :- [wat.type/i64 wat.type/String wat.type/String]
    (:wat::core::- (:wat::core::first edit) base)
    (:wat::core::second edit)
    (:wat::core::third edit)))

;; args-text — the fragment's own args-VECTOR span, re-spelled: every eligible keyword leaf
;; inside it is edited span-faithfully; everything else (including a non-24 type name like
;; `:wat::gen::Gen`, and any nested `[...]`/`(... :- [...])` structure) is BYTE-IDENTICAL to the
;; checker's own recorded text.
(:wat::core::defn :c69::args-text
  [fragment <- wat.type/AST
   frag-src <- wat.type/String]
  -> wat.type/String
  (:wat::core::let
    [frag-lines (:wat::string::split frag-src "\n")
     args-node  (:wat::core::nth (:wat::core::ast->children fragment) 2)
     base       (:wat::fix::fix-text-offset-of (:wat::core::ast-span args-node) frag-lines)
     args-src   (:wat::core::ast->source args-node)
     abs-edits  (:c69::collect-edits args-node frag-lines)
     rel-edits  (:wat::core::mapv
                  (:wat::core::fn [e <- (wat.type/Tuple :- [wat.type/i64 wat.type/String wat.type/String])]
                    -> (wat.type/Tuple :- [wat.type/i64 wat.type/String wat.type/String])
                    (:c69::rebase-edit e base))
                  abs-edits)
     sorted     (:wat::core::sort
                  (:wat::core::fn [a <- (wat.type/Tuple :- [wat.type/i64 wat.type/String wat.type/String])
                                   b <- (wat.type/Tuple :- [wat.type/i64 wat.type/String wat.type/String])]
                    -> wat.type/bool
                    (:wat::core::> (:wat::core::first a) (:wat::core::first b)))
                  rel-edits)]
    (:wat::fix::fix-text-apply args-src sorted)))

;; parse-fragment — the table's bridged `(HEAD :- [args…])` text, parsed to its own List node
;; (unwrapping `read-string`'s program-level Forms wrapper — the fragment is always exactly one
;; top-level form).
(:wat::core::defn :c69::parse-fragment [raw <- wat.type/String] -> wat.type/AST
  (:wat::core::let [tree (:wat::core::match (:wat::core::read-string raw)
                            [:wat::core::ReadOutcome.Forms {:forms __forms} __forms]
                            [:wat::core::ReadOutcome.Malformed {:cause __cause}
                              (:wat::kernel::assertion-failed! :message
                                (:wat::string::concat "typed-constructors: unparseable table fragment: " raw))])]
    (:wat::core::first (:wat::core::ast->children tree))))

;; head-text — the call's own head, converted through the SAME door directly (never via the
;; table): a `:wat::core::X` keyword becomes its `to-type-form` symbol; a head already spelled
;; `wat.type/X` (post-255.67 corpus sites) round-trips unchanged.
(:wat::core::defn :c69::head-text [head <- wat.type/AST] -> wat.type/String
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind head) "keyword")
    (:wat::core::write-forms (:wat::keyword::to-type-form head))
    (:wat::core::write-forms head)))

;; build-edit — the one point-edit for a hit whose table lookup succeeded.
(:wat::core::defn :c69::build-edit
  [node  <- wat.type/AST
   raw   <- wat.type/String
   lines <- (wat.type/Vector :- [wat.type/String])
   src   <- wat.type/String]
  -> (wat.type/Vector :- [(wat.type/Tuple :- [wat.type/i64 wat.type/String wat.type/String])])
  (:wat::core::let [ch   (:wat::core::ast->children node)
                    head (:wat::core::first ch)]
    (:wat::core::if (:wat::fix::source-matches-name? head lines src)
      (:wat::core::let
        [head-txt  (:c69::head-text head)
         fragment  (:c69::parse-fragment raw)
         args-txt  (:c69::args-text fragment raw)
         new-text  (:wat::string::concat head-txt (:wat::string::concat " :- " args-txt))
         off       (:wat::fix::fix-text-offset-of (:wat::core::ast-span head) lines)
         old-text  (:wat::core::ast-name head)]
        (wat.type/Vector :- [(wat.type/Tuple :- [wat.type/i64 wat.type/String wat.type/String])]
          (wat.type/Tuple :- [wat.type/i64 wat.type/String wat.type/String] off old-text new-text)))
      ;; reader-synthesized head (source text != ast-name) — never edit it.
      (:wat::fix::empty-edits))))

;; maybe-edit — the edit list for one node: empty unless it is a hit AND the table has its type.
(:wat::core::defn :c69::maybe-edit
  [node  <- wat.type/AST
   table <- (wat.type/HashMap :- [wat.type/String wat.type/String])
   lines <- (wat.type/Vector :- [wat.type/String])
   src   <- wat.type/String]
  -> (wat.type/Vector :- [(wat.type/Tuple :- [wat.type/i64 wat.type/String wat.type/String])])
  (:wat::core::if (:c69::is-hit? node)
    (:wat::core::match (:wat::hashmap::get table (:c69::table-key node))
      [:wat::core::Option.Some {:value raw} (:c69::build-edit node raw lines src)]
      [:wat::core::Option.None {} (:wat::fix::empty-edits)])
    (:wat::fix::empty-edits)))

;; node-edits / walk-seq — every node in the tree; recurse into structural nodes UNCONDITIONALLY
;; (a hit's own children can independently hold further, unrelated hits — e.g. a
;; PersistentVector-of-untyped-Tuple call — each is its own census site at its own position).
(:wat::core::defn :c69::node-edits
  [node  <- wat.type/AST
   table <- (wat.type/HashMap :- [wat.type/String wat.type/String])
   lines <- (wat.type/Vector :- [wat.type/String])
   src   <- wat.type/String]
  -> (wat.type/Vector :- [(wat.type/Tuple :- [wat.type/i64 wat.type/String wat.type/String])])
  (:wat::core::let [here (:c69::maybe-edit node table lines src)]
    (:wat::core::if (:wat::fix::structural? node)
      (:wat::core::concat here (:c69::walk-seq (:wat::core::ast->children node) table lines src))
      here)))

(:wat::core::defn :c69::walk-seq
  [items <- (wat.type/Vector :- [wat.type/AST])
   table <- (wat.type/HashMap :- [wat.type/String wat.type/String])
   lines <- (wat.type/Vector :- [wat.type/String])
   src   <- wat.type/String]
  -> (wat.type/Vector :- [(wat.type/Tuple :- [wat.type/i64 wat.type/String wat.type/String])])
  (:wat::core::if (:wat::core::empty? items)
    (:wat::fix::empty-edits)
    (:wat::core::concat
      (:c69::node-edits (:wat::core::first items) table lines src)
      (:c69::walk-seq (:wat::core::rest items) table lines src))))

;; convert — one file's source text, fully converted against its own table (a
;; `HashMap<String,String>`; a file with no table entries at all still round-trips — every hit
;; misses its lookup and is left alone, same as an unresolved/not-checked site).
(:wat::core::defn :c69::convert
  [src   <- wat.type/String
   table <- (wat.type/HashMap :- [wat.type/String wat.type/String])]
  -> wat.type/String
  (:wat::core::let
    [lines  (:wat::string::split src "\n")
     tree   (:wat::core::match (:wat::core::read-string src)
              [:wat::core::ReadOutcome.Forms {:forms __forms} __forms]
              [:wat::core::ReadOutcome.Malformed {:cause __cause}
                (:wat::kernel::assertion-failed! :message (:wat::core::Error/message __cause))])
     forms  (:wat::core::ast->children tree)
     edits  (:c69::walk-seq forms table lines src)
     sorted (:wat::core::sort
              (:wat::core::fn [a <- (wat.type/Tuple :- [wat.type/i64 wat.type/String wat.type/String])
                               b <- (wat.type/Tuple :- [wat.type/i64 wat.type/String wat.type/String])]
                -> wat.type/bool
                (:wat::core::> (:wat::core::first a) (:wat::core::first b)))
              edits)]
    (:wat::fix::fix-text-apply src sorted)))

;; ══ DRIVE — read paths (stdin, the SAME single-line protocol every recorded `wat-scripts/
;; fixes/*.wat` driver uses — `scripts/replay/delta.sh`/`census.sh` invoke a codemod this way
;; and feed it nothing else) → per path, convert against ITS OWN table entry → write ═══
(:wat::core::defn :c69::table-for
  [tables <- (wat.type/HashMap :- [wat.type/String (wat.type/HashMap :- [wat.type/String wat.type/String])])
   path   <- wat.type/String]
  -> (wat.type/HashMap :- [wat.type/String wat.type/String])
  (:wat::core::match (:wat::hashmap::get tables path)
    [:wat::core::Option.Some {:value t} t]
    [:wat::core::Option.None {} (wat.type/HashMap :- [wat.type/String wat.type/String])]))

(:wat::core::defn :user::apply-each
  [paths  <- (wat.type/Vector :- [wat.type/String])
   tables <- (wat.type/HashMap :- [wat.type/String (wat.type/HashMap :- [wat.type/String wat.type/String])])]
  -> wat.type/nil
  (:wat::core::if (:wat::core::empty? paths)
    nil
    (:wat::core::let [path (:wat::core::first paths)]
      (:wat::core::do
        (:wat::io::write-file path (:c69::convert (:wat::io::read-file path) (:c69::table-for tables path)))
        (:wat::kernel::println (:wat::string::concat "[typed-constructors] " path))
        (:user::apply-each (:wat::core::rest paths) tables)))))

;; The table (item 2 — an INPUT, never committed) is not on stdin: `paths` alone travels that
;; way, so this codemod's invocation is the SAME `printf '[…]' | wat <codemod.wat>` shape every
;; other recorded migration uses (and what `scripts/replay/{census,delta}.sh` already know how
;; to drive). The table is read from a FIXED, relative, gitignored side-channel path,
;; `.typed-constructors-table.edn`, written fresh before each invocation by the (uncommitted)
;; table-builder — never a file this codemod ships or a path baked into a committed artifact.
(:wat::core::defn :user::main [] -> wat.type/nil
  (:wat::core::let
    [paths (:wat::core::match (:wat::kernel::readln)
             [:wat::kernel::ReadlnOutcome.Datum {:v __paths} __paths]
             [:wat::kernel::ReadlnOutcome.Eof {} (:wat::kernel::assertion-failed! :message "readln: end of input (paths)")]
             [:wat::kernel::ReadlnOutcome.Stopped {} (:wat::kernel::assertion-failed! :message "readln: stop requested (paths)")])
     tables (:wat::edn::read (:wat::io::read-file ".typed-constructors-table.edn"))]
    (:user::apply-each paths tables)))
