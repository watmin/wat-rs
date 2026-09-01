;; wat/runtime-meta.wat — arc 255.1b-iv-c: closed-domain enum types for
;; :wat::runtime::metadata-of reflection surface.
;;
;; Three unit-only enums: Kind / DefinedIn / Layer.
;; Capitalized variants (§5 locked-record-model decision): avoids any `:fn`
;; keyword-legality question and signals these are closed-domain type values
;; (not data keywords).
;;
;; Loading order: no eval-deps beyond :wat::core::defenum (a builtin).
;; May be placed anywhere after wat/core.wat.

;; Kind — what kind of callable is this?
(:wat::core::defenum :wat::runtime::Kind :wat::enum::Pure
;; A user-defined `:wat::core::defmacro` — expands at compile time.
  :Macro
;; A user-defined `:wat::core::defn`.
  :Fn
;; Implemented in Rust, exposed under a `:wat::` FQDN.
  :Intrinsic
;; A substrate special form — no NativeHandler; dispatched by the runtime.
  :SpecialForm)

;; DefinedIn — implementation language.
(:wat::core::defenum :wat::runtime::DefinedIn :wat::enum::Pure
;; Written in wat — a user `defn` or `defmacro`.
  :Wat
;; Written in Rust — every intrinsic.
  :Rust)

;; Layer — where in the system stack does this live?
(:wat::core::defenum :wat::runtime::Layer :wat::enum::Pure
;; The kernel/stdlib layer — every intrinsic.
  :Substrate
;; User-written code above the substrate.
  :Userland)

;; Purity — declared purity of an intrinsic or special form.
(:wat::core::defenum :wat::runtime::Purity :wat::enum::Pure
;; Same output for the same input, with no observable side effect.
  :Pure
;; Has an observable side effect — I/O, mutation, a signal.
  :Effectful
;; A special form that PRESERVES the purity of its sub-forms rather than
;; having one of its own: `if` is pure exactly when its branches are.
  :Preserving)

;; Determinism — declared determinism of an intrinsic or special form.
(:wat::core::defenum :wat::runtime::Determinism :wat::enum::Pure
;; The same input always produces the same output.
  :Deterministic
;; The output may differ across calls — a clock read, a UUID, entropy.
  :Nondeterministic
;; A special form that PRESERVES the determinism of its sub-forms rather
;; than having one of its own.
  :Preserving)

;; Category — functional category.
;; Category — what a verb DOES in the language. Most commonly a runtime computation;
;; sometimes a program-level registration (:Declaration); sometimes a contract
;; discharged entirely at check time (:CheckGate). The axis is the DOING, not the
;; moment it happens.
;; ONE axis throughout: what the verb DOES. Not what it returns, not where its
;; input comes from, not which direction it crosses a type boundary — each of
;; those was proposed as a variant during arc 255 and rejected for mixing axes.
;;   :Transform   — returns the SAME value in another form (was :Encoding, renamed
;;                  2026-08-15: trim/to-lowercase/split are not encodings)
;;   :Reflection  — the program interrogating ITSELF (metadata-of, show-source)
;;   :ControlFlow — directs evaluation (if, and higher-order application)
;;   :Binding     — introduces a LOCAL, scoped name at runtime (let)
;;   :Entropic    — samples an unpredictable external source and effects nothing
;;                  (was :Clock; time::now, Uuid/v4 — measured by BOUNDING, not pinning)
;;   :Arithmetic  — math on numeric domain values
;;   :Io          — input/output on a stream
;;   :Probe       — interrogates a value, derives a FACT about it (empty?, length)
;;   :Combine     — builds a larger value of the same kind (concat, conj, assoc)
;;   :Declaration — registers a program-level entity (def, defclause,
;;                  declare-acronyms). Distinct from :Binding — a declaration
;;                  registers into the program, visible to everything after it.
;;
;; ⛔ THIS FILE IS THE SOURCE OF TRUTH FOR THE RUST ENUMS, not a mirror of them.
;; Every `defenum` above and below is read at COMPILE TIME by
;; `wat_enum_derive::wat_enum_from!` and becomes a Rust enum — variants, order, and
;; the `;;` prose on each variant (which becomes its `///`). Add a variant here and
;; the Rust type follows; every exhaustive `match` on it then fails to compile until
;; the new variant is handled.
;;
;; ⛔ CORRECTED 2026-08-19 (255.1c-taxonomy). This paragraph used to end: "There is no
;; second list and no drift gate, because a generated type cannot drift from its
;; generator." BOTH HALVES WERE FALSE, and adding five variants proved it by breaking
;; the build in three places.
;;
;; THERE ARE SECOND LISTS. The generated TYPE cannot drift — that much was true — but
;; every place that turns a `Category` VALUE back into something else is hand-written:
;;   crates/wat-macros/src/wat_intrinsic.rs      value -> `quote!` token, arm per variant
;;   crates/wat-macros/src/wat_special_form.rs   the same match, again
;;   crates/wat-doc/src/lib.rs                   the round-trip test’s own `all` + `match`
;;   crates/wat-doc/src/lib.rs `CATEGORY_LEGAL_VALUES`  a hand-written STRING of every name
;;
;; AND THERE IS A DRIFT GATE — `every_enum_variant_reaches_both_hand_lists`, whose NAME
;; says so. It was built 2026-08-15 because the lists HAD already drifted: `Transform`/
;; `Probe`/`Combine` were added and the old gate stayed green "because the two lists still
;; agreed with each other" — stale-to-stale.
;;
;; ★ THE PROPERTY THAT ACTUALLY HOLDS, and it is worth more than the false one: the second
;; lists CANNOT SILENTLY DRIFT. Every exhaustive `match` is covered by the compiler
;; (`E0004`, a hard error — `cargo build` for production code, `cargo test --no-run` for
;; test code); the one NON-match mirror, `CATEGORY_LEGAL_VALUES`, is covered by that gate.
;; Build green + test-build green + that gate passing = every mirror reached. Say THAT,
;; not that there is no second list.
(:wat::core::defenum :wat::runtime::Category :wat::enum::Pure
;; Returns the SAME value in another form — `Bytes::to-hex`, `epoch-seconds`,
;; `string::trim`. Was `:Encoding` until 2026-08-15: half its members were not
;; encodings at all (`trim` discards data, `to-lowercase` folds case, `split`
;; restructures). What unites them is that the OUTPUT IS A FORM OF THE INPUT.
  :Transform
;; The program interrogating ITSELF — `metadata-of`, `show-source`, `render-doc`.
;; NOT reading a clock or a stream: those come from outside the program.
  :Reflection
;; Directs evaluation — `if`, and applying a callable handed in as a value. Also verbs
;; that ABANDON evaluation rather than direct it — `raise!`/`assertion-failed!` never
;; return; they panic through the call stack instead of choosing which branch runs next.
  :ControlFlow
;; Introduces a LOCAL, scoped name at runtime — `let`. Contrast `:Declaration`,
;; which registers a program-level entity.
  :Binding
;; Samples an unpredictable external source and returns the sample — `now`,
;; `Uuid/v4`. Effects NOTHING, so `@Purity` stays `Pure`; the value cannot be
;; pinned, only BOUNDED, which is what makes conformance its measurement mode.
;; WHICH DEVICE the entropy is drawn from — wall clock, CSPRNG, /dev/urandom,
;; pid — is an implementation detail and NEVER the axis, the same way transport
;; is not `:Message`'s axis. This variant was `:Clock` until 2026-08-19, which
;; named the device and reserved a second slot for "random"; the builder ruled
;; them one DOING: "Time.now and SecureRandom.uuid are the same category.. they
;; are a syscall who is 'pure'". NOT `:Io`: Io moves DATA across the boundary in
;; either direction and effects the world (`println` out, `readln'` in); entropy
;; carries no data in, and leaves the world unchanged.
  :Entropic
;; Math on numeric domain values. NOT string concatenation — `Vector/concat` is
;; not math, and that absurdity is what exposed the mistake (2026-08-15).
  :Arithmetic
;; Input/output on a stream — `println`, `readln'`. The effect IS the point;
;; an encoding step along the way does not make it `:Transform`. Contrast
;; `:Message`: a peer is a typed value the caller holds a handle to, not an
;; OS stream.
  :Io
;; Interrogates a value and derives a FACT about it — `empty?`, `length`,
;; `contains?`. The output is a fact ABOUT the input, never a form of it.
;; NOT "returns a bool": `length` returns an i64 and belongs here. Sorting by
;; return type is the axis-mix that sank the proposed `:Predicate`.
  :Probe
;; Builds a larger value of the same kind from several — `concat`, `conj`,
;; `assoc`, `join`. A cross-type family spanning strings, vectors, sets, maps
;; and records.
  :Combine
;; Registers a program-level entity — `def`, `defclause`, `declare-acronyms`.
;; Distinct from `:Binding`: a declaration registers into the program and is
;; visible to everything after it; `let` is local and scoped.
  :Declaration
;; Acquires, releases, or ADMINISTERS a handle whose lifetime is tracked outside
;; value scope — `listener`, `connect`, `accept`, `pipe`, `spawn-thread`,
;; `spawn-process`, `after`, `HandlePool::{new,pop,finish}`, `close`,
;; `allow`, `deny`, `signal`. NOT what data moves through the handle (that is
;; `:Message`), NOT where the handle came from.
  :Resource
;; Delivers or receives a payload across a peer/channel boundary to another locus —
;; `send`, `try-send`, `recv`, `select`, `poll`. The locus is a TYPED VALUE (`(peer :- [I O])`)
;; the caller already holds — contrast `:Io`, whose target is an ambient OS stream with
;; no caller-held handle. The underlying transport (in-process channel, pipe, socket) is
;; an implementation detail, NEVER the axis — the same way `:Mutate` was refused for
;; `allow`/`deny`.
  :Message
;; Reads or writes process-global state that no value the caller holds addresses —
;; `stopped?`, `sigusr1?`, `sigusr2?`, `sighup?`, `reset-sigusr1!`, `reset-sigusr2!`,
;; `reset-sighup!`. NOT `:Entropic`: three of seven members are writes, and
;; `:Entropic`'s axis is which source a read draws from. NOT `:Probe`: the `sig*?` queries take no
;; input value to interrogate — they read a global `AtomicBool`, not a fact about
;; something the caller holds.
  :Ambient
;; Returns a COMPONENT of a compound value that was already there — `Failure/message`,
;; `Failure/location`, `LociDiedError/message`, and every hand-written record/struct
;; field accessor. The inverse of `:Combine`, which builds a larger value of the same
;; kind; nothing had named taking a part back out. NOT `:Probe`: a probe computes a
;; new fact (`empty?`, `length`); an accessor returns a part that already existed.
  :Projection
;; Refuses a call site at CHECK TIME; the contract is discharged before evaluation
;; ever runs — `require-wire-address`, which unifies its argument's transport marker
;; against `Wire` in `infer_require_wire_address` and raises a `TypeMismatch` naming
;; `Shared` when it does not fit. ONE axis: constrains which programs compile. The
;; runtime body is identity or otherwise incidental to the variant's purpose — minted
;; ahead of the totality campaign's `must-*` family on the builder's forward knowledge.
  :CheckGate)

;; Totality — is the verb DEFINED ON EVERY INPUT in its declared domain?
;;
;; Arc 255 / arc 278's `where`-fence needs this as a first-class axis and it has
;; never had a home: `pure` and `deterministic` live in the baseline, `total` lived
;; in THREE hand-lists that disagree (`rete/purity.rs`'s `intrinsic_meta`,
;; `macros/eval.rs`'s `is_pure_total`, `rete/vocabulary.rs`'s `RETE_OPS`).
;;
;; ⛔ ORTHOGONAL TO PURITY AND DETERMINISM, never derived from them: `i64::/` is
;; Pure AND Deterministic AND undefined at a zero divisor.
;;
;; ★ FOUR variants, and the fourth is the honest one. Two poles plus `:Preserving`
;; mirrors `Purity`/`Determinism` exactly. `:Unreviewed` exists because a verb
;; nobody has measured must NOT be recorded as either pole — collapsing "measured
;; partial" into "never looked at" is the failure `feedback_none_means_skip_
;; conflates_cannot_with_did_not_look` names, and a GUESSED `:Total` is a lie in a
;; fence that admits code into a `where`.
(:wat::core::defenum :wat::runtime::Totality :wat::enum::Pure
;; Defined on EVERY input of its declared domain — measured, by reading the
;; implementation, never inferred from the name. `f64::>` is total: its output is
;; a bool for any pair of floats.
  :Total
;; Undefined somewhere in its declared domain — measured. `i64::/` at a zero
;; divisor; `f64::*` overflowing to +/-Inf. ★ THIS VARIANT IS THE WORK LIST: the
;; totality endgame's census is `all_entries().filter(|e| e.totality == Partial)`.
  :Partial
;; A special form that PRESERVES the totality of its sub-forms rather than having
;; one of its own: `if` is total exactly when its branches are.
  :Preserving
;; NOBODY HAS MEASURED THIS VERB YET. Not a pole, not a guess. Default-deny: it
;; does NOT satisfy the `where`-fence, so an unreviewed verb is refused rather
;; than admitted. Shrinks to zero as the census runs; a migration state, and the
;; only variant expected to disappear.
  :Unreviewed)

;; ExpandTime — may this verb be CALLED from inside a `defmacro` body while that
;; macro is being expanded?
;;
;; ⛔ INDEPENDENT OF THE OTHER THREE AXES, and arc 255 Stone expand-1's audit of all
;; 202 entries in `macros/eval.rs`'s allow-list produced a witness for each:
;;
;;   `:wat::i64::/`             is @Totality PARTIAL          and legal — a zero divisor
;;                              at expand time is a COMPILE-time failure, strictly
;;                              better than a runtime one.
;;   `:wat::core::fresh-symbol` is @Determinism NONDETERMINISTIC and legal — minting a
;;                              different gensym per call is what makes hygienic
;;                              expansion possible.
;;   `:wat::hashmap::keys`      is NONDETERMINISTIC and legal — a pure projection of
;;                              pure data whose ORDER alone is unspecified.
;;   every @Purity Effectful verb is NOT legal — zero exceptions across 202 entries.
;;
;; So no combination of purity, determinism and totality predicts membership. The
;; LOCKED RECORD MODEL's Layer-1 baseline reserved `expand_time_legal` on 2026-06-21
;; and it was never built; a hand-curated allow-list carried it instead, and grew a
;; measured 174-verb gap that nothing could see — a false refusal only surfaces when
;; some macro body happens to call the verb.
(:wat::core::defenum :wat::runtime::ExpandTime :wat::enum::Pure
;; May be called inside a `defmacro` body during expansion. Says nothing about
;; purity, determinism or totality — a partial or nondeterministic verb can be
;; perfectly legal here, and three of them are.
  :Legal
;; Needs state that does not exist yet at expand time — IO, spawning, entropy, a
;; clock, a signal, or the evaluation of arbitrary submitted forms. Named for what
;; the verb IS rather than for being refused, the same way `Effectful` and `Partial`
;; name their poles.
  :RuntimeOnly
;; `RuntimeOnly`'s MIRROR, not a synonym for `Legal`: the verb has NO runtime call
;; site at all — its only legitimate caller is a `defmacro` body during expansion.
;; `Legal` means *also* callable there; this means *only* callable there. A
;; runnable `@example` for a verb in this pole is impossible by construction — a
;; runnable example is evaluated at RUNTIME, a tier where the verb does not exist —
;; so `@example-norun` is its correct and required form. Named for what the verb
;; IS, the same way `RuntimeOnly` names its own pole rather than the fact that it
;; gets refused.
  :ExpandOnly
;; A form whose expand-time legality is its SUB-FORMS' rather than its own: `if` is
;; legal at expand time exactly when its branches are. Mirrors `Purity`,
;; `Determinism` and `Totality`, which all carry this variant.
  :Preserving
;; NOBODY HAS MEASURED THIS VERB YET. Not a pole, not a guess. DEFAULT-DENY: it does
;; NOT satisfy the expand-time gate, so an unreviewed verb is refused rather than
;; admitted into a macro body. Shrinks to zero as the census runs; the only variant
;; expected to disappear.
  :Unreviewed)
