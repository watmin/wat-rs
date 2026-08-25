;; where-census-walker.wat — scout instrument for the `where` vocabulary census (arc 278, #57 mint).
;; Read-only reconnaissance: read-string + with-children-style structural recursion (fix.wat's own
;; machinery), NEVER a grep. Finds every literal (:wat::rete::where <expr>) form in each given file
;; and prints every call-head found inside <expr>, at every depth, tagged by file + occurrence + line.
;;
;; MATCH SPECIAL-CASE: a `(:wat::core::match scrutinee (pattern body) ...)` form's arm PATTERNS are
;; discriminators, not calls — a tagged-variant pattern like `(:wat::core::Some v)` must not be
;; logged as if `:wat::core::Some` were invoked. So `match` walks its scrutinee generically, counts
;; its own head once, then for each arm skips the pattern position and only descends into the body.
;; `fn`/`cond`/`if`/`let`/`do` need no such treatment — their sub-positions are all genuine exprs
;; (verified by hand against this corpus before writing this walker).
;;
;; Output lines:
;;   PARSE-FAIL <path>                      — STOP-1: read-string could not parse the file
;;   WHERE <path> #<n> @L<line>             — a where form was found (marks its existence + locus)
;;   HEAD <path> #<n> @L<line> <head>       — one call-head occurrence inside that where's expr

(:wat::core::defn :user::structural? [node <- :wat::WatAST] -> :wat::core::bool
  (:wat::core::let [k (:wat::core::ast-kind node)]
    (:wat::core::contains? (:wat::core::HashSet :wat::type::Infer "list" "vector" "map" "set") k)))

(:wat::core::defn :user::kw-name [n <- :wat::WatAST] -> :wat::core::String
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind n) "keyword")
    (:wat::core::ast-name n) ""))

(:wat::core::defn :user::head-name [node <- :wat::WatAST] -> :wat::core::String
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind node) "list")
    (:wat::core::let [ch (:wat::core::ast->children node)]
      (:wat::core::if (:wat::core::empty? ch) "" (:user::kw-name (:wat::core::first ch))))
    ""))

(:wat::core::defn :user::match-head? [h <- :wat::core::String] -> :wat::core::bool
  (:wat::core::if (:wat::core::= h ":wat::core::match") true
    (:wat::core::= h ":wat::rete::core::match")))

;; walk-match-arms — items are (pattern body...) lists (or malformed leftovers); skip position 0
;; (the pattern/discriminator), walk-collect over positions 1+ (the body).
(:wat::core::defn :user::walk-match-arms
  [arms <- (:wat::core::Vector :- [:wat::WatAST]) tag <- :wat::core::String] -> :wat::core::nil
  (:wat::core::if (:wat::core::empty? arms)
    nil
    (:wat::core::let [arm (:wat::core::first arms)]
      (:wat::core::do
        (:wat::core::if (:wat::core::= (:wat::core::ast-kind arm) "list")
          (:user::walk-collect-seq (:wat::core::into [] (:wat::core::rest (:wat::core::ast->children arm))) tag)
          nil)
        (:user::walk-match-arms (:wat::core::into [] (:wat::core::rest arms)) tag)))))

;; walk-collect — the generic head-collector. Prints this node's head (if it is a call), then
;; recurses: match gets the special arm-skipping treatment above; everything else (if/let/do/fn/
;; cond/quasiquote/unquote/plain calls/vectors/maps/sets) recurses into ALL children, because every
;; sub-position there is a genuine expression, not a discriminator.
(:wat::core::defn :user::walk-collect [node <- :wat::WatAST tag <- :wat::core::String] -> :wat::core::nil
  (:wat::core::do
    (:wat::core::if (:wat::core::= (:wat::core::ast-kind node) "list")
      (:wat::core::let [ch (:wat::core::ast->children node)]
        (:wat::core::if (:wat::core::empty? ch)
          nil
          (:wat::core::let [h (:user::head-name node)]
            (:wat::core::if (:wat::core::= h "") nil
              (:wat::kernel::println (:wat::string::concat tag h))))))
      nil)
    (:wat::core::if (:wat::core::= (:wat::core::ast-kind node) "list")
      (:wat::core::let [ch (:wat::core::ast->children node) h (:user::head-name node)]
        (:wat::core::if (:user::match-head? h)
          (:wat::core::if (:wat::core::< (:wat::core::length ch) 2)
            nil
            (:wat::core::do
              (:user::walk-collect (:wat::core::Option/expect (:wat::core::get ch 1) "match scrutinee") tag)
              (:user::walk-match-arms (:wat::core::into [] (:wat::core::drop ch 2)) tag)))
          (:wat::core::if (:user::structural? node)
            (:user::walk-collect-seq ch tag)
            nil)))
      (:wat::core::if (:user::structural? node)
        (:user::walk-collect-seq (:wat::core::ast->children node) tag)
        nil))))

(:wat::core::defn :user::walk-collect-seq
  [items <- (:wat::core::Vector :- [:wat::WatAST]) tag <- :wat::core::String] -> :wat::core::nil
  (:wat::core::if (:wat::core::empty? items)
    nil
    (:wat::core::do
      (:user::walk-collect (:wat::core::first items) tag)
      (:user::walk-collect-seq (:wat::core::into [] (:wat::core::rest items)) tag))))

;; find-wheres — full-tree walk looking for (:wat::rete::where <expr...>) forms. On a hit: print a
;; WHERE marker (path/index/line) then walk-collect over every child AFTER the head keyword (drop 1)
;; — `where`'s own head is never counted as a vocabulary head found INSIDE itself. Continues
;; recursing into the found node's children too (honest census: a nested where, however unlikely,
;; would still be caught).
(:wat::core::defn :user::find-wheres
  [node <- :wat::WatAST path <- :wat::core::String ctr <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::let [h (:user::head-name node)
                    is-where (:wat::core::= h ":wat::rete::where")
                    ctr2 (:wat::core::if is-where
                           (:wat::core::let
                             [n    (:wat::core::+ ctr 1)
                              ln   (:wat::core::Option/expect (:wat::core::HashMap/get (:wat::core::ast-span node) :line) "line")
                              tag  (:wat::string::concat path
                                     (:wat::string::concat " #"
                                       (:wat::string::concat (:wat::core::str n)
                                         (:wat::string::concat " @L"
                                           (:wat::string::concat (:wat::core::str ln) " ")))))]
                             (:wat::core::do
                               (:wat::kernel::println (:wat::string::concat "WHERE " tag))
                               (:user::walk-collect-seq
                                 (:wat::core::into [] (:wat::core::rest (:wat::core::ast->children node)))
                                 (:wat::string::concat "HEAD " tag))
                               n))
                           ctr)]
    (:wat::core::if (:user::structural? node)
      (:user::find-wheres-seq (:wat::core::ast->children node) path ctr2)
      ctr2)))

(:wat::core::defn :user::find-wheres-seq
  [items <- (:wat::core::Vector :- [:wat::WatAST]) path <- :wat::core::String ctr <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::if (:wat::core::empty? items)
    ctr
    (:wat::core::let [ctr2 (:user::find-wheres (:wat::core::first items) path ctr)]
      (:user::find-wheres-seq (:wat::core::into [] (:wat::core::rest items)) path ctr2))))

(:wat::core::defn :user::process-file [path <- :wat::core::String] -> :wat::core::nil
  (:wat::core::let [src (:wat::io::read-file path)]
    (:wat::core::match (:wat::core::read-string src)
      ((:wat::core::ReadOutcome::Forms __f)
        (:wat::core::do (:user::find-wheres-seq (:wat::core::ast->children __f) path 0) nil))
      ((:wat::core::ReadOutcome::Malformed __c)
        (:wat::core::do
          (:wat::kernel::println (:wat::string::concat "PARSE-FAIL " path))
          nil)))))

(:wat::core::defn :user::process-seq [paths <- (:wat::core::Vector :- [:wat::core::String])] -> :wat::core::nil
  (:wat::core::if (:wat::core::empty? paths)
    nil
    (:wat::core::do
      (:user::process-file (:wat::core::first paths))
      (:user::process-seq (:wat::core::into [] (:wat::core::rest paths))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:user::process-seq
    (:wat::core::Vector :wat::core::String
      "wat-scripts/perf/grid/where-boolean.wat"
      "wat-scripts/perf/grid/where-collection.wat"
      "wat-scripts/perf/grid/where-control.wat"
      "wat-scripts/perf/grid/where-multivar.wat"
      "wat-scripts/perf/grid/where-nesting.wat"
      "wat-scripts/perf/grid/where-numeric.wat"
      "wat-scripts/perf/grid/where-record.wat"
      "wat-scripts/perf/grid/where-shapes.wat"
      "wat-scripts/perf/grid/where-string.wat"
      "wat-scripts/perf/grid/min-finding.wat"
      "wat-scripts/perf/grid/node-share.wat"
      "wat-scripts/perf/grid/strat-neg.wat"
      "wat/rete.wat"
      "wat-scripts/fixes/rule-record-to-defrule.wat"
      "wat-scripts/fixes/to-faithful-clojure-rete.wat"
      "wat-scripts/fixes/to-faithful-clojure-net.wat"
      "wat-scripts/fixes/rete-truth-maintenance-probes/neg.wat"
      "wat-scripts/scratch-pad/probe-rules-rich.wat"
      "wat-scripts/scratch-pad/probe-overlay-refire-cost.wat"
      "wat-scripts/scratch-pad/probe-where-cond-fence-execution-split.wat"
      "wat-scripts/scratch-pad/probe-where-shape-spread.wat"
      "wat-scripts/scratch-pad/probe-stop-a-where-arith-path.wat"
      "wat-scripts/scratch-pad/probe-node-share-phase-split.wat"
      "wat-scripts/scratch-pad/probe-derive-chain-split.wat"
      "wat-scripts/scratch-pad/probe-arena-rich-graph.wat"
      "wat-scripts/scratch-pad/probe-node-share-dedup.wat"
      "tests/rete/probe_arc278_6b_ii_a_where_oracle_impure.wat"
      "tests/rete/probe_arc278_6b_ii_a_where_oracle_userfn.wat"
      "tests/rete/probe_arc300_2_fix_defrule.wat"
      "tests/rete/probe_fence_names_the_head_nondet.wat"
      "tests/rete/probe_arc278_7strat_native_differential.wat"
      "tests/rete/probe_arc278_6b_ii_a_where_oracle_cmp.wat"
      "tests/services/probe_arc278_sift_rules_arena.wat")))
