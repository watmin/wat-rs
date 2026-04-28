;; :wat::holon::HologramLRU — bounded therm-routed coordinate-cell store.
;; Composes :wat::holon::Hologram (substrate) and :wat::lru::LocalCache
;; (wat-lru). When the LRU evicts a key, the matching Hologram entry
;; is dropped via Hologram/remove.
;;
;; Arc 076 + 077: filter is bound at make time on the inner Hologram;
;; slot routing happens inside Hologram (no caller-supplied pos);
;; capacity = floor(sqrt(dim-count)) is read from the program's
;; ambient EncodingCtx via Hologram/make.
;;
;; Surface mirrors Hologram's:
;;
;;   Hologram/make         (filter :fn(f64)->bool) -> Hologram
;;   HologramLRU/make      (filter :fn(f64)->bool) (cap :i64) -> HologramLRU
;;
;;   Hologram/put          store key val -> ()
;;   HologramLRU/put       store key val -> ()      ;; ALSO updates LRU + drops evicted
;;
;;   Hologram/get          store probe -> Option<HolonAST>
;;   HologramLRU/get       store probe -> Option<HolonAST>  ;; ALSO bumps LRU on hit
;;
;;   {Hologram,HologramLRU}/{len, capacity} — same shape

(:wat::core::struct :wat::holon::HologramLRU
  (hologram :wat::holon::Hologram)
  (lru :wat::lru::LocalCache<wat::holon::HolonAST,()>))

;; ─── Construction ────────────────────────────────────────────────
;;
;; The filter and capacity (= floor(sqrt(dim-count))) come from the
;; ambient EncodingCtx via the inner Hologram. `cap` is the LRU's
;; global bound — when exceeded, the least-recently-used entry is
;; evicted from the LRU AND from the Hologram. A reasonable starting
;; point is `dim-capacity * 100` for ~100 entries per slot on average.
(:wat::core::define
  (:wat::holon::HologramLRU/make
    (filter :fn(f64)->bool)
    (cap :i64)
    -> :wat::holon::HologramLRU)
  (:wat::holon::HologramLRU/new
    (:wat::holon::Hologram/make filter)
    (:wat::lru::LocalCache::new cap)))

;; ─── put — insert + LRU bookkeeping ──────────────────────────────
;;
;; 1. Insert (key, val) into the inner Hologram (slot routing is
;;    internal — the Hologram inspects the key).
;; 2. Push key→() onto the LRU (V is unit; LRU only tracks freshness
;;    by key).
;; 3. If the LRU evicted an entry, call Hologram/remove on the
;;    evicted key — slot routing inside Hologram drops the matching
;;    cell entry.
(:wat::core::define
  (:wat::holon::HologramLRU/put
    (store :wat::holon::HologramLRU)
    (key :wat::holon::HolonAST)
    (val :wat::holon::HolonAST)
    -> :())
  (:wat::core::let*
    (((h :wat::holon::Hologram) (:wat::holon::HologramLRU/hologram store))
     ((lru :wat::lru::LocalCache<wat::holon::HolonAST,()>)
      (:wat::holon::HologramLRU/lru store))
     ((_ :()) (:wat::holon::Hologram/put h key val))
     ((evicted :Option<(wat::holon::HolonAST,())>)
      (:wat::lru::LocalCache::put lru key ())))
    (:wat::core::match evicted -> :()
      ((Some pair)
        (:wat::core::let*
          (((evicted-key :wat::holon::HolonAST) (:wat::core::first pair))
           ((_ :Option<wat::holon::HolonAST>)
            (:wat::holon::Hologram/remove h evicted-key)))
          ()))
      (:None ()))))

;; ─── get — find + filter + LRU bump on hit ───────────────────────
;;
;; Hologram/find returns Option<(matched-key, val)> on a passing-
;; filter hit. Bump the matched key in the LRU (LocalCache::put
;; updates freshness on existing keys) and return Some(val). On
;; miss (filter rejected or empty bracket-pair), return :None.
(:wat::core::define
  (:wat::holon::HologramLRU/get
    (store :wat::holon::HologramLRU)
    (probe :wat::holon::HolonAST)
    -> :Option<wat::holon::HolonAST>)
  (:wat::core::let*
    (((h :wat::holon::Hologram) (:wat::holon::HologramLRU/hologram store))
     ((lru :wat::lru::LocalCache<wat::holon::HolonAST,()>)
      (:wat::holon::HologramLRU/lru store)))
    (:wat::core::match
      (:wat::holon::Hologram/find h probe)
      -> :Option<wat::holon::HolonAST>
      ((Some pair)
        (:wat::core::let*
          (((matched-key :wat::holon::HolonAST) (:wat::core::first pair))
           ((val :wat::holon::HolonAST) (:wat::core::second pair))
           ((_ :Option<(wat::holon::HolonAST,())>)
            (:wat::lru::LocalCache::put lru matched-key ())))
          (Some val)))
      (:None :None))))

;; ─── len — total entries across all slots ────────────────────────
(:wat::core::define
  (:wat::holon::HologramLRU/len
    (store :wat::holon::HologramLRU)
    -> :i64)
  (:wat::holon::Hologram/len
    (:wat::holon::HologramLRU/hologram store)))

;; ─── capacity — slot count of the inner Hologram ─────────────────
(:wat::core::define
  (:wat::holon::HologramLRU/capacity
    (store :wat::holon::HologramLRU)
    -> :i64)
  (:wat::holon::Hologram/capacity
    (:wat::holon::HologramLRU/hologram store)))
