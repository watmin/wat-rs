;; Arc 279 follow-on — the disconfirming probe for `str` totality.
;;
;; 279's DESIGN.md:67 specifies `str` as rendering "ANY value unquoted
;; (String→itself, i64→digits, bool→true/false, …)". The shipped intrinsic is a
;; FIVE-ARM match (`String | i64 | f64 | bool | u8`, runtime.rs eval_str) that
;; RAISES on everything else. The `…` was never filled in.
;;
;; The target rendering is not invented here — it is what the EDN encoder already
;; emits for the same values (measured through `println`, 2026-08-14):
;;     :a-keyword    nil    [1 2 3]    ["a" "b"]    {:a 1 :b 2}
;; `show` is a THIRD renderer (value/observe.rs `render_value`) that duplicates
;; this badly in Rust `Debug` shape: `()` for nil, `[1, 2, 3]`, `{:a: 1}`.
;;
;; CONTROLS FIRST. Rows 1-3 are GREEN at HEAD and must stay green — without them a
;; red on rows 4-8 could mean "the harness is broken" rather than "str is partial".

;; ── CONTROLS (green at HEAD) ────────────────────────────────────────────────

;; A top-level String renders BARE through `str` — this is what makes `str` `str`.
(:wat::core::defn :t::control-str-string-is-bare [] -> :wat::core::String
  (:wat::core::str "abc"))

;; ...and QUOTED through `show`. This single difference is the whole distinction
;; between the two verbs; everything else about them should be identical.
(:wat::core::defn :t::control-show-string-is-quoted [] -> :wat::core::String
  (:wat::core::show "abc"))

;; A scalar inside the five-arm domain works today.
(:wat::core::defn :t::control-str-i64 [] -> :wat::core::String
  (:wat::core::str 42))

;; ── THE REDS (each raises at HEAD) ──────────────────────────────────────────

;; A keyword is a scalar by every reasonable reading and is NOT in the five.
(:wat::core::defn :t::probe-keyword [] -> :wat::core::String
  (:wat::core::str :a-keyword))

;; nil. `show` renders this as `()` — Rust's unit leaking through a wat verb.
(:wat::core::defn :t::probe-nil [] -> :wat::core::String
  (:wat::core::str nil))

;; A Vector. `show` renders `[1, 2, 3]` (comma-space: Rust Debug); EDN says `[1 2 3]`.
(:wat::core::defn :t::probe-vector [] -> :wat::core::String
  (:wat::core::str (:wat::core::Vector :- [:wat::core::i64] 1 2 3)))

;; A map. `show` renders `{:a: 1}` — a DOUBLED colon, which is nobody's syntax.
;; Key ORDER is deliberately not asserted: maps are unordered, and pinning order
;; here would be string equality standing in for data equality (builder's ruling,
;; 2026-08-14). One key only, so the assertion is order-free by construction.
(:wat::core::defn :t::probe-map [] -> :wat::core::String
  (:wat::core::str {:a 1}))

;; NESTED strings stay QUOTED even though the top-level one would not be. This is
;; the Clojure rule (`str` uses the readable form inside collections) and it is
;; the row that proves `str` is not merely "show with the quotes stripped".
(:wat::core::defn :t::probe-nested-string-stays-quoted [] -> :wat::core::String
  (:wat::core::str (:wat::core::Vector :- [:wat::core::String] "a")))

;; ── THE ROW THIS PROBE SHOULD HAVE HAD ON DAY ONE ───────────────────────────
;; `str` on a RECORD. The original probe sampled a map, a float, a keyword, nil and a
;; nested string — every shape EXCEPT the one that routes through the type registry —
;; so it certified `str` "total" while `(str <record>)` answered
;; `#t/Pt {:field-0 1 :field-1 2}`: positional keys, names silently discarded, while
;; `println` of the same value answered `{:x 1 :y 2}`. ONE VALUE, TWO FACES.
;; Root: `value_to_edn_string` hardcoded `None` for the registry (296's `field-N`
;; defect). That door is DELETED; this row is the wall that keeps it deleted.
(:wat::core::defrecord :t::Pt [x <- :wat::core::i64  y <- :wat::core::i64])

(:wat::core::defn :t::probe-record-named-fields [] -> :wat::core::String
  (:wat::core::str (:t::Pt :x 1 :y 2)))
