;; tests/types/probe_stone_118_11a_next.wat — co-located fixture for probe_stone_118_11a_next.rs
;;
;; Stone 118.11a — mint `:wat::stream::next` + `(:wat::stream::NextOutcome :- [T])` (additive; the
;; `forced: OnceLock` memo in src/stream/mod.rs is untouched, no existing walker migrates).
;; Covers gate rows 1, 2, 3, 4 of DESIGN-STONE-118.11a / BRIEF-STONE-118.11a /
;; EXPECTATIONS-STONE-118.11a.
;;
;; `:probe::row1` / `:probe::row2` / `:probe::row4` are plain zero-arg fns — Rust drives them via
;; `call_beside_value` and inspects the returned `(NextOutcome :- [T])` `Value::Enum` directly.
;;
;; `:user::main` is row 3, the ONE-FORCE-PER-CALL row — run as a real subprocess (`wat_cli.rs`'s
;; pattern) so the Rust side can count REAL stdout lines: `f` prints exactly once per call it is
;; given, so a single `next` on `(map f v)` must print exactly one line.

;; Row 1 — (next <3-element stream>) -> Item, value = first element.
(:wat::core::defn :probe::row1 [] -> (:wat::stream::NextOutcome :- [:wat::core::i64])
  (:wat::stream::next
    (:wat::stream::cons 1
      (:wat::stream::cons 2
        (:wat::stream::cons 3
          (:wat::stream::empty))))))

;; Row 2 — (next <exhausted stream>) -> Exhausted.
(:wat::core::defn :probe::row2 [] -> (:wat::stream::NextOutcome :- [:wat::core::i64])
  (:wat::stream::next (:wat::stream::empty)))

;; Row 4 — pulling `rest` from row 1's Item and calling `next` again yields the SECOND element.
(:wat::core::defn :probe::row4 [] -> (:wat::stream::NextOutcome :- [:wat::core::i64])
  (:wat::core::match (:probe::row1)
    ((:wat::stream::NextOutcome::Item value rest) (:wat::stream::next rest))
    (:wat::stream::NextOutcome::Exhausted
      (:wat::kernel::assertion-failed! "row1 must be Item — row4 fixture is broken"
        :wat::core::None :wat::core::None))))

;; Row 3 — with a printing `f`, ONE `next` on `(map f v)` prints EXACTLY ONE LINE.
;; `f` prints "CALLED" (via the primed `:wat::kernel::println`, which requires a running
;; program — hence the subprocess) once per invocation, then returns its arg unchanged. A single
;; `:wat::stream::next` call on `(map f v)` — never a second one — is the entire probe.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [v (:wat::stream::cons 1
         (:wat::stream::cons 2
           (:wat::stream::cons 3
             (:wat::stream::empty))))
     f (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::i64
         (:wat::core::do
           (:wat::kernel::println "CALLED")
           x))
     mapped (:wat::core::map f v)
     r (:wat::stream::next mapped)]
    (:wat::core::match r
      ((:wat::stream::NextOutcome::Item value rest) nil)
      (:wat::stream::NextOutcome::Exhausted
        (:wat::kernel::assertion-failed! "row3: next on (map f v) must be Item"
          :wat::core::None :wat::core::None)))))
