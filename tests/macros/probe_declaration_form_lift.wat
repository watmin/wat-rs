;; tests/macros/probe_declaration_form_lift.wat — co-located fixture for
;; probe_declaration_form_lift.rs, slurped via startup_beside(file!()).
;;
;; Four named launch functions (one per spawn-process test) + one :user::main.
;; Arc 170 slice 6 — all declaration kinds sit at program top-level alongside :user::main.

;; Test 2: defmacro in fn body do-prefix lifts to prologue.
(:wat::core::defn :my::launch-defmacro [] -> :wat::kernel::Process<wat::core::nil,wat::core::nil>
  (:wat::kernel::spawn-process
              (:wat::core::forms
                (:wat::core::defmacro :h::id-macro [x <- :wat::WatAST] -> :wat::WatAST `~x)
                (:wat::core::defn :user::main [] -> :wat::core::nil nil))))

;; Test 4: newtype in fn body do-prefix lifts to prologue.
(:wat::core::defn :my::launch-newtype [] -> :wat::kernel::Process<wat::core::nil,wat::core::nil>
  (:wat::kernel::spawn-process
              (:wat::core::forms
                (:wat::core::newtype :h::LocalAmount :wat::core::i64)
                (:wat::core::defn :user::main [] -> :wat::core::nil
                  (:wat::core::let [a (:h::LocalAmount 100)] nil)))))

;; Test 5: typealias in fn body do-prefix lifts to prologue.
(:wat::core::defn :my::launch-typealias [] -> :wat::kernel::Process<wat::core::nil,wat::core::nil>
  (:wat::kernel::spawn-process
              (:wat::core::forms
                (:wat::core::typealias :h::LocalCount :wat::core::i64)
                (:wat::core::defn :h::get-count [] -> :h::LocalCount 7)
                (:wat::core::defn :user::main [] -> :wat::core::nil
                  (:wat::core::let [_c (:h::get-count)] nil)))))

;; Test 6: mixed prelude covering 7 of 8 declaration form kinds.
(:wat::core::defn :my::launch-mixed [] -> :wat::kernel::Process<wat::core::nil,wat::core::nil>
  (:wat::kernel::spawn-process
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
                    [_p  (:h::MixPoint 1 2)
                     _d  :h::MixDir::Up
                     _a  (:h::MixAmount 10)
                     _n  (:h::mix-i64 7)]
                    nil)))))

