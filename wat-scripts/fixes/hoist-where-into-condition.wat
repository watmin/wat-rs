;; wat-scripts/fixes/hoist-where-into-condition.wat — strike-hoist-the-corpus, Phase 1/2.
;;
;; Self-hosted fix-wat codemod: no hand-editing of .wat files — use the tool.
;;
;; THE TRANSFORM (per DESIGN.md / SCORE-strike-where-fence-hoistable.md): a top-level `:when`-entry
;; `(:wat::rete::where <pred>)`, every `?var` of which is bound by EXACTLY ONE ordinary fact
;; pattern reachable from the same `:when` vector (transparently through `:wat::rete::and`
;; grouping), is DELETED from the `:when` vector and `<pred>` is appended, verbatim, as one more
;; clause on that ONE binder condition. An alpha test wearing a beta fence becomes an alpha test.
;;
;; ⛔ THE PREDICATE IS "COULD LEGALLY MOVE", NOT "SOME CONDITION BINDS THESE VARS" — mirrors
;; `check_where_hoistable` / `hoist_scope` in `src/rete/validate/mod.rs` (recoverable at
;; `f47a9fccc`) EXACTLY, independently reimplemented here in wat since a codemod cannot call into
;; the Rust checker. `:wat::rete::or` / `:wat::rete::not` / `:wat::rete::exists` / an accumulate's
;; `(?r <- (acc-form) :from inner)` are never descended into when collecting either hoist TARGETS
;; or hoistable `where` SITES — a var "bound" inside any of the three is never even offered as a
;; candidate (`:or` binds conditionally, `:exists` binds outward, an accumulate's result var has no
;; clause list to inline into). Under-collecting is the safe direction: this codemod MISSES a site
;; sooner than it mis-hoists one. `:wat::rete::and` is the one combinator it IS transparent
;; through — sequential grouping, not a scope boundary, matching the Rust `hoist_scope` thread.
;;
;; SHAPE CLASSIFICATION (`top-shape-tag`) mirrors `classify_rete_clause`
;; (`src/rete/clause.rs`) at exactly the subset this transform needs:
;;   keyword head :wat::rete::where/not/exists/or/and -> that tag; any other keyword head -> "plain"
;;   symbol head "?v" "<-" <keyword with "::">            -> "factbind"  (`(?p <- :ns::Type …)`)
;;   symbol head "?v" "<-" <anything> :from <anything> (5 items) -> "accumulate"
;;   anything else                                        -> "other" (never a target, never descended)
;; A "plain" or "factbind" condition's OWN bound vars are its fact-bind var (factbind only) plus
;; every inline `(?v <- :field)` clause (`bind-clause-var`, mirrors the `Bind` shape) — exactly
;; `plain_pattern_bound_vars`'s Rust definition, not "every symbol that looks like a var".
;;
;; MATCH RULE (mirrors `check_where_hoistable`): collect every `?var` the predicate reads
;; (`var-occurrences`, a plain recursive AST walk for `?`-prefixed symbols — no rete-specific
;; knowledge needed, a var occurrence is a var occurrence anywhere in the expression tree), then
;; find every hoist-target candidate whose bound-var set is a SUPERSET of that set. Hoist iff
;; exactly one candidate matches — zero (nothing binds it, or it's split across conditions: a real
;; beta join) or more than one (ambiguous) both leave the `where` untouched.
;;
;; TEXT-SPAN SPLICE, not re-serialization: the deleted `where` and the appended predicate are both
;; copied/removed as RAW SOURCE SPANS (`node-start-offset`/`node-end-offset`/`string::subs`), so
;; the original spelling, comments-adjacency and formatting of everything else survive untouched.
;;
;; ⚠ DEPARTURE FROM `wat/fix.wat`'s general "surviving whitespace is wat-fmt's job" doctrine: a
;; deletion here covers the `where` form's own span PLUS its leading indentation and the newline
;; before it (`backward-trim`, below) — the coordinator's Phase-1 refinement. Left at the form's
;; own span alone, 63 sites leave a whitespace-only line where the `where` used to sit (nothing
;; GATES it — no trailing-whitespace/line-length lint, and the corpus already has an 885-char
;; line — but this branch is the reference for an imminent merge, and that litter is noise in
;; exactly the files a merge will be resolving conflicts in). `backward-trim` is asymmetric
;; (backward only, never forward) SPECIFICALLY so N `where`s hoisting in a row (three, in
;; `probe_arc278_sift_rules_arena.wat`'s `:arena::suspect-rule`) produce N exactly-abutting,
;; never-overlapping deletion spans: site K's backward-trim always stops at site (K-1)'s own
;; closing paren (non-whitespace), never at anything site (K-1) itself deletes.
;;
;; Idempotent (re-run = 0 changes): once a `where` is hoisted, there is no more
;; `(:wat::rete::where …)` node at that site for a second pass to find.
;;
;; Usage (one EDN vector of paths on stdin):
;;   printf '["tests/rete/datamancer.src.wat" …]\n' | cargo wat ./wat-scripts/fixes/hoist-where-into-condition.wat

;; ── shape classification ─────────────────────────────────────────────────────────────────────

;; kw-name / head-name / calls-to? / node-start-offset / node-end-offset already live in
;; wat/fix.wat (:wat::fix::…) — reused directly, not re-defined.

;; bind-clause-var — `(?v <- :field)` (bare field keyword, no "::"): the var name, or "" if this
;; clause is not that shape. Mirrors the `Bind` arm of `classify_rete_clause`.
(:wat::core::defn :user::bind-clause-var [c <- :wat::WatAST] -> :wat::core::String
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind c) "list")
    (:wat::core::let [ch (:wat::core::ast->children c)]
      (:wat::core::if (:wat::core::= (:wat::core::length ch) 3)
        (:wat::core::let [h0 (:wat::core::first ch)
                          h1 (:wat::core::nth ch 1)
                          h2 (:wat::core::nth ch 2)]
          (:wat::core::if (:wat::core::if (:wat::core::= (:wat::core::ast-kind h0) "symbol")
                            (:wat::core::string::starts-with? (:wat::core::ast-name h0) "?") false)
            (:wat::core::if (:wat::core::if (:wat::core::= (:wat::core::ast-kind h1) "symbol")
                              (:wat::core::= (:wat::core::ast-name h1) "<-") false)
              (:wat::core::if (:wat::core::if (:wat::core::= (:wat::core::ast-kind h2) "keyword")
                                (:wat::core::string::contains? (:wat::core::ast-name h2) "::") false)
                ""
                (:wat::core::if (:wat::core::= (:wat::core::ast-kind h2) "keyword")
                  (:wat::core::ast-name h0)
                  ""))
              "")
            ""))
        ""))
    ""))

;; top-shape-tag — one of "where"/"not"/"exists"/"or"/"and"/"factbind"/"accumulate"/"plain"/"other".
;; Mirrors classify_rete_clause's top-level dispatch, restricted to what hoist-target/where-site
;; collection needs to know.
(:wat::core::defn :user::top-shape-tag [node <- :wat::WatAST] -> :wat::core::String
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind node) "list")
    (:wat::core::let [ch (:wat::core::ast->children node)]
      (:wat::core::if (:wat::core::empty? ch)
        "other"
        (:wat::core::let [head (:wat::core::first ch)]
          (:wat::core::if (:wat::core::= (:wat::core::ast-kind head) "keyword")
            (:wat::core::let [hn (:wat::core::ast-name head)]
              (:wat::core::cond
                ((:wat::core::= hn ":wat::rete::where") "where")
                ((:wat::core::= hn ":wat::rete::not") "not")
                ((:wat::core::= hn ":wat::rete::exists") "exists")
                ((:wat::core::= hn ":wat::rete::or") "or")
                ((:wat::core::= hn ":wat::rete::and") "and")
                (:else "plain")))
            (:wat::core::if (:wat::core::if (:wat::core::= (:wat::core::ast-kind head) "symbol")
                              (:wat::core::string::starts-with? (:wat::core::ast-name head) "?") false)
              (:wat::core::let [n (:wat::core::length ch)]
                (:wat::core::if (:wat::core::if (:wat::core::>= n 3)
                                  (:wat::core::if (:wat::core::= (:wat::core::ast-kind (:wat::core::nth ch 1)) "symbol")
                                    (:wat::core::= (:wat::core::ast-name (:wat::core::nth ch 1)) "<-") false)
                                  false)
                  (:wat::core::let [third (:wat::core::nth ch 2)]
                    (:wat::core::cond
                      ((:wat::core::if (:wat::core::= (:wat::core::ast-kind third) "keyword")
                         (:wat::core::string::contains? (:wat::core::ast-name third) "::") false)
                        "factbind")
                      ((:wat::core::if (:wat::core::= n 5)
                         (:wat::core::if (:wat::core::= (:wat::core::ast-kind (:wat::core::nth ch 3)) "keyword")
                           (:wat::core::= (:wat::core::ast-name (:wat::core::nth ch 3)) ":from") false)
                         false)
                        "accumulate")
                      (:else "other")))
                  "other"))
              "other")))))
    "other"))

;; bound-vars-of-plain / bound-vars-of-factbind — every `?var` a candidate hoist-target condition
;; binds. "plain": every inline bind-clause among its OWN clauses. "factbind": the fact-bind var
;; itself, plus every inline bind-clause. Mirrors `plain_pattern_bound_vars`.
(:wat::core::defn :user::bound-vars-of-plain [node <- :wat::WatAST] -> (:wat::core::Vector :- [:wat::core::String])
  (:wat::core::let [ch (:wat::core::ast->children node)
                    clauses (:wat::core::into [] (:wat::core::rest ch))]
    (:wat::core::foldl
      (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::core::String]) c <- :wat::WatAST]
        -> (:wat::core::Vector :- [:wat::core::String])
        (:wat::core::let [v (:user::bind-clause-var c)]
          (:wat::core::if (:wat::core::= v "") acc (:wat::core::conj acc v))))
      (:wat::core::Vector :wat::core::String)
      clauses)))

(:wat::core::defn :user::bound-vars-of-factbind [node <- :wat::WatAST] -> (:wat::core::Vector :- [:wat::core::String])
  (:wat::core::let [ch (:wat::core::ast->children node)
                    head-var (:wat::core::ast-name (:wat::core::first ch))
                    clauses (:wat::core::into [] (:wat::core::drop ch 3))]
    (:wat::core::foldl
      (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::core::String]) c <- :wat::WatAST]
        -> (:wat::core::Vector :- [:wat::core::String])
        (:wat::core::let [v (:user::bind-clause-var c)]
          (:wat::core::if (:wat::core::= v "") acc (:wat::core::conj acc v))))
      (:wat::core::conj (:wat::core::Vector :wat::core::String) head-var)
      clauses)))

;; var-occurrences — every `?`-prefixed symbol reachable under `node` (predicate expressions have
;; no rete-specific shape left to respect once we're inside them: a var occurrence is a var
;; occurrence anywhere in the tree). Self-recursive only.
(:wat::core::defn :user::var-occurrences [node <- :wat::WatAST] -> (:wat::core::Vector :- [:wat::core::String])
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind node) "symbol")
    (:wat::core::if (:wat::core::string::starts-with? (:wat::core::ast-name node) "?")
      (:wat::core::Vector :wat::core::String (:wat::core::ast-name node))
      (:wat::core::Vector :wat::core::String))
    (:wat::core::if (:wat::fix::structural? node)
      (:wat::core::foldl
        (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::core::String]) c <- :wat::WatAST]
          -> (:wat::core::Vector :- [:wat::core::String])
          (:wat::core::concat acc (:user::var-occurrences c)))
        (:wat::core::Vector :wat::core::String)
        (:wat::core::into [] (:wat::core::ast->children node)))
      (:wat::core::Vector :wat::core::String))))

(:wat::core::defn :user::dedup [items <- (:wat::core::Vector :- [:wat::core::String])] -> (:wat::core::Vector :- [:wat::core::String])
  (:wat::core::foldl
    (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::core::String]) x <- :wat::core::String]
      -> (:wat::core::Vector :- [:wat::core::String])
      (:wat::core::if (:wat::core::contains? acc x) acc (:wat::core::conj acc x)))
    (:wat::core::Vector :wat::core::String)
    items))

(:wat::core::defn :user::subset? [small <- (:wat::core::Vector :- [:wat::core::String]) big <- (:wat::core::Vector :- [:wat::core::String])] -> :wat::core::bool
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::bool x <- :wat::core::String]
      -> :wat::core::bool
      (:wat::core::if acc (:wat::core::contains? big x) false))
    true
    small))

;; ws-char? / backward-trim — a deleted `where` must take its own LINE with it, not leave a
;; whitespace-only line behind (the coordinator's Phase-1 refinement: nothing GATES a dangling
;; indented `]`, but 63 sites of it is noise in files someone will be resolving merge conflicts
;; in). `backward-trim` walks backward from a node's own start offset through every contiguous
;; whitespace character (space/tab/CR/LF — spanning as many blank lines as happen to be there,
;; though this corpus never has more than one), stopping at the first non-whitespace character
;; (or offset 0). The deletion then runs from THAT position to the node's own end — folding in
;; its leading indentation and the newline before it — while never touching anything forward of
;; the node (no symmetric forward-trim): that asymmetry is what keeps two adjacent hoisted
;; `where`s' deletion spans exactly abutting, never overlapping — see the file-level comment.
(:wat::core::defn :user::ws-char? [c <- :wat::core::String] -> :wat::core::bool
  (:wat::core::if (:wat::core::= c " ") true
    (:wat::core::if (:wat::core::= c "\t") true
      (:wat::core::if (:wat::core::= c "\n") true
        (:wat::core::= c "\r")))))

(:wat::core::defn :user::backward-trim [src <- :wat::core::String pos <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::if (:wat::core::<= pos 0)
    0
    (:wat::core::let [c (:wat::core::string::subs src (:wat::core::i64::- pos 1) pos)]
      (:wat::core::if (:user::ws-char? c)
        (:user::backward-trim src (:wat::core::i64::- pos 1))
        pos))))

;; ── collection, transparent through `:and` only ─────────────────────────────────────────────

;; collect-hoist-targets — every "plain"/"factbind" condition reachable from `items`, recursing
;; ONLY through "and". Never descends into where/not/exists/or/accumulate/other.
(:wat::core::defn :user::collect-hoist-targets [items <- (:wat::core::Vector :- [:wat::WatAST])] -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::WatAST (:wat::core::Vector :- [:wat::core::String])])])
  (:wat::core::if (:wat::core::empty? items)
    (:wat::core::Vector (:wat::core::Tuple :- [:wat::WatAST (:wat::core::Vector :- [:wat::core::String])]))
    (:wat::core::let [it (:wat::core::first items)
                      tl (:wat::core::into [] (:wat::core::rest items))
                      tag (:user::top-shape-tag it)]
      (:wat::core::cond
        ((:wat::core::= tag "and")
          (:wat::core::concat
            (:user::collect-hoist-targets (:wat::core::into [] (:wat::core::rest (:wat::core::ast->children it))))
            (:user::collect-hoist-targets tl)))
        ((:wat::core::= tag "plain")
          (:wat::core::conj (:user::collect-hoist-targets tl) (:wat::core::Tuple it (:user::bound-vars-of-plain it))))
        ((:wat::core::= tag "factbind")
          (:wat::core::conj (:user::collect-hoist-targets tl) (:wat::core::Tuple it (:user::bound-vars-of-factbind it))))
        (:else (:user::collect-hoist-targets tl))))))

;; collect-where-sites — every `(:wat::rete::where <pred>)` reachable from `items`, recursing ONLY
;; through "and" (same scope rule as collect-hoist-targets — a `where` inside `:or`/`:not`/
;; `:exists`/an accumulate is never even visited, so it can never be offered a hoist).
(:wat::core::defn :user::collect-where-sites [items <- (:wat::core::Vector :- [:wat::WatAST])] -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::WatAST :wat::WatAST])])
  (:wat::core::if (:wat::core::empty? items)
    (:wat::core::Vector (:wat::core::Tuple :- [:wat::WatAST :wat::WatAST]))
    (:wat::core::let [it (:wat::core::first items)
                      tl (:wat::core::into [] (:wat::core::rest items))
                      tag (:user::top-shape-tag it)]
      (:wat::core::cond
        ((:wat::core::= tag "where")
          (:wat::core::concat
            (:wat::core::Vector (:wat::core::Tuple :- [:wat::WatAST :wat::WatAST])
              (:wat::core::Tuple it (:wat::core::nth (:wat::core::ast->children it) 1)))
            (:user::collect-where-sites tl)))
        ((:wat::core::= tag "and")
          (:wat::core::concat
            (:user::collect-where-sites (:wat::core::into [] (:wat::core::rest (:wat::core::ast->children it))))
            (:user::collect-where-sites tl)))
        (:else (:user::collect-where-sites tl))))))

;; find-when-vector — the vector immediately after a `:when` keyword among `ch` (a defrule's /
;; defquery's own children). None if this form carries no `:when` (malformed, or not one of ours —
;; the caller already gated on the head keyword).
(:wat::core::defn :user::find-when-vector-at [ch <- (:wat::core::Vector :- [:wat::WatAST]) i <- :wat::core::i64] -> (:wat::core::Option :wat::WatAST)
  (:wat::core::if (:wat::core::>= (:wat::core::i64::+ i 1) (:wat::core::length ch))
    (:wat::core::None :wat::WatAST)
    (:wat::core::let [c (:wat::core::Option/expect (:wat::core::get ch i) "find-when-vector-at c")]
      (:wat::core::if (:wat::core::if (:wat::core::= (:wat::core::ast-kind c) "keyword")
                        (:wat::core::= (:wat::core::ast-name c) ":when") false)
        (:wat::core::Some (:wat::core::Option/expect (:wat::core::get ch (:wat::core::i64::+ i 1)) "find-when-vector-at next"))
        (:user::find-when-vector-at ch (:wat::core::i64::+ i 1))))))

(:wat::core::defn :user::find-when-vector [ch <- (:wat::core::Vector :- [:wat::WatAST])] -> (:wat::core::Option :wat::WatAST)
  (:user::find-when-vector-at ch 0))

;; ── edits ────────────────────────────────────────────────────────────────────────────────────

;; merge-insert-into — accumulate (offset, text) pairs, concatenating onto an EXISTING entry at
;; the same offset rather than emitting a second insert there. Needed because two `where`s can
;; hoist into the SAME target condition (`probe_arc278_sift_rules_arena.wat`'s 3-where rules,
;; `datamancer.src.wat`'s `:dm::four`) — `fix-text-apply` splices two same-offset inserts in
;; REVERSE of their list order (each later insert lands to the LEFT of the earlier one, since it
;; is applied to the already-spliced string at the same absolute offset), so building ONE insert
;; per target, in encounter (source) order, is what keeps the appended clauses in declaration
;; order instead of an accidental reversal.
(:wat::core::defn :user::merge-insert-into
  [acc <- (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String])])
   off <- :wat::core::i64 text <- :wat::core::String]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String])])
  (:wat::core::if (:wat::core::empty? acc)
    (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::i64 :wat::core::String]) (:wat::core::Tuple off text))
    (:wat::core::let [h (:wat::core::first acc) tl (:wat::core::into [] (:wat::core::rest acc))]
      (:wat::core::if (:wat::core::= (:wat::core::first h) off)
        (:wat::core::concat
          (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::i64 :wat::core::String])
            (:wat::core::Tuple off (:wat::core::string::concat (:wat::core::second h) text)))
          tl)
        (:wat::core::concat
          (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::i64 :wat::core::String]) h)
          (:user::merge-insert-into tl off text))))))

;; rule-form-edits — every hoist edit for ONE defrule/defquery form's `:when` vector. Deletions
;; are independent per `where` (each has its own unique span); insertions are MERGED per target
;; (via merge-insert-into) so N `where`s landing on the same condition become one insert with N
;; clauses appended in source order, not N separately-ordered inserts.
(:wat::core::defn :user::rule-form-edits [node <- :wat::WatAST src <- :wat::core::String lines <- (:wat::core::Vector :- [:wat::core::String])] -> (:wat::core::Vector :- [:wat::fix::Edit])
  (:wat::core::let [ch (:wat::core::into [] (:wat::core::ast->children node))
                    wvopt (:user::find-when-vector ch)]
    (:wat::core::match wvopt
      (:wat::core::None (:wat::core::Vector :wat::fix::Edit))
      ((:wat::core::Some wv)
        (:wat::core::let
          [items (:wat::core::into [] (:wat::core::ast->children wv))
           targets (:user::collect-hoist-targets items)
           wheres (:user::collect-where-sites items)
           result (:wat::core::foldl
                    (:wat::core::fn
                      [acc <- (:wat::core::Tuple :- [(:wat::core::Vector :- [:wat::fix::Edit]) (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String])])])
                       ws  <- (:wat::core::Tuple :- [:wat::WatAST :wat::WatAST])]
                      -> (:wat::core::Tuple :- [(:wat::core::Vector :- [:wat::fix::Edit]) (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String])])])
                      (:wat::core::let
                        [w (:wat::core::first ws) pred (:wat::core::second ws)
                         vars (:user::dedup (:user::var-occurrences pred))
                         matches (:wat::core::foldl
                                   (:wat::core::fn [macc <- (:wat::core::Vector :- [:wat::WatAST]) t <- (:wat::core::Tuple :- [:wat::WatAST (:wat::core::Vector :- [:wat::core::String])])]
                                     -> (:wat::core::Vector :- [:wat::WatAST])
                                     (:wat::core::if (:user::subset? vars (:wat::core::second t)) (:wat::core::conj macc (:wat::core::first t)) macc))
                                   (:wat::core::Vector :wat::WatAST) targets)
                         dels (:wat::core::first acc)
                         inss (:wat::core::second acc)]
                        (:wat::core::if (:wat::core::= (:wat::core::length matches) 1)
                          (:wat::core::let
                            [target (:wat::core::Option/expect (:wat::core::get matches 0) "rule-form-edits target")
                             w-start-raw (:wat::fix::node-start-offset w lines)
                             w-start (:user::backward-trim src w-start-raw)
                             w-end (:wat::fix::node-end-offset w lines)
                             pred-start (:wat::fix::node-start-offset pred lines)
                             pred-end (:wat::fix::node-end-offset pred lines)
                             pred-text (:wat::core::string::subs src pred-start pred-end)
                             ins-off (:wat::core::i64::- (:wat::fix::node-end-offset target lines) 1)]
                            (:wat::core::Tuple
                              (:wat::core::conj dels (:wat::core::Tuple w-start (:wat::core::i64::- w-end w-start) ""))
                              (:user::merge-insert-into inss ins-off (:wat::core::string::concat " " pred-text))))
                          acc)))
                    (:wat::core::Tuple (:wat::core::Vector :wat::fix::Edit) (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::i64 :wat::core::String])))
                    wheres)
           dels (:wat::core::first result)
           inss (:wat::core::second result)
           ins-edits (:wat::core::foldl
                       (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::fix::Edit]) t <- (:wat::core::Tuple :- [:wat::core::i64 :wat::core::String])]
                         -> (:wat::core::Vector :- [:wat::fix::Edit])
                         (:wat::core::conj acc (:wat::core::Tuple (:wat::core::first t) 0 (:wat::core::second t))))
                       (:wat::core::Vector :wat::fix::Edit)
                       inss)]
          (:wat::core::concat dels ins-edits))))))

;; walk-node — every defrule/defquery form's edits, anywhere in the tree (a rule may sit inside a
;; `:rules […]` vector passed to a macro, not only at top level — this is a pure text-span splice,
;; so where a rule form SITS syntactically makes no difference to the transform).
(:wat::core::defn :user::walk-node [node <- :wat::WatAST src <- :wat::core::String lines <- (:wat::core::Vector :- [:wat::core::String])] -> (:wat::core::Vector :- [:wat::fix::Edit])
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind node) "list")
    (:wat::core::let [hn (:wat::fix::head-name node)
                      this (:wat::core::if (:wat::core::if (:wat::core::= hn ":wat::rete::defrule") true (:wat::core::= hn ":wat::rete::defquery"))
                             (:user::rule-form-edits node src lines)
                             (:wat::core::Vector :wat::fix::Edit))]
      (:wat::core::concat this
        (:wat::core::foldl
          (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::fix::Edit]) c <- :wat::WatAST]
            -> (:wat::core::Vector :- [:wat::fix::Edit])
            (:wat::core::concat acc (:user::walk-node c src lines)))
          (:wat::core::Vector :wat::fix::Edit)
          (:wat::core::into [] (:wat::core::ast->children node)))))
    (:wat::core::if (:wat::fix::structural? node)
      (:wat::core::foldl
        (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::fix::Edit]) c <- :wat::WatAST]
          -> (:wat::core::Vector :- [:wat::fix::Edit])
          (:wat::core::concat acc (:user::walk-node c src lines)))
        (:wat::core::Vector :wat::fix::Edit)
        (:wat::core::into [] (:wat::core::ast->children node)))
      (:wat::core::Vector :wat::fix::Edit))))

;; ── per-file migrate ─────────────────────────────────────────────────────────────────────────
(:wat::core::defn :user::migrate [src <- :wat::core::String] -> :wat::core::String
  (:wat::core::let [lines (:wat::core::string::split src "\n")
                    tree (:wat::core::match (:wat::core::read-string src) ((:wat::core::ReadOutcome::Forms __forms) __forms) ((:wat::core::ReadOutcome::Malformed __cause) (:wat::kernel::assertion-failed! (:wat::core::Error/message __cause) :wat::core::None :wat::core::None)))
                    forms (:wat::core::into [] (:wat::core::ast->children tree))
                    edits (:wat::core::foldl
                            (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::fix::Edit]) f <- :wat::WatAST]
                              -> (:wat::core::Vector :- [:wat::fix::Edit])
                              (:wat::core::concat acc (:user::walk-node f src lines)))
                            (:wat::core::Vector :wat::fix::Edit)
                            forms)
                    ;; high-offset-first so a low insert/delete never shifts a pending higher one.
                    rev (:wat::core::reverse (:wat::core::sort edits))]
    (:wat::fix::fix-text-apply src rev)))

;; ── driver: one path per line on stdin (EDN vector of strings) ─────────────────────────────────
(:wat::core::defn :user::apply-each
  [paths <- (:wat::core::Vector :- [:wat::core::String])] -> :wat::core::nil
  (:wat::core::if (:wat::core::empty? paths)
    nil
    (:wat::core::let [path (:wat::core::first paths)]
      (:wat::core::do
        (:wat::io::write-file path
          (:user::migrate (:wat::io::read-file path)))
        (:wat::kernel::println (:wat::core::string::concat "[hoisted] " path))
        (:user::apply-each (:wat::core::rest paths))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:user::apply-each
    (:wat::core::match (:wat::kernel::readln ) ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum) (:wat::kernel::ReadlnOutcome::Eof (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None)) (:wat::kernel::ReadlnOutcome::Stopped (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)))))
