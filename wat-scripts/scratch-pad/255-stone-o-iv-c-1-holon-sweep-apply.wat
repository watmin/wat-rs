;; wat-scripts/scratch-pad/255-stone-o-iv-c-1-holon-sweep-apply.wat — arc 255
;; Stone O-iv-c-1, acceptance row 0. Drives every verb this stone migrated to ALGEBRA
;; through `:wat::core::apply` (the value door), via `:wat::eval-ast!` on a `quote`d form
;; so the static type checker (which would otherwise refuse a heterogeneous
;; `(:wat::core::Vector :wat::core::Any …)` args-vector at check time) never runs — the
;; SAME pattern `255-stone-o-iv-a-both-branches-proven-separately.wat` and
;; `255-stone-o-iv-c-0-require-family-wrong-type.wat` already use.
;;
;; BEFORE this stone: every row below reports the O-iv-a diagnostic
;; ("… is registered, but no handler taking EVALUATED arguments is registered under …",
;; kind = "runtime-error" — `NotValueDispatchable` has no dedicated EvalError kind).
;; AFTER: all 27 migrated rows answer with a real value.
;;
;; Four `:wat::holon::Engram/*` readers (`name`, `eigenvalue-signature`, `n`, `residual`)
;; are migrated but their TYPE is UNCONSTRUCTIBLE from wat today (no verb anywhere hands a
;; wat program a bare `Value::Engram` — `EngramLibrary/add` freezes one internally but
;; never returns it; `match-vec` returns `(name, residual)` tuples, not the engram) — a
;; pre-existing gap named in H-1a, not this stone's. Per the brief: exercised via their
;; wrong-TYPE path instead (`apply` with a non-Engram arg), which still proves the value
;; door now exists and dispatches into `require_engram`.
;;
;; The 8 handlers that STAY BINDING this stone (3 `hologram.rs` — `make`/`get`/`find`, via
;; `require_encoding_ctx`'s `&SymbolTable` need — plus 5 more this rider's census found:
;; `reckoner.rs`'s `new-discrete`/`new-continuous`/`resolve` and `hologram.rs`'s
;; `put`/`remove`, all of which locate a manual `TypeMismatch` at an ARGUMENT's own
;; `WatAST::span()`, unavailable from `&Value` — see the rider's report) are NOT exercised
;; here; they still report the O-iv-a diagnostic through apply, unchanged, same as before
;; this stone. Scratch, per holon/CLAUDE.md's .wat scratch convention.

(:wat::core::defn :probe::show
  [tag <- :wat::core::String r <- (:wat::core::Result :wat::core::Value :wat::core::EvalError)]
  -> :wat::core::nil
  (:wat::kernel::println (:wat::string::concat tag ": " (:wat::edn::write r))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [s   (:wat::holon::OnlineSubspace/new 10000 8)
     v   (:wat::holon::encode (:wat::holon::to-holon "x"))
     lib (:wat::holon::EngramLibrary/new 10000)
     sub (:wat::holon::OnlineSubspace/new 10000 4)
     _r0 (:wat::holon::OnlineSubspace/update sub v)
     _u  (:wat::holon::EngramLibrary/add lib "alpha" sub)
     labels (:wat::core::Vector :wat::holon::HolonAST
              (:wat::holon::to-holon "up") (:wat::holon::to-holon "down"))
     r   (:wat::holon::Reckoner/new-discrete "rec" 10000 1 labels)
     _u1 (:wat::holon::Reckoner/observe r v 0 1.0)
     store (:wat::holon::Hologram/make (:wat::holon::filter-accept-any))
     _p  (:wat::holon::Hologram/put store (:wat::holon::leaf :a) (:wat::holon::leaf :b))]
    (:wat::core::do
      ;; ── subspace.rs, 10 ──
      (:probe::show "subspace/new"
        (:wat::eval-ast! (:wat::core::quote
          (:wat::core::apply :wat::holon::OnlineSubspace/new
            (:wat::core::Vector :wat::core::Any 10000 8)))))
      (:probe::show "subspace/dim"
        (:wat::eval-ast! (:wat::core::quote
          (:wat::core::apply :wat::holon::OnlineSubspace/dim (:wat::core::Vector :wat::core::Any s)))))
      (:probe::show "subspace/k"
        (:wat::eval-ast! (:wat::core::quote
          (:wat::core::apply :wat::holon::OnlineSubspace/k (:wat::core::Vector :wat::core::Any s)))))
      (:probe::show "subspace/n"
        (:wat::eval-ast! (:wat::core::quote
          (:wat::core::apply :wat::holon::OnlineSubspace/n (:wat::core::Vector :wat::core::Any s)))))
      (:probe::show "subspace/threshold"
        (:wat::eval-ast! (:wat::core::quote
          (:wat::core::apply :wat::holon::OnlineSubspace/threshold (:wat::core::Vector :wat::core::Any s)))))
      (:probe::show "subspace/eigenvalues"
        (:wat::eval-ast! (:wat::core::quote
          (:wat::core::apply :wat::holon::OnlineSubspace/eigenvalues (:wat::core::Vector :wat::core::Any s)))))
      (:probe::show "subspace/update"
        (:wat::eval-ast! (:wat::core::quote
          (:wat::core::apply :wat::holon::OnlineSubspace/update (:wat::core::Vector :wat::core::Any s v)))))
      (:probe::show "subspace/residual"
        (:wat::eval-ast! (:wat::core::quote
          (:wat::core::apply :wat::holon::OnlineSubspace/residual (:wat::core::Vector :wat::core::Any s v)))))
      (:probe::show "subspace/project"
        (:wat::eval-ast! (:wat::core::quote
          (:wat::core::apply :wat::holon::OnlineSubspace/project (:wat::core::Vector :wat::core::Any s v)))))
      (:probe::show "subspace/reconstruct"
        (:wat::eval-ast! (:wat::core::quote
          (:wat::core::apply :wat::holon::OnlineSubspace/reconstruct (:wat::core::Vector :wat::core::Any s v)))))

      ;; ── engram.rs, 10 (4 wrong-type-only: type unconstructible from wat) ──
      (:probe::show "Engram/name (wrong-type)"
        (:wat::eval-ast! (:wat::core::quote
          (:wat::core::apply :wat::holon::Engram/name (:wat::core::Vector :wat::core::Any 5)))))
      (:probe::show "Engram/eigenvalue-signature (wrong-type)"
        (:wat::eval-ast! (:wat::core::quote
          (:wat::core::apply :wat::holon::Engram/eigenvalue-signature (:wat::core::Vector :wat::core::Any 5)))))
      (:probe::show "Engram/n (wrong-type)"
        (:wat::eval-ast! (:wat::core::quote
          (:wat::core::apply :wat::holon::Engram/n (:wat::core::Vector :wat::core::Any 5)))))
      (:probe::show "Engram/residual (wrong-type)"
        (:wat::eval-ast! (:wat::core::quote
          (:wat::core::apply :wat::holon::Engram/residual (:wat::core::Vector :wat::core::Any 5 v)))))
      (:probe::show "EngramLibrary/new"
        (:wat::eval-ast! (:wat::core::quote
          (:wat::core::apply :wat::holon::EngramLibrary/new (:wat::core::Vector :wat::core::Any 10000)))))
      (:probe::show "EngramLibrary/add"
        (:wat::eval-ast! (:wat::core::quote
          (:wat::core::apply :wat::holon::EngramLibrary/add (:wat::core::Vector :wat::core::Any lib "beta" sub)))))
      (:probe::show "EngramLibrary/match-vec"
        (:wat::eval-ast! (:wat::core::quote
          (:wat::core::apply :wat::holon::EngramLibrary/match-vec (:wat::core::Vector :wat::core::Any lib v 5 5)))))
      (:probe::show "EngramLibrary/len"
        (:wat::eval-ast! (:wat::core::quote
          (:wat::core::apply :wat::holon::EngramLibrary/len (:wat::core::Vector :wat::core::Any lib)))))
      (:probe::show "EngramLibrary/contains"
        (:wat::eval-ast! (:wat::core::quote
          (:wat::core::apply :wat::holon::EngramLibrary/contains (:wat::core::Vector :wat::core::Any lib "alpha")))))
      (:probe::show "EngramLibrary/names"
        (:wat::eval-ast! (:wat::core::quote
          (:wat::core::apply :wat::holon::EngramLibrary/names (:wat::core::Vector :wat::core::Any lib)))))

      ;; ── reckoner.rs, 5 migrated (new-discrete/new-continuous/resolve stay BINDING) ──
      (:probe::show "Reckoner/observe"
        (:wat::eval-ast! (:wat::core::quote
          (:wat::core::apply :wat::holon::Reckoner/observe (:wat::core::Vector :wat::core::Any r v 1 1.0)))))
      (:probe::show "Reckoner/predict"
        (:wat::eval-ast! (:wat::core::quote
          (:wat::core::apply :wat::holon::Reckoner/predict (:wat::core::Vector :wat::core::Any r v)))))
      (:probe::show "Reckoner/curve"
        (:wat::eval-ast! (:wat::core::quote
          (:wat::core::apply :wat::holon::Reckoner/curve (:wat::core::Vector :wat::core::Any r)))))
      (:probe::show "Reckoner/labels"
        (:wat::eval-ast! (:wat::core::quote
          (:wat::core::apply :wat::holon::Reckoner/labels (:wat::core::Vector :wat::core::Any r)))))
      (:probe::show "Reckoner/dims"
        (:wat::eval-ast! (:wat::core::quote
          (:wat::core::apply :wat::holon::Reckoner/dims (:wat::core::Vector :wat::core::Any r)))))

      ;; ── hologram.rs, 2 migrated (make/put/get/find/remove stay BINDING) ──
      (:probe::show "Hologram/len"
        (:wat::eval-ast! (:wat::core::quote
          (:wat::core::apply :wat::holon::Hologram/len (:wat::core::Vector :wat::core::Any store)))))
      (:probe::show "Hologram/capacity"
        (:wat::eval-ast! (:wat::core::quote
          (:wat::core::apply :wat::holon::Hologram/capacity (:wat::core::Vector :wat::core::Any store)))))

      ;; ── control: the 8 that STAY BINDING still report the O-iv-a diagnostic ──
      (:probe::show "STILL-BINDING Reckoner/new-discrete"
        (:wat::eval-ast! (:wat::core::quote
          (:wat::core::apply :wat::holon::Reckoner/new-discrete (:wat::core::Vector :wat::core::Any "rec" 10000 1 labels)))))
      (:probe::show "STILL-BINDING Reckoner/resolve"
        (:wat::eval-ast! (:wat::core::quote
          (:wat::core::apply :wat::holon::Reckoner/resolve (:wat::core::Vector :wat::core::Any r 0.5 true)))))
      (:probe::show "STILL-BINDING Hologram/put"
        (:wat::eval-ast! (:wat::core::quote
          (:wat::core::apply :wat::holon::Hologram/put (:wat::core::Vector :wat::core::Any store (:wat::holon::leaf :c) (:wat::holon::leaf :d))))))
      (:probe::show "STILL-BINDING Hologram/make"
        (:wat::eval-ast! (:wat::core::quote
          (:wat::core::apply :wat::holon::Hologram/make (:wat::core::Vector :wat::core::Any (:wat::holon::filter-accept-any)))))))))
