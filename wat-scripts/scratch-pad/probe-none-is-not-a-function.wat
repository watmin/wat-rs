;; probe-none-is-not-a-function.wat — THERE IS NO SUCH THING AS A TYPED NONE.
;;
;; The builder's question ended a false investigation: "wtf is a typed none? how does this
;; bear meaning? none... by definition... holds nothing."
;;
;; It holds nothing, and `:wat::core::None` is a KEYWORD, not a function. Measured:
;;
;;   :wat::core::None                              -> ":None"                      ✅
;;   (:wat::core::None :- [:wat::core::String])    -> RuntimeError UnknownFunction
;;   (:wat::core::None :wat::core::String)         -> CheckError (Doctrine 1)
;;   (:wat::core::None :wat::WatAST)               -> type-checks, then
;;                                                    RuntimeError UnknownFunction   ⛔
;;
;; ★ THE FINDING IS A PHANTOM FORM, and it is `cernere`'s class: a form that looks valid,
;; passes the checker, and has no definition at runtime. Doctrine 1 rejects a PRIMITIVE
;; type keyword in value position (`:wat::core::String`, `:wat::core::i64`) — so
;; (None :wat::core::String) is caught. It does NOT fire for a user/other type keyword, so
;; (None :wat::WatAST) and (None :some::Enum::Reply) sail through the checker and detonate
;; when evaluated.
;;
;; ⛔ ONE LIVE CORPUS SITE: wat-scripts/fixes/positional-to-kwargs.wat:27 —
;;      (:wat::core::None :wat::WatAST)
;;    in `:user::fieldvec-at`'s `i >= length` branch. It type-checks. If that branch is
;;    ever taken it raises UnknownFunction. A recorded, re-runnable migration with a
;;    latent detonation in a rarely-taken arm.
;;
;; ★ AND IT EXPLAINS AN ENTIRE FALSE INVESTIGATION. probe-reply-drop-is-userland.wat used
;; (:wat::core::None :cd::Drop::Reply) in a service arm's reply slot. It type-checked, then
;; raised UnknownFunction, which KILLED THE SERVICE — and I read the caller's resulting
;; LOST as "the reply was omitted and the caller was told". It was not. The service died of
;; my malformed code. There was never an Option defect, never a serve-loop anomaly, and
;; never two spellings of one value.
;;
;; expect: bare=:None;typed-detonates=yes

(:wat::config::set-redef! true)

(:wat::core::defn :nf::bare [] -> (:wat::core::Option :- [:wat::core::String])
  :wat::core::None)

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println
    (:wat::core::format "bare={b};typed-detonates=see-header"
      :b (:wat::core::show (:nf::bare)))))
