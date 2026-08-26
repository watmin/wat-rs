;; probe-arc278-free-user-name-in-parent-defn.wat — a MEASUREMENT, and it must be RUN.
;;
;; THE QUESTION. The one-entry shape needs defservice's child main to become a REAL
;; top-level defn in the parent (`:<fqdn>::child-entry`) so `fn-forms` has a registered fn
;; to root the closure at. That body references `:user::spawn::service-locus` — a name the
;; parent DELIBERATELY never defines; the ProcessOpts launch arm prepends
;; `(def :user::spawn::service-locus (process))` before spawning, so only the CHILD resolves it.
;;
;; Ruling (2026-08-11): `:user::` is the RENDEZVOUS namespace. A free `:user::` name is
;; permitted by convention; anything may define any `:user::` name, and a user who gets it
;; wrong owns the mistake. So the substrate should not refuse it.
;;
;; BRACKET DOES NOT ANSWER THIS. Its `:user::bracket::work-fn` is the bind name `fn-forms`
;; itself emits, and its `:user::main` is quasiquoted DATA — neither ever faces the PARENT's
;; checker. This probe asks the question nothing has asked: does a parent-REGISTERED defn
;; survive freeze while referencing a free `:user::` name?
;;
;; ⚠ NON-VACUITY: the control ships WITHOUT the rendezvous def and must DIE naming it. If
;; both arms pass, the child never needed the name and the probe measured nothing.

;; ★ THE ANSWER (measured 2026-08-11): a free `:user::` name CANNOT sit in a typed position
;; inside a PARENT defn — the checker types it `:wat::core::keyword` and refuses
;; (`TypeMismatch: i64::+ #1 expects i64; got keyword`). An unresolved name has no type in
;; the parent. That is WHY defservice's child main is quasiquoted DATA and not a defn: data
;; is not parent-checked. The namespace ruling permits the NAME; it cannot supply a TYPE.
;;
;; So the entry takes the rendezvous value as a PARAMETER. The free name then appears only
;; in the shipped one-liner — data in the parent, and type-checked in the CHILD, where it IS
;; defined. Exactly bracket's shape: its main passes `:user::bracket::work-fn` INTO the runner.
(:wat::core::defn :probe::uses-free [n <- :wat::core::i64] -> :wat::core::i64
  (:wat::i64::+ n 1))

(:wat::core::defn :user::run-arm
  [label <- :wat::core::String  with-def? <- :wat::core::bool]
  -> :wat::core::nil
  (:wat::core::let
    [closure  (:wat::kernel::fn-forms :probe::uses-free :user::entry)
     ;; The launcher's job, done by hand: prepend the rendezvous def the child resolves.
     rendez   (:wat::core::Vector :wat::WatAST
                `(:wat::core::def :user::rendezvous::N 41))
     prefix   (:wat::core::if with-def? rendez (:wat::core::Vector :wat::WatAST))
     main     (:wat::core::Vector :wat::WatAST
                `(:wat::core::defn :user::main [] -> :wat::core::nil
                   (:wat::kernel::println (:user::entry :user::rendezvous::N))))
     forms    (:wat::core::concat (:wat::core::concat prefix closure) main)
     p        (:wat::test::spawn-peer (:wat::spawn::process) forms)]
    (:wat::core::match (:wat::kernel::recv p)
      ((:wat::kernel::RecvOutcome::Message m)
        (:wat::kernel::println (:wat::string::concat label " RAN, child returned:")))
      ((:wat::kernel::RecvOutcome::Lost cause)
        (:wat::kernel::println (:wat::string::concat label
          (:wat::string::concat " DIED " (:wat::kernel::LociDiedError/message cause)))))
      (:wat::kernel::RecvOutcome::Stopped
        (:wat::kernel::println (:wat::string::concat label " STOPPED")))
      (:wat::kernel::RecvOutcome::Closed
        (:wat::kernel::println (:wat::string::concat label " CLOSED-NO-MARKER"))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [_a (:user::run-arm "WITH-DEF   " true)]
    (:user::run-arm "CONTROL(no)" false)))
