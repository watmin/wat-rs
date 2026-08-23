;; wat-tests/core/record-def.wat — deftest-green ward for :wat::core::Record::def (BASE)
;; and :wat::holon::Record::def (HOLONIC), arc 245.3c-b.
;;
;; Grounded on:
;;   - macro spec:     wat/Record.wat (expansion diagrams, accessor naming, class-guard)
;;   - probe (Rust):   tests/probe_arc237_sC3_macro_split.rs (18/18 green)
;;   - probe (Rust):   tests/probe_arc234_stone2b_defrecord_macro.rs (6/6 green)
;;   - failure idiom:  wat-tests/core/option-expect.wat (run-thread + RunResult/failure)
;;   - deftest idiom:  wat-tests/core/core-equality.wat (plain deftest, empty prelude)
;;
;; Coverage:
;;   construct          — BASE construct returns correct field values via SLASH accessor
;;   slash-accessor     — :test::rd::Pt/x SLASH form (not bare :x)
;;   predicate-true     — (:test::rd::is-Pt? p) = true
;;   predicate-false    — cross-call: wrong class -> false
;;   class-guard        — accessor on wrong-class receiver panics with "got class"
;;   holonic-construct  — HOLONIC construct + slash-accessor
;;   holonic-to-holon   — (:wat::holon::to-holon h) succeeds for holonic, errors for base
;;   liskov             — defn [v <- :wat::core::Record] accepts holonic instance
;;
;; Record types and helpers are declared at the FILE TOP LEVEL (not in any
;; deftest prelude) so they appear exactly once in the compiled program.
;; Deftests use empty preludes () and reference these top-level declarations.

;; ─── Top-level type declarations ────────────────────────────────────────────

;; BASE record: two i64 fields.
(:wat::core::defrecord :test::rd::Pt [x <- :wat::core::i64  y <- :wat::core::i64])

;; Second BASE record (different class_fqdn, one field) — used in predicate-false
;; and class-guard tests.
(:wat::core::defrecord :test::rd::Box [w <- :wat::core::i64])

;; HOLONIC record: two i64 fields.
(:wat::holon::defrecord :test::rd::HPt [x <- :wat::core::i64  y <- :wat::core::i64])

;; Liskov helper: accepts ANY :wat::core::Record (base OR holonic) and returns true.
(:wat::core::defn :test::rd::accepts-base? [v <- :wat::core::Record] -> :wat::core::bool true)


;; ─── BASE: construct + slash-accessor (x) ───────────────────────────────────

(:wat::test::deftest :wat-tests::core::record-def::base-construct-x
  
  (:wat::core::let
    [p (:test::rd::Pt :x 3 :y 4)]
    (:wat::test::assert-eq (:test::rd::Pt/x p) 3)))

;; ─── BASE: slash-accessor (y) ────────────────────────────────────────────────

(:wat::test::deftest :wat-tests::core::record-def::base-construct-y
  
  (:wat::core::let
    [p (:test::rd::Pt :x 3 :y 4)]
    (:wat::test::assert-eq (:test::rd::Pt/y p) 4)))

;; ─── Predicate: true on matching class ──────────────────────────────────────

(:wat::test::deftest :wat-tests::core::record-def::predicate-true
  
  (:wat::core::let
    [p (:test::rd::Pt :x 3 :y 4)]
    (:wat::test::assert-eq (:test::rd::is-Pt? p) true)))

;; ─── Predicate: false on non-matching class ──────────────────────────────────
;;
;; Constructs a :test::rd::Box; calls :test::rd::is-Pt? on it; asserts false.
;; Validates that predicate discriminates via class_fqdn, not struct shape.

(:wat::test::deftest :wat-tests::core::record-def::predicate-false-cross-class
  
  (:wat::core::let
    [b (:test::rd::Box :w 99)]
    (:wat::test::assert-eq (:test::rd::is-Pt? b) false)))

;; ─── Class-safety guard — wrong-class receiver panics with "got class" ───────
;;
;; The accessor (:test::rd::Pt/x v) checks (= (type v) "test::rd::Pt") at
;; runtime and panics with a "got class" message on mismatch.
;; This is the load-bearing ward: a regression removing the guard turns it RED.
;;
;; Uses a nested run-thread inside the deftest's outer run-thread to catch
;; the runtime panic and surface it as RunResult/failure.
;;
;; Type-check: :test::rd::Box is also a :wat::core::Record, so passing it to
;; :test::rd::Pt/x (which takes :wat::core::Record) passes type-check. The
;; runtime class guard fires because Box is not Pt.


(:wat::test::deftest :wat-tests::core::record-def::class-guard-panics-got-class
  
  (:wat::core::let
    [p (:wat::test::spawn-peer (:wat::spawn::thread)
         (:wat::core::fn [self <- (:wat::kernel::ThreadSelfPeer :- [:wat::core::i64 :wat::core::i64])] -> :wat::core::nil
           ;; Accessor returns i64; do discards it and returns nil.
           ;; The class guard fires before the nil is reached — that's the point;
           ;; the crash reaches the parent's recv' as Lost before the completion send'.
           (:wat::core::do
             (:wat::core::do (:test::rd::Pt/x (:test::rd::Box :w 5)) nil)
             (:wat::core::match (:wat::kernel::send self 0)
               (:wat::kernel::SendOutcome::Sent   nil)
               (:wat::kernel::SendOutcome::Closed nil)
               ;; arc 278 #73 — same body as Sent/Closed: this send-outcome wall just
               ;; needs to proceed regardless; the class-guard panic above already
               ;; fired before this line could even run.
               (:wat::kernel::SendOutcome::Stopped nil)
               ((:wat::kernel::SendOutcome::Lost _c) nil)))))]
    (:wat::core::match (:wat::kernel::recv p)
      ((:wat::kernel::RecvOutcome::Message _m)
        (:wat::kernel::assertion-failed!
          "expected class-guard panic on wrong-class receiver; got Success"
          :wat::core::None :wat::core::None))
      ((:wat::kernel::RecvOutcome::Lost cause)
        (:wat::test::assert-contains
          (:wat::kernel::LociDiedError/message cause)
          "got class"))
      (:wat::kernel::RecvOutcome::Stopped
        (:wat::kernel::assertion-failed!
          "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open"
          :wat::core::None :wat::core::None))
      (:wat::kernel::RecvOutcome::Closed
        (:wat::kernel::assertion-failed!
          "expected class-guard panic on wrong-class receiver; got Success"
          :wat::core::None :wat::core::None)))))

;; ─── HOLONIC: construct + slash-accessor ─────────────────────────────────────

(:wat::test::deftest :wat-tests::core::record-def::holonic-construct-accessor
  
  (:wat::core::let
    [h (:test::rd::HPt :x 7 :y 8)]
    (:wat::test::assert-eq (:test::rd::HPt/x h) 7)))

;; ─── HOLONIC: to-holon succeeds ──────────────────────────────────────────────
;;
;; Holonic record has a holon_form; (:wat::holon::to-holon h) returns HolonAST.
;; We discard the result (_h) and do a sentinel assert-eq true true.
;; If to-holon panics the deftest's outer run-thread surfaces the failure.

(:wat::test::deftest :wat-tests::core::record-def::holonic-to-holon-ok
  
  ;; to-holon returns HolonAST. Bind the result and assert-coincident
  ;; it is coincident with itself — proves the call succeeded AND that
  ;; the returned HolonAST is a valid point in HD space (self-coincident
  ;; is the minimal geometric sanity check on any HolonAST).
  (:wat::core::let
    [h (:test::rd::HPt :x 1 :y 2)
     v (:wat::holon::to-holon h)]
    (:wat::test::assert-coincident v v)))

;; ─── BASE: to-holon errors at runtime ────────────────────────────────────────
;;
;; Base record has no holon_form; to-holon fires a MalformedForm RuntimeError.
;; run-thread catches the panic; match on failure; assert Some.


(:wat::test::deftest :wat-tests::core::record-def::base-to-holon-errors
  
  (:wat::core::let
    [p (:wat::test::spawn-peer (:wat::spawn::thread)
         (:wat::core::fn [self <- (:wat::kernel::ThreadSelfPeer :- [:wat::core::i64 :wat::core::i64])] -> :wat::core::nil
           ;; to-holon panics at runtime on base record; do discards result and
           ;; returns nil. The runtime error fires before the nil is reached — the
           ;; crash reaches the parent's recv' as Lost before the completion send'.
           (:wat::core::do
             (:wat::core::let
               [p (:test::rd::Pt :x 3 :y 4)]
               (:wat::core::do (:wat::holon::to-holon p) nil))
             (:wat::core::match (:wat::kernel::send self 0)
               (:wat::kernel::SendOutcome::Sent   nil)
               (:wat::kernel::SendOutcome::Closed nil)
               ;; arc 278 #73 — same body as Sent/Closed: this send-outcome wall just
               ;; needs to proceed regardless; the to-holon runtime error above already
               ;; fired before this line could even run.
               (:wat::kernel::SendOutcome::Stopped nil)
               ((:wat::kernel::SendOutcome::Lost _c) nil)))))]
    (:wat::core::match (:wat::kernel::recv p)
      ((:wat::kernel::RecvOutcome::Message _m)
        (:wat::kernel::assertion-failed!
          "expected to-holon runtime error on BASE record; got Success"
          :wat::core::None :wat::core::None))
      ((:wat::kernel::RecvOutcome::Lost _cause) nil)
      (:wat::kernel::RecvOutcome::Stopped
        (:wat::kernel::assertion-failed!
          "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open"
          :wat::core::None :wat::core::None))
      (:wat::kernel::RecvOutcome::Closed
        (:wat::kernel::assertion-failed!
          "expected to-holon runtime error on BASE record; got Success"
          :wat::core::None :wat::core::None)))))

;; ─── Liskov: [v <- :wat::core::Record] accepts a HOLONIC instance ──────────────────
;;
;; :test::rd::accepts-base? takes v <- :wat::core::Record (declared at file top level).
;; Passes a :test::rd::HPt (holonic) instance.
;; If the call passes type-check (holonic <: base) and evaluates, returns true.

(:wat::test::deftest :wat-tests::core::record-def::liskov-holonic-into-base
  
  (:wat::core::let
    [h (:test::rd::HPt :x 5 :y 6)]
    (:wat::test::assert-eq (:test::rd::accepts-base? h) true)))
