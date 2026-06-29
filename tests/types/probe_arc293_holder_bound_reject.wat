;; tests/types/probe_arc293_holder_bound_reject.wat — negative fixture (must cite :env::Holon)
;;
;; Arc 293 holder bound — a CORE record is rejected by a :holder :wat::holon::Record surface.
;; The rejection must cite the surface :env::Holon (not an incidental MalformedDecl).

(:wat::core::defsurface :env::Holon
  :holder :wat::holon::Record
  [slot <- :wat::core::i64])
(:wat::core::defrecord :env::CEnv [slot <- :wat::core::i64])
(:wat::core::defn :env::wants-holon [x <- :env::Holon] -> :wat::core::bool
  true)
(:wat::core::defn :user::main [] -> :wat::core::bool
  (:env::wants-holon (:env::CEnv 1)))
