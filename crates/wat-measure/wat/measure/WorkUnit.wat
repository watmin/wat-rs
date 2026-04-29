;; :wat::measure::WorkUnit — measurement-scope state surface.
;;
;; The Rust shim at :rust::measure::WorkUnit holds the four pieces
;; every scope tracks: counters (HashMap<Value, i64>), durations
;; (HashMap<Value, Vec<f64>>), `started: Instant`, and `uuid:
;; String`. Mutation is in place via ThreadOwnedCell — same Tier-2
;; zero-mutex pattern wat-lru's LocalCache uses.
;;
;; Slice 3 ships the data primitives:
;;   - WorkUnit::new                                     -> WorkUnit
;;   - WorkUnit/uuid       wu                            -> String
;;   - WorkUnit/incr!      wu (name :HolonAST)           -> ()
;;   - WorkUnit/append-dt! wu (name :HolonAST) (s :f64)  -> ()
;;   - WorkUnit/counter    wu (name :HolonAST)           -> i64
;;   - WorkUnit/durations  wu (name :HolonAST)           -> Vec<f64>
;;
;; Slice 4 will add WorkUnit/scope<T> (the HOF that opens a fresh
;; wu, runs body, computes elapsed, walks counters+durations to
;; build LogEntry::Metric rows, ships them through the consumer's
;; Service handles, returns body's value).
;;
;; The `!` suffix on incr!/append-dt! follows wat's mutation
;; convention (cf. :wat::core::set!). Reads have no suffix.
;;
;; Keys are HolonAST. A wat keyword like `:requests` becomes
;; HolonAST via `(:wat::holon::Atom :requests)`. Per arc 057 the
;; substrate's hashmap_key accepts any hashable Value, so the
;; runtime is permissive — the wat-level type discipline is what
;; keeps slice 4's edn-write-notag rendering clean.
;;
;; Arc 091 slice 3.

(:wat::core::use! :rust::measure::WorkUnit)

(:wat::core::typealias :wat::measure::WorkUnit :rust::measure::WorkUnit)


;; A single tag's K,V shape. Aliased so type signatures that
;; need to name the pair (e.g. `:Vec<wat::measure::Tag>` for a
;; tag-list before HashMap-ification) read cleanly.
;;
;; Note: `:wat::core::HashMap`'s constructor checks its first
;; argument as a LITERAL tuple form `:(K,V)` and does NOT expand
;; typealiases at that site (the check is form-level, not
;; type-system-level). So at HashMap construction the verbose
;; `:(wat::holon::HolonAST,wat::holon::HolonAST)` is required;
;; the alias still serves declarations elsewhere.
(:wat::core::typealias :wat::measure::Tag
  :(wat::holon::HolonAST,wat::holon::HolonAST))


;; The wu's tag map shape — arbitrary HolonAST→HolonAST pairs that
;; ride on every emitted Event row as a queryable EDN map. Aliased
;; here per arc 077's "nested generics get a typealias" convention
;; so the verbose `:HashMap<wat::holon::HolonAST,wat::holon::HolonAST>`
;; doesn't smear across the WorkUnit + Event surface.
(:wat::core::typealias :wat::measure::Tags
  :HashMap<wat::holon::HolonAST,wat::holon::HolonAST>)


(:wat::core::define
  (:wat::measure::WorkUnit::new
    (tags :wat::measure::Tags)
    -> :wat::measure::WorkUnit)
  (:rust::measure::WorkUnit::new tags))


(:wat::core::define
  (:wat::measure::WorkUnit/uuid
    (wu :wat::measure::WorkUnit) -> :String)
  (:rust::measure::WorkUnit::uuid wu))


(:wat::core::define
  (:wat::measure::WorkUnit/incr!
    (wu :wat::measure::WorkUnit)
    (name :wat::holon::HolonAST)
    -> :())
  (:rust::measure::WorkUnit::incr wu name))


(:wat::core::define
  (:wat::measure::WorkUnit/append-dt!
    (wu :wat::measure::WorkUnit)
    (name :wat::holon::HolonAST)
    (secs :f64)
    -> :())
  ;; The Rust shim's path mirrors its Rust ident verbatim — the
  ;; #[wat_dispatch] macro uses `method.sig.ident` directly, so the
  ;; path is `append_dt` (underscore), not `append-dt` (kebab). The
  ;; wat-side wrapper here owns the kebab name; the rust path is
  ;; an internal detail.
  (:rust::measure::WorkUnit::append_dt wu name secs))


(:wat::core::define
  (:wat::measure::WorkUnit/counter
    (wu :wat::measure::WorkUnit)
    (name :wat::holon::HolonAST)
    -> :i64)
  (:rust::measure::WorkUnit::counter wu name))


(:wat::core::define
  (:wat::measure::WorkUnit/durations
    (wu :wat::measure::WorkUnit)
    (name :wat::holon::HolonAST)
    -> :Vec<f64>)
  (:rust::measure::WorkUnit::durations wu name))


;; ─── Slice 4 accessors — read state needed by WorkUnit/scope ────

(:wat::core::define
  (:wat::measure::WorkUnit/started-epoch-nanos
    (wu :wat::measure::WorkUnit) -> :i64)
  ;; Rust ident `started_epoch_nanos`; the macro registers with
  ;; underscore (cf. slice-3's append_dt). The wat-side keeps the
  ;; kebab name.
  (:rust::measure::WorkUnit::started_epoch_nanos wu))


(:wat::core::define
  (:wat::measure::WorkUnit/counters-keys
    (wu :wat::measure::WorkUnit) -> :Vec<wat::holon::HolonAST>)
  (:rust::measure::WorkUnit::counters_keys wu))


(:wat::core::define
  (:wat::measure::WorkUnit/durations-keys
    (wu :wat::measure::WorkUnit) -> :Vec<wat::holon::HolonAST>)
  (:rust::measure::WorkUnit::durations_keys wu))


;; ─── Tags — the third concern, IMMUTABLE for the scope ─────────
;;
;; Tags are declared upfront at WorkUnit::new and are immutable
;; for the scope's lifetime. There is NO assoc/disassoc — every
;; Log line emitted in the scope must carry the same tag set so
;; rows correlate via a stable queryable shape. The user can read
;; the map natively with `:wat::core::get`, `:wat::core::keys`,
;; etc. — no per-key accessor needed.
;;
;; The map serializes to the SQL `tags` column as a clean EDN
;; map: `{:asset :BTC, :stage :market-eval}`. Slice 4 picks the
;; field-type shape that drives that rendering.

(:wat::core::define
  (:wat::measure::WorkUnit/tags
    (wu :wat::measure::WorkUnit) -> :wat::measure::Tags)
  (:rust::measure::WorkUnit::tags wu))
