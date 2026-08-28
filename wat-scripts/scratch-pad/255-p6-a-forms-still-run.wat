;; arc 255 Stone P6-a — Row 4 acceptance evidence: THE FORMS STILL RUN.
;; `if` and `let` behave identically to before the #[wat_special_form_impl]
;; annotations landed (STOP-1: no behaviour change), including a
;; TAIL-POSITION case deep enough (200,000) to prove the TCO path is
;; untouched — the tail match still calls eval_if_tail / eval_let_tail
;; DIRECTLY (STOP-2: this stone adds submissions, it does not reroute a
;; call). Depth matches the existing TCO gate's own proof
;; (tests/rete/probe_arc278_55_slice_one_vocabulary.rs, rows 7+8).
;;
;; Scratch, per holon/CLAUDE.md's `.wat` scratch convention (not the ephemeral session tmp).

;; non-tail if
(:wat::core::defn :probe::if-true [] -> :wat::core::i64
  (:wat::core::if true 1 2))
(:wat::core::defn :probe::if-false [] -> :wat::core::i64
  (:wat::core::if false 1 2))

;; sequential-binder let
(:wat::core::defn :probe::let-sequential [] -> :wat::core::i64
  (:wat::core::let [x 1 y 2] (:wat::i64::+ x y)))

;; tail-position if AND let, both exercised at depth 200000 in one
;; recursion: the outer if is the fn's tail form (eval_if_tail); its else
;; branch is a let whose sole body form is the recursive call
;; (eval_let_tail), so a native-stack SIGSEGV at this depth would prove
;; either gate broken.
(:wat::core::defn :probe::countdown-if-let-tail [n <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::if (:wat::i64::<= n 0)
    0
    (:wat::core::let [next (:wat::i64::- n 1)]
      (:probe::countdown-if-let-tail next))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    (:wat::kernel::println "if-true (expect 1):")
    (:wat::kernel::println (:probe::if-true))
    (:wat::kernel::println "if-false (expect 2):")
    (:wat::kernel::println (:probe::if-false))
    (:wat::kernel::println "let-sequential (expect 3):")
    (:wat::kernel::println (:probe::let-sequential))
    (:wat::kernel::println "tail if+let at depth 200000 (expect 0, no SIGSEGV):")
    (:wat::kernel::println (:probe::countdown-if-let-tail 200000))))
