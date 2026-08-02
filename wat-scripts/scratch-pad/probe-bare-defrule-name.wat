;; wat-scripts/scratch-pad/probe-bare-defrule-name.wat — probe: is a bare (non-namespaced)
;; defrule name legal, and does the derived Rule/name come out bare (no "wsh::" prefix)?
;; Throwaway reconnaissance for the where-shapes defrule migration (arc 278).

(:wat::core::defrecord :wsh::probe::Req [k <- :wat::core::i64])
(:wat::core::defrecord :wsh::probe::Hit [k <- :wat::core::i64])

(:wat::rete::defrule :arith
  :when
  [(:wsh::probe::Req (?k <- :k))]
  :then
  (:wat::rete::insert (:wsh::probe::Hit ?k)))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println (:wat::rete::Rule/name (:arith))))
