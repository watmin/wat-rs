;; wat-scripts/perf/grid/deep-cascade.wat — GRID AXIS A0: deep forward-chain cascade, IN WAT.
;;
;; Adapted from the legacy `wat-scripts/perf/deep-cascade.wat` (R4's original A0 bench) into the
;; Clara-grid contract (`run-axis.sh`, DESIGN-STONE-a0-a1-into-the-grid.md): stdin becomes a plain
;; i64 vector (not a `:perf::Params` record), and `:derived` becomes the FULL SORTED canonical fact
;; vector (not a count) so the accuracy differential can actually catch a wrong answer that happens
;; to have the right cardinality. The legacy file is untouched — R4's record, left in place.
;;
;; Shape (unchanged from the legacy bench): a depth-N x width-M cascade where every level is a
;; 2-way JOIN on the prior level's DERIVED facts — M independent join-chains, each N levels deep.
;;   Node(0,i), Tag(0,i)     — seeded for i in [0, width).            [input, level 0]
;;   Node(k,id), Tag(k,id)   :- Node(k-1,id) AND Tag(k-1,id)          for k in [1, depth]
;; Every id survives every level (the joins never drop anyone), so exactly 2*depth*width facts are
;; DERIVED (level 0 is input, excluded from the witness). `depth` is the swept dial; `width` is
;; held fixed at 100. The ladder first shipped with width 5 — 300 derived facts, a 4 ms fire —
;; which is an order of magnitude under every other grid axis and below R4's own 30x10 top cell.
;; At that size the ratio measured JIT and scheduler jitter: the same rung read 5.96x and 12.12x
;; twenty minutes apart, and reported 4-12x :us over a cell that, sized honestly, measured
;; :winner :clara. Width 100 puts every rung in the grid's own measurement class (~6-38 ms).
;;
;; Fires the NATIVE production verb `:wat::rete::fire-rules` (the differential-tested fast path,
;; the same verb every other grid axis uses) — NOT the legacy file's two-timing wat-oracle-vs-native
;; split; the grid measures one number, `:native-ns`.
;;
;; :derived is the full sorted i64 vector of every derived Node/Tag fact past level 0, each fact
;; canonicalized as kind*1e15 + level*1e9 + id (kind: 0=Node, 1=Tag) — same encoding SHAPE as
;; accum.wat's `:acc::enc`, injective at grid scale (level <= 50, id < width <= 100).
;;
;; Usage (stdin = an i64 vector [depth width]; stdout = one #grid/Result EDN line):
;;   echo '[10 100]' | cargo wat ./wat-scripts/perf/grid/deep-cascade.wat
;;   => #grid/Result {:axis "deep-cascade" :size [10 100] :derived [...] :native-ns N}

(:wat::core::defrecord :cascade::Node [level <- :wat::core::i64  id <- :wat::core::i64])
(:wat::core::defrecord :cascade::Tag  [level <- :wat::core::i64  id <- :wat::core::i64])

(:wat::core::defrecord :grid::Result
  [axis      <- :wat::core::String
   size      <- (:wat::core::PersistentVector :- [:wat::core::i64])
   derived   <- (:wat::core::PersistentVector :- [:wat::core::i64])
   native-ns      <- :wat::core::i64
   ;; THREE-WAY: the wat SPEC's own answer, so the runner can render :oracle-accuracy
   ;; (spec vs Clara) and :port-accuracy (spec vs native) instead of one verdict.
   oracle-derived <- (:wat::core::PersistentVector :- [:wat::core::i64])
   oracle-ns      <- :wat::core::i64
   insert-ns      <- :wat::core::i64
   fire-ns        <- :wat::core::i64
   query-ns       <- :wat::core::i64
   protocol-ns    <- :wat::core::i64])

(:wat::rete::defquery :cascade::q-Node
  :params []
  :when [(?fact <- :cascade::Node)])


(:wat::rete::defquery :cascade::q-Tag
  :params []
  :when [(?fact <- :cascade::Tag)])


;; build-rule k — the k-th cascade level: join Node⋈Tag at level (k-1) on ?id, derive Node,Tag at
;; level k. The level literals (k-1 in the conditions, k in the inserts) are spliced via
;; quasiquote/unquote (byte-identical in shape to the legacy file's :perf::build-rule).
(:wat::core::defn :dc::build-rule [k <- :wat::core::i64] -> :wat::rete::Rule
  (:wat::core::let [prev (:wat::core::i64::- k 1)
                    c1 (:wat::core::quasiquote (:cascade::Node (?id <- :id) (?l <- :level) (:wat::core::= ?l (:wat::core::unquote prev))))
                    c2 (:wat::core::quasiquote (:cascade::Tag  (?id <- :id) (?m <- :level) (:wat::core::= ?m (:wat::core::unquote prev))))
                    t1 (:wat::core::quasiquote (:cascade::Node (:wat::core::unquote k) ?id))
                    t2 (:wat::core::quasiquote (:cascade::Tag  (:wat::core::unquote k) ?id))]
    (:wat::rete::Rule :name (:wat::core::i64::to-string k)
      :lhs (:wat::core::PersistentVector c1 c2)
      :rhs (:wat::core::PersistentVector t1 t2))))

;; build-rules depth — the rule set [rule1 .. rule depth], folding build-rule over (range 1 depth+1).
(:wat::core::defn :dc::build-rules [depth <- :wat::core::i64] -> (:wat::core::PersistentVector :- [:wat::rete::Rule])
  (:wat::core::foldl
    (:wat::core::fn [acc <- (:wat::core::PersistentVector :- [:wat::rete::Rule])  k <- :wat::core::i64] -> (:wat::core::PersistentVector :- [:wat::rete::Rule])
      (:wat::core::PersistentVector/conj acc (:dc::build-rule k)))
    (:wat::core::PersistentVector (:dc::build-rule 1))
    (:wat::core::range 2 (:wat::core::i64::+ depth 1))))

;; seed-level-0 session width — stage Node(0,i)+Tag(0,i) for i in [0, width), threading the session.
;; Staged with the BATCH verb — one `insert-all` (native, one rebuild) rather than `insert` x 2N.
(:wat::core::defn :dc::level-0-facts [width <- :wat::core::i64] -> (:wat::core::PersistentVector :- [:wat::core::Record])
  (:wat::core::foldl
    (:wat::core::fn [acc <- (:wat::core::PersistentVector :- [:wat::core::Record])  i <- :wat::core::i64]
                    -> (:wat::core::PersistentVector :- [:wat::core::Record])
      (:wat::core::PersistentVector/conj
        (:wat::core::PersistentVector/conj acc (:cascade::Node :level 0 :id i))
        (:cascade::Tag :level 0 :id i)))
    (:wat::core::PersistentVector)
    (:wat::core::range 0 width)))

(:wat::core::defn :dc::seed-level-0 [session <- :wat::rete::Session  width <- :wat::core::i64] -> :wat::rete::Session
  (:wat::rete::insert-all session (:dc::level-0-facts width)))

;; enc kind level id — canonical single-i64 witness for one derived fact (mirrors accum.wat's
;; :acc::enc: kind*1e15 + level*1e9 + id).
(:wat::core::defn :dc::enc [kind <- :wat::core::i64  level <- :wat::core::i64  id <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::i64::+
    (:wat::core::i64::+ (:wat::core::i64::* kind 1000000000000000) (:wat::core::i64::* level 1000000000))
    id))

;; vec->pvec v — materialize a (Vector :- [i64]) into a (PersistentVector :- [i64]). DESIGN-STONE-into-pv-
;; from-vector.md: `into` now has a native ((PersistentVector :- [T]), (Vector :- [T])) clause backed by one
;; `PersistentVector/concat` call — retiring the N-interpreted-closure-invocation conj-fold.
(:wat::core::defn :dc::vec->pvec [v <- (:wat::core::Vector :- [:wat::core::i64])] -> (:wat::core::PersistentVector :- [:wat::core::i64])
  (:wat::core::into (:wat::core::PersistentVector) v))

;; codes fired — every DERIVED Node/Tag fact (level > 0), canonically encoded, into one (Vector :- [i64]).
;; Level-0 facts are the seeded input, excluded from the witness (mirrors accum/negation excluding
;; their own seed types from the derived witness).
(:wat::core::defn :dc::codes [fired <- :wat::rete::Session] -> (:wat::core::Vector :- [:wat::core::i64])
  (:wat::core::let
    [c0 (:wat::core::into (:wat::core::Vector :wat::core::i64)
          (:wat::core::map (:wat::core::fn [p <- :wat::core::PersistentMap] -> :wat::core::i64 (:wat::core::let [f (:wat::core::Option/expect (:wat::core::PersistentMap/get p "?fact") "query: ?fact")] (:dc::enc 0 (:cascade::Node/level f) (:cascade::Node/id f))))
            (:wat::core::filter (:wat::core::fn [p <- :wat::core::PersistentMap] -> :wat::core::bool (:wat::core::let [f (:wat::core::Option/expect (:wat::core::PersistentMap/get p "?fact") "query: ?fact")] (:wat::core::i64::> (:cascade::Node/level f) 0)))
              (:wat::rete::query fired (:cascade::q-Node)))))
     c1 (:wat::core::into c0
          (:wat::core::map (:wat::core::fn [p <- :wat::core::PersistentMap] -> :wat::core::i64 (:wat::core::let [f (:wat::core::Option/expect (:wat::core::PersistentMap/get p "?fact") "query: ?fact")] (:dc::enc 1 (:cascade::Tag/level f) (:cascade::Tag/id f))))
            (:wat::core::filter (:wat::core::fn [p <- :wat::core::PersistentMap] -> :wat::core::bool (:wat::core::let [f (:wat::core::Option/expect (:wat::core::PersistentMap/get p "?fact") "query: ?fact")] (:wat::core::i64::> (:cascade::Tag/level f) 0)))
              (:wat::rete::query fired (:cascade::q-Tag)))))]
    c1))

;; derived-vector fired — the sorted i64 accuracy witness (the full derived set, not a count).
(:wat::core::defn :dc::derived-vector [fired <- :wat::rete::Session] -> (:wat::core::PersistentVector :- [:wat::core::i64])
  (:dc::vec->pvec (:wat::core::sort (:dc::codes fired))))

;; ns-between t0 t1 — nanoseconds between two Instants (mirrors accum.wat's ns-between).
(:wat::core::defn :dc::ns-between [t0 <- :wat::time::Instant  t1 <- :wat::time::Instant] -> :wat::core::i64
  (:wat::core::i64::- (:wat::time::epoch-nanos t1) (:wat::time::epoch-nanos t0)))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let [params  (:wat::core::match (:wat::kernel::readln ) ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum) (:wat::kernel::ReadlnOutcome::Eof (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None)) (:wat::kernel::ReadlnOutcome::Stopped (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)))
                    depth   (:wat::core::Option/expect (:wat::core::get params 0) "stdin: [depth width]")
                    width   (:wat::core::Option/expect (:wat::core::get params 1) "stdin: [depth width]")
                    rules   (:dc::build-rules depth)
                    session (:wat::rete::compile-all rules (:wat::core::PersistentVector (:cascade::q-Node) (:cascade::q-Tag)))
                    facts   (:dc::level-0-facts width)
                    p0      (:wat::time::now)
                    staged  (:wat::rete::insert-all session facts)
                    i1      (:wat::time::now)
                    fired   (:wat::rete::fire-rules staged)
                    f1      (:wat::time::now)
                    derived (:dc::derived-vector fired)
                    q1      (:wat::time::now)
                    ins-ns  (:dc::ns-between p0 i1)
                    fir-ns  (:dc::ns-between i1 f1)
                    qry-ns  (:dc::ns-between f1 q1)
                    proto-ns (:dc::ns-between p0 q1)
                    ;; ORACLE — fired on the SAME staged session. Value semantics make the
                    ;; two fires independent: `staged` is unchanged by either.
                    o0      (:wat::time::now)
                    ofired  (:wat::rete::fire-rules$oracle staged)
                    o1      (:wat::time::now)]
    (:wat::kernel::println
      (:grid::Result :axis "deep-cascade" :size (:wat::core::PersistentVector depth width) :derived derived :native-ns fir-ns :oracle-derived (:dc::derived-vector ofired) :oracle-ns (:dc::ns-between o0 o1) :insert-ns ins-ns :fire-ns fir-ns :query-ns qry-ns :protocol-ns proto-ns))))
