;; Fixture: probe 5 — mixed declaration prelude now includes def.
;; All 7 declaration forms lift together: def, struct, enum, newtype, typealias, defn, defmacro.
;;
;; Arc 278 IPC de-prime — driver migrated to `spawn-program' (process)` + `recv'`; every
;; declaration under test is unchanged, in the same order. The child now exercises ALL of them
;; and folds the results into ONE i64 the parent reads back, so a single asserted number proves
;; each form registered: 99 (def) + 1+2 (struct accessors) + 10 (enum match) + 10 (newtype /0)
;; + 7 (typealias-returning fn, through the macro) = 129.
(:wat::core::defn :my::launch [] -> :wat::core::i64
  (:wat::core::let
    [p (:wat::test::spawn-peer (:wat::spawn::process)
         (:wat::core::forms
           (:wat::core::def :h::def-answer 99)
           (:wat::core::defstruct :h::MixPoint8
             [x <- :wat::core::i64
              y <- :wat::core::i64])
           (:wat::core::defenum :h::MixDir8 :wat::enum::Pure
             :Up
             :Down)
           (:wat::core::newtype :h::MixAmount8 :wat::core::i64)
           (:wat::core::typealias :h::MixCount8 :wat::core::i64)
           (:wat::core::defn :h::mix-i64-fn8 [v <- :wat::core::i64] -> :h::MixCount8
             v)
           (:wat::core::defmacro :h::mix-id8 [z <- :wat::WatAST] -> :wat::WatAST `~z)
           (:wat::core::defn :user::main [] -> :wat::core::nil
             (:wat::core::let
               [ans  :h::def-answer
                pt   (:h::MixPoint8 :x 1 :y 2)
                d    :h::MixDir8::Up
                a    (:h::MixAmount8 10)
                dv   (:wat::core::match d
                       (:h::MixDir8::Up 10)
                       (:h::MixDir8::Down 20))
                n    (:wat::i64::+
                       (:wat::i64::+
                         ans
                         (:wat::i64::+ (:h::MixPoint8/x pt) (:h::MixPoint8/y pt)))
                       (:wat::i64::+
                         (:wat::i64::+ dv (:h::MixAmount8/0 a))
                         (:h::mix-i64-fn8 (:h::mix-id8 7))))
                _out (:wat::kernel::println n)]
               nil))))]
    (:wat::core::match (:wat::kernel::recv p)
      ((:wat::kernel::RecvOutcome::Message m) m)
      ((:wat::kernel::RecvOutcome::Lost cause)
        (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
      (:wat::kernel::RecvOutcome::Stopped
        (:wat::kernel::assertion-failed! "launch: stop requested before the child sent its value — the child was alive" :wat::core::None :wat::core::None))
      (:wat::kernel::RecvOutcome::Closed
        (:wat::kernel::assertion-failed! "launch: child closed before sending its value" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::TimedOut (:wat::kernel::assertion-failed! "recv: timed out — the peer is alive and silent" :wat::core::None :wat::core::None)))))
