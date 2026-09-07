;; probe-a-local-fn-can-be-generic.wat — arc 278, drawn for
;; `one retry OUTCOME, three call sites`.
;;
;; ⛔ THE ANSWER IS NO — and that answer shaped the stone.
;;
;; circuit.wat's worker holds three hand-rolled retry ladders (check :513, mark
;; :574, ack :591). Two speak to a `Seen` peer, one to a `Queue` peer. A single
;; shared combinator would have to be GENERIC over the peer's [Op Reply] pair, and
;; it would have to be a LOCAL `fn`: the worker runs in a process child with
;; `:env-fn "(:wat::program::EmptyEnv)"` and cannot see a top-level `defn`.
;;
;; The generic local form was tried first:
;;
;;   twice (:wat::core::fn :- [:wat::core::T]
;;            [x <- :wat::core::T  f <- [:wat::core::T :-> :wat::core::T]]
;;            -> :wat::core::T  (f (f x)))
;;   … (twice 1 bump) …  (twice "hi" shout)
;;
;; It DEFINES cleanly and then dies at every call site:
;;
;;   (value head): parameter #1 expects :wat::core::T; got :wat::core::i64
;;   (value head): parameter #2 expects [:wat::core::T :-> :wat::core::T];
;;                 got [:wat::core::i64 :-> :wat::core::i64]
;;
;; A local `fn` bound in a `let` is a monomorphic VALUE; its type parameter is
;; never instantiated by application. `wat/core.wat:1349` emits that same form, but
;; from inside a macro, where the types are already concrete.
;;
;; ★ So the shape that must stop being re-hand-rolled is not the LOOP — it is the
;; OUTCOME. A `bool` retry flag can be dropped, and at two of the three ladders it
;; IS dropped. An enum variant cannot be: the match must name it. This probe is the
;; worked reference for that shape.
(:wat::core::defenum :fanout::probe::RetryOutcome :wat::enum::Pure
  :Got       [value <- :wat::core::i64]
  :Exhausted [attempts <- :wat::core::i64])

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [;; one monomorphic combinator, two call sites, exhaustion as a VALUE
     try-until (:wat::core::fn
                 [start <- :wat::core::i64  limit <- :wat::core::i64]
                 -> :fanout::probe::RetryOutcome
                 (:wat::core::if (:wat::i64::>= start limit)
                   (:fanout::probe::RetryOutcome::Exhausted limit)
                   (:fanout::probe::RetryOutcome::Got start)))
     ;; the caller CANNOT ignore Exhausted — the match must name it
     render (:wat::core::fn [o <- :fanout::probe::RetryOutcome] -> :wat::core::String
              (:wat::core::match o
                ((:fanout::probe::RetryOutcome::Got v)
                  (:wat::core::format "got={v}" :v v))
                ((:fanout::probe::RetryOutcome::Exhausted a)
                  (:wat::core::format "EXHAUSTED after {a}" :a a))))
     ok   (render (try-until 1 3))
     dead (render (try-until 5 3))]
    (:wat::kernel::println (:wat::core::format "{a} | {b}" :a ok :b dead))))
