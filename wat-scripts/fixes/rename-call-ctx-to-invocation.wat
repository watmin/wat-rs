;; wat-scripts/fixes/rename-call-ctx-to-invocation.wat — arc 278: the call context gets its ratified name.
;; Self-hosted fix-wat codemod: no hand-editing of .wat files — use the tool.
;;
;; Renames the per-dispatch context record from its PLACEHOLDER name to the ratified one:
;;   :wat::service::CallCtx  ->  :wat::service::Invocation
;;
;; WHY the rename (the intueri cast, 2026-08-09): `CallCtx` is a Level-2 mumble on two axes.
;; `Ctx` fails intueri's own carve-out — "ctx is acceptable when the TYPE speaks", and here `Ctx`
;; IS the type, so the abbreviation stands in for nothing. And the record braids three lifetimes
;; (caller-id is per-CONNECTION; request-id/start-ns are per-CALL; namespace/operation are
;; per-SERVICE constants), so no field-lifetime name can be honest. `Invocation` names the EVENT
;; instead of any one field's lifetime — and an invocation legitimately has a caller.
;; Ratified by the builder: "Invocation reads better than CallCtx."
;;
;; SURGICAL: `rename-keyword-prefix` is boundary-aware, so the ONE prefix rewrite covers both
;; the type head and every accessor built on it:
;;   (:wat::core::defrecord :wat::service::CallCtx  …)   the declaration
;;   (:wat::service::CallCtx  id ns op rid ns)           the ctor the macro emits
;;   (:wat::service::CallCtx/caller-id ctx)              the accessors — prefix + "/field"
;;
;; NOT renamed, deliberately: the user-facing BINDER stays `ctx`. intueri judged it separately
;; and it earns its brevity by SCOPE, not by mirroring the type — it is the fixed middle slot of
;; `[s ctx req]`, a positional role name exactly like `s` and `req`, in every opt-in arm of every
;; service. The type name and the binder do not have to track each other.
;;
;; NOT renamed: the FIELD `caller-id`. The docs (BRIEF/DESIGN-STONE/SEAM) all say `conn-id` and
;; the SEAM claimed a cast had ratified it, but the shipped code has said `caller-id` since
;; c8fcfe0d and the builder ruled the divergence in the code's favour — "code wins nearly every
;; time." The DOCS are what was stale; they are corrected alongside this migration.
;;
;; The codemod is idempotent (re-run = 0 changes). Kept in wat-scripts/fixes/ as the recorded
;; migration, alongside rename-record-def-to-defrecord.wat (the shape this copies).
;;
;; Usage (one EDN vector of paths on stdin — list EVERY path):
;;   printf '["wat/service.wat" "tests/services/probe_arc278_call_context.wat"]\n' \
;;     | cargo wat ./wat-scripts/fixes/rename-call-ctx-to-invocation.wat

;; Order matters: the QUALIFIED accessor first (":wat::service::Invocation/caller-id"), then the
;; bare kwarg keyword (":caller-id"). Reversed, the bare rename cannot reach the accessor (whose
;; keyword does not START with ":caller-id"), and the qualified rename would already have run.
(:wat::core::defn :user::migrate
  [src <- :wat::core::String] -> :wat::core::String
  (:wat::fix::rename-keyword-prefix ":caller-id" ":conn-id"
    (:wat::fix::rename-keyword-prefix ":wat::service::Invocation/caller-id" ":wat::service::Invocation/conn-id"
      (:wat::fix::rename-keyword-prefix ":wat::service::CallCtx" ":wat::service::Invocation"
        src))))

(:wat::core::defn :user::apply-each
  [paths <- (:wat::core::Vector :- [:wat::core::String])] -> :wat::core::nil
  (:wat::core::if (:wat::core::empty? paths)
    nil
    (:wat::core::let [path (:wat::core::first paths)]
      (:wat::core::do
        (:wat::io::write-file path
          (:user::migrate (:wat::io::read-file path)))
        (:wat::kernel::println (:wat::string::concat "[renamed] " path))
        (:user::apply-each (:wat::core::rest paths))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:user::apply-each
    (:wat::core::match (:wat::kernel::readln )
      ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum)
      (:wat::kernel::ReadlnOutcome::Eof (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None))
      (:wat::kernel::ReadlnOutcome::Stopped (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)))))
