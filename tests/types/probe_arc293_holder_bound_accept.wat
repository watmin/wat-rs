;; tests/types/probe_arc293_holder_bound_accept.wat — positive fixture
;;
;; Arc 293 nature bound — a holon record satisfies a :nature :wat::holon::Record surface.
;; RED at HEAD: :nature makes defsurface a >2-arg form → MalformedDecl.

(:wat::core::defsurface :env::Holon
  :nature :wat::holon::Record
  :features [slot <- :wat::core::i64])
(:wat::holon::defrecord :env::HEnv [slot <- :wat::core::i64])
(:wat::core::defn :env::wants-holon [x <- :env::Holon] -> :wat::core::bool
  true)
(:wat::core::defn :probe::drive [] -> :wat::core::bool
  (:env::wants-holon (:env::HEnv :slot 1)))
