;; PROBE — row 4 of the Span-fact stone, RE-RUN on a quiescent tree.
;;
;; Does a rete RHS construct a record with a NESTED record field, with LHS bindings flowing into
;; the nested constructor? Specifically: the user's RHS assembling a `:wat::core::Span` (which
;; carries `:file` — a property of the run, not of a node) from bound `?line`/`?col` plus a
;; filename supplied in the RHS itself. This is the assembly step the Span-fact stone's whole
;; ergonomics argument rests on — `:fx::Span` is flat because the USER reassembles the nested
;; record downstream, not the extractor.
;;
;; The predecessor of this probe ran beside a live writer mid-flight in the same tree (recorded
;; in DESIGN-STONE-the-span-fact.md) and is UNCREDITED for exactly that reason. This is the fresh,
;; quiescent-tree run. If the output differs from the recorded shape in any way, the difference is
;; the finding, not the old record.
;;
;; Recorded shape to reproduce:
;;   #p/Hit {:span #wat.core/Span {:file "a.wat" :line 7 :col 1 :end …} :why "…"}
;;
;; ⛔ AND THE CORRECTION THAT MATTERS — `:end` IS NOT None HERE. Builder, 2026-08-24:
;;   *"end set to none … that's reserved for rust code where we cannot know … in wat we always
;;    know … end must be optional as rust doesn't have a tool for its end of line coords."*
;;
;; The Option on `:end` exists for RUST's benefit, not wat's. The substrate says so in its own
;; hand at `crates/wat-reader/src/span.rs:69` — *"`end` is `Some(Pos)` when the lexer or parser
;; computed a real range (wat-source tokens and structural forms); `None` for point-spans from
;; Rust call sites (`rust_caller_span!()`) where no end is available"* — and splits the two
;; constructors on exactly that line: `Span::new` for `rust_caller_span!()`, `Span::with_end`
;; for the lexer and the parser (arc 281).
;;
;; So `None` is a PROVENANCE MARKER: it asserts "Rust built me, and Rust has no instrument for
;; the end." A wat-built Span carrying None is a lie about its own origin. `ast-end-span` is
;; TOTAL — wat always knows — so every Span a wat rule assembles carries `Some(Pos)`.
;;
;; That also makes this row STRICTLY HARDER, which is the point: it is no longer one level of
;; nesting but three — Pos inside Some inside Span — with LHS bindings flowing all the way to
;; the innermost constructor. The earlier `:end None` version proved a weaker claim than the one
;; wat-grep actually needs.

;; the LHS fact carries BOTH ends — exactly what `:fx::Span` emits per node.
(:wat::core::defrecord :p::Loc
  [line     <- :wat::core::i64
   col      <- :wat::core::i64
   end-line <- :wat::core::i64
   end-col  <- :wat::core::i64])

(:wat::core::defrecord :p::Hit
  [span <- :wat::core::Span
   why  <- :wat::core::String])

;; the RHS: LHS binds all four coordinates from a plain fact; the filename "a.wat" is supplied
;; IN the RHS (it is a property of the run, exactly as the DESIGN argues); `:end` is a real
;; `Some(Pos)` built from the bound end coords, because a wat-built Span always knows its end.
(:wat::rete::defrule :p::build-hit
  :when [(:p::Loc (?l <- :line) (?c <- :col) (?el <- :end-line) (?ec <- :end-col))]
  :then [(:p::Hit
           :span (:wat::core::Span
                   :file "a.wat"
                   :line ?l
                   :col  ?c
                   ;; ⚠ QUALIFIED, and it is not a style choice — see the finding at the foot.
                   :end  (:wat::core::Option::Some (:wat::core::Pos :line ?el :col ?ec)))
           :why "complete Span — Pos inside Some inside Span, all four coords LHS-bound")])

(:wat::rete::defquery :p::q-Hit
  :params []
  :when [(?fact <- :p::Hit)])

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [rules (:wat::core::PersistentVector (:p::build-hit))
     s0    (:wat::rete::insert
             (:wat::rete::compile-all rules (:wat::core::PersistentVector (:p::q-Hit)))
             (:p::Loc :line 7 :col 1 :end-line 7 :end-col 26))
     fired (:wat::core::match (:wat::rete::fire-rules s0) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))
     hits  (:wat::rete::query fired (:p::q-Hit))
     hit   (:wat::core::Option/expect
             (:wat::core::PersistentMap/get (:wat::core::first hits) "?fact")
             "q-Hit: ?fact")]
    (:wat::kernel::println hit)))

;; ─── NOTE FOR RULE AUTHORS (2026-08-24) — USE DECLARED ENUMS, NOT CORE'S ALIASES ─────────────
;;
;; Builder: *"rete has their own enums - use them."*
;;
;; A rete `:then` admits a head through `head_ok`'s FIRST door, `constructor_meta`, and Law A
;; exempts a DECLARATION-DERIVED head from the rete-namespace requirement entirely — "a record's
;; constructor and its field accessors exist by construction of the type." So anything that is a
;; real declaration sails through, whatever namespace it is spelled in:
;;
;;   a rule's OWN enum          (:g::End::Known ?l ?c)              → #g.End/Known [7 26]
;;   a declared core variant    (:wat::core::Option::Some …)        → compiles
;;   the bare core alias        (:wat::core::Some …)                → REFUSED
;;
;; The bare `:wat::core::{Some,Ok,Err}` are NOT declarations. They are special-cased by hardcoded
;; string equality in the checker and the runtime (`src/check.rs:6611`, `src/runtime.rs:30626` —
;; `matches!(s, ":wat::core::Some" | ":wat::core::Ok" | ":wat::core::Err")`), so there is nothing
;; for the constructor door to read and the head is judged as COMPUTATION, where default-deny is
;; correct. The message says "is not pure", which describes the axis that happened to be tested
;; first, not the reason — `Some` is not impure, it is unseen. That is the design working, not a
;; defect: the qualified path exists and is the one to write.
;;
;; ⚠ AND THE ONE THAT WILL BITE: a TAGGED VARIANT CONSTRUCTOR IS POSITIONAL, not kwargs.
;; `(:g::End::Known :line ?l :col ?c)` is an arity error — `expects 2 positional argument(s); got 4`.
;; Records take kwargs; tagged variants take positions. Both appear in one `:then` and they do not
;; look different at the call site.
