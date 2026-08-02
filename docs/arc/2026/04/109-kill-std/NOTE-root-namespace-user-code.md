# ~~NOTE — user code may live in ROOT~~ · **SUPERSEDED 2026-08-02**

> ## ⛔ SUPERSEDED. The direction below was overturned by a builder ruling. Read this first.
>
> **The ruling (builder, 2026-08-02, verbatim):**
>
> > *"i no longer **accept** user symbols in the root namespace unless they are in args or lets or
> > similar … anything that's a def declaration must be namespaced … symbols may only be
> > un-namespaced when they are in use in some kind of dsl like… a match against enum argv or let
> > bindings or rete `?name` or… you get the idea… def's gotta be namespaced… **the root is for
> > scoped bindings**."*
>
> **The doctrine, stated positively — and it is a clean split, not a restriction:**
>
> | a name is… | may it be bare? | because |
> |---|---|---|
> | a **declaration** (`def`, `defn`, `defrecord`, `defenum`, `defsurface`, `defrule`, `typealias`, …) | **NO — must be namespaced** | it is a globally-visible registration in a shared world |
> | a **scoped binding** — fn args, `let`, `match` patterns, argv destructuring, rete `?vars`, any DSL binder | **YES — bare is its home** | it is LEXICAL; it never registers and cannot collide |
>
> **`::` means declared. Bare means scoped.** The root namespace is not a place to *install*
> things; it is where scoped bindings live.
>
> **This retires this note's direction entirely** — there is to be no root-namespace *resolution*
> path for top-level definitions, and `(wat.core/defn my-fn …)` calling bare `(my-fn)` is NOT the
> target. What the note got right and what survives: the **reserved set is exactly `wat` and
> `rust`** (confirmed on the disk, `src/resolve/reserved.rs:25-27`); every other namespace,
> including `:user::`, is free — `:user::` being *purposed* for rendezvous points, not privileged.
>
> **Built, not queued.** `72a1ac3d` armed `Registration::Unnamespaced` at the shared gate; the
> corpus was migrated by codemod the same day. The wall's own claim — *"args and let-bindings are
> lexical and never reach the gate, so there are no exceptions to carve"* — is exactly this ruling,
> and the ruling widens it: **every DSL binder is lexical too.** That the corpus went green
> (4266/4266) with the wall armed is the evidence that no binder reaches a registration door.
>
> *The original note is kept below, unedited. It was the builder's own queued direction and it was
> reversed by him; hiding it would erase the reversal, and the reversal is the record.*

---

## The original note, as filed (2026-07-21) — SUPERSEDED, do not act on

**Filed 2026-07-21 (surfaced mid arc-278 item (c), designing a marker for reactor-internal
service ops).** Queued, **NOT built** — a future refinement of the arc-109 FQDN doctrine, at
the builder's direction: *"we should allow installing things in root namespace… only `^wat` is
reserved… `^rust` is reserved for interop stuff… not a now thing."* Named-not-lost.

## The refinement (builder-stated)

Arc 109 settled the FQDN doctrine — *"the FQDN IS the namespace; verbosity is the design."*
This note **refines** it: the FQDN requirement is **reserved-tree-scoped**, not universal. Two
top-level namespaces are RESERVED; everything else — including the **root** — is user space:

| prefix | reserved for | examples |
|---|---|---|
| **`wat`** (`^wat`) | the substrate stdlib | `wat.core/`, `wat.kernel/`, `wat.type/`, `wat.rete/`, `wat.telemetry/`, `wat.service/`, … |
| **`rust`** (`^rust`) | Rust interop shims | `rust.sqlite/`, … (`:rust::*` interop verbs) |
| *(anything else, incl. root)* | **the user** | `my-fn`, `my.app/handler`, … |

So a user writes and calls bare, no ceremony:

```clojure
(wat.core/defn my-fn [] -> wat.type/i64 (wat.core/+ 2 2))
(my-fn)   ;; ⇒ 4  — resolves in ROOT; NO `user/` prefix required
```

The stdlib stays FQDN (verbosity IS the design, for the substrate's own tree — the reason the
doctrine exists: a reserved, unambiguous, collision-proof namespace for what ships in the box).
User code does **not** pay that tax: root/bare is theirs, and only a `wat`-or-`rust` prefix is
walled off. FQDN-verbosity for the substrate; ergonomic bare names for the user.

## Grounded current behavior (what needs to change)

Today **every** top-level definition requires an FQDN — root/bare does not resolve. Grounded
(builder's live probe, 2026-07-21):

```clojure
(wat.core/defn -name [] -> wat.type/nil …) (main-form calls (-name))
;; ⇒ #wat.runtime/UnboundSymbol {:message "unbound symbol: -name" …}   — bare name does NOT bind
(wat.core/defn user/-name [] -> wat.type/nil …) (… (user/-name))
;; ⇒ 4   — the SAME def, namespaced, resolves fine
```

The failure is not name-specific (the dash is a red herring — see the arc-278 cross-ref below);
a bare `main` (without `user/`) fails identically. The resolver requires an FQDN for every
top-level symbol; there is no root/bare resolution path.

## The direction (when it comes)

- Add a **root-namespace resolution path**: a bare top-level name (no `/` / no `::`) installs
  into and resolves from root.
- **Reserve the two prefixes** at the checker/resolver: a user `def` under a `wat`-rooted or
  `rust`-rooted FQDN is a located error (the substrate owns those trees); everything else is
  permitted, including root.
- Keep the stdlib exactly as-is (FQDN under `wat.*`) — this is purely *additive* headroom for
  user code, not a change to how the substrate names itself.

## Why it surfaced (cross-ref — arc 278 item (c))

Designing a marker for a **reactor-internal service op** (a timer-fired `:impls` arm no client
can call — arc-278 self-scheduling defservices), we reached for Clojure's leading-dash
convention (`-flush-tick`, the `gen-class` `-main` flavor: "the runtime invokes this"). The
builder's probe showed the dash *parses* fine in wat, but a bare `-name` needs an FQDN — which
felt wrong, and grounding revealed the real friction is **FQDN-everything forcing ceremony on
user-authored names**, not the dash. This note captures the doctrine fix; the arc-278 marker
decision is independent of it (a bare-name marker only reads clean once root/bare names are
first-class — so the two align, but the marker choice does not wait on this).
