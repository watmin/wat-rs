;; probe-brief-get-is-total-by-fallback.wat — RUN proof for
;; docs/arc/2026/06/278-rules-engine/BRIEF-get-is-total-by-fallback.md.
;;
;; Covers the brief's thirteen-row scorecard end to end:
;;   - rows 2/3/5/6 — happy path, out-of-range, empty container, all three containers
;;   - row 4        — NON-VACUITY: the SAME out-of-range expression, run twice with
;;                    DIFFERENT `:undefined` fallbacks. Rows 2/3/5 alone pass if the
;;                    new `Value::Option` arm merely returned a constant — only this
;;                    pair proves it returns the CALLER'S value.
;;   - row 7        — the seam still composes: the EXACT `where`-shaped expression at
;;                    the top of the brief (`:wat::rete::core::i64::>` wrapping
;;                    `:wat::rete::core::PersistentVector/get … :undefined -1`),
;;                    inside a REAL `:wat::rete::defrule`/`:wat::rete::where` clause,
;;                    type-checks (this file loads under `every_wat_scripts_file_loads`)
;;                    and evaluates (fires without raising, on a fact whose vector is
;;                    EMPTY — the exact shape that would abort the whole `fire-rules`
;;                    call if the fallback arm did not fire).
;;   - row 8        — i64/f64/holon fallbacks unregressed by editing the SHARED arm.
;;   - row 9        — `first` unregressed (STOP-2: its rows were untouched).
;;
;; A vacuous probe (no `:user::main`) proves nothing — this file has a real
;; `:user::main`, printing one line per assertion so the transcript is the proof.

;; ── row 7's fixture: a real rule using the brief's exact nested expression ─────────
(:wat::core::defrecord :g278get::PV [v <- (:wat::core::PersistentVector :- [:wat::core::i64])])
(:wat::core::defrecord :g278get::Hit [n <- :wat::core::i64])

;; Hit(1) :- PV(v) AND (PersistentVector/get v 0 :undefined -1) > 5.
;; Facts below: [7 8 9] (get 0 = 7, hits), [1 2 3] (get 0 = 1, no hit), []
;; (out-of-range at index 0 — the fallback -1 fires, -1 > 5 is false, no hit, and
;; critically the whole `fire-rules` call does NOT abort the way a raising `first`
;; would — that is exactly the totality the brief's ruling buys).
(:wat::rete::defrule :g278get::big-at-0
  :when
  [(:g278get::PV (?v <- :v))
   (:wat::rete::where (:wat::rete::core::i64::> (:wat::rete::core::PersistentVector/get ?v 0 :undefined -1) 5))]
  :then
  [(:g278get::Hit 1)])

(:wat::rete::defquery :g278get::q-Hit
  :params []
  :when [(?fact <- :g278get::Hit)])


(:wat::core::defn :g278get::row7 [] -> :wat::core::nil
  (:wat::core::let
    [s0    (:wat::rete::compile-all (:wat::rete::collect-rules :g278get) (:wat::core::PersistentVector (:g278get::q-Hit)))
     s1    (:wat::rete::insert s0 (:g278get::PV (:wat::core::PersistentVector 7 8 9)))
     s2    (:wat::rete::insert s1 (:g278get::PV (:wat::core::PersistentVector 1 2 3)))
     s3    (:wat::rete::insert s2 (:g278get::PV (:wat::core::PersistentVector)))
     fired (:wat::rete::fire-rules$oracle s3)]
    (:wat::kernel::println
      (:wat::string::concat "row7 seam-composes Hit-count (expect 1) = "
        (:wat::core::str (:wat::core::length (:wat::rete::query fired (:g278get::q-Hit))))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [pv       (:wat::core::PersistentVector 7 8 9)
     empty-pv (:wat::core::PersistentVector)
     vec      (:wat::core::Vector :wat::core::i64 7 8 9)
     lst      (:wat::core::List/of 7 8 9)
     h        (:wat::holon::to-holon "some-atom")
     other    (:wat::holon::to-holon "an-entirely-different-atom")
     zero     (:wat::holon::Blend h h 1.0 -1.0)]
    (:wat::core::do
      ;; ── row 2 — in-range returns the element, fallback NOT taken ──────────────
      (:wat::kernel::println
        (:wat::string::concat "row2 in-range (expect 8) = "
          (:wat::core::str (:wat::rete::core::PersistentVector/get pv 1 :undefined -1))))

      ;; ── row 3 — out-of-range takes the fallback ───────────────────────────────
      (:wat::kernel::println
        (:wat::string::concat "row3 out-of-range (expect -1) = "
          (:wat::core::str (:wat::rete::core::PersistentVector/get pv 9 :undefined -1))))

      ;; ── row 4 — NON-VACUITY: the SAME out-of-range expression, two DIFFERENT
      ;; fallback values. Rows 2/3/5 all pass if the arm returns a constant; only
      ;; this pair proves it returns the caller's own value.
      (:wat::kernel::println
        (:wat::string::concat "row4 run-a :undefined -1 (expect -1) = "
          (:wat::core::str (:wat::rete::core::PersistentVector/get pv 9 :undefined -1))))
      (:wat::kernel::println
        (:wat::string::concat "row4 run-b :undefined 42 (expect 42) = "
          (:wat::core::str (:wat::rete::core::PersistentVector/get pv 9 :undefined 42))))

      ;; ── row 5 — empty container ────────────────────────────────────────────────
      (:wat::kernel::println
        (:wat::string::concat "row5 empty-container (expect -1) = "
          (:wat::core::str (:wat::rete::core::PersistentVector/get empty-pv 0 :undefined -1))))

      ;; ── row 6 — all three containers behave identically ───────────────────────
      (:wat::kernel::println
        (:wat::string::concat "row6 Vector/get in-range (expect 8) = "
          (:wat::core::str (:wat::rete::core::Vector/get vec 1 :undefined -1))))
      (:wat::kernel::println
        (:wat::string::concat "row6 List/get in-range (expect 8) = "
          (:wat::core::str (:wat::rete::core::List/get lst 1 :undefined -1))))
      (:wat::kernel::println
        (:wat::string::concat "row6 Vector/get out-of-range (expect -1) = "
          (:wat::core::str (:wat::rete::core::Vector/get vec 9 :undefined -1))))
      (:wat::kernel::println
        (:wat::string::concat "row6 List/get out-of-range (expect -1) = "
          (:wat::core::str (:wat::rete::core::List/get lst 9 :undefined -1))))

      ;; ── row 7 — the seam still composes (real defrule, above) ─────────────────
      (:g278get::row7)

      ;; ── row 8 — i64/f64/holon fallbacks UNREGRESSED (this strike edits the
      ;; SHARED `Fallback` arm all four families run through) ────────────────────
      (:wat::kernel::println
        (:wat::string::concat "row8 i64::/ 1 0 :undefined -1 (expect -1) = "
          (:wat::core::str (:wat::rete::core::i64::/ 1 0 :undefined -1))))
      (:wat::kernel::println
        (:wat::string::concat "row8 f64::/ 0.0 0.0 :undefined -1.0 (expect -1) = "
          (:wat::core::str (:wat::rete::core::f64::/ 0.0 0.0 :undefined -1.0))))
      (:wat::kernel::println
        (:wat::string::concat "row8 holon::cosine degenerate :undefined -1.0 (expect -1) = "
          (:wat::core::str (:wat::rete::holon::cosine zero other :undefined -1.0))))

      ;; ── row 9 — `first` unregressed (STOP-2: its three rows were untouched) ───
      (:wat::kernel::println
        (:wat::string::concat "row9 PersistentVector/first empty :undefined -1 (expect -1) = "
          (:wat::core::str (:wat::rete::core::PersistentVector/first empty-pv :undefined -1))))

      nil)))
