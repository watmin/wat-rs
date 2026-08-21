# DESIGN STONE — 299.3-entropy-is-a-category · `:Clock` was the DEVICE. Rename it `:Entropic`.

## ⛔ THIS REDIRECTS 299.3 AS DRAWN. Read this section first.

`299/DESIGN.md` scopes stone **299.3** as *"refine `Purity` → `Pure | Effectful | Entropic`; tag the
entropy sources; derive entropy transitively — **HARD, 23 files**."* **This stone does not do that,
and the redirect is the builder's**, on the argument recorded below. Three measurements support it:

1. **299 R1 cites the wrong `Purity`.** R1 reads *"`Purity` today is `Pure | Impure`"*. There are TWO
   types with that name: `types::Purity` (`Pure | Impure` — the `:wat::enum::` marker on a **defenum**,
   about whether a DATA TYPE crosses address spaces, 293.W) and `wat_doc::Purity` (a **verb's**
   declared `@Purity`). R1's content is entirely about verbs — *"disk, network, stdio"* vs
   *"time, random, pid"* — so it means the latter and quotes the former. The "HARD, 23 files"
   estimate belongs to `types::Purity`, which is threaded through `is_pure_type`, the containment
   pass and the rete axis. It was never this axis's number.

2. **Half the split had already landed before 299 was drawn.** At `e172a423` — the commit that wrote
   299's DESIGN, 2026-07-02 — `wat_doc::Purity` was ALREADY `Pure | Effectful | Preserving`. R1's
   diagnosis (*"Impure fuses effect and entropy"*) was true of a world that, on the verb axis, had
   already been half-dissolved. `Effectful` was split out. What remained was never a purity variant.

3. **The substrate already ruled where source-naming lives — in `:Clock`'s own prose**, written during
   255.1c-taxonomy: *"Names WHICH external source a Nondeterministic verb draws from — **which
   `:Determinism` alone cannot say**."* `Category` is the axis that names the source. Putting
   `Entropic` on `Purity` would encode *where the value comes from* onto the purity axis — the same
   axis `Category`'s own header lists as REJECTED, and no better one variant over.

**So: `Entropic` does NOT eat `Nondeterministic`, and the two do not awkwardly coexist. The
distinction is a `Category`, and `Purity`/`Determinism` are untouched by this stone.**

## THE BUILDER'S ARGUMENT — `:Clock` names a DEVICE

> *"Clock is a bad label then.. its a measure of entropy as much as random is.. calling Time.now and
> SecureRandom.uuid are the same category.. they are a syscall who is 'pure'"*
>
> *"and println /is/ an IO here... it effects the world"*

Demonstrated at a Ruby REPL: two `Time.now` calls and two `SecureRandom.uuid` calls, each pair
differing, each a syscall, none with an observable effect. **The wall clock is an entropy source
exactly as `/dev/urandom` is.** `:Clock` named *which device* — and naming the device is the
over-fine cut this taxonomy has refused twice already: the transport was refused as `:Message`'s axis
(*"in-process channel, pipe, socket — an implementation detail, NEVER the axis"*), and in-place-vs-
fresh was refused as `:Mutate`'s. **`:Clock` is that same defect, shipped.**

Its own prose even contains the correction, one sentence later: *"Entropy gets its own variant when a
random verb actually registers; do not widen this to cover it."* That reserved a SECOND variant for a
verb whose DOING is identical. The builder has overruled the second half: **widen it — they are one.**

## The resulting grid — all four cells, nothing new required

```
              DETERMINISTIC                     NONDETERMINISTIC
  no effect   pure computation                  :Entropic  time::now · Uuid/v4
              Purity=Pure                       Purity=Pure          — samples; effects nothing
                                                                     — measure by CONFORMANCE

  effect      :Io   println · pprintln          :Io         readln' · read-frame
              Purity=Effectful                  Purity=Effectful     — the world hands you DATA
                                                                     — measure by INJECTION
```

`:Io` is *touches the world, either direction*; `:Entropic` is *samples an unpredictable source and
effects nothing*. `readln'` and `time::now` — the two cells our own docs have been conflating —
differ in **Category**, and always could have. The six stdio rows already carry `:Io` correctly
(home #3); **this stone changes no `@Purity` and no `@Determinism` anywhere.**

## The one contract decision, pinned

**RENAME `:Clock` → `:Entropic` in place. Do not add a variant and retire the old one.**

`Category` is closed and append-only — but that rule protects **ordinals** (T2: *"inserting one
mid-list renumbers the generated enum"*). A rename preserves all fifteen positions and every
ordinal; add-and-retire would strand a dead `:Clock` slot permanently, because a closed enum cannot
drop a variant. The rename is the only move that leaves the enum honest.

**Name: `:Entropic`, the ADJECTIVE — a verb so tagged PRODUCES AN ENTROPIC VALUE.** `:Clock`'s prose
reserved the slot under the noun *"Entropy"*; the builder corrected the form: *"I think Entropic is
the name, not Entropy?.. these produce entropic values?"* — and that is right on two counts. It is
consistent with `:Ambient`, already an adjective in this same enum; and it names the verb's OUTPUT,
which is the axis (`Category` classifies what the verb DOES), where a bare noun names the substance
sampled. It also matches the arc's own name — 299 is `entropic-values`. `:Sample` and `:Entropy`
were considered and rejected: the first invents a word the record does not use, the second is the
noun the builder corrected.

## Blast radius — measured, and it is the 255.1c-taxonomy shape

```
wat/runtime-meta.wat        :127  the variant + its prose (REWRITTEN, see below)
                            :70   the header's variant list
                            :165  `:Ambient`'s prose says "NOT `:Clock`" — must follow the rename
crates/wat-macros/src/wat_intrinsic.rs:380      one match arm
crates/wat-macros/src/wat_special_form.rs:84    one match arm
crates/wat-doc/src/lib.rs:71,1159,1170          CATEGORY_LEGAL_VALUES · the gate's `all` · its match
src/intrinsic/time.rs                           17 rows: @Category Clock → Entropic
```

**Nothing else.** Measured: no `.edn` golden, no test fixture, and no `.wat` outside `runtime-meta`
pins the string `Clock` as a Category. `Uuid/v4` is NOT registered — it is still a literal arm at
`runtime.rs:5947` — so it inherits `:Entropic` for free whenever it carves, and this stone does not
carve it.

## The prose to ship, replacing `:Clock`'s

```
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
```

## The four questions

- **Obvious?** YES — one variant renamed to the name its own comment reserved.
- **Simple?** YES — one axis, one rename, no row's purity or determinism moves.
- **Honest?** YES. It also retires a shipped mis-cut rather than building over it, and it says so in
  the prose instead of quietly renaming.
- **Good UX?** YES — `metadata-of` stops telling a caller that `now` is a clock thing and `Uuid/v4`
  would be some other thing; 299.4's mode-derivation gets a single tag to key on.

## Out of scope — affirmative cuts, homes named

- **Carving `Uuid/v4` into the registry.** It is a literal arm; carving it is a registry stone, and it
  is the verb whose registration `:Clock`'s prose named as the admission condition. It inherits
  `:Entropic` when it lands.
- **`@Purity Entropic`.** Redirected by this stone, with the argument recorded above. Arc 299's
  DESIGN keeps its text — what is designed is designed; this stone's redirect lives here.
- **The `cannot-world-fault` weld** — `278/REALIZATIONS.md:9155` records the builder's contribution
  that *"entropic IO gets NO outcome enum; only failing IO does."* That is a WALL, not a label, and it
  is the structural discriminator between the two nondeterministic cells. Named here; not built here.
- **The three conflating comments.** `kernel_stdio.rs:36-39` justifies `readln'`'s nondeterminism as
  *"exactly as `:wat::time::now` reading the wall clock does"*, and `kernel_ambient.rs` repeats it.
  Those sentences are now known-wrong — the two ARE different cells. In scope for the rider as a
  correction, since this stone is what makes them false.
