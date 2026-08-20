;; wat-scripts/perf/deep-cascade.wat — DEEP forward-chain cascade perf, IN WAT.
;;
;; The real perf measurement on hard data, wat-native (wat orchestrates the Rust engine).
;; A depth-N × width-M cascade where every level is a 2-way JOIN on the prior level's DERIVED
;; facts — so every rule activation is both a hash-join (the heart) AND unlocked only by a
;; lower rule's output (forward chaining N deep). M independent join-chains, each N levels deep;
;; 2·N·M derived facts; the deepest level (Node@N) is reachable only after N cascade rounds.
;;
;; Two CONSTANT record types (Node, Tag), the level lives in a field — so depth is parameterized
;; by building N rules with `quasiquote` splicing the level literal (no per-level types, no codegen
;; in another language; wat builds its own rule set via foldl over a range).
;;
;; Times the wat oracle `fire-rules` (re-run-from-scratch) against the native `fire-rules'`
;; (P4a cascade fixpoint) on the SAME staged session. Emits one EDN result map per line on stdout.
;;
;; Usage (stdin = an EDN :perf::Params; stdout = an EDN map):
;;   echo '(:perf::Params 20 3)' | cargo wat ./wat-scripts/perf/deep-cascade.wat
;;   => {:depth 20 :width 3 :derived 120 :deepest 3 :wat-ns N :native-ns M}

(:wat::core::defrecord :cascade::Node [level <- :wat::core::i64  id <- :wat::core::i64])
(:wat::core::defrecord :cascade::Tag  [level <- :wat::core::i64  id <- :wat::core::i64])
(:wat::core::defrecord :perf::Result
  [depth     <- :wat::core::i64
   width     <- :wat::core::i64
   derived   <- :wat::core::i64
   deepest   <- :wat::core::i64
   wat-ns    <- :wat::core::i64
   native-ns <- :wat::core::i64])

(:wat::rete::defquery :cascade::q-Node
  :params []
  :when [(?fact <- :cascade::Node)])


;; build-rule k — the k-th cascade level: join Node⋈Tag at level (k-1) on ?id, derive Node,Tag at level k.
;; The level literals (k-1 in the conditions, k in the inserts) are spliced via quasiquote/unquote.
(:wat::core::defn :perf::build-rule [k <- :wat::core::i64] -> :wat::rete::Rule
  (:wat::core::let [prev (:wat::core::i64::- k 1)
                    c1 (:wat::core::quasiquote (:cascade::Node (?id <- :id) (?l <- :level) (:wat::core::= ?l (:wat::core::unquote prev))))
                    c2 (:wat::core::quasiquote (:cascade::Tag  (?id <- :id) (?m <- :level) (:wat::core::= ?m (:wat::core::unquote prev))))
                    t1 (:wat::core::quasiquote (:cascade::Node (:wat::core::unquote k) ?id))
                    t2 (:wat::core::quasiquote (:cascade::Tag  (:wat::core::unquote k) ?id))]
    (:wat::rete::Rule :name (:wat::core::i64::to-string k)
      :lhs (:wat::core::PersistentVector c1 c2)
      :rhs (:wat::core::PersistentVector t1 t2))))

;; build-rules depth — the rule set [rule1 .. rule depth], built by folding build-rule over (range 1 depth+1).
(:wat::core::defn :perf::build-rules [depth <- :wat::core::i64] -> :wat::core::PersistentVector<wat::rete::Rule>
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::PersistentVector<wat::rete::Rule>  k <- :wat::core::i64] -> :wat::core::PersistentVector<wat::rete::Rule>
      (:wat::core::PersistentVector/conj acc (:perf::build-rule k)))
    (:wat::core::PersistentVector (:perf::build-rule 1))
    (:wat::core::range 2 (:wat::core::i64::+ depth 1))))

;; seed-level-0 session width — stage Node(0,i)+Tag(0,i) for i in 0..width, threading the staging session.
(:wat::core::defn :perf::seed-level-0 [session <- :wat::rete::Session  width <- :wat::core::i64] -> :wat::rete::Session
  (:wat::core::foldl
    (:wat::core::fn [s <- :wat::rete::Session  i <- :wat::core::i64] -> :wat::rete::Session
      (:wat::rete::insert (:wat::rete::insert s (:cascade::Node :level 0 :id i)) (:cascade::Tag :level 0 :id i)))
    session
    (:wat::core::range 0 width)))

;; count-at-level fired lvl — how many Node facts were derived at exactly `lvl` (the deepest = width iff full closure).
(:wat::core::defn :perf::count-at-level [fired <- :wat::rete::Session  lvl <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::length
    (:wat::core::filter
      (:wat::core::fn [p <- :wat::core::PersistentMap] -> :wat::core::bool (:wat::core::let [n (:wat::core::Option/expect (:wat::core::PersistentMap/get p "?fact") "query: ?fact")] (:wat::core::= (:cascade::Node/level n) lvl)))
      (:wat::rete::query fired (:cascade::q-Node)))))

;; elapsed-ns thunk-result-start-end — nanoseconds between two Instants.
(:wat::core::defn :perf::ns-between [t0 <- :wat::time::Instant  t1 <- :wat::time::Instant] -> :wat::core::i64
  (:wat::core::i64::- (:wat::time::epoch-nanos t1) (:wat::time::epoch-nanos t0)))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let [params  (:wat::core::match (:wat::kernel::readln ) ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum) (:wat::kernel::ReadlnOutcome::Eof (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None)) (:wat::kernel::ReadlnOutcome::Stopped (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)))
                    depth   (:wat::core::Option/expect   (:wat::core::get params 0) "stdin: [depth width]")
                    width   (:wat::core::Option/expect   (:wat::core::get params 1) "stdin: [depth width]")
                    rules   (:perf::build-rules depth)
                    staged  (:perf::seed-level-0 (:wat::rete::compile-all rules (:wat::core::PersistentVector (:cascade::q-Node))) width)
                    ;; time the wat SPEC engine fire-rules-spec (re-run-from-scratch reference)
                    w0      (:wat::time::now)
                    fired-w (:wat::rete::fire-rules$oracle staged)
                    w1      (:wat::time::now)
                    ;; time the native fire-rules' (P4a cascade fixpoint) on the SAME staged session
                    n0      (:wat::time::now)
                    fired-n (:wat::rete::fire-rules staged)
                    n1      (:wat::time::now)
                    deepest (:perf::count-at-level fired-n depth)
                    derived (:wat::core::i64::* 2 (:wat::core::i64::* depth width))
                    wat-ns  (:perf::ns-between w0 w1)
                    nat-ns  (:perf::ns-between n0 n1)]
    ;; println is ∀T → EDN: hand it the Result record, the stdout service renders it to EDN.
    (:wat::kernel::println (:perf::Result :depth depth :width width :derived derived :deepest deepest :wat-ns wat-ns :native-ns nat-ns))))
