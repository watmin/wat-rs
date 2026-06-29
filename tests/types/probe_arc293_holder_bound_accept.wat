;; tests/types/probe_arc293_holder_bound_accept.wat — positive fixture
;;
;; Arc 293 holder bound — a holon record satisfies a :holder :wat::holon::Record surface.
;; RED at HEAD: :holder makes defsurface a >2-arg form → MalformedDecl.

(:wat::core::defsurface :env::Holon
  :holder :wat::holon::Record
  :features [slot <- :wat::core::i64])
(:wat::holon::defrecord :env::HEnv [slot <- :wat::core::i64])
(:wat::core::defn :env::wants-holon [x <- :env::Holon] -> :wat::core::bool
  true)
(:wat::core::defn :user::main [] -> :wat::core::bool
  (:env::wants-holon (:env::HEnv 1)))
