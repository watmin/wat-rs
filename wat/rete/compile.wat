;; wat/rete/compile.wat — interpreted compile (rule-set → network).
;;
;; compile-condition / compile-rule / compile-all, Axis / AxisViolation.
;; Loads after wat/rete.wat (records). This file is the compile oracle
;; (no compile$native verb). Native compile of conds/rhs happens in
;; `arm-session` (`compile_condition_local` / `compile_rhs` / `compile_cond_driver`).
;;
;; Namespace: :wat::rete::

;; ─── compile — rule-set → shared connected network ──────────────────────────

;; CompileState — internal state threaded through compile's rule + condition folds.
;; network: the id→Node PersistentMap built so far.
;; next-id: the next free node id.
;; dedup:   (HashMap :- [String i64]) — maps a structural key to the existing node id;
;;          avoids rescanning the network to detect shareable nodes.
;; WHY a record: cleaner than a Tuple at call sites; fields are domain nouns.
(:wat::core::defrecord :wat::rete::CompileState
  [network <- :wat::core::PersistentMap
   next-id <- :wat::core::i64
   dedup   <- (:wat::core::HashMap :- [:wat::core::String :wat::core::i64])])

;; MintResult — result of find-or-mint: the resolved node id + updated state.
;; WHY a record: named fields communicate intent at call sites better than positional.
(:wat::core::defrecord :wat::rete::MintResult
  [id    <- :wat::core::i64
   state <- :wat::rete::CompileState])

;; network-add-child — add child-id to the children of the node at node-id in network.
;; Returns the updated PersistentMap.
;; SET SEMANTICS: children is a set of out-edges. Adding a child-id already present is a
;; no-op — a rete edge means "propagate to this child", and a second identical edge would mean
;; "propagate twice", which no caller wants. When two rules share a compiled node (routine under
;; the find-or-mint-* dedup), this keeps the node's out-degree from growing once per rule.
;; WHY: wiring edges = conj child-id onto the existing children PersistentVector and
;; re-assoc the node; :wat::core::Record/assoc does name-based field update on any Record.
(:wat::core::defn :wat::rete::network-add-child
  [network  <- :wat::core::PersistentMap
   node-id  <- :wat::core::i64
   child-id <- :wat::core::i64]
  -> :wat::core::PersistentMap
  (:wat::core::let [node   (:wat::core::Option/expect
                                  (:wat::core::PersistentMap/get network node-id)
                                  "network-add-child: node not found")
                    old-ch (:wat::rete::node-children-ids node)]
    (:wat::core::if (:wat::core::PersistentVector/contains? old-ch child-id)
      network
      (:wat::core::let [new-ch   (:wat::core::PersistentVector/conj old-ch child-id)
                        new-node (:wat::core::Record/assoc node :children new-ch)]
        (:wat::core::PersistentMap/assoc network node-id new-node)))))

;; find-or-mint-alpha — find an existing AlphaNode whose tests == cond, or mint a new one.
;; Dedup key: "alpha:<write-forms cond>".
;; Returns a MintResult(id, updated-state).
;; WHY write-forms for key: gives a canonical string from the WatAST form; structural
;; equality on the form is span-agnostic so identical conditions always produce the same key.
(:wat::core::defn :wat::rete::find-or-mint-alpha
  [cond  <- :wat::WatAST
   state <- :wat::rete::CompileState]
  -> :wat::rete::MintResult
  (:wat::core::let [cond-text (:wat::core::write-forms cond)
                    dkey      (:wat::string::interpolate "alpha:{cond-text}" :cond-text cond-text)
                    network   (:wat::rete::CompileState/network state)
                    next-id   (:wat::rete::CompileState/next-id state)
                    dedup     (:wat::rete::CompileState/dedup   state)
                    found-opt (:wat::core::HashMap/get dedup dkey)]
    (:wat::core::match found-opt 
      ((:wat::core::Some existing-id)
       (:wat::rete::MintResult :id existing-id :state state))
      (:wat::core::None
       (:wat::core::let [alpha     (:wat::rete::AlphaNode
                                      :id next-id
                                      :tests (:wat::core::PersistentVector cond)
                                      :children (:wat::core::PersistentVector))
                         new-net   (:wat::core::PersistentMap/assoc network next-id alpha)
                         new-dedup (:wat::core::HashMap/assoc dedup dkey next-id)
                         new-state (:wat::rete::CompileState
                                      :network new-net
                                      :next-id (:wat::core::i64::+ next-id 1)
                                      :dedup new-dedup)]
         (:wat::rete::MintResult :id next-id :state new-state))))))

;; exists-uses-alpha-probe? — a fact-shaped inner (Reading(?g), Maintenance(?loc))
;; is what alpha already stored. `:where` / `:and` / `:or` / `:not` / `:exists`
;; stay on exists-cond-under (where-not-where is eval-test; leftover `?v < ?m`
;; after accum is a where, not a reason to scan facts for the fact-shaped case).
(:wat::core::defn :wat::rete::exists-uses-alpha-probe?
  [cond <- :wat::WatAST] -> :wat::core::bool
  (:wat::core::let [head-nm (:wat::core::ast-name
                              (:wat::core::first (:wat::core::ast->children cond)))]
    (:wat::core::not
      (:wat::core::or (:wat::core::= head-nm ":wat::rete::where")
        (:wat::core::or (:wat::core::= head-nm ":wat::rete::and")
          (:wat::core::or (:wat::core::= head-nm ":wat::rete::or")
            (:wat::core::or (:wat::core::= head-nm ":wat::rete::not")
              (:wat::core::= head-nm ":wat::rete::exists"))))))))

;; cond-children — wrapper arms of `(:wat::rete::and …)` / `(:or …)`.
(:wat::core::defn :wat::rete::cond-children
  [form <- :wat::WatAST]
  -> (:wat::core::PersistentVector :- [:wat::WatAST])
  (:wat::core::let [ch (:wat::core::ast->children form)]
    (:wat::core::foldl
      (:wat::core::fn [acc <- (:wat::core::PersistentVector :- [:wat::WatAST])
                       i   <- :wat::core::i64]
        -> (:wat::core::PersistentVector :- [:wat::WatAST])
        (:wat::core::PersistentVector/conj acc
          (:wat::core::Option/expect
            (:wat::core::get ch i)
            "cond-children")))
      (:wat::core::PersistentVector)
      (:wat::core::range 1 (:wat::core::length ch)))))

;; mint-leaf-alphas — a `:not` / `:exists` inner that is `:and` / `:or` / `:not`
;; has no useful single-fact alpha. Mint an alpha for each fact-shaped leaf so
;; binding-extensions can probe the right bag instead of the session fact vector.
(:wat::core::defn :wat::rete::mint-leaf-alphas
  [cond  <- :wat::WatAST
   state <- :wat::rete::CompileState]
  -> :wat::rete::CompileState
  (:wat::core::if (:wat::rete::exists-uses-alpha-probe? cond)
    (:wat::rete::MintResult/state (:wat::rete::find-or-mint-alpha cond state))
    (:wat::core::let [head-nm (:wat::core::ast-name
                                (:wat::core::first (:wat::core::ast->children cond)))]
      (:wat::core::cond
        ((:wat::core::= head-nm ":wat::rete::where")
         state)
        ((:wat::core::or (:wat::core::= head-nm ":wat::rete::and")
                         (:wat::core::= head-nm ":wat::rete::or"))
         (:wat::core::foldl
           (:wat::core::fn [st  <- :wat::rete::CompileState
                            kid <- :wat::WatAST]
             -> :wat::rete::CompileState
             (:wat::rete::mint-leaf-alphas kid st))
           state
           (:wat::rete::cond-children cond)))
        ((:wat::core::or (:wat::core::= head-nm ":wat::rete::not")
                         (:wat::core::= head-nm ":wat::rete::exists"))
         (:wat::rete::mint-leaf-alphas
           (:wat::core::second (:wat::core::ast->children cond))
           state))
        (:else
         (:wat::rete::MintResult/state (:wat::rete::find-or-mint-alpha cond state)))))))

;; find-or-mint-root-join — find or mint a RootJoinNode for the first condition.
;; Dedup key: "rootjoin:<cond-text>".
;; WHY split from hash-join: if-branching between different record types (RootJoinNode vs
;; HashJoinNode) cannot be unified by the type checker; two typed fns avoid the mismatch.
(:wat::core::defn :wat::rete::find-or-mint-root-join
  [cond  <- :wat::WatAST
   state <- :wat::rete::CompileState]
  -> :wat::rete::MintResult
  (:wat::core::let [cond-text (:wat::core::write-forms cond)
                    dkey      (:wat::string::interpolate "rootjoin:{cond-text}" :cond-text cond-text)
                    network   (:wat::rete::CompileState/network state)
                    next-id   (:wat::rete::CompileState/next-id state)
                    dedup     (:wat::rete::CompileState/dedup   state)
                    found-opt (:wat::core::HashMap/get dedup dkey)]
    (:wat::core::match found-opt 
      ((:wat::core::Some existing-id)
       (:wat::rete::MintResult :id existing-id :state state))
      (:wat::core::None
       (:wat::core::let [join-node (:wat::rete::RootJoinNode
                                      :id next-id
                                      :children (:wat::core::PersistentVector))
                         new-net   (:wat::core::PersistentMap/assoc network next-id join-node)
                         new-dedup (:wat::core::HashMap/assoc dedup dkey next-id)
                         new-state (:wat::rete::CompileState
                                      :network new-net
                                      :next-id (:wat::core::i64::+ next-id 1)
                                      :dedup new-dedup)]
         (:wat::rete::MintResult :id next-id :state new-state))))))

;; find-or-mint-hash-join — find or mint a HashJoinNode for a non-first condition.
;; Dedup key: "hashjoin:<parent-id>:<cond-text>" — both condition AND left parent must match.
(:wat::core::defn :wat::rete::find-or-mint-hash-join
  [cond      <- :wat::WatAST
   parent-id <- :wat::core::i64
   state     <- :wat::rete::CompileState]
  -> :wat::rete::MintResult
  (:wat::core::let [cond-text (:wat::core::write-forms cond)
                    pid-s     (:wat::core::i64::to-string parent-id)
                    dkey      (:wat::string::interpolate "hashjoin:{pid-s}:{cond-text}" :pid-s pid-s :cond-text cond-text)
                    network   (:wat::rete::CompileState/network state)
                    next-id   (:wat::rete::CompileState/next-id state)
                    dedup     (:wat::rete::CompileState/dedup   state)
                    found-opt (:wat::core::HashMap/get dedup dkey)]
    (:wat::core::match found-opt 
      ((:wat::core::Some existing-id)
       (:wat::rete::MintResult :id existing-id :state state))
      (:wat::core::None
       (:wat::core::let [join-node (:wat::rete::HashJoinNode
                                      :id next-id
                                      :children (:wat::core::PersistentVector))
                         new-net   (:wat::core::PersistentMap/assoc network next-id join-node)
                         new-dedup (:wat::core::HashMap/assoc dedup dkey next-id)
                         new-state (:wat::rete::CompileState
                                      :network new-net
                                      :next-id (:wat::core::i64::+ next-id 1)
                                      :dedup new-dedup)]
         (:wat::rete::MintResult :id next-id :state new-state))))))

;; compile-condition — fold step: process one condition form in a rule.
;; acc = (CompileState, PV of parent node-ids). Empty PV = no parent yet (first
;; condition → RootJoin). Condition `:or` leaves N arm terminals in the PV;
;; the next condition fans out (one HashJoin / one Test / one :not per parent)
;; and Clara does not require `:or` to be last.
(:wat::core::defrecord :wat::rete::CondFoldAcc
  [state      <- :wat::rete::CompileState
   parent-ids <- (:wat::core::PersistentVector :- [:wat::core::i64])])

;; wire-parents — hang `child` off every parent (condition `:or` leaves N terminals).
(:wat::core::defn :wat::rete::wire-parents
  [network <- :wat::core::PersistentMap
   pids    <- (:wat::core::PersistentVector :- [:wat::core::i64])
   child   <- :wat::core::i64]
  -> :wat::core::PersistentMap
  (:wat::core::foldl
    (:wat::core::fn [net <- :wat::core::PersistentMap
                     pid <- :wat::core::i64]
      -> :wat::core::PersistentMap
      (:wat::rete::network-add-child net pid child))
    network
    pids))

;; Axis — BRIEF-the-fence-names-the-head, builder-ruled: the CLOSED-SET RULE
;; (REALIZATIONS.md:2676 — "a closed set is an enum, name holds value; open identifiers stay
;; Keyword/String"). WHICH axis `:wat::rete::axis-violation` is being asked about. `:Total` landed
;; BRIEF-total-t1-the-axis-unarmed — and, as designed, minting it broke `axis-violation-message`'s
;; exhaustive match below until its arm was added: the checker enumerated its own consumers instead
;; of anyone remembering by hand. `:Total` is ARMED — `compile-condition` consults
;; `:wat::rete::total?` as the third conjunct of the four-axis fence (pure ∧ det ∧ total ∧ rete).
;;
;; ★ NAME COLLISION — read before editing this line: the nature marker `:wat::enum::Pure` right
;; after the type name is UNRELATED to the `:Pure` VARIANT two lines under it. The marker declares
;; THIS ENUM's own runtime representation holds pure data (trivially true — every variant here is a
;; bare unit tag, no payload, so it can never carry a live resource, same as
;; `:wat::runtime::Purity`/`:wat::runtime::Determinism` in runtime-meta.wat). The `:Pure` VARIANT is
;; the axis constant meaning "check effect-freedom." Same word, two unrelated things — do not
;; conflate them when you read or extend this form.
(:wat::core::defenum :wat::rete::Axis :wat::enum::Pure
  :Pure
  :Deterministic
  :Total
  ;; #57 LAW A — the head is a rete primitive. Armed on `where` / accumulate / `:then`.
  ;; Its own variant because reusing :Pure would make the refusal LIE — `:wat::core::>` IS
  ;; pure, deterministic and total, and is refused (on those arms) for one reason only: it
  ;; is not from rete. The name is a WORD IN THE SENTENCE `axis-violation-message` builds
  ;; ("is not a rete primitive"), not a label — an earlier spelling, `:Vocabulary`, was cast
  ;; to intueri and failed exactly there: "is not vocabulary" does not parse, and it named the
  ;; table we check rather than the law we hold.
  :RetePrimitive)

;; AxisViolation — BRIEF-the-fence-names-the-head. The result of `:wat::rete::axis-violation`:
;; the offending head when a `where`/accumulator expr falsifies an axis (pure, deterministic,
;; total, or rete-primitive).
;;   head: the violating verb's fqdn (e.g. ":wat::io::IOReader/open-file", ":wat::core::Uuid/v4").
;;   axis: which axis was asked about (`:Pure` / `:Deterministic` / `:Total` / `:RetePrimitive`) —
;;         echoed back for self-description.
;;   span: the failing call's source Location. Native stubs with no body AST use
;;         rust_caller_span so the field is never omitted.
;; Fields: head (fqdn), axis (which fence conjunct failed), span (call site).
(:wat::core::defrecord :wat::rete::AxisViolation
  [head <- :wat::core::String
   axis <- :wat::rete::Axis
   span <- :wat::kernel::Location])

;; first-failing-axis — given the SAME booleans a fence's `and` already computed, names WHICH axis
;; to explain, mirroring `and`'s left-to-right short-circuit: report the FIRST conjunct that failed,
;; because that is the one the caller must fix first. Never called when all hold — every fence call
;; site reaches this only inside the branch where the accept check already failed.
;;
;; ★ THE CHAIN OF RESTRICTIONS, and the ORDER IS THE MESSAGE (builder-ruled 2026-08-05):
;;
;;     is-pure  →  is-deterministic  →  is-total  →  is-rete
;;
;; Each is measured STRICTLY and separately — *"verbosity is our shield"* (R63/R65: the same
;; exhaustiveness we pay for in keystrokes is what lets us change meaning later). They are NOT
;; collapsed even where one arguably implies another: `is-total` is not folded into `is-rete` on
;; the theory that every rete primitive is total by minting discipline, because
;;   (a) that discipline is a GATE, not a convention — `every_rete_row_is_total` is a red
;;       build on any `total: false` row; partial core ops enter only as `Fallback` +
;;       `:undefined`. The conjunct still fires on a user form whose body walks a partial
;;       core op that is not a rete twin; and
;;   (b) `total?` is a GENERAL language capability, not a rete detail. The `where` fence
;;       is its first real consumer (`compile-condition` consults it). Proving it here is
;;       what earns the right to lean on it elsewhere
;;       ([[300 ALIVS ARGVIT]] — the consumer is the crucible).
;; Builder: *"the is-total will have utility beyond rete — prove it works here so we have our
;; reliable toolkit for further language usage — this is not a one off."*
(:wat::core::defn :wat::rete::first-failing-axis
  [is-pure  <- :wat::core::bool
   is-det   <- :wat::core::bool
   is-total <- :wat::core::bool
   is-rete  <- :wat::core::bool]
  -> :wat::rete::Axis
  ;; `cond`, not a nested-`if` ladder — the chain reads top-to-bottom in the SAME order the
  ;; conjunction short-circuits, so the code and the law have one shape. (A nested `if` here would
  ;; also trip our own `lint_finds_the_nested_if_ladder`.) Builder: *"use cond over if chaining."*
  ;; Fourth argument is Law A: `:else` is RetePrimitive, never Total. A wrap that names
  ;; RetePrimitive outside this fn can drop and a core `i64::>` where is refused as Total.
  (:wat::core::cond
    ((:wat::core::not is-pure)  :wat::rete::Axis::Pure)
    ((:wat::core::not is-det)   :wat::rete::Axis::Deterministic)
    ((:wat::core::not is-total) :wat::rete::Axis::Total)
    (:else                      :wat::rete::Axis::RetePrimitive)))

;; axis-violation-message — build a human-actionable fence message from an ALREADY-DECIDED
;; rejection. `context` names the fenced site ("where" / "accumulator" / "then"); `failing-axis` is the axis
;; `first-failing-axis` picked. This function NEVER changes accept/reject (STOP-3) — the three call sites
;; (where, accumulator, then-item-fence) reach it only after `(and is-pure is-det is-total is-rete)` was already false — it only names
;; the failure that was already found. `:wat::core::Option/expect`'s message argument is evaluated
;; lazily (only on the None/rejected branch — `expect_panic` in runtime.rs), so this walk never
;; runs on an accepted expr even though it is wired as a plain argument expression.
;;
;; The `match` below is EXHAUSTIVE over `:wat::rete::Axis` — that is the payoff of the enum over a
;; free keyword: BRIEF-total-t1-the-axis-unarmed minted `:Total` and this match went non-exhaustive
;; until the arm below was added — the checker enumerated its own consumer instead of anyone
;; remembering by hand. `total?` is ARMED: `compile-condition` computes a `:Total` `failing-axis`
;; via `first-failing-axis` when the first two conjuncts hold and totality fails. The `:Total`
;; arm is live. The `:RetePrimitive` arm is live when the first three conjuncts hold.
(:wat::core::defn :wat::rete::axis-violation-message
  [context      <- :wat::core::String
   expr         <- :wat::WatAST
   failing-axis <- :wat::rete::Axis]
  -> :wat::core::String
  (:wat::core::match failing-axis
    (:wat::rete::Axis::Pure
     (:wat::core::match (:wat::rete::axis-violation expr :wat::rete::Axis::Pure)
       ((:wat::core::Some v)
        (:wat::string::concat "compile-condition: " context " expr is not pure — '"
                                     (:wat::rete::AxisViolation/head v) "' is not pure"))
       (:wat::core::None
        (:wat::core::format "compile-condition: {context} expr is not pure (offending head could not be attributed)" :context context))))
    (:wat::rete::Axis::Deterministic
     (:wat::core::match (:wat::rete::axis-violation expr :wat::rete::Axis::Deterministic)
       ((:wat::core::Some v)
        (:wat::string::concat "compile-condition: " context " expr is not deterministic — '"
                                     (:wat::rete::AxisViolation/head v) "' is not deterministic"))
       (:wat::core::None
        (:wat::core::format "compile-condition: {context} expr is not deterministic (offending head could not be attributed)" :context context))))
    (:wat::rete::Axis::Total
     (:wat::core::match (:wat::rete::axis-violation expr :wat::rete::Axis::Total)
       ((:wat::core::Some v)
        (:wat::string::concat "compile-condition: " context " expr is not total — '"
                                     (:wat::rete::AxisViolation/head v) "' is not total"))
       (:wat::core::None
        (:wat::core::format "compile-condition: {context} expr is not total (offending head could not be attributed)" :context context))))
    ;; #57 LAW A — the sentence the name was CHOSEN for. The three arms above read "is not pure" /
    ;; "is not deterministic" / "is not total"; this one reads "is not a rete primitive", which IS
    ;; the law ("the entire rete query language may only be composed from rete primitives") and
    ;; tells the author what to do without a lookup. The remedy is named explicitly because a
    ;; refusal that withholds the cure makes the reader hunt (R29 RVINA ERVDIT — the checker
    ;; educates); the rete twin of a core op is its name with `rete::` inserted after `wat::`.
    (:wat::rete::Axis::RetePrimitive
     (:wat::core::match (:wat::rete::axis-violation expr :wat::rete::Axis::RetePrimitive)
       ((:wat::core::Some v)
        (:wat::string::concat "compile-condition: " context " expr is not a rete primitive — '"
                                     (:wat::rete::AxisViolation/head v)
                                     "' is not a rete primitive; a " context " admits only :wat::rete:: ops"))
       (:wat::core::None
        (:wat::core::format "compile-condition: {context} expr is not a rete primitive (offending head could not be attributed)" :context context))))))

(:wat::core::defn :wat::rete::compile-condition
  [acc  <- :wat::rete::CondFoldAcc
   cond <- :wat::WatAST]
  -> :wat::rete::CondFoldAcc
  (:wat::core::let [state0    (:wat::rete::CondFoldAcc/state     acc)
                    parent-ids (:wat::rete::CondFoldAcc/parent-ids acc)
                    ;; TOP: detect (:wat::rete::where <expr>) form
                    ;; Keyword-headed wrappers (`:where`/`:not`/`:exists`/`:or`/`:and`)
                    ;; or a symbol-headed fact-bind / accumulate. Non-empty list.
                    cond-ch   (:wat::core::ast->children cond)
                    head      (:wat::core::first cond-ch)
                    head-nm        (:wat::core::ast-name head)
                    is-where       (:wat::core::= head-nm ":wat::rete::where")
                    is-not         (:wat::core::= head-nm ":wat::rete::not")
                    is-exists      (:wat::core::= head-nm ":wat::rete::exists")
                    is-or          (:wat::core::= head-nm ":wat::rete::or")
                    is-and         (:wat::core::= head-nm ":wat::rete::and")
                    ;; Accumulate: `?` head AND not fact-bind `(?p <- :ns::Type …)`.
                    ;; Fact-bind shares the `?` head; `:from` / a list after `<-` is accumulate.
                    is-accumulate  (:wat::rete::cond-is-accumulate cond)]
    (:wat::core::if is-or
      ;; Each arm is its own left chain from the SAME incoming parents.
      ;; Terminals of every arm become the outgoing parent-ids (Clara :or
      ;; of activations). Nested `:or` recurses through compile-condition.
      (:wat::core::let [or-ch (:wat::core::ast->children cond)
                        arms  (:wat::core::foldl
                                 (:wat::core::fn [acc <- (:wat::core::PersistentVector :- [:wat::WatAST])
                                                  i   <- :wat::core::i64]
                                   -> (:wat::core::PersistentVector :- [:wat::WatAST])
                                   (:wat::core::PersistentVector/conj acc
                                     (:wat::core::Option/expect
                                       (:wat::core::get or-ch i)
                                       "compile-condition: or arm")))
                                 (:wat::core::PersistentVector)
                                 (:wat::core::range 1 (:wat::core::length or-ch)))
                        _or-n (:wat::core::Option/expect
                                 (:wat::core::if (:wat::core::i64::> (:wat::core::length arms) 0)
                                   (:wat::core::Some nil)
                                   :wat::core::None)
                                 "compile-condition: or of conditions has no arms")
                        incoming parent-ids]
        (:wat::core::foldl
          (:wat::core::fn [fold-acc <- :wat::rete::CondFoldAcc
                           arm      <- :wat::WatAST]
            -> :wat::rete::CondFoldAcc
            (:wat::core::let [arm-acc (:wat::rete::compile-condition
                                        (:wat::rete::CondFoldAcc
                                          :state (:wat::rete::CondFoldAcc/state fold-acc)
                                          :parent-ids incoming)
                                        arm)]
              (:wat::rete::CondFoldAcc
                :state (:wat::rete::CondFoldAcc/state arm-acc)
                :parent-ids (:wat::core::PersistentVector/concat
                              (:wat::rete::CondFoldAcc/parent-ids fold-acc)
                              (:wat::rete::CondFoldAcc/parent-ids arm-acc)))))
          (:wat::rete::CondFoldAcc :state state0 :parent-ids (:wat::core::PersistentVector))
          arms))
    (:wat::core::if is-and
      ;; Sequential group (Clara `:and` inside `:or` / `:not`). Each child
      ;; sees the previous child's terminals — same as listing them in :when.
      (:wat::core::let [and-ch (:wat::core::ast->children cond)
                        kids   (:wat::core::foldl
                                 (:wat::core::fn [acc <- (:wat::core::PersistentVector :- [:wat::WatAST])
                                                  i   <- :wat::core::i64]
                                   -> (:wat::core::PersistentVector :- [:wat::WatAST])
                                   (:wat::core::PersistentVector/conj acc
                                     (:wat::core::Option/expect
                                       (:wat::core::get and-ch i)
                                       "compile-condition: and child")))
                                 (:wat::core::PersistentVector)
                                 (:wat::core::range 1 (:wat::core::length and-ch)))
                        _and-n (:wat::core::Option/expect
                                 (:wat::core::if (:wat::core::i64::> (:wat::core::length kids) 0)
                                   (:wat::core::Some nil)
                                   :wat::core::None)
                                 "compile-condition: and of conditions has no children")]
        (:wat::core::foldl :wat::rete::compile-condition acc kids))
    (:wat::core::if is-where
      ;; ── where branch (6b-ii-a) ──────────────────────────────────────────────
      (:wat::core::let [expr      (:wat::core::second cond-ch)
                        ;; ★ THE FULL CHAIN OF RESTRICTIONS — pure ∧ deterministic ∧ total ∧ rete.
                        ;; Each measured STRICTLY and separately; see `first-failing-axis` for why
                        ;; none is folded into another. Raise at compile if any fails. The message
                        ;; names the offending head + axis (BRIEF-the-fence-names-the-head); it is
                        ;; computed lazily by `Option/expect` (only on the None/reject branch), so
                        ;; the diagnostic walk never runs on an accepted expr.
                        is-pure   (:wat::rete::pure? expr)
                        is-det    (:wat::rete::deterministic? expr)
                        ;; TOTAL, ARMED — the first real consumer of `total?`, which shipped with
                        ;; T1 callable and unarmed. A partial op inside a `where` is the hazard the
                        ;; whole endeavour exists to remove: there is no jump-table opcode for
                        ;; "raises", so #49a's compiled executor cannot dispatch one.
                        is-total  (:wat::rete::total? expr)
                        ;; #57 LAW A, ARMED on this arm (`where` / accumulate / `:then`).
                        ;; Fact-pattern constraints: freeze wall (`validate.rs` NonReteConstraint)
                        ;; AND native `compile_condition_local` (arm-session / compile-all intern)
                        ;; refuse CoreGeneric — this fenced expr is the same axis.
                        is-rete   (:wat::rete::primitive? expr)
                        _fence    (:wat::core::Option/expect
                                      (:wat::core::if (:wat::core::and is-pure is-det is-total is-rete)
                                        (:wat::core::Some nil)
                                        :wat::core::None)
                                      (:wat::rete::axis-violation-message "where" expr
                                        ;; the axis is EXACT: first-failing-axis walks all four
                                        ;; conjuncts; `:else` is Law A (RetePrimitive), never Total.
                                        (:wat::rete::first-failing-axis is-pure is-det is-total is-rete)))
                        ;; #49 — lower at rule-compile. A form that cannot lower never
                        ;; enters the network. Fire only executes the circuit.
                        _lowered  (:wat::rete::lower expr)
                        ;; mint the TestNode
                        network0  (:wat::rete::CompileState/network state0)
                        next-id0  (:wat::rete::CompileState/next-id state0)
                        dedup0    (:wat::rete::CompileState/dedup   state0)
                        test-node (:wat::rete::TestNode :id next-id0 :expr expr :children (:wat::core::PersistentVector))
                        net1      (:wat::core::PersistentMap/assoc network0 next-id0 test-node)
                        state1    (:wat::rete::CompileState
                                     :network net1
                                     :next-id (:wat::core::i64::+ next-id0 1)
                                     :dedup dedup0)
                        ;; wire every parent → test (`:or` may leave N terminals)
                        net2      (:wat::rete::wire-parents
                                     (:wat::rete::CompileState/network state1)
                                     parent-ids
                                     next-id0)
                        state2    (:wat::rete::CompileState
                                     :network net2
                                     :next-id (:wat::rete::CompileState/next-id state1)
                                     :dedup (:wat::rete::CompileState/dedup   state1))]
        (:wat::rete::CondFoldAcc :state state2
          :parent-ids (:wat::core::PersistentVector/conj (:wat::core::PersistentVector) next-id0)))
      (:wat::core::if is-not
        ;; ── :not branch (7-a) ───────────────────────────────────────────────────
        ;; Leading :not is legal (Clara negated conjunction matches the empty world).
        ;; Empty parent-ids: filter seeds one empty-binding token.
        (:wat::core::let [;; Extract <inner> — the 2nd child of (:wat::rete::not <inner>)
                          inner       (:wat::core::second cond-ch)
                          ;; find-or-mint an AlphaNode for <inner> (so alpha pass populates it)
                          alpha-res   (:wat::rete::find-or-mint-alpha inner state0)
                          neg-alpha-id (:wat::rete::MintResult/id    alpha-res)
                          state1      (:wat::rete::mint-leaf-alphas inner
                                        (:wat::rete::MintResult/state alpha-res))
                          ;; mint the NegationNode
                          network1    (:wat::rete::CompileState/network state1)
                          next-id1    (:wat::rete::CompileState/next-id state1)
                          dedup1      (:wat::rete::CompileState/dedup   state1)
                          neg-node    (:wat::rete::NegationNode :id next-id1 :negated-alpha-id neg-alpha-id :children (:wat::core::PersistentVector))
                          net2        (:wat::core::PersistentMap/assoc network1 next-id1 neg-node)
                          state2      (:wat::rete::CompileState
                                         :network net2
                                         :next-id (:wat::core::i64::+ next-id1 1)
                                         :dedup dedup1)
                          net3        (:wat::rete::wire-parents
                                          (:wat::rete::CompileState/network state2)
                                          parent-ids
                                          next-id1)
                          state3      (:wat::rete::CompileState
                                         :network net3
                                         :next-id (:wat::rete::CompileState/next-id state2)
                                         :dedup (:wat::rete::CompileState/dedup   state2))]
          (:wat::rete::CondFoldAcc :state state3
            :parent-ids (:wat::core::PersistentVector/conj (:wat::core::PersistentVector) next-id1)))
        (:wat::core::if is-exists
          ;; ── :exists branch (7-exists) ────────────────────────────────────────────
          ;; Leading :exists is legal (Clara test-simple-exists). Empty parent-ids:
          ;; filter emits one token per distinct inner binding, not a dummy seed.
          (:wat::core::let [;; Extract <inner> — the 2nd child of (:wat::rete::exists <inner>)
                            inner        (:wat::core::second cond-ch)
                            ;; find-or-mint an AlphaNode for <inner> (so alpha pass populates it)
                            alpha-res    (:wat::rete::find-or-mint-alpha inner state0)
                            ex-alpha-id  (:wat::rete::MintResult/id    alpha-res)
                            state1       (:wat::rete::mint-leaf-alphas inner
                                           (:wat::rete::MintResult/state alpha-res))
                            ;; mint the ExistsNode
                            network1     (:wat::rete::CompileState/network state1)
                            next-id1     (:wat::rete::CompileState/next-id state1)
                            dedup1       (:wat::rete::CompileState/dedup   state1)
                            ex-node      (:wat::rete::ExistsNode :id next-id1 :exists-alpha-id ex-alpha-id :children (:wat::core::PersistentVector))
                            net2         (:wat::core::PersistentMap/assoc network1 next-id1 ex-node)
                            state2       (:wat::rete::CompileState
                                           :network net2
                                           :next-id (:wat::core::i64::+ next-id1 1)
                                           :dedup dedup1)
                            net3         (:wat::rete::wire-parents
                                            (:wat::rete::CompileState/network state2)
                                            parent-ids
                                            next-id1)
                            state3       (:wat::rete::CompileState
                                           :network net3
                                           :next-id (:wat::rete::CompileState/next-id state2)
                                           :dedup (:wat::rete::CompileState/dedup   state2))]
            (:wat::rete::CondFoldAcc :state state3
              :parent-ids (:wat::core::PersistentVector/conj (:wat::core::PersistentVector) next-id1)))
        (:wat::core::if is-accumulate
          ;; ── accumulate branch (8-a) ─────────────────────────────────────────────
          ;; Form: (?result-var <- (<acc-form>) :from (<inner>))
          ;; children: [?result-var, <-, acc-form, :from, inner]
          ;; Leading accumulate is legal (Clara test-count: empty world → count 0).
          ;; Empty parent-ids: accumulate-pass seeds one empty-binding token.
          (:wat::core::let [;; result-var: stored WITH the `?` prefix (`head-nm` as bound).
                            result-var   head-nm
                            ;; acc-form: items[2]
                            acc-form     (:wat::core::Option/expect  
                                             (:wat::core::get cond-ch 2)
                                             "compile-condition: accumulate missing acc-form")
                            ;; 8-custom FENCE: the acc-form head selects the fold. A built-in
                            ;; head (:wat::rete::acc::*) is trusted (skip the fence wholesale).
                            ;; Any other head is a USER fold fn → the same four-axis check
                            ;; `where` / `:then` use: pure ∧ det ∧ total ∧ rete. Build a
                            ;; synthetic call `(<acc-hd> __acc__)` and run the four predicates
                            ;; on it — head_ok classifies the user fn transitively
                            ;; (purity.rs:classify_fn).
                            acc-ch       (:wat::core::ast->children acc-form)
                            acc-hd       (:wat::core::first acc-ch)
                            acc-hd-nm    (:wat::core::ast-name acc-hd)
                            is-builtin   (:wat::string::starts-with? acc-hd-nm ":wat::rete::acc::")
                            fence-call   (:wat::core::quasiquote
                                            ((:wat::core::unquote acc-hd) __acc__))
                            ;; fence: pure ∧ det ∧ total ∧ rete (skipped for :wat::rete::acc::*
                            ;; builtins, which are trusted). The four predicates are computed
                            ;; unconditionally — same
                            ;; shape as the `where` fence above — which is safe even for a builtin
                            ;; acc-hd: pure?/deterministic? default-deny gracefully on an unrecognized
                            ;; head, never panic. The message names the offending head + axis
                            ;; (BRIEF-the-fence-names-the-head, "accumulator" not "where"); it is
                            ;; computed lazily by `Option/expect` (only on the None/reject branch).
                            is-pure      (:wat::rete::pure? fence-call)
                            is-det       (:wat::rete::deterministic? fence-call)
                            ;; TOTAL, ARMED — the accumulator fence is the `where` fence's sibling
                            ;; (the stone scopes the vocabulary to "a `where` (and the accumulator
                            ;; fence)"), so it measures the same chain.
                            is-total     (:wat::rete::total? fence-call)
                            ;; #83 LAW A, ARMED HERE TOO — the fourth conjunct, closing the gap that
                            ;; left this fence at three where `where` and `:then` had four. The prior
                            ;; revision of this comment said "`is-rete` is NOT added here … widening it
                            ;; to this surface is its own strike"; that was a deferral written AGAINST
                            ;; the stone, which scopes the vocabulary to "a `where` (AND THE ACCUMULATOR
                            ;; FENCE)". This IS that strike.
                            ;;
                            ;; Note the `is-builtin` short-circuit below exempts a `:wat::rete::acc::*`
                            ;; head WHOLESALE (all four conjuncts), so the population law A newly reaches
                            ;; here is exactly the USER fold fn — which stays admissible transitively
                            ;; whenever its body bottoms out in rete primitives (the composition door,
                            ;; purity.rs:classify_fn). What it now refuses is a core-spelled fold.
                            is-rete      (:wat::rete::primitive? fence-call)
                            _acc-fence   (:wat::core::Option/expect
                                             (:wat::core::if is-builtin
                                               (:wat::core::Some nil)
                                               (:wat::core::if (:wat::core::and is-pure is-det is-total is-rete)
                                                 (:wat::core::Some nil)
                                                 :wat::core::None))
                                             (:wat::rete::axis-violation-message "accumulator" fence-call
                                               ;; the axis is EXACT: first-failing-axis walks all four
                                               ;; conjuncts; `:else` is Law A (RetePrimitive), never Total.
                                               (:wat::rete::first-failing-axis is-pure is-det is-total is-rete)))
                            ;; assert items[3] is :from (structural validation)
                            from-kw      (:wat::core::Option/expect  
                                             (:wat::core::get cond-ch 3)
                                             "compile-condition: accumulate missing :from")
                            _from-check  (:wat::core::Option/expect  
                                             (:wat::core::if (:wat::core::= (:wat::core::ast-name from-kw) ":from")
                                               (:wat::core::Some nil)
                                               :wat::core::None)
                                             "compile-condition: accumulate expected :from at position 3")
                            ;; inner: items[4] — the :from fact-pattern condition
                            inner        (:wat::core::Option/expect  
                                             (:wat::core::get cond-ch 4)
                                             "compile-condition: accumulate missing :from inner condition")
                            ;; find-or-mint an AlphaNode for the :from inner condition
                            alpha-res    (:wat::rete::find-or-mint-alpha inner state0)
                            from-alpha-id (:wat::rete::MintResult/id    alpha-res)
                            state1       (:wat::rete::MintResult/state alpha-res)
                            ;; mint the AccumulateNode
                            network1     (:wat::rete::CompileState/network state1)
                            next-id1     (:wat::rete::CompileState/next-id state1)
                            dedup1       (:wat::rete::CompileState/dedup   state1)
                            acc-node     (:wat::rete::AccumulateNode
                                             :id next-id1
                                             :result-var result-var
                                             :acc-form acc-form
                                             :from-alpha-id from-alpha-id
                                             :children (:wat::core::PersistentVector))
                            net2         (:wat::core::PersistentMap/assoc network1 next-id1 acc-node)
                            state2       (:wat::rete::CompileState
                                            :network net2
                                            :next-id (:wat::core::i64::+ next-id1 1)
                                            :dedup dedup1)
                            net3         (:wat::rete::wire-parents
                                             (:wat::rete::CompileState/network state2)
                                             parent-ids
                                             next-id1)
                            state3       (:wat::rete::CompileState
                                            :network net3
                                            :next-id (:wat::rete::CompileState/next-id state2)
                                            :dedup (:wat::rete::CompileState/dedup   state2))]
            (:wat::rete::CondFoldAcc :state state3
              :parent-ids (:wat::core::PersistentVector/conj (:wat::core::PersistentVector) next-id1)))
          ;; ── alpha+join: first condition → RootJoin; later → one HashJoin per parent
          (:wat::core::let [alpha-res  (:wat::rete::find-or-mint-alpha cond state0)
                        alpha-id   (:wat::rete::MintResult/id    alpha-res)
                        state1     (:wat::rete::MintResult/state alpha-res)
                        is-first   (:wat::core::= (:wat::core::length parent-ids) 0)]
            (:wat::core::if is-first
              (:wat::core::let [join-res (:wat::rete::find-or-mint-root-join cond state1)
                                join-id  (:wat::rete::MintResult/id    join-res)
                                state2   (:wat::rete::MintResult/state join-res)
                                net3     (:wat::rete::network-add-child
                                           (:wat::rete::CompileState/network state2)
                                           alpha-id
                                           join-id)
                                state3   (:wat::rete::CompileState
                                           :network net3
                                           :next-id (:wat::rete::CompileState/next-id state2)
                                           :dedup (:wat::rete::CompileState/dedup state2))]
                (:wat::rete::CondFoldAcc :state state3
                  :parent-ids (:wat::core::PersistentVector/conj (:wat::core::PersistentVector) join-id)))
              (:wat::core::let [fan (:wat::core::foldl
                                      (:wat::core::fn [acc <- :wat::rete::CondFoldAcc
                                                       pid <- :wat::core::i64]
                                        -> :wat::rete::CondFoldAcc
                                        (:wat::core::let [st0 (:wat::rete::CondFoldAcc/state acc)
                                                          jr  (:wat::rete::find-or-mint-hash-join cond pid st0)
                                                          jid (:wat::rete::MintResult/id jr)
                                                          st1 (:wat::rete::MintResult/state jr)
                                                          n1  (:wat::rete::network-add-child
                                                                (:wat::rete::CompileState/network st1)
                                                                alpha-id
                                                                jid)
                                                          n2  (:wat::rete::network-add-child n1 pid jid)
                                                          st2 (:wat::rete::CompileState
                                                                :network n2
                                                                :next-id (:wat::rete::CompileState/next-id st1)
                                                                :dedup (:wat::rete::CompileState/dedup st1))]
                                          (:wat::rete::CondFoldAcc :state st2
                                            :parent-ids (:wat::core::PersistentVector/conj
                                                          (:wat::rete::CondFoldAcc/parent-ids acc)
                                                          jid))))
                                      (:wat::rete::CondFoldAcc :state state1
                                        :parent-ids (:wat::core::PersistentVector))
                                      parent-ids)]
                fan)))))))))))

;; then-item-fence — Stone B (arc 278 DESIGN-STONE-then-is-a-vector-of-singular-facts.md §
;; "Stone B"): the RHS fence, mirroring `compile-condition`'s `where` fence (above,
;; :wat::rete::compile-condition's `is-where` branch) and the accumulator fence's synthetic-call
;; trick (the `is-accumulate` branch) — except a `:then` item is ALREADY a call form
;; `(<head> arg…)`, so no synthesis is needed: `pure?`/`deterministic?` on the item itself walks
;; the head (constructor_meta / sym.functions, `purity.rs::head_ok`) AND recurses into every
;; operand. ONE check covers BOTH widenings named in `BRIEF-then-user-forms.md`: (a) the item
;; head may be a fn (not only a fact-type constructor — `head_ok` dispatches on either via the
;; SAME declaration-derived / sym.functions doors already used for `where`) and (b) an operand
;; may be a composed expression, not only `?var`/`:field`/literal (classify_expr's generic
;; list-call arm already recurses into every argument on the same axis).
;;
;; SECOND check, new to `:then` (not shared with `where`, which never claims to produce
;; anything): the item must RETURN A FACT. Evaluate the head to its fn value — `eval-ast!`, since
;; `item-ch`'s head is already a `:wat::WatAST` value — then read its declared return type
;; (`return-type-of`) and confirm that type is a registered record/struct
;; (`field-names-of` raises otherwise — the SAME registry `validate_then_form`'s
;; `lookup_fields` reads on the Rust side, reached here in wat because that Rust validator
;; carries `types` but not `sym`, per `BRIEF-then-user-forms.md`'s "the fence goes where the
;; where fence already is").
;;
;; A bare record-type keyword (e.g. `:usr::Rate`) evaluates to a KEYWORD, not its constructor fn
;; (arc 294 item 9a's construction flip — `runtime.rs::eval_return_type_of`'s own doc) — resolve
;; through the PRIME `:T'` in that case, the :then constructor door only (`return-type-of`).
;; `query` is a defmacro over `query-read`; q is a Query, no type-keyword, no T'.
;; A plain `:wat::core::defn` has no such indirection and already resolved to
;; a fn on the bare read.
;;
;; foldl-compatible: `(acc, item) -> acc`, so `compile-rule` folds this straight over `rhs`
;; without a lambda wrapper — the accumulator is a throwaway `i64`, unused except to satisfy
;; foldl's shape; every check here is a side-effecting raise (an axis violation panics via
;; `Option/expect`, exactly like `where`'s fence; "does not return a fact" raises normally,
;; via `field-names-of`'s own diagnostic — both are freeze-time-only, never per derived fact).
(:wat::core::defn :wat::rete::then-item-fence
  [acc  <- :wat::core::i64
   item <- :wat::WatAST]
  -> :wat::core::i64
  (:wat::core::let [is-pure   (:wat::rete::pure? item)
                    is-det    (:wat::rete::deterministic? item)
                    ;; TOTAL, ARMED. A `:then` item that can raise aborts the fire mid-derivation,
                    ;; which is the same hazard by a different door.
                    is-total  (:wat::rete::total? item)
                    ;; ★ LAW A, ARMED ON THE RHS TOO (2026-08-05, the builder's call after the
                    ;; measurement below). The `where` fence got `is-rete` first and this one was
                    ;; deliberately left at three conjuncts — which MEASURED as a real gap:
                    ;;
                    ;;     :then item using (:wat::core::i64::+ …)                      -> refused (not total)
                    ;;     :then item using (:wat::core::if (:wat::core::i64::> n 5) …) -> COMPILED
                    ;;
                    ;; A core-spelled TOTAL op sailed straight through. The ruling is general —
                    ;; *"rete forms must only be composed of rete forms and primitives"* — and the
                    ;; RHS is not exempt: `compiled_rhs.rs` ALREADY EXISTS, so the RHS is already a
                    ;; compiled surface and wants the same closed head-space `where` does. There is
                    ;; no opcode for a head the vocabulary does not list.
                    ;;
                    ;; This does NOT narrow what a `:then` may CONSTRUCT: a user record/fact
                    ;; constructor is admitted by the declaration-derived constructor door, which
                    ;; law A never consults a namespace for (`head_ok`'s first door). What it
                    ;; refuses is core-spelled COMPUTATION inside the item.
                    is-rete   (:wat::rete::primitive? item)
                    _fence    (:wat::core::Option/expect
                                  (:wat::core::if (:wat::core::and is-pure is-det is-total is-rete)
                                    (:wat::core::Some nil)
                                    :wat::core::None)
                                  (:wat::rete::axis-violation-message "then" item
                                    ;; exact: first-failing-axis walks all four conjuncts;
                                    ;; `:else` is Law A (RetePrimitive), never Total.
                                    (:wat::rete::first-failing-axis is-pure is-det is-total is-rete)))
                    item-ch   (:wat::core::ast->children item)
                    head      (:wat::core::first item-ch)
                    head-val0 (:wat::core::Result/expect
                                  (:wat::eval-ast! head)
                                  "compile-rule: :then item head failed to evaluate")
                    ;; `:wat::core::type` returns the COLON-FREE FQDN (`Value::declared_type_name`)
                    ;; — compare against "wat::core::fn", not ":wat::core::fn".
                    is-fn-val (:wat::core::= (:wat::core::type head-val0) "wat::core::fn")
                    ;; A bare record-type keyword evaluates to a KEYWORD — re-resolve through its
                    ;; PRIME `:T'` to reach the constructor fn (see this defn's doc). A plain
                    ;; `defn` already resolved to a fn above and takes the `is-fn-val` branch.
                    prime-kw  (:wat::core::keyword-node
                                  (:wat::string::concat (:wat::core::ast-name head) "'"))
                    head-fn   (:wat::core::if is-fn-val
                                  head-val0
                                  (:wat::core::Result/expect
                                     (:wat::eval-ast! prime-kw)
                                     "compile-rule: :then item head failed to resolve to a fn"))
                    ;; return-type-of raises "unknown type" itself if head-fn is STILL a bare
                    ;; keyword (a genuinely unrecognised head) — no separate check needed here.
                    ret-ty    (:wat::runtime::return-type-of head-fn)
                    ;; `keyword/from-string` wants COLON-FREE input (it adds the sigil itself);
                    ;; `return-type-of` already returns a colon-free FQDN — do not re-prepend one.
                    ret-kw    (:wat::core::keyword/from-string ret-ty)
                    ;; raises unless ret-kw names a registered record/struct — "produces a fact."
                    _fact-ty  (:wat::runtime::field-names-of ret-kw)]
    acc))

;; ast-qvars — every `?var` symbol under a condition AST (binds and uses).
(:wat::core::defn :wat::rete::ast-qvars
  [ast <- :wat::WatAST]
  -> (:wat::core::PersistentVector :- [:wat::core::String])
  (:wat::core::let [k (:wat::core::ast-kind ast)]
    (:wat::core::if (:wat::core::= k "symbol")
      (:wat::core::let [nm (:wat::core::ast-name ast)]
        (:wat::core::if (:wat::string::starts-with? nm "?")
          (:wat::core::PersistentVector/conj (:wat::core::PersistentVector) nm)
          (:wat::core::PersistentVector)))
      (:wat::core::if
        (:wat::core::if (:wat::core::= k "list")
          true
          (:wat::core::= k "vector"))
        (:wat::core::let [ch (:wat::core::ast->children ast)
                          n  (:wat::core::length ch)]
          (:wat::core::foldl
            (:wat::core::fn [acc <- (:wat::core::PersistentVector :- [:wat::core::String])
                             i   <- :wat::core::i64]
              -> (:wat::core::PersistentVector :- [:wat::core::String])
              (:wat::core::let [kid (:wat::core::Option/expect
                                      (:wat::core::get ch i)
                                      "ast-qvars")]
                (:wat::core::foldl
                  (:wat::core::fn [out <- (:wat::core::PersistentVector :- [:wat::core::String])
                                   nm  <- :wat::core::String]
                    -> (:wat::core::PersistentVector :- [:wat::core::String])
                    (:wat::core::if (:wat::core::PersistentVector/contains? out nm)
                      out
                      (:wat::core::PersistentVector/conj out nm)))
                  acc
                  (:wat::rete::ast-qvars kid))))
            (:wat::core::PersistentVector)
            (:wat::core::range 0 n)))
        (:wat::core::PersistentVector)))))

;; cond-bind-keys — `?var` names this condition BINDS (`(?v <- :field)`,
;; fact-bind `(?p <- :ns::Type …)`, accum result, `:from` inner, `:exists`
;; inner). `:not` / `:where` bind nothing.
(:wat::core::defn :wat::rete::cond-bind-keys
  [cond <- :wat::WatAST]
  -> (:wat::core::PersistentVector :- [:wat::core::String])
  (:wat::core::if (:wat::core::not (:wat::core::= (:wat::core::ast-kind cond) "list"))
    (:wat::core::PersistentVector)
    (:wat::core::let [ch (:wat::core::ast->children cond)
                      n  (:wat::core::length ch)]
      (:wat::core::if (:wat::core::= n 0)
        (:wat::core::PersistentVector)
        (:wat::core::let [head   (:wat::core::first ch)
                          head-k (:wat::core::ast-kind head)]
          (:wat::core::if (:wat::core::= head-k "symbol")
            (:wat::core::let [hnm (:wat::core::ast-name head)]
              (:wat::core::if (:wat::rete::cond-is-fact-bind cond)
                (:wat::core::foldl
                  (:wat::core::fn [acc <- (:wat::core::PersistentVector :- [:wat::core::String])
                                   i   <- :wat::core::i64]
                    -> (:wat::core::PersistentVector :- [:wat::core::String])
                    (:wat::core::let [kid (:wat::core::Option/expect
                                            (:wat::core::get ch i)
                                            "cond-bind-keys: fact-bind clause")]
                      (:wat::core::foldl
                        (:wat::core::fn [out <- (:wat::core::PersistentVector :- [:wat::core::String])
                                         nm  <- :wat::core::String]
                          -> (:wat::core::PersistentVector :- [:wat::core::String])
                          (:wat::core::if (:wat::core::PersistentVector/contains? out nm)
                            out
                            (:wat::core::PersistentVector/conj out nm)))
                        acc
                        (:wat::rete::cond-bind-keys kid))))
                  (:wat::core::PersistentVector/conj (:wat::core::PersistentVector) hnm)
                  (:wat::core::range 3 n))
                (:wat::core::if
                  (:wat::core::if (:wat::string::starts-with? hnm "?")
                    (:wat::core::if (:wat::core::= n 3)
                      (:wat::core::if (:wat::core::= (:wat::core::ast-kind
                                                      (:wat::core::Option/expect
                                                        (:wat::core::get ch 1)
                                                        "cond-bind-keys: bind arrow"))
                                                    "symbol")
                        (:wat::core::= (:wat::core::ast-name
                                        (:wat::core::Option/expect
                                          (:wat::core::get ch 1)
                                          "cond-bind-keys: bind arrow"))
                                      "<-")
                        false)
                      false)
                    false)
                  (:wat::core::PersistentVector/conj (:wat::core::PersistentVector) hnm)
                  (:wat::core::if
                    (:wat::core::if (:wat::string::starts-with? hnm "?")
                      (:wat::core::if (:wat::core::= n 5)
                        (:wat::core::= (:wat::core::ast-name
                                        (:wat::core::Option/expect
                                          (:wat::core::get ch 3)
                                          "cond-bind-keys: :from"))
                                      ":from")
                        false)
                      false)
                    (:wat::core::foldl
                      (:wat::core::fn [acc <- (:wat::core::PersistentVector :- [:wat::core::String])
                                       nm  <- :wat::core::String]
                        -> (:wat::core::PersistentVector :- [:wat::core::String])
                        (:wat::core::if (:wat::core::PersistentVector/contains? acc nm)
                          acc
                          (:wat::core::PersistentVector/conj acc nm)))
                      (:wat::core::PersistentVector/conj (:wat::core::PersistentVector) hnm)
                      (:wat::rete::cond-bind-keys
                        (:wat::core::Option/expect
                          (:wat::core::get ch 4)
                          "cond-bind-keys: :from inner")))
                    (:wat::core::PersistentVector)))))
            (:wat::core::if (:wat::core::= head-k "keyword")
              (:wat::core::let [hnm (:wat::core::ast-name head)]
                (:wat::core::cond
                  ((:wat::core::= hnm ":wat::rete::not")
                   (:wat::core::PersistentVector))
                  ((:wat::core::= hnm ":wat::rete::where")
                   (:wat::core::PersistentVector))
                  ((:wat::core::= hnm ":wat::rete::exists")
                   (:wat::rete::cond-bind-keys
                     (:wat::core::second ch)))
                  (:else
                   (:wat::core::foldl
                     (:wat::core::fn [acc <- (:wat::core::PersistentVector :- [:wat::core::String])
                                      i   <- :wat::core::i64]
                       -> (:wat::core::PersistentVector :- [:wat::core::String])
                       (:wat::core::let [kid (:wat::core::Option/expect
                                               (:wat::core::get ch i)
                                               "cond-bind-keys: child")]
                         (:wat::core::foldl
                           (:wat::core::fn [out <- (:wat::core::PersistentVector :- [:wat::core::String])
                                            nm  <- :wat::core::String]
                             -> (:wat::core::PersistentVector :- [:wat::core::String])
                             (:wat::core::if (:wat::core::PersistentVector/contains? out nm)
                               out
                               (:wat::core::PersistentVector/conj out nm)))
                           acc
                           (:wat::rete::cond-bind-keys kid))))
                     (:wat::core::PersistentVector)
                     (:wat::core::range 1 n)))))
              (:wat::core::PersistentVector))))))))

;; cond-is-fact-bind — `(?p <- :ns::Type …)` (Clara `[?p <- Type]`). Type keyword has `::`.
(:wat::core::defn :wat::rete::cond-is-fact-bind
  [cond <- :wat::WatAST]
  -> :wat::core::bool
  (:wat::core::if (:wat::core::not (:wat::core::= (:wat::core::ast-kind cond) "list"))
    false
    (:wat::core::let [ch (:wat::core::ast->children cond)]
      (:wat::core::if (:wat::core::< (:wat::core::length ch) 3)
        false
        (:wat::core::if
          (:wat::core::if (:wat::core::= (:wat::core::ast-kind (:wat::core::first ch)) "symbol")
            (:wat::string::starts-with? (:wat::core::ast-name (:wat::core::first ch)) "?")
            false)
          (:wat::core::if
            (:wat::core::if (:wat::core::= (:wat::core::ast-kind
                                            (:wat::core::Option/expect
                                              (:wat::core::get ch 1)
                                              "cond-is-fact-bind: arrow"))
                                          "symbol")
              (:wat::core::= (:wat::core::ast-name
                               (:wat::core::Option/expect
                                 (:wat::core::get ch 1)
                                 "cond-is-fact-bind: arrow"))
                             "<-")
              false)
            (:wat::core::if (:wat::core::= (:wat::core::ast-kind
                                            (:wat::core::Option/expect
                                              (:wat::core::get ch 2)
                                              "cond-is-fact-bind: type"))
                                          "keyword")
              (:wat::string::contains?
                (:wat::core::ast-name
                  (:wat::core::Option/expect
                    (:wat::core::get ch 2)
                    "cond-is-fact-bind: type"))
                "::")
              false)
            false)
          false)))))

;; cond-is-accumulate — `?result` head that is NOT a fact-bind.
(:wat::core::defn :wat::rete::cond-is-accumulate
  [cond <- :wat::WatAST]
  -> :wat::core::bool
  (:wat::core::if (:wat::rete::cond-is-fact-bind cond)
    false
    (:wat::core::if (:wat::core::not (:wat::core::= (:wat::core::ast-kind cond) "list"))
      false
      (:wat::core::let [ch (:wat::core::ast->children cond)]
        (:wat::core::if (:wat::core::= (:wat::core::length ch) 0)
          false
          (:wat::core::if (:wat::core::= (:wat::core::ast-kind (:wat::core::first ch)) "symbol")
            (:wat::string::starts-with? (:wat::core::ast-name (:wat::core::first ch)) "?")
            false))))))

;; sort-lhs — Clara defers accumulators so a later fact can bind the group
;; key (test-count-none-joined: Wind first, then count Temps at ?loc → 0).
;; Non-accums that mention an accum result-var stay after the accum (`:where`
;; on ?c). Relative order inside each partition is preserved.
(:wat::core::defn :wat::rete::sort-lhs
  [lhs <- (:wat::core::PersistentVector :- [:wat::WatAST])]
  -> (:wat::core::PersistentVector :- [:wat::WatAST])
  (:wat::core::let [result-vars
                    (:wat::core::foldl
                      (:wat::core::fn [acc  <- (:wat::core::PersistentVector :- [:wat::core::String])
                                       cond <- :wat::WatAST]
                        -> (:wat::core::PersistentVector :- [:wat::core::String])
                        (:wat::core::if (:wat::rete::cond-is-accumulate cond)
                          (:wat::core::let [ch (:wat::core::ast->children cond)]
                            (:wat::core::PersistentVector/conj
                              acc
                              (:wat::core::ast-name (:wat::core::first ch))))
                          acc))
                      (:wat::core::PersistentVector)
                      lhs)
                    uses-result?
                    (:wat::core::fn [cond <- :wat::WatAST] -> :wat::core::bool
                      (:wat::core::let [qs (:wat::rete::ast-qvars cond)]
                        (:wat::core::foldl
                          (:wat::core::fn [hit <- :wat::core::bool
                                           rv  <- :wat::core::String]
                            -> :wat::core::bool
                            (:wat::core::if hit
                              true
                              (:wat::core::PersistentVector/contains? qs rv)))
                          false
                          result-vars)))
                    independent
                    (:wat::core::foldl
                      (:wat::core::fn [acc  <- (:wat::core::PersistentVector :- [:wat::WatAST])
                                       cond <- :wat::WatAST]
                        -> (:wat::core::PersistentVector :- [:wat::WatAST])
                        (:wat::core::if
                          (:wat::core::if (:wat::rete::cond-is-accumulate cond)
                            false
                            (:wat::core::not (uses-result? cond)))
                          (:wat::core::PersistentVector/conj acc cond)
                          acc))
                      (:wat::core::PersistentVector)
                      lhs)
                    accums
                    (:wat::core::foldl
                      (:wat::core::fn [acc  <- (:wat::core::PersistentVector :- [:wat::WatAST])
                                       cond <- :wat::WatAST]
                        -> (:wat::core::PersistentVector :- [:wat::WatAST])
                        (:wat::core::if (:wat::rete::cond-is-accumulate cond)
                          (:wat::core::PersistentVector/conj acc cond)
                          acc))
                      (:wat::core::PersistentVector)
                      lhs)
                    rest
                    (:wat::core::foldl
                      (:wat::core::fn [acc  <- (:wat::core::PersistentVector :- [:wat::WatAST])
                                       cond <- :wat::WatAST]
                        -> (:wat::core::PersistentVector :- [:wat::WatAST])
                        (:wat::core::if
                          (:wat::core::if (:wat::rete::cond-is-accumulate cond)
                            false
                            (uses-result? cond))
                          (:wat::core::PersistentVector/conj acc cond)
                          acc))
                      (:wat::core::PersistentVector)
                      lhs)]
    (:wat::core::PersistentVector/concat
      independent
      (:wat::core::PersistentVector/concat accums rest))))

;; compile-rule — fold step: process one Rule into the network.
;; WHY: folds over the rule's lhs conditions with compile-condition, then mints
;; the ProductionNode as a child of every remaining parent (one join after a
;; linear :when; N arm terminals after a condition `:or`).
;;
;; Arc 278 Stone B — fences `rhs` (the rule's `:then` items) via `then-item-fence` BEFORE folding
;; `lhs`, so a malformed RHS is caught before any network nodes are minted for this rule. Mirrors
;; how `where`/accumulate are fenced inline during the LHS fold — this is the RHS's own
;; freeze-time-only pass, over the SAME `rule` this fn already receives.
(:wat::core::defn :wat::rete::compile-rule
  [state <- :wat::rete::CompileState
   rule  <- :wat::rete::Rule]
  -> :wat::rete::CompileState
  (:wat::core::let [lhs        (:wat::rete::Rule/lhs rule)
                    rhs        (:wat::rete::Rule/rhs rule)
                    rname      (:wat::rete::Rule/name rule)
                    _rhs-fence (:wat::core::foldl :wat::rete::then-item-fence 0 rhs)
                    lhs-sorted (:wat::rete::sort-lhs lhs)
                    init-acc   (:wat::rete::CondFoldAcc
                                 :state state
                                 :parent-ids (:wat::core::PersistentVector))
                    final-acc  (:wat::core::foldl :wat::rete::compile-condition init-acc lhs-sorted)
                    state2     (:wat::rete::CondFoldAcc/state      final-acc)
                    pids       (:wat::rete::CondFoldAcc/parent-ids final-acc)
                    network2   (:wat::rete::CompileState/network state2)
                    next-id2   (:wat::rete::CompileState/next-id state2)
                    prod       (:wat::rete::ProductionNode :id next-id2 :rule-name rname)
                    net3       (:wat::core::PersistentMap/assoc network2 next-id2 prod)
                    net4       (:wat::rete::wire-parents net3 pids next-id2)]
    (:wat::rete::CompileState :network net4 :next-id (:wat::core::i64::+ next-id2 1)
      :dedup (:wat::rete::CompileState/dedup state2))))

;; compile-query — same LHS fold as compile-rule; terminal is a QueryNode.
(:wat::core::defn :wat::rete::compile-query
  [state <- :wat::rete::CompileState
   q     <- :wat::rete::Query]
  -> :wat::rete::CompileState
  (:wat::core::let [lhs        (:wat::rete::sort-lhs (:wat::rete::Query/lhs q))
                    qname      (:wat::rete::Query/name q)
                    init-acc   (:wat::rete::CondFoldAcc
                                 :state state
                                 :parent-ids (:wat::core::PersistentVector))
                    final-acc  (:wat::core::foldl :wat::rete::compile-condition init-acc lhs)
                    state2     (:wat::rete::CondFoldAcc/state      final-acc)
                    pids       (:wat::rete::CondFoldAcc/parent-ids final-acc)
                    network2   (:wat::rete::CompileState/network state2)
                    next-id2   (:wat::rete::CompileState/next-id state2)
                    qnode      (:wat::rete::QueryNode
                                 :id next-id2
                                 :query-name qname
                                 :param-keys (:wat::rete::Query/params q))
                    net3       (:wat::core::PersistentMap/assoc network2 next-id2 qnode)
                    net4       (:wat::rete::wire-parents net3 pids next-id2)]
    (:wat::rete::CompileState :network net4 :next-id (:wat::core::i64::+ next-id2 1)
      :dedup (:wat::rete::CompileState/dedup state2))))

;; compile — rules only (existing callers). Use compile-all to add queries.
(:wat::core::defn :wat::rete::compile
  [rules <- (:wat::core::PersistentVector :- [:wat::rete::Rule])]
  -> :wat::rete::Session
  (:wat::rete::compile-all rules (:wat::core::PersistentVector)))

;; compile-all — rules + queries (Clara mk-session mixes both).
(:wat::core::defn :wat::rete::compile-all
  [rules   <- (:wat::core::PersistentVector :- [:wat::rete::Rule])
   queries <- (:wat::core::PersistentVector :- [:wat::rete::Query])]
  -> :wat::rete::Session
  (:wat::core::let [init-state (:wat::rete::CompileState
                                  :network (:wat::core::PersistentMap)
                                  :next-id 0
                                  :dedup (:wat::core::HashMap :wat::core::String :wat::core::i64))
                    after-rules (:wat::core::foldl :wat::rete::compile-rule init-state rules)
                    final-state (:wat::core::foldl :wat::rete::compile-query after-rules queries)
                    network  (:wat::rete::CompileState/network final-state)
                    next-id  (:wat::rete::CompileState/next-id final-state)
                    empty-pm (:wat::core::PersistentMap)
                    empty-pv (:wat::core::PersistentVector)]
    ;; Intern the rust InternedNetwork under the network identity so first fire-rules HIT
    ;; (`DESIGN-STONE-arm-at-compile`). Session bytes unchanged.
    (:wat::rete::arm-session
      (:wat::rete::Session
         :network network
         :rules rules
         :alpha-memory empty-pm
         :beta-memory empty-pm
         :production-memory empty-pm
         :facts empty-pv
         :next-id next-id
         :query-memory empty-pm))))
