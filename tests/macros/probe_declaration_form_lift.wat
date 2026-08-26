;; tests/macros/probe_declaration_form_lift.wat — co-located fixture for
;; probe_declaration_form_lift.rs, slurped via startup_beside(file!()).
;;
;; Four named launch functions (one per probe). Arc 170 slice 6 — all declaration kinds sit at
;; program top-level alongside :user::main.
;;
;; Arc 278 IPC de-prime — the DRIVER migrated off the non-prime `:wat::kernel::spawn-process`
;; onto the composed primes (`spawn-program' (process)` + `recv'`); every declaration under test
;; is unchanged, in the same order, at the same position. The OBSERVATION also changed: instead
;; of checking the child's exit code by field-poking the concrete `Process` struct (which the
;; opaque `Process'` peer has no analog for), each child `println`s an i64 derived from the
;; declaration under test and the parent reads it back via `recv'`. A registration failure now
;; surfaces as a `Lost` cause carrying the child's real reason.

;; Test 2: defmacro in fn body do-prefix lifts to prologue.
;; The child now INVOKES the macro — `(:h::id-macro 5)` expands to `5` — so the asserted value
;; proves the defmacro registered AND expanded, not merely that the child survived.
(:wat::core::defn :my::launch-defmacro [] -> :wat::core::i64
  (:wat::core::let
    [p (:wat::test::spawn-peer (:wat::spawn::process)
         (:wat::core::forms
           (:wat::core::defmacro :h::id-macro [x <- :wat::WatAST] -> :wat::WatAST `~x)
           (:wat::core::defn :user::main [] -> :wat::core::nil
             (:wat::kernel::println (:h::id-macro 5)))))]
    (:wat::core::match (:wat::kernel::recv p)
      ((:wat::kernel::RecvOutcome::Message m) m)
      ((:wat::kernel::RecvOutcome::Lost cause)
        (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
      (:wat::kernel::RecvOutcome::Stopped
        (:wat::kernel::assertion-failed! "launch-defmacro: stop requested before the child sent its value — the child was alive" :wat::core::None :wat::core::None))
      (:wat::kernel::RecvOutcome::Closed
        (:wat::kernel::assertion-failed! "launch-defmacro: child closed before sending its value" :wat::core::None :wat::core::None)))))

;; Test 4: newtype in fn body do-prefix lifts to prologue.
;; The child constructs the newtype and reads it back through the synthesized `/0` accessor,
;; so the asserted 100 proves the newtype registered AND constructs AND its accessor resolves.
(:wat::core::defn :my::launch-newtype [] -> :wat::core::i64
  (:wat::core::let
    [p (:wat::test::spawn-peer (:wat::spawn::process)
         (:wat::core::forms
           (:wat::core::newtype :h::LocalAmount :wat::core::i64)
           (:wat::core::defn :user::main [] -> :wat::core::nil
             (:wat::core::let
               [a    (:h::LocalAmount 100)
                _out (:wat::kernel::println (:h::LocalAmount/0 a))]
               nil))))]
    (:wat::core::match (:wat::kernel::recv p)
      ((:wat::kernel::RecvOutcome::Message m) m)
      ((:wat::kernel::RecvOutcome::Lost cause)
        (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
      (:wat::kernel::RecvOutcome::Stopped
        (:wat::kernel::assertion-failed! "launch-newtype: stop requested before the child sent its value — the child was alive" :wat::core::None :wat::core::None))
      (:wat::kernel::RecvOutcome::Closed
        (:wat::kernel::assertion-failed! "launch-newtype: child closed before sending its value" :wat::core::None :wat::core::None)))))

;; Test 5: typealias in fn body do-prefix lifts to prologue.
;; A typealias is transparent, so the child can println the aliased-return value directly —
;; the asserted 7 proves both the typealias and the fn declared against it registered.
(:wat::core::defn :my::launch-typealias [] -> :wat::core::i64
  (:wat::core::let
    [p (:wat::test::spawn-peer (:wat::spawn::process)
         (:wat::core::forms
           (:wat::core::typealias :h::LocalCount :wat::core::i64)
           (:wat::core::defn :h::get-count [] -> :h::LocalCount 7)
           (:wat::core::defn :user::main [] -> :wat::core::nil
             (:wat::core::let
               [c    (:h::get-count)
                _out (:wat::kernel::println c)]
               nil))))]
    (:wat::core::match (:wat::kernel::recv p)
      ((:wat::kernel::RecvOutcome::Message m) m)
      ((:wat::kernel::RecvOutcome::Lost cause)
        (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
      (:wat::kernel::RecvOutcome::Stopped
        (:wat::kernel::assertion-failed! "launch-typealias: stop requested before the child sent its value — the child was alive" :wat::core::None :wat::core::None))
      (:wat::kernel::RecvOutcome::Closed
        (:wat::kernel::assertion-failed! "launch-typealias: child closed before sending its value" :wat::core::None :wat::core::None)))))

;; Test 6: mixed prelude covering 7 of 8 declaration form kinds.
;; The child exercises every declaration and folds them into ONE value: 1+2 from the struct's
;; accessors, 10 from the enum match, 7 through the typealias-returning fn, and the macro's
;; expansion — so a single asserted number proves all of them registered, in order.
(:wat::core::defn :my::launch-mixed [] -> :wat::core::i64
  (:wat::core::let
    [p (:wat::test::spawn-peer (:wat::spawn::process)
         (:wat::core::forms
           (:wat::core::defstruct :h::MixPoint
             [x <- :wat::core::i64
              y <- :wat::core::i64])
           (:wat::core::defenum :h::MixDir :wat::enum::Pure
             :Up
             :Down)
           (:wat::core::newtype :h::MixAmount :wat::core::i64)
           (:wat::core::typealias :h::MixCount :wat::core::i64)
           (:wat::core::defn :h::mix-i64 [v <- :wat::core::i64] -> :h::MixCount
             v)
           (:wat::core::defmacro :h::mix-id [z <- :wat::WatAST] -> :wat::WatAST `~z)
           (:wat::core::defn :user::main [] -> :wat::core::nil
             (:wat::core::let
               [pt   (:h::MixPoint :x 1 :y 2)
                d    :h::MixDir::Up
                a    (:h::MixAmount 10)
                dv   (:wat::core::match d
                       (:h::MixDir::Up 10)
                       (:h::MixDir::Down 20))
                n    (:wat::i64::+
                       (:wat::i64::+ (:h::MixPoint/x pt) (:h::MixPoint/y pt))
                       (:wat::i64::+
                         (:wat::i64::+ dv (:h::MixAmount/0 a))
                         (:h::mix-i64 (:h::mix-id 7))))
                _out (:wat::kernel::println n)]
               nil))))]
    (:wat::core::match (:wat::kernel::recv p)
      ((:wat::kernel::RecvOutcome::Message m) m)
      ((:wat::kernel::RecvOutcome::Lost cause)
        (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
      (:wat::kernel::RecvOutcome::Stopped
        (:wat::kernel::assertion-failed! "launch-mixed: stop requested before the child sent its value — the child was alive" :wat::core::None :wat::core::None))
      (:wat::kernel::RecvOutcome::Closed
        (:wat::kernel::assertion-failed! "launch-mixed: child closed before sending its value" :wat::core::None :wat::core::None)))))
