;; tests/types/probe_arc293_holder_bound_reject.wat — negative fixture (must cite :env::Holon)
;;
;; Arc 293 nature bound — a CORE record is rejected by a :nature :wat::holon::Record surface.
;; The rejection must cite the surface :env::Holon (not an incidental MalformedDecl).

(:wat::core::defsurface :env::Holon
  :nature :wat::holon::Record
  :features [slot <- wat.type/i64])
(:wat::core::defrecord :env::CEnv [slot <- wat.type/i64])
(:wat::core::defn :env::wants-holon [x <- :env::Holon] -> wat.type/bool
  true)
(:wat::core::defn :probe::drive [] -> wat.type/bool
  (:env::wants-holon (:env::CEnv :slot 1)))
