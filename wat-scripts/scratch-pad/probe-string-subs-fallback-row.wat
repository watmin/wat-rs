;; probe-string-subs-fallback-row.wat — arc 278 #57 round 2, the last outstanding row:
;; `:wat::rete::core::string::subs`. `:wat::core::string::subs` is PARTIAL (raises
;; `MalformedForm` with `head: ":wat::core::string::subs"` on an out-of-range index); the new
;; row buys totality via `dispatch_rete_op`'s `Fallback` arm, same mechanism as `first`/`get`/
;; the i64-f64-holon quartets — this is the FIRST 3-real-arg Fallback row, proving the arm's
;; `op.params.len()`-derived arity split holds at this shape too.
;;
;; ROW 2 — happy path: in-range indices, fallback NOT taken.
;; ROW 3 — out-of-range indices, fallback FIRES.
;; ROW 4 — NON-VACUITY: the SAME out-of-range call with two DIFFERENT `:undefined` values.
;;   Rows 2/3 alone pass if the arm just returns a constant; only this pair proves the arm
;;   returns the CALLER's fallback value.
;; ROW 6 — every other Fallback family, unregressed by this strike: i64::/, f64::/,
;;   PersistentVector/get, PersistentVector/first, holon::cosine (degenerate).

(:wat::core::defn :probe::run [] -> :wat::core::nil
  (:wat::core::let
    [h     (:wat::holon::to-holon "some-atom")
     other (:wat::holon::to-holon "an-entirely-different-atom")
     zero  (:wat::holon::Blend h h 1.0 -1.0)

     ;; ROW 2 — happy path, fallback not taken.
     row2-happy (:wat::rete::string::subs "hello" 1 3 :undefined "?")

     ;; ROW 3 — out-of-range, fallback fires.
     row3-out-of-range (:wat::rete::string::subs "hello" 2 99 :undefined "?")

     ;; ROW 4 — same out-of-range call, two different fallback values.
     row4-run-a (:wat::rete::string::subs "hello" 2 99 :undefined "?")
     row4-run-b (:wat::rete::string::subs "hello" 2 99 :undefined "gone")

     ;; ROW 6 — every other Fallback family, unregressed.
     row6-i64-div    (:wat::rete::i64::/ 1 0 :undefined -1)
     row6-f64-div    (:wat::rete::f64::/ 0.0 0.0 :undefined -1.0)
     row6-pv-get     (:wat::rete::core::PersistentVector/get (:wat::core::PersistentVector 7 8 9) 99 :undefined -1)
     row6-pv-first   (:wat::rete::core::PersistentVector/first (:wat::core::PersistentVector) :undefined -1)
     row6-cosine     (:wat::rete::holon::cosine zero other :undefined -1.0)]

    (:wat::core::do
      (:wat::kernel::println (:wat::core::PersistentMap :row2-happy row2-happy))
      (:wat::kernel::println (:wat::core::PersistentMap :row3-out-of-range row3-out-of-range))
      (:wat::kernel::println (:wat::core::PersistentMap :row4-run-a row4-run-a))
      (:wat::kernel::println (:wat::core::PersistentMap :row4-run-b row4-run-b))
      (:wat::kernel::println (:wat::core::PersistentMap :row6-i64-div row6-i64-div))
      (:wat::kernel::println (:wat::core::PersistentMap :row6-f64-div row6-f64-div))
      (:wat::kernel::println (:wat::core::PersistentMap :row6-pv-get row6-pv-get))
      (:wat::kernel::println (:wat::core::PersistentMap :row6-pv-first row6-pv-first))
      (:wat::kernel::println (:wat::core::PersistentMap :row6-cosine row6-cosine)))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:probe::run))
