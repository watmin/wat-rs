# Arc 278 — the wat rules engine (RETE / Clara-shaped, VSA-matched)

> **STATUS: STUB — queued, BLOCKED behind arc 277.** Surfaced 2026-06-17 the moment `wat-lint`'s rule
> abstraction (`form → findings`, a registry, run over a fact base) was recognized as a *production-rules
> system*. The builder: *"did we just declare the need for something like Clara?… wat-rules basically
> just described itself."* The plan, set explicitly: **once `wat-lint` (arc 277) is done and proven
> working, build the RETE/Clara engine and swap `wat-lint` onto it.** Not started; do NOT start before
> 277 lands.

## The trigger

`wat-lint` (arc 277) defines a rule as `(form → Vector<Finding>)` — an `LHS` pattern (match a form) →
`RHS` action (emit a finding/fix), a registry of rules, run over a fact base (the parsed forms). That is
the production-rules paradigm, named. The current `wat-lint` impl is *rule-based* (map the rule set over
the forms) but **not** a rules-*engine* — no RETE network, no incremental re-matching, no rule **chaining**
(one rule's output feeding another's LHS), no fact base richer than "the forms." Arc 278 builds the
engine; `wat-lint` is its proving ground and first consumer.

## Prior art (collision, recorded straight — and it is the builder's OWN tool)

- **Clara** — the Clojure RETE rules engine the builder ran **at AWS** for the Shield DDoS pipeline (per
  BOOK.md / the chronicle: "Clojure for the Kinesis KCL interop and the rule engine, Clara"). The
  reach-and-find pattern at career scale: reached for a clean way to organize lint rules, landed on a
  great he'd already shipped in production.
- **Drools / CLIPS** — the JVM and the classic production-rules systems.
- **Forgy's RETE (1982)** — the incremental pattern-matching algorithm underneath all of them: a
  discrimination network that re-evaluates only what changed, so N rules over M facts is not N×M per tick.
- The coordinate recurs across the builder's work: Clara @ AWS → the **eBPF tail-call rule-trees** in
  `holon-lab-ddos` (1M rules in O(tree depth)) → `wat-lint` rules over AST. A place he stands.

## What is genuinely ours (the two reasons to build it, not just import the idea)

1. **Homoiconic rules.** wat is EDN — a `defrule` is *data* natively (quote/unquote, the same tongue you
   compute in). Clara's rules are Clojure data too, but ours are AST-native and live in the substrate
   that already self-rewrites (`fix.wat`) and self-analyzes (`deporder`). The rules engine is one more
   self-hosted tool, configured in the language it runs on.
2. **The VSA-matched LHS — the real novelty.** Swap RETE's *exact* pattern match for **coincidence**
   (`coincident?`, similarity over a floor) as the matcher: rules fired by *similarity*, not equality —
   a **fuzzy / probabilistic production-rules engine**, the holon substrate as the fact network, `bundle`
   as the working memory, `cosine` as the match. That is not Clara; it is Clara's matcher replaced by the
   thing holon already is. (This is the deep one — note it; the exact-match engine ships first.)

## The plan (the swap — author-adjacent / prime-drop)

1. **277 first.** `wat-lint` ships and is *proven working* (rules catch real bad forms across the corpus,
   `wat-fix` applies, `wat-fmt` formats). This proves the rule abstraction is correct before any engine
   is built — the proving ground.
2. **Build the engine beside the working lint.** A general `defrule` + RETE matcher (exact-match first),
   rule chaining, an incremental fact network. It does NOT touch `wat-lint` yet — it sits proven on its
   own deftests, primed.
3. **Swap `wat-lint` onto it.** The naive map-over-rules retires; lint's rules re-home onto the engine
   (their `(form → findings)` shape is already the engine's rule shape, so the migration is mechanical).
   `wat-lint` becomes the engine's first real consumer; the rule-of-three is satisfied by construction.
4. **Then the other consumers.** `deporder`, the DDoS detection lab (the eBPF rule-trees' wat sibling),
   and the verification market — each a fact base + a rule set over the one engine.

## Out of scope / discipline

- **Do NOT build before 277 is proven.** The whole point of the sequence is that lint earns the
  abstraction empirically first; building the engine speculatively is exactly the forcing-function the
  project forbids. Blocked, on purpose.
- **Exact-match RETE first; VSA-matched second.** The fuzzy/coincidence matcher is the novel horizon, not
  the v1 — ship the deterministic engine, prove the swap, then explore VSA-fired rules.
- **No config** (per the toolchain doctrine, arc 277): the engine has one correct behavior; rules are
  data, suppression is a rune, not a knob.

## Four questions (sketch, to weigh when it opens)

- **Obvious?** A `defrule` reads as `LHS → RHS`; the engine runs rules over facts and fires actions —
  the production-rules paradigm everyone in the lineage already knows.
- **Simple?** RETE is *not* simple — it earns its complexity only at scale (incremental, chaining,
  many-rules). The discipline: build it when a consumer's scale demands it (lint at corpus scale + a
  second/third consumer), not before. The naive engine is simpler and correct for small rule sets.
- **Honest?** Mark the seam — what we have today is rule-based mapping; the engine is RETE; do not call
  the map an engine until the network exists.
- **Good UX?** One `defrule` surface, homoiconic; lint/deporder/DDoS all consumers of one engine; the
  fuzzy matcher (if built) makes "rules over similarity" a first-class, novel affordance.
