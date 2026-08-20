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
;; Directs evaluation — `if`, and applying a callable handed in as a value.
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
;; `spawn-process`, `after`, `HandlePool::{new,pop,finish}`, `close`, `drop`,
;; `allow`, `deny`, `signal`. NOT what data moves through the handle (that is
;; `:Message`), NOT where the handle came from. `drop` is a documented NO-OP —
;; it does not force teardown while other references remain.
  :Resource
;; Delivers or receives a payload across a peer/channel boundary to another locus —
;; `send`, `try-send`, `recv`, `select`, `poll`. The locus is a TYPED VALUE (`peer<I,O>`)
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
;; ever runs — `require-wire-address`. ONE axis: constrains which programs compile.
;; The runtime body is identity or otherwise incidental to the variant's purpose.
;; One member today, deliberately: minted ahead of the totality campaign's `must-*`
;; family on the builder's forward knowledge; revisit this variant at the second
;; member rather than treating the thin membership as an error.
  :CheckGate)
