;; Arc 255 (`DESIGN-the-composition-door-gets-a-wat-surface`, BRIEF-compose-variant) —
;; smoke probe: any wat program forces the whole `wat/*.wat` stdlib (incl. `wat/service.wat`)
;; to load and type-check. Trivial body; the point is that `--check` on THIS file must
;; succeed, proving the corpus loads after `:wat::runtime::compose-variant` landed and the
;; thirteen `wat/service.wat` string-interpolate sites were rewritten to use it.
(:wat::core::+ 1 1)
