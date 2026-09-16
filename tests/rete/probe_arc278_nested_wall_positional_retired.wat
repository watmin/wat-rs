;; strike-nested-wall — KIND 4 of 4: `RhsPositionalConstructionRetired`, at the NESTED-CONSTRUCTOR
;; producer. Two positional args at a bare aggregate name — neither kwargs nor the single-value
;; passthrough.
;;
;; ⛔ READ THE PROBE'S DOC BEFORE TRUSTING THIS KIND'S MESSAGE. Driven at this HEAD, this exact
;; form COMPILED AND FIRED, deriving a correctly-valued fact: rete fire does not reach
;; `eval_kwargs_construct`'s retirement arm at all (`rhs_must_compile`, `kernel/arm.rs`, refuses to
;; walk `build_insert_fact`; the compiled path lowers through `expr_ir::lower_construct`, whose
;; `rete_kwargs_value_asts` treats positional args as declaration order and constructs happily).
;; So the variant's own doc — "that dispatch unconditionally retires multi-arg RAW POSITIONAL
;; construction" — describes the INTERPRETER path, not the one rete fire takes. The refusal below
;; is this wall enforcing the doctrine the fire path does not. Reported, not fixed here.

(:wat::core::defrecord :nwp::Src   [k <- :wat::core::i64])
(:wat::core::defrecord :nwp::Inner [x <- :wat::core::i64  y <- :wat::core::i64])
(:wat::core::defrecord :nwp::Outer [k <- :wat::core::i64  inner <- :nwp::Inner])

(:wat::rete::defrule :nwp::r
  :when [(:nwp::Src (?k <- :k))]
  :then [(:nwp::Outer :k ?k :inner (:nwp::Inner ?k ?k))])

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println "the wall refuses before main runs"))
