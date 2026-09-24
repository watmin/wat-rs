;; Stone 255.25 (C-b4) — the markers ALONE are pure: `:wat::kernel::Transport` is a declared
;; `:wat::enum::Pure` family, and each variant is a type the moment its enum is (255.25a).
;; Pre-stone the markers were empty `defstruct`s, and a struct is impure: `:wat::kernel::Shared`
;; as a `Record` field was refused. The control that this is read from the DECLARATION (not the
;; "unknown path => type parameter => pure" arm) is _impure_variant_field.wat.bad.
(:wat::core::defrecord :probe::HoldsMarkers
  [shared    <- :wat::kernel::Transport.Shared
   wire      <- :wat::kernel::Transport.Wire
   transport <- :wat::kernel::Transport])
