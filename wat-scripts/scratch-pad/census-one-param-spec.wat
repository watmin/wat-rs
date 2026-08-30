;; wat-scripts/fixes/one-param-spec.wat — arc 109 Stone 1 (of 3): ONE PARAM-SPEC.
;;
;; Builder's ruling, 2026-08-29: "there is exactly one way to confer a parametric type. it is
;; `:- [...]`. all others must die." This codemod rewrites the other two spellings, in the
;; `.wat` corpus, into the one:
;;
;;   (:wat::core::Vector :wat::core::i64 1 2 3)      -> (:wat::core::Vector :- [:wat::core::i64] 1 2 3)   BARE keyword
;;   (:wat::core::HashMap :k :v k1 v1)               -> (:wat::core::HashMap :- [:k :v] k1 v1)            BARE keyword
;;   (:wat::core::Vector [:wat::core::i64] 1 2 3)    -> (:wat::core::Vector :- [:wat::core::i64] 1 2 3)   unmarked bracket
;;
;; ★ THE CONTRACT — arity comes from a SOURCE, never from counting leading keywords
;; (`(:wat::core::Vector :wat::core::keyword :a :b :c)` is ONE type param and THREE keyword
;; VALUES — counting keywords would wreck it). Two sources, built by `:user::arity-table`:
;;
;;   1. SUBSTRATE (`:user::substrate-arity`) — a small named table, one entry per built-in
;;      parametric with NO wat declaration of its own: Vector 1, HashSet 1, PersistentVector 1,
;;      Option 1, HashMap 2, PersistentMap 2, Result 2. `Tuple` is DELIBERATELY ABSENT — its
;;      param count equals its value count, so a bare `(:wat::core::Tuple k v)` cannot be split
;;      by count alone; those sites are REPORTED (`:user::tuple-ambiguous?`), never guessed.
;;   2. USER TYPES (`:user::collect-arity`) — every `defrecord` / `holon::defrecord` /
;;      `defstruct` / `defenum` / `defsurface` / `defservice` / `typealias` / `newtype` /
;;      `typeunion` / `recordtype` / `aggregatetype` / `structtype` declaring
;;      `Name :- [T …]` gives `Name` arity `(length [T …])`. Collected in a FIRST PASS over the
;;      whole corpus (the CONTEXT paths — every `.wat` file, so a type declared in file A
;;      resolves correctly for a call site in file B), before any rewriting happens.
;;
;;   A head found in NEITHER source is left untouched and reported (never guessed) — this is
;;   `:user::classify`'s kind 7, `bracket-unknown-head` (an unmarked `Head [k…]` bracket whose
;;   head is not a recognised parametric; there is no structural marker at all for an unknown
;;   BARE head, so that direction is a strict subset of "not a match").
;;
;; ⚠ `:wat::core::fn`'s `[...]` is its PARAMETER LIST, not a param-spec — 1053 sites. This
;; codemod NEVER special-cases `fn` by name-exclusion; the safety is structural: the bracket
;; path (`classify` kind 4) only fires when `arity-lookup` succeeds, and `:wat::core::fn` is
;; never a key in the table (neither substrate nor corpus-declared), so it can only ever land
;; on kind 0 (not-a-match) or kind 7 (report-only) — never kind 2 or kind 4, the only kinds that
;; emit an edit. The same argument protects every OTHER unrecognised call head: no key, no edit.
;;
;; ★ THE ORACLE is `target/release/wat --check`, not this script's own confidence — a wrong
;; split fails to type-check because a value in a type-param slot is never a type. Run it over
;; every rewritten file after applying (README / brief, not this file's job to invoke).
;;
;; ── mechanics — PURE INSERTION, not text reconstruction ─────────────────────────────────
;; Every edit this script emits is a zero-width insertion: `(offset "" new-text)`. For the bare
;; form, that's two insertions per site — `":- ["` immediately before the first type-arg's own
;; span, and `"]"` immediately after the last type-arg's own span — so the ORIGINAL tokens
;; (including comments and whitespace between them) are never touched, only bracketed in place.
;; For the unmarked-bracket form, one insertion — `":- "` immediately before the vector's own
;; span. `:wat::fix::fix-text-apply`'s verify-before-splice check accepts an empty old-text
;; trivially (`subs(src,off,off) == ""`), so this rides the SAME framework as every other
;; recorded migration, just with the "old text" always empty.
;;
;; Recursion is total and independent of whether a node itself matched: `collect-edits` always
;; walks head, every type-arg, and every value-arg (bare form) or the whole list including the
;; bracket's own contents (unmarked-bracket form) — so a nested parametric buried inside an
;; outer one's type-arg or value list is found and fixed in the same pass
;; (`(:wat::core::Vector (:wat::cache::Entry :wat::core::i64 :wat::core::String))` fixes BOTH
;; Vector's and Entry's param-specs). Insertion offsets nest correctly by construction: an inner
;; match's own span sits strictly inside its parent's span, so its insertion offsets are always
;; strictly greater (open) / strictly less (close) than the parent's own — no offset collisions,
;; ascending order preserved for the left-to-right collect -> reverse -> splice discipline every
;; recorded migration uses.
;;
;; ── usage — R21: dry-run on a /tmp copy FIRST, diff it, THEN apply to the corpus ──────────
;; Two EDN vectors of paths on stdin: line 1 = CONTEXT (read-only, arity source — pass the
;; WHOLE corpus so cross-file type declarations resolve), line 2 = TARGET (read+write, the
;; files actually rewritten). For a pilot: context = the whole corpus, target = just the /tmp
;; copy. For the real run: both lines are the same full corpus path list.
;;
;;   printf '["ctxA" "ctxB" …]\n["/tmp/pilot.wat"]\n' | cargo wat ./wat-scripts/fixes/one-param-spec.wat
;;   diff original.wat /tmp/pilot.wat
;;
;; Idempotent: a site already in `:- [...]` form has its first arg-child as the `:-` symbol
;; (`classify` kind 1, `already-marked`) — no edit emitted — so a second run over the same
;; paths is a byte-identical no-op.

;; ── span/offset helpers (identical shape to every recorded migration) ───────────────────
(:wat::core::defn :user::start-off
  [n <- :wat::WatAST lines <- (:wat::core::Vector :- [:wat::core::String])] -> :wat::core::i64
  (:wat::fix::fix-text-offset-of (:wat::core::ast-span n) lines))
(:wat::core::defn :user::end-off
  [n <- :wat::WatAST lines <- (:wat::core::Vector :- [:wat::core::String])] -> :wat::core::i64
  (:wat::fix::fix-text-offset-of (:wat::core::ast-end-span n) lines))
(:wat::core::defn :user::node-line [n <- :wat::WatAST] -> :wat::core::i64
  (:wat::core::Option/expect (:wat::hashmap::get (:wat::core::ast-span n) :line) "one-param-spec: :line"))

;; ── arity source 1 — the substrate table ─────────────────────────────────────────────────
;; Every entry justified: these seven built-ins have NO wat-level `defrecord`/`defstruct` of
;; their own to declare arity, so the table is the only source. `Tuple` is deliberately absent.
(:wat::core::defn :user::substrate-arity []
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::String :wat::core::i64])])
  (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::String :wat::core::i64])
    (:wat::core::Tuple ":wat::core::Vector" 1)           ;; Vector<T>
    (:wat::core::Tuple ":wat::core::HashSet" 1)          ;; HashSet<T>
    (:wat::core::Tuple ":wat::core::PersistentVector" 1) ;; PersistentVector<T>
    (:wat::core::Tuple ":wat::core::Option" 1)           ;; Option<T>
    (:wat::core::Tuple ":wat::core::HashMap" 2)          ;; HashMap<K,V>
    (:wat::core::Tuple ":wat::core::PersistentMap" 2)    ;; PersistentMap<K,V>
    (:wat::core::Tuple ":wat::core::Result" 2)))         ;; Result<Ok,Err>

;; substrate-head? — true for exactly the seven names above. Load-bearing distinction (found
;; live, `wat-tests/edn/roundtrip.wat`): a user-declared record/struct's POSITIONAL
;; bare-keyword-then-values construction is RETIRED — `--check` on the substrate's own
;; `:wat::core::kwargs-construct` says so verbatim ("bare-positional construction of
;; :test::Wrapper is retired ... write kwargs `(:test::Wrapper :field value …)`"). So
;; `(:test::Wrapper :label "score" :value 42)` is a KWARGS call whose leading `:label` is a
;; FIELD NAME, not a type keyword — structurally identical to a genuine bare param-spec site,
;; but semantically nothing alike. The substrate seven have their OWN positional
;; bare-type-then-values construction grammar (never kwargs), so `got > n` is safe evidence
;; there ONLY; for a corpus-declared type, `got > n` is presumed kwargs and left untouched
;; (classify's bare-branch gates on this).
(:wat::core::defn :user::substrate-head?
  [name <- :wat::core::String] -> :wat::core::bool
  (:wat::core::contains?
    (:wat::core::HashSet :wat::type::Infer
      ":wat::core::Vector" ":wat::core::HashSet" ":wat::core::PersistentVector"
      ":wat::core::Option" ":wat::core::HashMap" ":wat::core::PersistentMap" ":wat::core::Result")
    name))

;; ── arity source 2 — corpus-declared user types ──────────────────────────────────────────
;; declarator-head-keyword set, restricted to heads that declare a CONSTRUCTIBLE TYPE (not
;; `defn` — a generic FUNCTION's own `:- [T]` param list is a different concept entirely,
;; arc 109's turbofish/angle-bracket stones' business, never this one's).
(:wat::core::defn :user::decl-head?
  [name <- :wat::core::String] -> :wat::core::bool
  (:wat::core::contains?
    (:wat::core::HashSet :wat::type::Infer
      ":wat::core::defrecord"
      ":wat::holon::defrecord"
      ":wat::core::defstruct"
      ":wat::core::defenum"
      ":wat::core::defsurface"
      ":wat::service::defservice"
      ":wat::core::typealias"
      ":wat::core::newtype"
      ":wat::core::typeunion"
      ":wat::core::recordtype"
      ":wat::core::aggregatetype"
      ":wat::core::structtype")
    name))

;; collect-arity — recursively find every `(DeclHead Name :- [T …] …)` at ANY depth (messages
;; nested inside a `defsurface`'s `:messages` vector, a `defrecord` inside a `do`, …) and emit
;; a (Name, arity) tuple. A shape that doesn't match (no `:-`/vector right after the name, or
;; the name slot is a macro-template unquote symbol, not a literal keyword) contributes nothing
;; — it is simply not a source of arity, not an error.
(:wat::core::defn :user::collect-arity
  [node <- :wat::WatAST] -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::String :wat::core::i64])])
  (:wat::core::let
    [here (:wat::core::if (:wat::core::= (:wat::core::ast-kind node) "list")
            (:wat::core::let [ch (:wat::core::ast->children node)]
              (:wat::core::if (:wat::i64::< (:wat::core::length ch) 4)
                (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::String :wat::core::i64]))
                (:wat::core::let [h (:wat::core::nth ch 0)]
                  (:wat::core::if (:wat::core::if (:wat::core::= (:wat::core::ast-kind h) "keyword")
                                    (:user::decl-head? (:wat::core::ast-name h))
                                    false)
                    (:wat::core::let [nm (:wat::core::nth ch 1) arr (:wat::core::nth ch 2) brk (:wat::core::nth ch 3)]
                      (:wat::core::if (:wat::core::if (:wat::core::= (:wat::core::ast-kind nm) "keyword")
                                        (:wat::core::if (:wat::core::= (:wat::core::ast-kind arr) "keyword")
                                          (:wat::core::if (:wat::core::= (:wat::core::ast-name arr) ":-")
                                            (:wat::core::= (:wat::core::ast-kind brk) "vector")
                                            false)
                                          false)
                                        false)
                        (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::String :wat::core::i64])
                          (:wat::core::Tuple (:wat::core::ast-name nm) (:wat::core::length (:wat::core::ast->children brk))))
                        (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::String :wat::core::i64]))))
                    (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::String :wat::core::i64]))))))
            (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::String :wat::core::i64])))]
    (:wat::core::concat here (:user::collect-arity-seq (:wat::core::ast->children node)))))

(:wat::core::defn :user::collect-arity-seq
  [items <- (:wat::core::Vector :- [:wat::WatAST])] -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::String :wat::core::i64])])
  (:wat::core::if (:wat::core::empty? items)
    (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::String :wat::core::i64]))
    (:wat::core::concat (:user::collect-arity (:wat::core::first items)) (:user::collect-arity-seq (:wat::core::rest items)))))

;; arity-lookup — first match in `table`, or -1 (sentinel: no source names this head).
(:wat::core::defn :user::arity-lookup
  [table <- (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::String :wat::core::i64])])
   name  <- :wat::core::String]
  -> :wat::core::i64
  (:wat::core::if (:wat::core::empty? table)
    -1
    (:wat::core::let [h (:wat::core::first table)]
      (:wat::core::if (:wat::core::= (:wat::core::first h) name)
        (:wat::core::second h)
        (:user::arity-lookup (:wat::core::rest table) name)))))

;; bracket-type-shaped? — the unmarked `[...]` bracket must look like TYPE ARGS (every element
;; a bare keyword or a compound type-reference LIST), never a param/binding vector: `let`,
;; `fn`, `defn` and every other binder-shaped form ALSO puts a `[...]` right after its head, but
;; those vectors always contain at least one SYMBOL (a bound name, `<-`, `&`) — never present in
;; a genuine type-param bracket. This is the guard that keeps the unmarked-bracket / unknown-
;; head detection from firing on every `(:wat::core::let [x v] ...)` in the corpus.
;; type-shaped-elem? — a KEYWORD is always presumed type-shaped (the grammar itself expects a
;; type in this position; the arity check backstops a plain data keyword). A LIST is
;; type-shaped ONLY if ITS OWN HEAD is itself a keyword this codemod recognises as a type
;; (in `table`, or the literal `:wat::core::Tuple`) — never "any list at all". Found live
;; (`tests/rete/probe_arc278_4a_production_fire.wat`):
;; `(:wat::core::PersistentVector (:weather::q-ColdAndWindy))` is PersistentVector's
;; "no type spec, infer from values" spelling with a SINGLE VALUE that happens to be a
;; zero-arg function CALL, not a nested type reference — `:weather::q-ColdAndWindy` names
;; neither arity source, so under the old "any List counts" rule this was wrongly wrapped as
;; `:- [(:weather::q-ColdAndWindy)]`, corrupting a value into a type-param slot. Requiring the
;; nested list's OWN head to be a recognised type name closes this without losing the genuine
;; nested-parametric case (`(:wat::core::Vector (:pilot::Entry :wat::core::i64 :wat::core::String))`
;; — `:pilot::Entry` IS in `table`).
(:wat::core::defn :user::type-shaped-elem?
  [node <- :wat::WatAST table <- (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::String :wat::core::i64])])]
  -> :wat::core::bool
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind node) "keyword") true
    (:wat::core::if (:wat::core::= (:wat::core::ast-kind node) "list")
      (:wat::core::let [ch (:wat::core::ast->children node)]
        (:wat::core::if (:wat::core::empty? ch)
          false
          (:wat::core::let [h (:wat::core::first ch)]
            (:wat::core::if (:wat::core::= (:wat::core::ast-kind h) "keyword")
              (:wat::core::let [hn (:wat::core::ast-name h)]
                (:wat::core::let [_unused-tuple-note nil]
                  ;; `:wat::core::Tuple` gets NO blanket pass here — it is EXCLUDED from the
                  ;; arity table by design (param count == value count), so a bare
                  ;; `(:wat::core::Tuple v1 v2)` is ALWAYS a value construction, never
                  ;; type-shaped evidence. Found live (`wat/rete/oracle/pass.wat`):
                  ;; `(:wat::core::PersistentVector (:wat::core::Tuple (fact) alpha-id))` is
                  ;; PersistentVector's "no type spec, infer from values" spelling whose single
                  ;; VALUE is a 2-tuple — a prior blanket `hn == Tuple -> true` wrapped this
                  ;; VALUE in `:- [...]`, corrupting a stdlib file baked into every test binary.
                  ;; Tuple now falls through to the SAME general rule as everything else below:
                  ;; only an ALREADY-`:-`-marked, nothing-trailing Tuple reference
                  ;; (`(:wat::core::Tuple :- [T1 T2])`) counts as type-shaped.
                  ;; already `:- [...]`-marked, AND NOTHING TRAILS the bracket -> a PURE type
                  ;; reference (`(Head :- [T…])`), no ambiguity left. The trailing-nothing
                  ;; check is load-bearing: `(:wat::core::Vector :- [T] v1 v2)` is ALSO
                  ;; `:-`-marked but is a constructed VALUE (a vector holding v1, v2), not a
                  ;; type at all — found live
                  ;; (`wat-tests/service-cache-hologram.wat`, only on a SECOND corpus-wide
                  ;; pass once run 1 had already `:-`-marked this exact Vector):
                  ;; `(:wat::cache::Cache::PutRequest :entries (:wat::core::Vector :- [...] e1 e2))`
                  ;; — treating the already-marked-but-value-bearing Vector as type-shaped
                  ;; wrongly wrapped PutRequest's own kwargs call in `:- [...]`.
                  (:wat::core::if (:wat::core::if (:wat::core::= (:wat::core::length ch) 3)
                                    (:wat::core::if (:wat::core::= (:wat::core::ast-kind (:wat::core::nth ch 1)) "keyword")
                                      (:wat::core::= (:wat::core::ast-name (:wat::core::nth ch 1)) ":-")
                                      false)
                                    false)
                    true
                    ;; NOT yet `:-`-marked: a known USER-declared type is unambiguous evidence
                    ;; (positional construction is retired for those — kwargs-retirement note
                    ;; above), but a SUBSTRATE head here is NOT: it is exactly as likely to be
                    ;; a nested VALUE construction as a nested type reference (found live,
                    ;; `wat-scripts/scratch-pad/probe-zero-magnitude-reachable.wat`:
                    ;; `(:wat::core::PersistentMap :k (:wat::core::PersistentMap :expected e :got g))`
                    ;; is a values-only 2-entry map whose VALUE is itself a values-only map —
                    ;; treating that nested call as "type-shaped" wrongly converted the OUTER
                    ;; map). So a bare (unmarked) substrate-headed nested list is conservatively
                    ;; NOT type-shaped — real nested substrate-in-substrate TYPE sites are
                    ;; missed rather than risking corruption; `--check`/row-4's skip report
                    ;; would have caught a genuine loss as a "not converted" site, and none was
                    ;; found in the corpus for this shape.
                    (:wat::core::if (:wat::core::if (:wat::i64::>= (:user::arity-lookup table hn) 0)
                                      (:wat::core::if (:wat::core::= (:wat::core::length ch) 2)
                                        (:wat::core::= (:wat::core::ast-kind (:wat::core::nth ch 1)) "vector")
                                        false)
                                      false)
                      ;; hn is a RECOGNISED type, its first (and ONLY) arg is itself an
                      ;; unmarked `[...]` bracket, and NOTHING trails it — a pure type
                      ;; reference again (the same trailing-nothing discipline as the
                      ;; already-`:-`-marked arm just above: `(Head [T] v1 v2)` WITH trailing
                      ;; values is a VALUE construction, not a type). Unambiguous regardless of
                      ;; substrate-vs-user (brackets are exclusively a type spelling in this
                      ;; grammar, never a "values-only" positional call; the substrate/bare
                      ;; ambiguity above is specific to the BARE keyword form). The
                      ;; `arity-lookup >= 0` guard is load-bearing: without it this branch also
                      ;; matched
                      ;; `(:wat::core::let [bindvec] body)` — ANY binder-shaped form's
                      ;; `[...]` second child — wrongly treating a LET-EXPRESSION VALUE as
                      ;; type-shaped (found live,
                      ;; `wat-scripts/scratch-pad/probe-overlay-refire-cost.wat`:
                      ;; `(:wat::core::PersistentVector (:wat::core::let [conds ...] ...))`).
                      ;; This is also what makes the codemod idempotent in ONE pass: a nested
                      ;; unmarked bracket that this SAME run is about to convert (its own edit
                      ;; hasn't landed yet — edits are text-spliced after the whole walk, the
                      ;; AST here is still pre-edit) must still count as type-shaped for the
                      ;; OUTER bracket's own classification, or the outer site is missed in
                      ;; this pass and only picked up on a second run (found live,
                      ;; `wat-scripts/scratch-pad/probe-stone-2a-bracket-mechanics.wat`).
                      true
                      (:wat::core::if (:wat::i64::< (:user::arity-lookup table hn) 0)
                        false
                        (:wat::core::not (:user::substrate-head? hn)))))))
              false))))
      false)))
(:wat::core::defn :user::all-type-shaped?
  [items <- (:wat::core::Vector :- [:wat::WatAST])
   table <- (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::String :wat::core::i64])])]
  -> :wat::core::bool
  (:wat::core::if (:wat::core::empty? items) true
    (:wat::core::if (:user::type-shaped-elem? (:wat::core::first items) table)
      (:user::all-type-shaped? (:wat::core::rest items) table)
      false)))
(:wat::core::defn :user::bracket-type-shaped?
  [vec-node <- :wat::WatAST table <- (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::String :wat::core::i64])])]
  -> :wat::core::bool
  (:wat::core::let [ch (:wat::core::ast->children vec-node)]
    (:wat::core::if (:wat::core::empty? ch) false (:user::all-type-shaped? ch table))))

;; take-n — the first n elements of items (n<=0 or items exhausted -> []). Used to gate the
;; bare-form check on ALL of the WOULD-BE type-arg slots, not just arg[0]: found live
;; (`tests/collection/probe_arc278_0a_persistent_map.wat`) —
;; `(:wat::core::PersistentMap :a 1 :b 2)` is PersistentMap's own "no type spec, infer from
;; values" construction (n=2, arity source 1's substrate table), whose FIRST value (`:a`, a map
;; KEY) happens to be keyword-shaped while its SECOND (`1`) plainly is not. Checking only
;; arg[0] passed this and produced `:- [:a 1]` — a value wrapped as a type. Every one of the
;; first n args must be type-shaped before this is treated as an attempted param-spec.
(:wat::core::defn :user::take-n
  [items <- (:wat::core::Vector :- [:wat::WatAST]) n <- :wat::core::i64] -> (:wat::core::Vector :- [:wat::WatAST])
  (:wat::core::if (:wat::core::if (:wat::i64::<= n 0) true (:wat::core::empty? items))
    (:wat::core::Vector :wat::WatAST)
    (:wat::core::concat (:wat::core::Vector :wat::WatAST (:wat::core::first items))
      (:user::take-n (:wat::core::rest items) (:wat::i64::- n 1)))))

;; bracket-all-keyword? — STRICTER than bracket-type-shaped?: every element is a literal
;; Keyword (matches `src/check.rs`'s `is_type_bracket_candidate` exactly, no nested-List
;; allowance). Used ONLY for the unknown-head (kind 7) signal: a quasiquote/quote template's
;; unquoted (`~name`) bindings desugar to List nodes with no distinct ast-kind (`wat/fix.wat`'s
;; own note), so `(:wat::core::let [~a ~b] ...)`-shaped forms inside a macro template can look
;; permissively "type-shaped" (List elements) despite being ordinary bind-vectors — measured
;; false positives on `:wat::core::let` / `:wat::core::quote` / `:wat::core::quasiquote` /
;; `:wat::core::fn` in the real corpus. A KNOWN head still gets the permissive nested-List
;; check (`bracket-type-shaped?`) because genuine sites nest compound type refs
;; (`(:wat::core::PersistentVector [(:wat::core::PersistentMap [...])])`); an UNKNOWN head has
;; no other evidence at all, so the strict all-keyword rule is the only defensible bar for
;; "this looks enough like a type bracket to be worth a human's attention."
(:wat::core::defn :user::all-keyword?
  [items <- (:wat::core::Vector :- [:wat::WatAST])] -> :wat::core::bool
  (:wat::core::if (:wat::core::empty? items) true
    (:wat::core::if (:wat::core::= (:wat::core::ast-kind (:wat::core::first items)) "keyword")
      (:user::all-keyword? (:wat::core::rest items))
      false)))
(:wat::core::defn :user::bracket-all-keyword? [vec-node <- :wat::WatAST] -> :wat::core::bool
  (:wat::core::let [ch (:wat::core::ast->children vec-node)]
    (:wat::core::if (:wat::core::empty? ch) false (:user::all-keyword? ch))))

;; ── classify — the ONE decision, shared by both the edit-emitter and the reporter ────────
;; Returns (kind, n, got):
;;   0 not-a-match           — unrecognised head, or not a list at all
;;   1 already-marked        — first arg IS the `:-` symbol; correct already, no edit
;;   2 bare-ok               — bare keyword form, enough args; rewrite
;;   3 bare-insufficient     — recognised head, fewer args than its declared arity; REPORT
;;   4 bracket-ok            — unmarked `[T…]` bracket whose length matches the source; rewrite
;;   5 bracket-mismatch      — unmarked bracket whose length does NOT match the source; REPORT
;;   6 tuple-ambiguous       — `(:wat::core::Tuple <keyword> …)`, not `:-`-marked: param count
;;                             equals value count, cannot disambiguate by count; REPORT
;;   7 bracket-unknown-head  — unmarked `[k…]` bracket whose head names neither arity source;
;;                             REPORT, never guessed (row 2's "reported, never guessed")
(:wat::core::defn :user::classify
  [node  <- :wat::WatAST
   table <- (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::String :wat::core::i64])])]
  -> (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::i64])
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind node) "list")
    (:wat::core::let [ch (:wat::core::ast->children node)]
      (:wat::core::if (:wat::core::empty? ch)
        (:wat::core::Tuple 0 0 0)
        (:wat::core::let [h (:wat::core::first ch)]
          (:wat::core::if (:wat::core::= (:wat::core::ast-kind h) "keyword")
            (:wat::core::let [hn   (:wat::core::ast-name h)
                              args (:wat::core::into [] (:wat::core::rest ch))]
              (:wat::core::if (:wat::core::= hn ":wat::core::Tuple")
                ;; Tuple: excluded from the arity table BY DESIGN (param count == value
                ;; count); a non-`:-` leading keyword is ambiguous, never guessed.
                (:wat::core::if (:wat::core::empty? args)
                  (:wat::core::Tuple 0 0 0)
                  (:wat::core::let [a0 (:wat::core::first args)]
                    (:wat::core::if (:wat::core::if (:wat::core::= (:wat::core::ast-kind a0) "keyword")
                                      (:wat::core::= (:wat::core::ast-name a0) ":-")
                                      false)
                      (:wat::core::Tuple 1 0 0)
                      (:wat::core::if (:wat::core::= (:wat::core::ast-kind a0) "keyword")
                        (:wat::core::Tuple 6 0 0)
                        (:wat::core::Tuple 0 0 0)))))
                (:wat::core::let [n (:user::arity-lookup table hn)]
                  (:wat::core::if (:wat::core::empty? args)
                    ;; zero args at all — no evidence of an ATTEMPTED type-spec (a mandatory-
                    ;; typed head like Vector/HashMap/HashSet with truly zero args is already
                    ;; MalformedForm at the checker; a head that allows "no spec, infer from
                    ;; values" — PersistentVector/PersistentMap — has nothing to convert here
                    ;; either way). Not this stone's business; never guess an arity out of thin air.
                    (:wat::core::Tuple 0 0 0)
                    (:wat::core::let [a0 (:wat::core::first args)]
                      (:wat::core::if (:wat::core::if (:wat::core::= (:wat::core::ast-kind a0) "keyword")
                                        (:wat::core::= (:wat::core::ast-name a0) ":-")
                                        false)
                        (:wat::core::Tuple 1 0 0)
                        (:wat::core::if (:wat::core::= (:wat::core::ast-kind a0) "vector")
                          (:wat::core::if (:wat::i64::< n 0)
                            ;; unknown head — the STRICT all-keyword bar only (never the
                            ;; permissive nested-List one; see bracket-all-keyword?'s header —
                            ;; a quasiquote-template bind-vector's unquoted names desugar to
                            ;; List nodes with no distinct ast-kind, which would otherwise pass
                            ;; the permissive check and false-flag :wat::core::let/quote/
                            ;; quasiquote/fn as unknown parametric heads).
                            (:wat::core::if (:user::bracket-all-keyword? a0)
                              (:wat::core::Tuple 7 n (:wat::core::length (:wat::core::ast->children a0)))
                              (:wat::core::Tuple 0 0 0))
                            ;; known head — the permissive nested-type-shaped bar (real sites
                            ;; nest compound type refs), then compare length against the source.
                            (:wat::core::if (:user::bracket-type-shaped? a0 table)
                              (:wat::core::let [m (:wat::core::length (:wat::core::ast->children a0))]
                                (:wat::core::if (:wat::core::= m n) (:wat::core::Tuple 4 n m) (:wat::core::Tuple 5 n m)))
                              (:wat::core::Tuple 0 0 0)))
                          ;; bare keyword/compound form — but ONLY if arg[0] itself is
                          ;; TYPE-SHAPED (a keyword or a compound type-reference list). A
                          ;; head like PersistentVector/PersistentMap legitimately allows
                          ;; NO type spec at all, values only, type inferred
                          ;; (`(:wat::core::PersistentVector 1 2 3 4 5)` is legal AS-IS,
                          ;; confirmed live: `target/release/wat --check` on
                          ;; wat-tests/core/core-foldl-spec.wat is clean). Without this
                          ;; gate, arg[0] being a plain VALUE (an int, a symbol, …) was
                          ;; being counted as "1 leading type keyword" purely because
                          ;; `got >= n` for n=1 — a false population site that this
                          ;; codemod would have corrupted by wrapping a VALUE in `:- [...]`.
                          ;; ALSO: `got > n` is only trustworthy evidence for a SUBSTRATE head
                          ;; (Vector/HashMap/... — genuine bare-type-then-values positional
                          ;; construction). For a corpus-declared type, positional construction
                          ;; is RETIRED (kwargs only), so `got > n` there is a KWARGS call whose
                          ;; leading keyword is a FIELD NAME, not a type — see substrate-head?'s
                          ;; header. `got == n` (a pure type reference, no trailing values) is
                          ;; unambiguous regardless of head kind and always converts.
                          ;; `got < n` is likewise only trustworthy as "insufficient" evidence
                          ;; for a SUBSTRATE head. Found live (`wat/spawn.wat`):
                          ;; `(:wat::spawn::Launched :handle sp :address (...))` is a KWARGS
                          ;; construction (arity 5, only 2 kwargs pairs given = 4 args < 5) —
                          ;; NOT an under-supplied bare param-spec. For a corpus-declared type,
                          ;; only `got == n` (a pure, unambiguous type reference) is this
                          ;; stone's business at all; anything else is presumed kwargs and left
                          ;; alone, matching the `got > n` reasoning just above.
                          (:wat::core::if (:wat::i64::< n 0)
                            (:wat::core::Tuple 0 0 0)
                            (:wat::core::let [got (:wat::core::length args)]
                              (:wat::core::if (:user::substrate-head? hn)
                                (:wat::core::if (:wat::i64::< got n)
                                  (:wat::core::Tuple 3 n got)
                                  (:wat::core::if (:user::all-type-shaped? (:user::take-n args n) table)
                                    (:wat::core::Tuple 2 n got)
                                    (:wat::core::Tuple 0 0 0)))
                                (:wat::core::if (:wat::core::if (:wat::core::= got n) (:user::all-type-shaped? (:user::take-n args n) table) false)
                                  (:wat::core::Tuple 2 n got)
                                  (:wat::core::Tuple 0 0 0))))))))))))
            (:wat::core::Tuple 0 0 0)))))
    (:wat::core::Tuple 0 0 0)))

;; ── edit collection ───────────────────────────────────────────────────────────────────────
;; args-edits-split — walk the args of a `bare-ok` site with a live idx/n split: open-bracket
;; insertion before arg[0]'s span, one recursive `collect-edits` per arg (type AND value args
;; alike — nested matches inside either are found), close-bracket insertion right after the
;; last TYPE arg's span. Concatenation is already in ascending source-offset order (depth-first
;; left-to-right over a properly nested AST), matching every recorded migration's
;; collect-ascending -> reverse -> splice discipline.
(:wat::core::defn :user::args-edits-split
  [args  <- (:wat::core::Vector :- [:wat::WatAST])
   idx   <- :wat::core::i64
   n     <- :wat::core::i64
   table <- (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::String :wat::core::i64])])
   lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::if (:wat::core::empty? args)
    (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String]))
    (:wat::core::let [h      (:wat::core::first args)
                      tl     (:wat::core::into [] (:wat::core::rest args))
                      open-e (:wat::core::if (:wat::core::= idx 0)
                               (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])
                                 (:wat::core::Tuple (:user::start-off h lines) "" ":- ["))
                               (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])))
                      h-e    (:user::collect-edits h table lines)
                      close-e (:wat::core::if (:wat::core::= idx (:wat::i64::- n 1))
                                (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])
                                  (:wat::core::Tuple (:user::end-off h lines) "" "]"))
                                (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])))
                      rest-e (:user::args-edits-split tl (:wat::i64::+ idx 1) n table lines)]
      (:wat::core::concat open-e (:wat::core::concat h-e (:wat::core::concat close-e rest-e))))))

;; collect-edits — the ONE walk. `classify` decides; kind 2/4 emit an edit (plus recurse for
;; nested matches), everything else is edit-free but STILL recurses (a not-a-match / already-
;; marked / reported node can still contain a nested match elsewhere in its subtree).
(:wat::core::defn :user::collect-edits
  [node  <- :wat::WatAST
   table <- (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::String :wat::core::i64])])
   lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind node) "list")
    (:wat::core::let [ch   (:wat::core::ast->children node)
                       cls  (:user::classify node table)
                       kind (:wat::core::first cls)]
      (:wat::core::if (:wat::core::= kind 2)
        (:user::args-edits-split (:wat::core::into [] (:wat::core::rest ch)) 0 (:wat::core::second cls) table lines)
        (:wat::core::if (:wat::core::= kind 4)
          (:wat::core::let [vec-node (:wat::core::nth ch 1)]
            (:wat::core::concat
              (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])
                (:wat::core::Tuple (:user::start-off vec-node lines) "" ":- "))
              (:user::collect-edits-seq ch table lines)))
          (:user::collect-edits-seq ch table lines))))
    (:wat::core::if (:wat::fix::structural? node)
      (:user::collect-edits-seq (:wat::core::ast->children node) table lines)
      (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])))))

(:wat::core::defn :user::collect-edits-seq
  [items <- (:wat::core::Vector :- [:wat::WatAST])
   table <- (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::String :wat::core::i64])])
   lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::if (:wat::core::empty? items)
    (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String]))
    (:wat::core::concat (:user::collect-edits (:wat::core::first items) table lines)
                        (:user::collect-edits-seq (:wat::core::rest items) table lines))))

;; ── report collection — kinds 3/5/6/7, never an edit; SILENCE IS THE FAILURE MODE this
;; stone must not have, so every non-rewritten candidate site is named: line, head, reason. ──
(:wat::core::defn :user::classify-message
  [kind <- :wat::core::i64 hn <- :wat::core::String n <- :wat::core::i64 got <- :wat::core::i64] -> :wat::core::String
  (:wat::core::if (:wat::core::= kind 3)
    (:wat::string::interpolate "bare-insufficient: head={h} declared-arity={n} got-args={g} (fewer args than the declared type-param count — cannot split, not guessing)" :h hn :n (:wat::i64::to-string n) :g (:wat::i64::to-string got))
    (:wat::core::if (:wat::core::= kind 5)
      (:wat::string::interpolate "bracket-mismatch: head={h} declared-arity={n} bracket-length={g} (unmarked [..] length disagrees with the declared type-param count)" :h hn :n (:wat::i64::to-string n) :g (:wat::i64::to-string got))
      (:wat::core::if (:wat::core::= kind 6)
        "tuple-ambiguous: (:wat::core::Tuple <keyword> ...) not `:-`-marked — Tuple's param count equals its value count, so a bare leading keyword cannot be disambiguated from a bare leading VALUE; excluded by design, hand-review required"
        (:wat::string::interpolate "bracket-unknown-head: head={h} unmarked [..] bracket, bracket-length={g} — head names NEITHER the substrate table NOR any corpus defrecord/defstruct/... :- [..] declaration; reported, never guessed" :h hn :g (:wat::i64::to-string got))))))

(:wat::core::defn :user::collect-reports
  [node  <- :wat::WatAST
   table <- (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::String :wat::core::i64])])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind node) "list")
    (:wat::core::let [ch   (:wat::core::ast->children node)
                       cls  (:user::classify node table)
                       kind (:wat::core::first cls)
                       here (:wat::core::if (:wat::core::if (:wat::core::= kind 3) true
                                              (:wat::core::if (:wat::core::= kind 5) true
                                                (:wat::core::if (:wat::core::= kind 6) true
                                                  (:wat::core::= kind 7))))
                              (:wat::core::let [hn (:wat::core::if (:wat::core::empty? ch) "" (:wat::core::ast-name (:wat::core::first ch)))]
                                (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])
                                  (:wat::core::Tuple (:user::node-line node) hn
                                    (:user::classify-message kind hn (:wat::core::second cls) (:wat::core::third cls)))))
                              (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])))]
      (:wat::core::concat here (:user::collect-reports-seq ch table)))
    (:wat::core::if (:wat::fix::structural? node)
      (:user::collect-reports-seq (:wat::core::ast->children node) table)
      (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])))))

(:wat::core::defn :user::collect-reports-seq
  [items <- (:wat::core::Vector :- [:wat::WatAST])
   table <- (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::String :wat::core::i64])])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::if (:wat::core::empty? items)
    (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String]))
    (:wat::core::concat (:user::collect-reports (:wat::core::first items) table)
                        (:user::collect-reports-seq (:wat::core::rest items) table))))

;; ── per-file passes ───────────────────────────────────────────────────────────────────────
(:wat::core::defn :user::parse-forms [src <- :wat::core::String] -> (:wat::core::Vector :- [:wat::WatAST])
  (:wat::core::ast->children
    (:wat::core::match (:wat::core::read-string src)
      ((:wat::core::ReadOutcome::Forms __forms) __forms)
      ((:wat::core::ReadOutcome::Malformed __cause)
        (:wat::kernel::assertion-failed! (:wat::core::Error/message __cause) :wat::core::None :wat::core::None)))))

(:wat::core::defn :user::scan-file-arity [path <- :wat::core::String]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::String :wat::core::i64])])
  (:user::collect-arity-seq (:user::parse-forms (:wat::io::read-file path))))

(:wat::core::defn :user::scan-all-arity [paths <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::String :wat::core::i64])])
  (:wat::core::if (:wat::core::empty? paths)
    (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::String :wat::core::i64]))
    (:wat::core::concat (:user::scan-file-arity (:wat::core::first paths))
                        (:user::scan-all-arity (:wat::core::rest paths)))))

;; ── census main — prints "KIND HEAD" per matched (kind != 0) list node ──────
(:wat::core::defn :census::walk-seq
  [items <- (:wat::core::Vector :- [:wat::WatAST])
   table <- (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::String :wat::core::i64])])]
  -> :wat::core::nil
  (:wat::core::if (:wat::core::empty? items)
    nil
    (:wat::core::do
      (:census::walk (:wat::core::first items) table)
      (:census::walk-seq (:wat::core::rest items) table))))

(:wat::core::defn :census::walk
  [node <- :wat::WatAST
   table <- (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::String :wat::core::i64])])]
  -> :wat::core::nil
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind node) "list")
    (:wat::core::let [ch   (:wat::core::ast->children node)
                       cls  (:user::classify node table)
                       kind (:wat::core::first cls)]
      (:wat::core::do
        (:wat::core::if (:wat::core::= kind 0)
          nil
          (:wat::kernel::println
            (:wat::string::interpolate "{k} {h}"
              :k (:wat::i64::to-string kind)
              :h (:wat::core::if (:wat::core::empty? ch) "" (:wat::core::ast-name (:wat::core::first ch))))))
        (:census::walk-seq ch table)))
    (:wat::core::if (:wat::fix::structural? node)
      (:census::walk-seq (:wat::core::ast->children node) table)
      nil)))

(:wat::core::defn :census::file
  [path <- :wat::core::String
   table <- (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::String :wat::core::i64])])]
  -> :wat::core::nil
  (:census::walk-seq (:user::parse-forms (:wat::io::read-file path)) table))

(:wat::core::defn :census::files
  [paths <- (:wat::core::Vector :- [:wat::core::String])
   table <- (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::String :wat::core::i64])])]
  -> :wat::core::nil
  (:wat::core::if (:wat::core::empty? paths)
    nil
    (:wat::core::do
      (:census::file (:wat::core::first paths) table)
      (:census::files (:wat::core::rest paths) table))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let [ctx   (:wat::core::match (:wat::kernel::readln)
                             ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum)
                             (:wat::kernel::ReadlnOutcome::Eof (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None))
                             (:wat::kernel::ReadlnOutcome::Stopped (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)))
                    table (:wat::core::concat (:user::substrate-arity) (:user::scan-all-arity ctx))]
    (:census::files ctx table)))
