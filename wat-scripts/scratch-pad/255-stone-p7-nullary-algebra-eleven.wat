;; Scratch probe — arc 255 Stone P7, acceptance rows 2, 3, 5.
;;
;; Exercises DIRECT (AST-door) and APPLY (value-door) calls for all eleven verbs the
;; `sniff_kind` nullary-ALGEBRA predicate fix (BRIEF-STONE-P7) unblocks:
;;   :wat::uuid::v4 · :wat::uuid::nil · :wat::time::now · :wat::math::pi ·
;;   :wat::kernel::stopped? · sigusr1? · sigusr2? · sighup? ·
;;   :wat::kernel::reset-sigusr1! · reset-sigusr2! · reset-sighup!
;;
;; Wraps every call through `:wat::eval-ast!` (Result-returning) so a NotValueDispatchable
;; error is CAUGHT and printed, not an uncaught process death — run BEFORE the migration
;; (apply column shows the NotValueDispatchable text for all eleven) and AFTER (apply
;; column shows a real value for all eleven, direct column unchanged in shape).
;;
;; uuid::v4 / time::now are Nondeterministic — DIRECT and APPLY values will legitimately
;; differ call-to-call; only the "ok:" shape (not the literal payload) is comparable across
;; runs. The three reset-*! mutate global signal flags but grant no new capability (already
;; directly callable) — direct-call here is itself an intentional exercise of that mutation.
;;
;; Run: ./target/release/wat wat-scripts/scratch-pad/255-stone-p7-nullary-algebra-eleven.wat

(wat.core/defn probe/outcome [r :- (wat.core/Result :- [wat.type/Value wat.core/EvalError])]
  :- wat.type/String
  (wat.core/match r
    [wat.core/Result.Ok {:value v}  (wat.string/concat "ok:" (wat.edn/write v))]
    [wat.core/Result.Err {:error e} (wat.string/concat "err:" (wat.core.EvalError/message e))]))

(wat.core/defn probe/show [name   :- wat.type/String
                                 direct :- wat.type/AST
                                 thru   :- wat.type/AST]
  :- wat.type/nil
  (wat.core/let
    [d (probe/outcome (wat/eval-ast! direct))
     a (probe/outcome (wat/eval-ast! thru))]
    (wat.kernel/println (wat.string/concat name "  DIRECT=" d "  APPLY=" a))))

(wat.core/defn user/main [] :- wat.type/nil
  (wat.core/let
    [_01 (probe/show "uuid::v4              "
           (wat.core/quote (wat.uuid/v4))
           (wat.core/quote (wat.core/apply wat.uuid/v4 (wat.type/Vector :- [wat.type/Value]))))

     _02 (probe/show "uuid::nil             "
           (wat.core/quote (wat.uuid/nil))
           (wat.core/quote (wat.core/apply wat.uuid/nil (wat.type/Vector :- [wat.type/Value]))))

     _03 (probe/show "time::now             "
           (wat.core/quote (wat.time/now))
           (wat.core/quote (wat.core/apply wat.time/now (wat.type/Vector :- [wat.type/Value]))))

     _04 (probe/show "math::pi              "
           (wat.core/quote (wat.math/pi))
           (wat.core/quote (wat.core/apply wat.math/pi (wat.type/Vector :- [wat.type/Value]))))

     _05 (probe/show "kernel::stopped?      "
           (wat.core/quote (wat.kernel/stopped?))
           (wat.core/quote (wat.core/apply wat.kernel/stopped? (wat.type/Vector :- [wat.type/Value]))))

     _06 (probe/show "kernel::sigusr1?      "
           (wat.core/quote (wat.kernel/sigusr1?))
           (wat.core/quote (wat.core/apply wat.kernel/sigusr1? (wat.type/Vector :- [wat.type/Value]))))

     _07 (probe/show "kernel::sigusr2?      "
           (wat.core/quote (wat.kernel/sigusr2?))
           (wat.core/quote (wat.core/apply wat.kernel/sigusr2? (wat.type/Vector :- [wat.type/Value]))))

     _08 (probe/show "kernel::sighup?       "
           (wat.core/quote (wat.kernel/sighup?))
           (wat.core/quote (wat.core/apply wat.kernel/sighup? (wat.type/Vector :- [wat.type/Value]))))

     _09 (probe/show "kernel::reset-sigusr1!"
           (wat.core/quote (wat.kernel/reset-sigusr1!))
           (wat.core/quote (wat.core/apply wat.kernel/reset-sigusr1! (wat.type/Vector :- [wat.type/Value]))))

     _10 (probe/show "kernel::reset-sigusr2!"
           (wat.core/quote (wat.kernel/reset-sigusr2!))
           (wat.core/quote (wat.core/apply wat.kernel/reset-sigusr2! (wat.type/Vector :- [wat.type/Value]))))

     _11 (probe/show "kernel::reset-sighup! "
           (wat.core/quote (wat.kernel/reset-sighup!))
           (wat.core/quote (wat.core/apply wat.kernel/reset-sighup! (wat.type/Vector :- [wat.type/Value]))))]
    nil))
