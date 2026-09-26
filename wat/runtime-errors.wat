;; wat/runtime-errors.wat — excursus 003 envelope step 3a: every `RuntimeErrorKind`
;; variant becomes a declared `:wat::runtime::<Kind>` record. Ruling 2026-09-26
;; (option A), builder verbatim: "Define the new records in wat code and use the
;; existing tooling to macro rust code from them."
;;
;; Each record opens with the `:wat::core::Error` floor — `message location causes`
;; (`wat/core.wat:2169`'s `:wat::core::Error` defsurface) — so it structurally
;; satisfies `:wat::core::Error`, followed by the kind's own fields, named exactly
;; as today's EDN keys (kebab, honouring `#[to_edn(key = …)]`). `message` is the
;; span-free headline (`WatError::error_edn`'s `first_line(kind.to_string())`),
;; `location` is the raising site's `:wat::core::Span`, `causes` is empty unless
;; the kind wraps a nested error (see EvalVerificationFailed / MacroExpansionFailed
;; below).
;;
;; This step ships NO envelope change — `RuntimeErrorKind`'s `#[derive(ToEdn)]`
;; still writes the wire until step 3b. These records exist so 3b has a wat VALUE
;; to put a runtime error's structure into; `RuntimeError::to_record` (src/value/
;; runtime_records.rs) is the only consumer today.
;;
;; Loads immediately after `wat/kernel/diagnostics.wat` (needs `:wat::kernel::
;; ClauseAttempt`, declared there per the builder's namespace ruling — it already
;; ships as `#wat.kernel/…`) and after `wat/core.wat` (`:wat::core::Error` /
;; `:wat::core::Span`). See `src/load/stdlib.rs`.

;; ─── :wat::runtime::Provenance — where a snapshotted value came from ─────────
;;
;; Mirrors `crate::value::observe::Provenance` (`src/value/observe.rs:25`, NOT
;; `src/edn/error.rs` — that file only holds the serializer, `provenance_to_edn`).
;; `Provenance::Unknown` (the default — no provenance attached) carries no data,
;; so it is not a variant here: `:wat::runtime::ValueSnapshot/provenance` is an
;; `(Option :- [Provenance])`, and `Unknown` is `:None`.
(:wat::core::defenum :wat::runtime::Provenance :wat::enum::Pure
;; The value appeared as a literal in source; `span` is the literal's own span.
  :Literal      [span <- :wat::core::Span]
;; The value was resolved from a symbol lookup; `binding-span` is where the
;; binding was defined, `head-span` is where the symbol appeared in the call.
  :SymbolBound  [binding-span <- :wat::core::Span
                 head-span    <- :wat::core::Span]
;; The value was constructed by a producer function at runtime (e.g.
;; `keyword/from-string`); `producer` names it, `call-span` is the call site.
  :RuntimeBuilt [producer  <- :wat::core::String
                 call-span <- :wat::core::Span])

;; ─── :wat::runtime::ValueSnapshot — a captured value, for diagnostics ────────
;;
;; Mirrors `crate::value::observe::ValueSnapshot` (`src/value/observe.rs:94`).
;; Was written on the wire as an UNTAGGED map (`{:type :rendered :provenance}`) —
;; a record-shaped value with no tag, which the builder has ruled out
;; ("record-shaped values are always tagged"). Declared here so `got` (on
;; `NotCallable` / `TypeMismatch` / `BadCondition`), `called-args` (on
;; `NoMatchingClause`) and `returned-value` (on `PostconditionFailed`) carry a
;; real tag, `#wat.runtime/ValueSnapshot`, instead.
(:wat::core::defrecord :wat::runtime::ValueSnapshot
  [type       <- :wat::core::String
;; The value's rendered textual form (`render_value`), for a human reader.
   rendered   <- :wat::core::String
;; Where the value came from, when known.
   provenance <- (:wat::core::Option :- [:wat::runtime::Provenance])])

;; ─── :wat::runtime::ReteCeilingKind — the closed set of rete ceiling breaches ─
;;
;; The payload enum for the `ReteCeiling` RuntimeErrorKind variant below. Mirrors
;; `crate::value::signal::ReteCeiling` (`src/value/signal.rs:269`) — four
;; variants, every field a scalar (`usize`/`String`), confirmed by direct
;; measurement (excursus 003 STOP-and-report, then the builder's ruling).
;;
;; Named `ReteCeilingKind`, not `ReteCeiling`, on the builder's ruling
;; (2026-09-26): the WRAPPER record below keeps `:wat::runtime::ReteCeiling` —
;; that is today's wire tag and G1's key — so the payload gets the second name,
;; mirroring the `RuntimeError`/`RuntimeErrorKind` split. The Rust type stays
;; named `ReteCeiling`; only the wat declaration and the inner EDN tags
;; (`#wat.runtime/ReteCeilingKind.<Variant>`) carry the new name.
(:wat::core::defenum :wat::runtime::ReteCeilingKind :wat::enum::Pure
;; A `fire-rules` round boundary found the session past `max-session-bytes`.
;; `rounds` completed before the breach — `0` means the growth was per-round
;; fanout, not depth.
  :SessionMemoryCeilingExceeded         [limit  <- :wat::core::i64
                                          used   <- :wat::core::i64
                                          rounds <- :wat::core::i64]
;; `insert` / `insert-all` grew its session past `max-session-bytes`. `staged`
;; is the fact count already held when the breach hit — the insert door's
;; answer to "how far had this got".
  :SessionMemoryCeilingExceededOnInsert [limit  <- :wat::core::i64
                                          used   <- :wat::core::i64
                                          staged <- :wat::core::i64]
;; A rule set refused at `compile-all` because it cannot be proven to
;; terminate: `rule`'s `:then` computes a head value inside a derivation
;; cycle, on `fact-type`.
  :RuleSetMayNotTerminate               [rule      <- :wat::core::String
                                          fact-type <- :wat::core::String]
;; The cascade fixpoint ran past its round `cap`; `still-deriving` is the
;; number of facts still being derived in the round that hit it — evidence
;; the fixpoint was still growing, not merely deep.
  :FixpointRoundCapExceeded             [cap            <- :wat::core::i64
                                          still-deriving <- :wat::core::i64])

;; ─── The 40 RuntimeErrorKind records ─────────────────────────────────────────
;;
;; One `defrecord` per variant of `crate::value::signal::RuntimeErrorKind`
;; (`src/value/signal.rs:376`, 40 variants — the first draft of the brief this
;; file implements said 31; the executor's grep found the other nine). Field
;; order: the floor (`message location causes`), then the kind's own fields in
;; their Rust declaration order.
;;
;; Two shapes intentionally carry ONLY the floor, no kind fields:
;;   - a Rust variant with no payload at all (`DivisionByZero`, `UserMainMissing`,
;;     `WriteStopped`);
;;   - a variant whose only kind field WAS `message`, now merged into the floor's
;;     own `message` (`MacroAbort`) — see the note there — or whose only kind
;;     field is a nested error, now moved to `causes` (`EvalVerificationFailed`).
;; Both are fine per the brief ("if a record would then have no kind field left,
;; it keeps only the floor").

;; `:wat::core::unbound-symbol` — `name` is the unresolved symbol.
(:wat::core::defrecord :wat::runtime::UnboundSymbol
  [message <- :wat::core::String
   location <- :wat::core::Span
   causes <- (:wat::core::Vector :- [:wat::core::Error])
;; The symbol that had no binding.
   name <- :wat::core::String])

;; `path` is the unresolved function's FQDN.
(:wat::core::defrecord :wat::runtime::UnknownFunction
  [message <- :wat::core::String
   location <- :wat::core::Span
   causes <- (:wat::core::Vector :- [:wat::core::Error])
;; The path that named no registered function.
   path <- :wat::core::String])

;; A BINDING handler (takes `env`/`sym`, not evaluated args) was reached through
;; a value-dispatch call site, which has no AST left to hand it.
(:wat::core::defrecord :wat::runtime::NotValueDispatchable
  [message <- :wat::core::String
   location <- :wat::core::Span
   causes <- (:wat::core::Vector :- [:wat::core::Error])
;; The handler's registered name.
   name <- :wat::core::String])

;; A call's head evaluated to a non-Function value.
(:wat::core::defrecord :wat::runtime::NotCallable
  [message <- :wat::core::String
   location <- :wat::core::Span
   causes <- (:wat::core::Vector :- [:wat::core::Error])
;; The value that was called, snapshotted for diagnosis.
   got <- :wat::runtime::ValueSnapshot])

;; A typed operation received a value of the wrong type.
(:wat::core::defrecord :wat::runtime::TypeMismatch
  [message <- :wat::core::String
   location <- :wat::core::Span
   causes <- (:wat::core::Vector :- [:wat::core::Error])
;; The operation that rejected the value.
   op <- :wat::core::String
;; The type name it wanted.
   expected <- :wat::core::String
;; The value it got instead, snapshotted.
   got <- :wat::runtime::ValueSnapshot])

;; A call passed the wrong number of arguments.
(:wat::core::defrecord :wat::runtime::ArityMismatch
  [message <- :wat::core::String
   location <- :wat::core::Span
   causes <- (:wat::core::Vector :- [:wat::core::Error])
;; The operation whose arity was violated.
   op <- :wat::core::String
;; The arity it declared.
   expected <- :wat::core::i64
;; The arity it was called with.
   got <- :wat::core::i64])

;; An `if`/`when` condition was not a `:wat::core::bool`.
(:wat::core::defrecord :wat::runtime::BadCondition
  [message <- :wat::core::String
   location <- :wat::core::Span
   causes <- (:wat::core::Vector :- [:wat::core::Error])
;; The non-bool value the condition evaluated to, snapshotted.
   got <- :wat::runtime::ValueSnapshot])

;; A special form's surface shape was invalid.
(:wat::core::defrecord :wat::runtime::MalformedForm
  [message <- :wat::core::String
   location <- :wat::core::Span
   causes <- (:wat::core::Vector :- [:wat::core::Error])
;; The form's head keyword.
   head <- :wat::core::String
;; What was wrong with it.
   reason <- :wat::core::String])

;; A `fn`/`defn` parameter name shadowed a `:wat::core` builtin.
(:wat::core::defrecord :wat::runtime::ParamShadowsBuiltin
  [message <- :wat::core::String
   location <- :wat::core::Span
   causes <- (:wat::core::Vector :- [:wat::core::Error])
;; The shadowing parameter's name.
   name <- :wat::core::String])

;; `i64::/` (or `%`) at a zero divisor. No kind fields — the floor message
;; already names the operation ("division by zero").
(:wat::core::defrecord :wat::runtime::DivisionByZero
  [message <- :wat::core::String
   location <- :wat::core::Span
   causes <- (:wat::core::Vector :- [:wat::core::Error])])

;; `i64 + - *` overflowed 64 bits.
(:wat::core::defrecord :wat::runtime::IntegerOverflow
  [message <- :wat::core::String
   location <- :wat::core::Span
   causes <- (:wat::core::Vector :- [:wat::core::Error])
;; The overflowing operator.
   op <- :wat::core::String
;; The left operand.
   a <- :wat::core::i64
;; The right operand.
   b <- :wat::core::i64])

;; A top-level name was declared twice with a non-equivalent body.
(:wat::core::defrecord :wat::runtime::DuplicateDefine
  [message <- :wat::core::String
   location <- :wat::core::Span
   causes <- (:wat::core::Vector :- [:wat::core::Error])
;; The redeclared name.
   name <- :wat::core::String])

;; A top-level name used a reserved namespace prefix.
(:wat::core::defrecord :wat::runtime::ReservedPrefix
  [message <- :wat::core::String
   location <- :wat::core::Span
   causes <- (:wat::core::Vector :- [:wat::core::Error])
;; The reserved prefix that was used.
   prefix <- :wat::core::String])

;; A `defclause` arm can never be selected because an earlier arm subsumes it.
(:wat::core::defrecord :wat::runtime::UnreachableClause
  [message <- :wat::core::String
   location <- :wat::core::Span
   causes <- (:wat::core::Vector :- [:wat::core::Error])
;; The defclause's declared name.
   name <- :wat::core::String
;; 0-based index of the arm that can never fire.
   clause-index <- :wat::core::i64
;; 0-based index of the earlier arm that subsumes it.
   subsumed-by <- :wat::core::i64
;; The unreachable arm's declared parameter types, formatted.
   declared-arg-types <- (:wat::core::Vector :- [:wat::core::String])])

;; A top-level name reached a registration gate with no namespace.
(:wat::core::defrecord :wat::runtime::UnnamespacedName
  [message <- :wat::core::String
   location <- :wat::core::Span
   causes <- (:wat::core::Vector :- [:wat::core::Error])
;; The bare name that was registered.
   name <- :wat::core::String])

;; A top-level name's last segment contained a `.` — the wire discriminator
;; for a tagged-enum variant, and so reserved.
(:wat::core::defrecord :wat::runtime::DottedName
  [message <- :wat::core::String
   location <- :wat::core::Span
   causes <- (:wat::core::Vector :- [:wat::core::Error])
;; The dotted name that was registered.
   name <- :wat::core::String])

;; A declaration form (`define`, `defmacro`, …) appeared in expression position.
(:wat::core::defrecord :wat::runtime::DeclarationInExpressionPosition
  [message <- :wat::core::String
   location <- :wat::core::Span
   causes <- (:wat::core::Vector :- [:wat::core::Error])
;; The misplaced declaration form's head.
   head <- :wat::core::String])

;; A constrained `eval` found a mutation-inducing form in the AST it was
;; asked to evaluate.
(:wat::core::defrecord :wat::runtime::EvalForbidsMutationForm
  [message <- :wat::core::String
   location <- :wat::core::Span
   causes <- (:wat::core::Vector :- [:wat::core::Error])
;; The forbidden form's head.
   head <- :wat::core::String])

;; `:user::main` was not registered at startup. Freeze-pair — no kind fields;
;; the floor message alone names the failure.
(:wat::core::defrecord :wat::runtime::UserMainMissing
  [message <- :wat::core::String
   location <- :wat::core::Span
   causes <- (:wat::core::Vector :- [:wat::core::Error])])

;; `:wat::eval-digest!`/`eval-signed!` verification failed. The wrapped
;; `HashError` (`src/hash.rs`) is a nested ERROR, per the brief's rule 2: it
;; moves into `causes` as a `:wat::core::Fault` (its `Display` text, the
;; RAISING SITE's span — `HashError` carries no location of its own — and no
;; further causes), so no kind field survives. `HashError`'s own shape (which
;; of eight failure kinds) is OUT OF SCOPE for this step; naming that loss is
;; the point of `causes` carrying a Fault rather than the richer type.
(:wat::core::defrecord :wat::runtime::EvalVerificationFailed
  [message <- :wat::core::String
   location <- :wat::core::Span
   causes <- (:wat::core::Vector :- [:wat::core::Error])])

;; `:wat::kernel::join` reaped a spawned program whose thread panicked before
;; yielding a result.
(:wat::core::defrecord :wat::runtime::ChannelDisconnected
  [message <- :wat::core::String
   location <- :wat::core::Span
   causes <- (:wat::core::Vector :- [:wat::core::Error])
;; The operation that observed the disconnect.
   op <- :wat::core::String])

;; A rete session breached one of its closed-set ceilings (see
;; `:wat::runtime::ReteCeilingKind` above). Excursus 003 doc note (`src/value/
;; signal.rs:248-262`): nesting the four former flat variants under this one
;; changed their EDN tag (`#wat.runtime/ReteCeiling {:ceiling …}` instead of a
;; flat `#wat.runtime/<Variant>`) — measured to touch no golden and no test;
;; `tests/lint/no_ceiling_raise_in_rete.rs`'s `CEILING_VARIANTS` keys on the
;; INNER variant names, unaffected by the wrapper's or the payload enum's name.
(:wat::core::defrecord :wat::runtime::ReteCeiling
  [message <- :wat::core::String
   location <- :wat::core::Span
   causes <- (:wat::core::Vector :- [:wat::core::Error])
;; Which ceiling breached, and its measurement.
   ceiling <- :wat::runtime::ReteCeilingKind])

;; A vector-level primitive ran with no `EncodingCtx` attached to the
;; `SymbolTable`.
(:wat::core::defrecord :wat::runtime::NoEncodingCtx
  [message <- :wat::core::String
   location <- :wat::core::Span
   causes <- (:wat::core::Vector :- [:wat::core::Error])
;; The primitive that needed the encoding context.
   op <- :wat::core::String])

;; A file-reading primitive ran with no source loader attached.
(:wat::core::defrecord :wat::runtime::NoSourceLoader
  [message <- :wat::core::String
   location <- :wat::core::Span
   causes <- (:wat::core::Vector :- [:wat::core::Error])
;; The primitive that needed file I/O.
   op <- :wat::core::String])

;; `macroexpand`/`macroexpand-1` ran with no macro registry attached.
(:wat::core::defrecord :wat::runtime::NoMacroRegistry
  [message <- :wat::core::String
   location <- :wat::core::Span
   causes <- (:wat::core::Vector :- [:wat::core::Error])
;; The primitive that needed macro expansion.
   op <- :wat::core::String])

;; `macroexpand`/`macroexpand-1` surfaced a macro-expansion error. The wrapped
;; `MacroError` (`src/macros/error.rs`) is a nested ERROR, per rule 2: it
;; moves into `causes` as a `:wat::core::Fault` (its `Display` text, the
;; RAISING SITE's span, no further causes) rather than a kind field. `op`
;; survives as the one non-error kind field. `MacroError`'s own richer shape
;; (it separately implements `WatError` with its own location) is OUT OF
;; SCOPE here — the same uniform Fault-wrapping this file gives every nested
;; cause, not a special case for the one nested error that happens to carry
;; more.
(:wat::core::defrecord :wat::runtime::MacroExpansionFailed
  [message <- :wat::core::String
   location <- :wat::core::Span
   causes <- (:wat::core::Vector :- [:wat::core::Error])
;; The primitive whose expansion failed.
   op <- :wat::core::String])

;; A `match` ran with no arm whose pattern matched the scrutinee's shape.
(:wat::core::defrecord :wat::runtime::PatternMatchFailed
  [message <- :wat::core::String
   location <- :wat::core::Span
   causes <- (:wat::core::Vector :- [:wat::core::Error])
;; The scrutinee's runtime type name.
   value-type <- :wat::core::String])

;; `:wat::eval-step!` saw a form whose head is an effectful op, which the
;; stepwise evaluator deliberately refuses.
(:wat::core::defrecord :wat::runtime::EffectfulInStep
  [message <- :wat::core::String
   location <- :wat::core::Span
   causes <- (:wat::core::Vector :- [:wat::core::Error])
;; The effectful op that was refused.
   op <- :wat::core::String])

;; `:wat::eval-step!` saw a form shape not yet covered by a step rule.
(:wat::core::defrecord :wat::runtime::NoStepRule
  [message <- :wat::core::String
   location <- :wat::core::Span
   causes <- (:wat::core::Vector :- [:wat::core::Error])
;; The unrecognized op.
   op <- :wat::core::String])

;; `:wat::kernel::assertion-failed!` fired outside a sandbox catching it.
;;
;; The kind's own `message` field collided with the floor's — measured (not
;; guessed): `RuntimeError` has two EDN writers, and the wire-crossing one
;; (`WatError::error_edn`, `src/edn/contract.rs:109`) strips a variant's own
;; `:message`/`:location`/`:causes` and inserts the floor's, whose `message`
;; is `first_line(kind.to_string())` — for `AssertionFailed` that is
;; `"assertion failed: <the raw message>"`, NOT the raw message alone. So the
;; wire that actually crosses a process boundary today already carries only
;; ONE `:message` — the headline. This record matches that: no separate raw-
;; message kind field; the raw text survives inside the floor headline.
;; `actual`/`expected` stay as their own structured fields.
(:wat::core::defrecord :wat::runtime::AssertionFailed
  [message <- :wat::core::String
   location <- :wat::core::Span
   causes <- (:wat::core::Vector :- [:wat::core::Error])
;; The failed assertion's actual value, rendered, when the caller supplied one.
   actual <- (:wat::core::Option :- [:wat::core::String])
;; The failed assertion's expected value, rendered, when the caller supplied one.
   expected <- (:wat::core::Option :- [:wat::core::String])])

;; A sub-program's `UnknownFunction` name is registered in the OUTER scope —
;; the "you defined this outside the sandbox" diagnostic.
(:wat::core::defrecord :wat::runtime::SandboxScopeLeak
  [message <- :wat::core::String
   location <- :wat::core::Span
   causes <- (:wat::core::Vector :- [:wat::core::Error])
;; The name that leaked from the outer scope.
   offending-name <- :wat::core::String
;; Where the outer-scope define lives.
   outer-define-span <- :wat::core::Span])

;; A thread-aware stdio helper ran on a thread with no `ThreadIO` installed.
(:wat::core::defrecord :wat::runtime::ServiceNotRunning
  [message <- :wat::core::String
   location <- :wat::core::Span
   causes <- (:wat::core::Vector :- [:wat::core::Error])
;; The stdio helper that found no service.
   op <- :wat::core::String])

;; `:wat::kernel::readln`'s EDN→typed coercion found a shape mismatch. `path`
;; is a `(Vector :- [String])` on the wire today (`#[to_edn(via =
;; edn_path_segments)]`, `src/edn/error.rs:56`), not a bare string — this
;; record matches the WIRE shape, not the Rust field's own `String` type.
(:wat::core::defrecord :wat::runtime::EdnCoerceMismatch
  [message <- :wat::core::String
   location <- :wat::core::Span
   causes <- (:wat::core::Vector :- [:wat::core::Error])
;; The op whose declared return type the wire value didn't match.
   op <- :wat::core::String
;; The wat type the caller's `-> :T` annotation asked for.
   expected <- :wat::core::String
;; The EDN shape that actually arrived.
   got <- :wat::core::String
;; Dot-path segments naming the nested sub-field that failed, empty for a
;; top-level mismatch.
   path <- (:wat::core::Vector :- [:wat::core::String])])

;; `Record/assoc` was invoked with a field key absent from the record's class.
(:wat::core::defrecord :wat::runtime::UnknownField
  [message <- :wat::core::String
   location <- :wat::core::Span
   causes <- (:wat::core::Vector :- [:wat::core::Error])
;; The record class's bare FQDN (no leading colon).
   record-class <- :wat::core::String
;; The field name that was attempted.
   field <- :wat::core::String
;; The record class's actually-available field names.
   available <- (:wat::core::Vector :- [:wat::core::String])])

;; A `defclause` call matched no clause. `attempted-clauses` carries WHY each
;; clause was skipped (`:wat::kernel::ClauseAttempt`, declared in
;; `wat/kernel/diagnostics.wat` — it already ships as `#wat.kernel/…`, so it
;; keeps that namespace rather than moving to `:wat::runtime::`).
(:wat::core::defrecord :wat::runtime::NoMatchingClause
  [message <- :wat::core::String
   location <- :wat::core::Span
   causes <- (:wat::core::Vector :- [:wat::core::Error])
;; The defclause's declared name.
   name <- :wat::core::String
;; The number of arguments the call actually supplied.
   called-arity <- :wat::core::i64
;; The call's actual arguments, snapshotted.
   called-args <- (:wat::core::Vector :- [:wat::runtime::ValueSnapshot])
;; One entry per declared clause, naming why it was skipped.
   attempted-clauses <- (:wat::core::Vector :- [:wat::kernel::ClauseAttempt])])

;; A `defclause` clause's `:ensure` postcondition returned false.
(:wat::core::defrecord :wat::runtime::PostconditionFailed
  [message <- :wat::core::String
   location <- :wat::core::Span
   causes <- (:wat::core::Vector :- [:wat::core::Error])
;; The defclause's declared name.
   defclause-name <- :wat::core::String
;; 0-based index of the clause whose postcondition failed.
   clause-index <- :wat::core::i64
;; The `:ensure :fn` expression's rendered form, captured at dispatch time.
   ensure-expr-snapshot <- :wat::core::String
;; The clause body's return value, snapshotted — what the postcondition rejected.
   returned-value <- :wat::runtime::ValueSnapshot
;; Where the `:ensure :fn` was declared (the secondary span).
   ensure-span <- :wat::core::Span])

;; `(:wat::core::macro-error "msg")` aborted macro expansion with a user
;; diagnostic. Same message collision as `AssertionFailed`, same resolution:
;; the floor's `message` already carries the raw text (the wire's
;; `error_edn()` strips the kind's own `:message` and substitutes the
;; headline, which for this variant equals the raw text verbatim — there is
;; no extra prefix in its `Display` impl). No kind field survives.
(:wat::core::defrecord :wat::runtime::MacroAbort
  [message <- :wat::core::String
   location <- :wat::core::Span
   causes <- (:wat::core::Vector :- [:wat::core::Error])])

;; A pipe write was preempted by a process-wide stop request, not a broken
;; pipe. No kind fields — the floor message names the condition.
(:wat::core::defrecord :wat::runtime::WriteStopped
  [message <- :wat::core::String
   location <- :wat::core::Span
   causes <- (:wat::core::Vector :- [:wat::core::Error])])

;; A `(:wat::rete::core::defn …)` body failed one of the four purity-fence
;; axes, checked once at the definition site.
(:wat::core::defrecord :wat::runtime::ReteDefnAxisViolation
  [message <- :wat::core::String
   location <- :wat::core::Span
   causes <- (:wat::core::Vector :- [:wat::core::Error])
;; The declared helper's own FQDN.
   name <- :wat::core::String
;; The failing axis's variant name (Pure / Deterministic / Total / Law A).
   axis <- :wat::core::String
;; The specific violating sub-expression's head.
   head <- :wat::core::String])

;; A `(:wat::rete::core::defn …)` body (transitively) calls itself, or
;; participates in a cycle of rete-defns.
(:wat::core::defrecord :wat::runtime::ReteDefnRecursive
  [message <- :wat::core::String
   location <- :wat::core::Span
   causes <- (:wat::core::Vector :- [:wat::core::Error])
;; The declared helper that is (transitively) recursive.
   name <- :wat::core::String
;; The callee that closed the cycle (equal to `name` for self-recursion).
   head <- :wat::core::String])
