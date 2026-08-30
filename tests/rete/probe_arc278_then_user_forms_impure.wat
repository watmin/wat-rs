;; tests/rete/probe_arc278_then_user_forms_impure.wat — Stone B RED world, "prove it both
;; directions." Loaded via startup_from_file. The `:then` item's HEAD is a user fn
;; (`:tf::make-rate-bad`) composed of a CORE-namespaced, genuinely impure op
;; (`:wat::io::IOReader/open-file`) — mirrors probe_arc278_6b_ii_a_where_oracle_impure.wat's own
;; shape exactly, one layer down (RHS instead of LHS). The compile fence must refuse it, naming
;; the offending head AND axis — the identical `then-item-fence` mechanism as the GREEN worlds,
;; on a fn whose body does not bottom out in admitted ops.

(:wat::core::defrecord :tf::In   [n <- :wat::core::i64])
(:wat::core::defrecord :tf::Rate [count <- :wat::core::i64])

(:wat::core::defn :tf::make-rate-bad
  [r <- :tf::Rate]
  -> :tf::Rate
  (:wat::core::if (:wat::core::record? (:wat::io::IOReader/open-file "x")) r r))

(:wat::rete::defrule :tf::compute-bad
  :when [(:tf::In (?n <- :n))]
  :then [(:tf::make-rate-bad (:tf::Rate :count ?n))])

;; Compiling ALONE must panic (Option/expect -> panic_any) before ever inserting/firing anything —
;; this is the freeze-time-only claim (BRIEF-then-user-forms.md's "Freeze-time, never fire-time").
(:wat::core::defn :user::run-compile [] -> :wat::core::i64
  (:wat::core::let
    [rules   (:wat::rete::collect-rules :tf)
     session (:wat::core::match (:wat::rete::compile rules) ((:wat::rete::CompileOutcome::Compiled __session) __session) ((:wat::rete::CompileOutcome::MayNotTerminate __rule __fact-type) (:wat::kernel::assertion-failed! "compile: the rule set may not terminate" :wat::core::None :wat::core::None)))]
    (:wat::core::length (:wat::rete::Session/facts session))))
