;; wat/kernel/assertion.wat — the `assertion-failed!` kwargs macro.
;;
;; WHY THIS FILE EXISTS. `assertion-failed!'` is the kernel-restricted positional
;; primitive (message, actual, expected). The user-facing name is a kwargs macro
;; because a plain fail must not carry two unreadable positional `:None`s.
;; kwargs-is-always-a-macro: the surface is
;;   (assertion-failed! :message m)
;;   (assertion-failed! :message m :actual a :expected e)
;; and it lowers to `(assertion-failed!' m a e)` with `:actual`/`:expected`
;; defaulting to `:wat::core::None`.
;;
;; Load-order-free: `assertion-failed!'` is a Rust intrinsic (available at
;; expand time). This file has no eval-dep on any other wat file.
;;
;; F5: this program-body must not CALL `:wat::core::None` / `:wat::core::Some`
;; as heads. Optional slots are 0-or-1 vectors; a missing required value is
;; `get` past the end, then `Option/expect`.

(:wat::core::defmacro :wat::kernel::assertion-failed!
  [& args <- (:wat::core::Vector :- [:wat::WatAST])]
  -> :wat::WatAST
  (:wat::core::let
    [n (:wat::core::length args)]
    (:wat::core::if
      (:wat::core::if (:wat::core::= n 0) true
        (:wat::core::if (:wat::core::not (:wat::core::= (:wat::i64::rem n 2) 0)) true
          (:wat::core::not
            (:wat::core::= (:wat::core::ast-kind
                             (:wat::core::Option/expect
                               (:wat::core::get args 0)
                               "assertion-failed!: empty"))
                           "keyword"))))
      (:wat::core::Option/expect
        (:wat::core::get args n)
        "assertion-failed! takes kwargs :message / :actual / :expected; the positional (message actual expected) form is retired")
      (:wat::core::let
        [nkv (:wat::i64::/ n 2)
         empty (:wat::core::Vector :- [:wat::WatAST])
         slots
           (:wat::core::foldl
             (:wat::core::fn
               [acc <- (:wat::core::Vector :- [(:wat::core::Vector :- [:wat::WatAST])])
                i   <- :wat::core::i64]
               -> (:wat::core::Vector :- [(:wat::core::Vector :- [:wat::WatAST])])
               (:wat::core::let
                 [k (:wat::core::Option/expect
                      (:wat::core::get args (:wat::i64::* i 2))
                      "assertion-failed!: key")
                  v (:wat::core::Option/expect
                      (:wat::core::get args (:wat::i64::+ (:wat::i64::* i 2) 1))
                      "assertion-failed!: value")
                  kn (:wat::core::ast-name k)
                  msg-slot (:wat::core::Option/expect (:wat::core::get acc 0) "slot 0")
                  act-slot (:wat::core::Option/expect (:wat::core::get acc 1) "slot 1")
                  exp-slot (:wat::core::Option/expect (:wat::core::get acc 2) "slot 2")]
                 (:wat::core::if (:wat::core::not (:wat::core::= (:wat::core::ast-kind k) "keyword"))
                   (:wat::core::Option/expect
                     (:wat::core::get args n)
                     "assertion-failed! kwargs keys must be keywords")
                   (:wat::core::if (:wat::core::= kn ":message")
                     (:wat::core::Vector :- [(:wat::core::Vector :- [:wat::WatAST])]
                       (:wat::core::conj msg-slot v) act-slot exp-slot)
                     (:wat::core::if (:wat::core::= kn ":actual")
                       (:wat::core::Vector :- [(:wat::core::Vector :- [:wat::WatAST])]
                         msg-slot (:wat::core::conj act-slot v) exp-slot)
                       (:wat::core::if (:wat::core::= kn ":expected")
                         (:wat::core::Vector :- [(:wat::core::Vector :- [:wat::WatAST])]
                           msg-slot act-slot (:wat::core::conj exp-slot v))
                         (:wat::core::Option/expect
                           (:wat::core::get args n)
                           "assertion-failed! unknown kwarg — write :message / :actual / :expected")))))))
             (:wat::core::Vector :- [(:wat::core::Vector :- [:wat::WatAST])]
               empty empty empty)
             (:wat::core::range 0 nkv))
         msg-slot (:wat::core::Option/expect (:wat::core::get slots 0) "msg-slot")
         act-slot (:wat::core::Option/expect (:wat::core::get slots 1) "act-slot")
         exp-slot (:wat::core::Option/expect (:wat::core::get slots 2) "exp-slot")
         msg (:wat::core::Option/expect
               (:wat::core::get msg-slot 0)
               "assertion-failed! requires :message")
         has-act (:wat::i64::> (:wat::core::length act-slot) 0)
         has-exp (:wat::i64::> (:wat::core::length exp-slot) 0)
         ;; A string literal kwarg is wrapped in Some in the TEMPLATE
         ;; (quasiquote — F5 does not see it as a call). An already-Option
         ;; form (`(:wat::core::Some …)` / a computed Option) is spliced as-is.
         str? (:wat::core::fn [n <- :wat::WatAST] -> :wat::core::bool
                (:wat::core::= (:wat::core::ast-kind n) "string"))]
        (:wat::core::if has-act
          (:wat::core::let [a (:wat::core::Option/expect (:wat::core::get act-slot 0) "actual")]
            (:wat::core::if has-exp
              (:wat::core::let [e (:wat::core::Option/expect (:wat::core::get exp-slot 0) "expected")]
                (:wat::core::if (str? a)
                  (:wat::core::if (str? e)
                    `(:wat::kernel::assertion-failed!' ~msg (:wat::core::Option::Some {:value ~a}) (:wat::core::Option::Some {:value ~e}))
                    `(:wat::kernel::assertion-failed!' ~msg (:wat::core::Option::Some {:value ~a}) ~e))
                  (:wat::core::if (str? e)
                    `(:wat::kernel::assertion-failed!' ~msg ~a (:wat::core::Option::Some {:value ~e}))
                    `(:wat::kernel::assertion-failed!' ~msg ~a ~e))))
              (:wat::core::if (str? a)
                `(:wat::kernel::assertion-failed!' ~msg (:wat::core::Option::Some {:value ~a}) :wat::core::Option::None)
                `(:wat::kernel::assertion-failed!' ~msg ~a :wat::core::Option::None))))
          (:wat::core::if has-exp
            (:wat::core::let [e (:wat::core::Option/expect (:wat::core::get exp-slot 0) "expected")]
              (:wat::core::if (str? e)
                `(:wat::kernel::assertion-failed!' ~msg :wat::core::Option::None (:wat::core::Option::Some {:value ~e}))
                `(:wat::kernel::assertion-failed!' ~msg :wat::core::Option::None ~e)))
            `(:wat::kernel::assertion-failed!' ~msg :wat::core::Option::None :wat::core::Option::None)))))))
