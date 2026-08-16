;; Scratch probe for arc 300 stone C5c — NaN is UNORDERED gate, all 12 rows.
;; Run directly against a freshly built ./target/release/wat binary (bypassing the
;; long-running MCP eval servers, which are stale relative to this session's HEAD).
;; Not a committed test — reconnaissance only, kept loadable per the scratch-pad convention.

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println
    (:wat::core::PersistentMap
      ;; row 1/4 — polymorphic <, > with NaN on the right
      :row1-lt-1-nan   (:wat::core::< 1 (:wat::core::f64::/ 0.0 0.0))
      :row4-gt-1-nan   (:wat::core::> 1 (:wat::core::f64::/ 0.0 0.0))
      ;; row 2/3 — the defect
      :row2-le-1-nan   (:wat::core::<= 1 (:wat::core::f64::/ 0.0 0.0))
      :row3-ge-1-nan   (:wat::core::>= 1 (:wat::core::f64::/ 0.0 0.0))
      ;; row 5 — NaN on the LEFT, all four ops
      :row5-lt-nan-1   (:wat::core::< (:wat::core::f64::/ 0.0 0.0) 1)
      :row5-le-nan-1   (:wat::core::<= (:wat::core::f64::/ 0.0 0.0) 1)
      :row5-gt-nan-1   (:wat::core::> (:wat::core::f64::/ 0.0 0.0) 1)
      :row5-ge-nan-1   (:wat::core::>= (:wat::core::f64::/ 0.0 0.0) 1)
      ;; row 6 — NaN vs NaN
      :row6-lt-nan-nan (:wat::core::< (:wat::core::f64::/ 0.0 0.0) (:wat::core::f64::/ 0.0 0.0))
      :row6-le-nan-nan (:wat::core::<= (:wat::core::f64::/ 0.0 0.0) (:wat::core::f64::/ 0.0 0.0))
      ;; row 7 — = / not= untouched (IEEE exception)
      :row7-noteq      (:wat::core::not= 1 (:wat::core::f64::/ 0.0 0.0))
      :row7-eq         (:wat::core::= 1 (:wat::core::f64::/ 0.0 0.0))
      ;; row 8 — Inf unchanged
      :row8-lt-inf     (:wat::core::< 1 (:wat::core::f64::/ 1.0 0.0))
      :row8-le-inf     (:wat::core::<= 1 (:wat::core::f64::/ 1.0 0.0))
      ;; row 9 — C5b exactness intact
      :row9-exact      (:wat::core::< 9007199254740992.0 9007199254740993)
      ;; row 10 — a non-numeric ordering, spot check
      :row10-string    (:wat::core::< "abc" "abd")
      ;; row 11 — per-type spellings, i64 (structurally NaN-immune) + f64 (own engine)
      :row11-f64-le    (:wat::core::f64::<= 1.0 (:wat::core::f64::/ 0.0 0.0))
      :row11-f64-ge    (:wat::core::f64::>= 1.0 (:wat::core::f64::/ 0.0 0.0))
      )))
