;; probe-node-share-dedup.wat — DISCONFIRMING PROBE: does the shared join-prefix actually dedup?
;;
;; THE QUESTION. Grid axis A8 (node-share) measures N rules that all share the SAME leading
;; [A(?k)] ⋈ [B(?k)] join, differentiated only by a trailing per-rule `where`. Measured
;; 2026-07-30 against the local build, M fixed at 500:
;;
;;     N= 5  ratio 0.0775   (Clara ~13x faster)
;;     N=10  ratio 0.0074   (Clara ~135x faster)
;;     N=20  SIGKILL — >4 GiB to join 500 facts against 20 rules
;;
;; Doubling N cost ~10x, not 2x, and the next doubling exhausted memory. That is the signature of
;; per-rule join materialization, NOT of a shared subtree with a bad constant.
;;
;; THE CONTRADICTION THIS SETTLES. The record disagrees with itself:
;;   - `find-or-mint-hash-join` (wat/rete.wat:483+) dedups on "hashjoin:<parent-id>:<cond-text>",
;;     and CLARA-TRANSLATIONS.md §A8 says rules sharing a join-prefix "collapse onto the same beta
;;     subtree" — i.e. sharing EXISTS.
;;   - the arc's own perf note says "we share ALPHA nodes but NOT beta/join-prefix subtrees;
;;     N rules -> N x join work" — i.e. sharing DOESN'T.
;; Both cannot be true. Reading the compiler against two disagreeing docs only produces a third
;; opinion, so this COUNTS THE NODES instead.
;;
;; WHAT IT PROVES, either way. Compile N rules; count network nodes by kind.
;;   sharing WORKS  -> Alpha and HashJoin counts are FLAT in N; only Test/Production grow (1 each
;;                     per rule, since ProductionNodes are never shared, wat/rete.wat:781).
;;   sharing BROKEN -> Alpha and/or HashJoin grow WITH N, and which one grows names the missed
;;                     dedup: a growing HashJoin with flat Alpha means the join key misses; both
;;                     growing means the alpha key misses and the join inherits distinct parents.
;;
;; SAFE BY CONSTRUCTION: compile ONLY — no seed, no fire. The blowup lives in firing, so this
;; cannot repeat the crash that produced the numbers above. Run it at any N.
;;
;; stdin : [n]        (a one-element i64 vector — the rule count)
;; stdout: one #probe/NodeCounts EDN line
;;
;; The rule shape is copied VERBATIM from wat-scripts/perf/grid/node-share.wat's build-rule — if it
;; drifts from the axis, this probe stops describing the thing that was measured.

(:wat::core::defrecord :nsp::A [k <- :wat::core::i64])
(:wat::core::defrecord :nsp::B [k <- :wat::core::i64])
(:wat::core::defrecord :nsp::Out [k <- :wat::core::i64])

(:wat::core::defrecord :probe::NodeCounts
  [n       <- :wat::core::i64
   total   <- :wat::core::i64
   next-id <- :wat::core::i64
   kinds   <- (:wat::core::HashMap :- [:wat::core::String :wat::core::i64])])

;; build-rule i n — VERBATIM the axis's rule: Out(k) :- A(k) AND B(k) AND (i == k mod n).
;; The leading two conditions are byte-identical across every i (no i splices into them), so they
;; are exactly the shareable prefix under test. Only the trailing `where` carries the per-rule
;; literal. `mod` is written as the truncating-division idiom (no native i64 mod).
(:wat::core::defn :nsp::build-rule [i <- :wat::core::i64  n <- :wat::core::i64] -> :wat::rete::Rule
  (:wat::core::let [a-c     (:wat::core::quasiquote (:nsp::A (?k <- :k)))
                    b-c     (:wat::core::quasiquote (:nsp::B (?k <- :k)))
                    where-c (:wat::core::quasiquote
                              (:wat::rete::where
                                (:wat::core::= (:wat::core::unquote i)
                                  (:wat::i64::- ?k
                                    (:wat::i64::* (:wat::i64::/ ?k (:wat::core::unquote n)) (:wat::core::unquote n))))))
                    ins     (:wat::core::quasiquote (:nsp::Out ?k))]
    (:wat::rete::Rule :name (:wat::i64::to-string i)
      :lhs (:wat::core::PersistentVector a-c b-c where-c)
      :rhs (:wat::core::PersistentVector ins))))

(:wat::core::defn :nsp::build-rules [n <- :wat::core::i64] -> (:wat::core::PersistentVector :- [:wat::rete::Rule])
  (:wat::core::foldl
    (:wat::core::fn [acc <- (:wat::core::PersistentVector :- [:wat::rete::Rule])  i <- :wat::core::i64]
      -> (:wat::core::PersistentVector :- [:wat::rete::Rule])
      (:wat::vector::conj acc (:nsp::build-rule i n)))
    (:wat::core::PersistentVector)
    (:wat::core::range 0 n)))

;; count-kinds — fold the network map into kind-label -> count. `node-kind-label` (wat/rete.wat:290)
;; takes the last `::` segment of the node record's own type FQDN, so this needs no per-kind
;; enumeration and will surface a node kind this probe's author never thought of.
(:wat::core::defn :nsp::count-kinds
  [session <- :wat::rete::Session] -> (:wat::core::HashMap :- [:wat::core::String :wat::core::i64])
  (:wat::core::let [network (:wat::rete::Session/network session)
                    keys    (:wat::map::keys network)]
    (:wat::core::foldl
      (:wat::core::fn [acc <- (:wat::core::HashMap :- [:wat::core::String :wat::core::i64])
                       k   <- :wat::core::i64]
        -> (:wat::core::HashMap :- [:wat::core::String :wat::core::i64])
        (:wat::core::let [node (:wat::core::Option/expect
                                 (:wat::map::get network k)
                                 "count-kinds: node not found")
                          kind (:wat::rete::node-kind-label node)
                          cur  (:wat::core::match (:wat::hashmap::get acc kind)
                                 ((:wat::core::Some v) v)
                                 (:wat::core::None 0))]
          (:wat::hashmap::assoc acc kind (:wat::i64::+ cur 1))))
      ;; the empty HashMap takes its KEY and VALUE types as arguments (cf. rete.wat:801's dedup)
      (:wat::core::HashMap :- [:wat::core::String :wat::core::i64])
      keys)))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let [params  (:wat::core::match (:wat::kernel::readln )
                              ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum)
                              (:wat::kernel::ReadlnOutcome::Eof
                                (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None))
                              (:wat::kernel::ReadlnOutcome::Stopped
                                (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)))
                    n       (:wat::core::Option/expect (:wat::core::get params 0) "stdin: [n]")
                    rules   (:nsp::build-rules n)
                    ;; COMPILE ONLY — never seed, never fire. That is what makes this safe at any N.
                    session (:wat::rete::compile rules)
                    kinds   (:nsp::count-kinds session)
                    network (:wat::rete::Session/network session)
                    total   (:wat::core::length (:wat::map::keys network))]
    (:wat::kernel::println
      (:probe::NodeCounts :n n :total total
        :next-id (:wat::rete::Session/next-id session)
        :kinds kinds))))
