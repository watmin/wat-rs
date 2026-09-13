;; Fixture BESIDE probe_arc278_stratified_query_replay.rs.
;;
;; THE PATH UNDER TEST: `harvest_stratified_queries`. After a STRATIFIED fire, a query
;; that is not a plain class scan does NOT read the accumulated per-stratum beta. It
;; builds a FRESH session with empty alpha/beta/production memory over the final facts
;; and replays with `FireKind::Once` — a single round.
;;
;; WHY IT NEEDS ITS OWN GATE. `complectens` named it as a SECOND masking layer, and it
;; is a sharp one: under stratified fire this replay would have hidden the leading-`:not`
;; /`:exists` duplication ENTIRELY. That bug appended one token per fixpoint round to a
;; cumulative beta — and a single-round replay over final facts cannot accumulate a
;; per-round duplicate. `probe_arc278_where_is_positionally_free` caught it only because
;; it is SINGLE-stratum; the same rule under stratification would have come back clean.
;;
;; And nothing exercised the branch: every pre-existing stratified differential queries
;; only plain class-scan classes, so `class_scans_cover_queries` was always true and the
;; replay never ran. Verified by arming a panic in it — this fixture reaches it, the old
;; corpus did not.
;;
;; THE ROWS.
;;   q-scan   a plain class scan — the IN-PLACE harvest. Control: proves the two paths
;;            are being compared, not one path twice.
;;   q-join   a join, so it routes through the REPLAY.
;;   q-exists a LEADING `:exists`, also routed through the replay — and this is the shape
;;            the replay would mask. If leading-filter multiplicity ever regresses, the
;;            single-stratum probe reddens; this one guards that the STRATIFIED reading
;;            of the same rule stays right too.

(:wat::core::defrecord :sqr::Item [k <- :wat::core::i64  name <- :wat::core::String])
(:wat::core::defrecord :sqr::Wind [loc <- :wat::core::String])
(:wat::core::defrecord :sqr::Bad  [k <- :wat::core::i64])
(:wat::core::defrecord :sqr::Ok   [k <- :wat::core::i64])

(:wat::rete::defrule :sqr::mark-bad
  :when [(:sqr::Item (?k <- :k)) (:wat::rete::where (:wat::rete::i64::= ?k 2))]
  :then [(:sqr::Bad :k ?k)])

;; stratum 2 — negation over the DERIVED Bad, which is what forces stratification
(:wat::rete::defrule :sqr::mark-ok
  :when [(:sqr::Item (?k <- :k)) (:wat::rete::not (:sqr::Bad (?k <- :k)))]
  :then [(:sqr::Ok :k ?k)])

(:wat::rete::defquery :sqr::q-scan :params [] :when [(?fact <- :sqr::Ok)])
(:wat::rete::defquery :sqr::q-join :params []
  :when [(:sqr::Ok (?k <- :k)) (:sqr::Item (?k <- :k) (?n <- :name))])
(:wat::rete::defquery :sqr::q-exists :params []
  :when [(:wat::rete::exists (:sqr::Wind (?loc <- :loc)))])

(:wat::core::defn :sqr::staged [] -> :wat::rete::Session
  (:wat::rete::insert-all
    (:wat::rete::insert-all
      (:wat::rete::compile-all (:wat::rete::collect-rules :sqr)
        (:wat::core::PersistentVector (:sqr::q-scan) (:sqr::q-join) (:sqr::q-exists)))
      (:wat::core::PersistentVector (:sqr::Item :k 1 :name "a") (:sqr::Item :k 2 :name "b")
                                    (:sqr::Item :k 3 :name "c")))
    ;; two Winds sharing one loc => ONE distinct inner binding
    (:wat::core::PersistentVector (:sqr::Wind :loc "MCI") (:sqr::Wind :loc "MCI"))))

(:wat::core::defn :sqr::counts [s <- :wat::rete::Session] -> (:wat::core::PersistentVector :- [:wat::core::i64])
  (:wat::core::PersistentVector
    (:wat::core::length (:wat::rete::query s (:sqr::q-scan)))
    (:wat::core::length (:wat::rete::query s (:sqr::q-join)))
    (:wat::core::length (:wat::rete::query s (:sqr::q-exists)))))

;; [scan, join, exists] native, then the same under $oracle. Expect 2 2 1 twice.
(:wat::core::defn :user::native-and-oracle [] -> (:wat::core::Vector :- [:wat::core::i64])
  (:wat::core::mapv
    (:wat::core::fn [n <- :wat::core::i64] -> :wat::core::i64 n)
    (:wat::vector::concat
      (:sqr::counts (:wat::rete::fire-rules (:sqr::staged)))
      (:sqr::counts (:wat::rete::fire-rules$oracle (:sqr::staged))))))
