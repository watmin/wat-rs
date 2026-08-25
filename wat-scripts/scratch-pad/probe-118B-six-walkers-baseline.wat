;; probe-118B-six-walkers-baseline.wat — BASELINE for the "migrate the six" stone (118.B2b).
;;
;; ⛔ WHY THIS EXISTS: two of the five verbs about to be migrated have a DEGENERATE case whose
;; current behaviour I could not read off the source with confidence, and the seam's standing
;; alarm is *"do not design this tier from reading."* This probe pins what HEAD actually does,
;; BEFORE the migration, so the stone's "behaviour preserved" claim is a comparison and not an
;; assertion.
;;
;;   1. `take-nth` with n <= 0. Today it recurses on `(drop coll n)` — dropping from the FULL
;;      coll, head included — and `drop` CLAMPS a negative n to 0 (src/collection/transform.rs:201).
;;      So n=0 re-consumes the same collection forever: an infinite repeat of the first element.
;;      That happens to be clojure-faithful. Any `next`-based rewrite that emits `value` and then
;;      recurses on `rest` SILENTLY CHANGES THIS to "every element" — a real semantic change on a
;;      degenerate input, which is exactly the kind of thing a green floor would not catch
;;      (nothing in the corpus calls take-nth with n=0).
;;
;;   2. `reductions`' 2-arity on an EMPTY input. Its own comment claims an empty coll "raises via
;;      `first`'s out-of-range failure rather than a silent 0-arity dispatch". That is checkable
;;      for a Vector. It is NOT obviously true for a Stream — `first` on an exhausted Stream is a
;;      tracked B5 defect (it returns a bare nil), which would make the 2-arity Stream arm yield a
;;      one-element stream containing nil instead of raising. Row 3 asks HEAD directly.
;;
;; Every row prints; nothing asserts. The point is to RECORD HEAD's answers, not to judge them.
;; Run CAPPED (the seam's standing rule); row 1 builds an infinite stream and MUST stay behind a
;; `take`.
;;
;; ⛔ STONE 118.B4-iii — THE WALL (2026-08-18): row 2 (`:probe::first-of-empty-stream`) asked what
;; `first` returns on an EXHAUSTED Stream. That question is now MOOT, not merely unanswered:
;; `first` refuses every Stream, exhausted or not, at compile time — `:wat::core::first: parameter
;; #1 expects a lazy Stream<T> has no first/second/third — advance it with :wat::stream::next`. The
;; tracked B5 defect the row existed to pin (first-of-exhausted silently returning nil) is
;; permanently unreachable dead code now — the wall answers it by making it unaskable. Row
;; retired below; row 3 (`reductions` on an empty Stream) is untouched — its call site never went
;; through `first`/`rest`/`empty?`/`nth` directly, so it still type-checks.

;; ─── row 1 — take-nth's degenerate n ───────────────────────────────────────────────────────────
;; n=0 over [1 2 3], forced to 5 elements. If HEAD repeats the head forever this prints 1,1,1,1,1.
(:wat::core::defn :probe::take-nth-zero [] -> :wat::core::String
  (:wat::string::join ","
    (:wat::core::into []
      (:wat::core::take (:wat::core::take-nth 0 (:wat::core::Vector :wat::core::i64 1 2 3)) 5))))

;; n=1 over [1 2 3] — every element. The control that says row 1 is about n=0, not about take-nth.
(:wat::core::defn :probe::take-nth-one [] -> :wat::core::String
  (:wat::string::join ","
    (:wat::core::into []
      (:wat::core::take (:wat::core::take-nth 1 (:wat::core::Vector :wat::core::i64 1 2 3)) 5))))

;; n=2 over [1..6] — indices 0,2,4 => 1,3,5. The ordinary case, so a migration that breaks it
;; cannot hide behind "only the degenerate case moved".
(:wat::core::defn :probe::take-nth-two [] -> :wat::core::String
  (:wat::string::join ","
    (:wat::core::into []
      (:wat::core::take-nth 2 (:wat::core::Vector :wat::core::i64 1 2 3 4 5 6)))))

;; ─── row 2 — RETIRED by stone 118.B4-iii (THE WALL) — see the header note above.

;; ─── row 3 — reductions 2-arity over an EMPTY Stream ───────────────────────────────────────────
;; If the doc comment is right this RAISES. If `first` returns a bare nil it yields one element.
(:wat::core::defn :probe::reductions-empty-stream [] -> :wat::core::String
  (:wat::string::join ","
    (:wat::core::into []
      (:wat::core::take
        (:wat::core::reductions
          (:wat::core::fn [a <- :wat::core::i64 b <- :wat::core::i64] -> :wat::core::i64
            (:wat::core::+ a b))
          (:wat::stream::empty))
        5))))

;; ─── the ordinary reductions rows — the behaviour the migration must hold byte-for-byte ────────
(:wat::core::defn :probe::reductions-3arity [] -> :wat::core::String
  (:wat::string::join ","
    (:wat::core::into []
      (:wat::core::reductions
        (:wat::core::fn [a <- :wat::core::i64 b <- :wat::core::i64] -> :wat::core::i64
          (:wat::core::+ a b))
        0
        (:wat::core::Vector :wat::core::i64 1 2 3 4)))))

(:wat::core::defn :probe::reductions-2arity [] -> :wat::core::String
  (:wat::string::join ","
    (:wat::core::into []
      (:wat::core::reductions
        (:wat::core::fn [a <- :wat::core::i64 b <- :wat::core::i64] -> :wat::core::i64
          (:wat::core::+ a b))
        (:wat::core::Vector :wat::core::i64 1 2 3 4)))))

;; ─── the three stateless verbs — ordinary behaviour, all four containers' worth of shape ───────
(:wat::core::defn :probe::remove-evens [] -> :wat::core::String
  (:wat::string::join ","
    (:wat::core::into []
      (:wat::core::remove
        (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::bool
          (:wat::core::= 0 (:wat::core::mod x 2)))
        (:wat::core::Vector :wat::core::i64 1 2 3 4 5 6)))))

(:wat::core::defn :probe::take-while-lt4 [] -> :wat::core::String
  (:wat::string::join ","
    (:wat::core::into []
      (:wat::core::take-while
        (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::bool (:wat::core::< x 4))
        (:wat::core::Vector :wat::core::i64 1 2 3 4 1 2)))))

(:wat::core::defn :probe::drop-while-lt4 [] -> :wat::core::String
  (:wat::string::join ","
    (:wat::core::into []
      (:wat::core::drop-while
        (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::bool (:wat::core::< x 4))
        (:wat::core::Vector :wat::core::i64 1 2 3 4 1 2)))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    (:wat::kernel::println (:wat::string::concat "take-nth 0     : " (:probe::take-nth-zero)))
    (:wat::kernel::println (:wat::string::concat "take-nth 1     : " (:probe::take-nth-one)))
    (:wat::kernel::println (:wat::string::concat "take-nth 2     : " (:probe::take-nth-two)))
    (:wat::kernel::println (:wat::string::concat "remove even    : " (:probe::remove-evens)))
    (:wat::kernel::println (:wat::string::concat "take-while <4  : " (:probe::take-while-lt4)))
    (:wat::kernel::println (:wat::string::concat "drop-while <4  : " (:probe::drop-while-lt4)))
    (:wat::kernel::println (:wat::string::concat "reductions/3   : " (:probe::reductions-3arity)))
    (:wat::kernel::println (:wat::string::concat "reductions/2   : " (:probe::reductions-2arity)))
    (:wat::kernel::println (:wat::string::concat "reductions/2 [] : " (:probe::reductions-empty-stream)))))
