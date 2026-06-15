# Arc 265 (STUB) — the acronym registry: restoring the disparate case (WebACL ⇄ web-acl ⇄ WebACL)

> **Status: STUB — banked** (captured 2026-06-14, builder: *"get the improvement queued up — we'll
> work on it after"*). Refines the arc-209 PascalCase⇄kebab naming tooling (the basic converter is
> the disciplined-subset floor; this makes it handle real-world acronyms). Builder's motivating case:
> *"a forever problem at AWS and it still exists — WebACL is always mishandled."*

## The problem

`pascal->kebab` is a **lossy projection**: `WebACL` and `WebAcl` collapse toward `web-acl`, and once
at `web-acl` the original casing is gone. No pure function can invert it — which is why
`PASCAL-KEBAB-CONVERSION.md` said "discipline the namespace (write `WebAcl`, not `WebACL`)." But you
don't control AWS's names; discipline fails for external identifiers. `WebACL → web-acl → WebAcl`
(wrong) is the forever-bug.

## The fix — a registry of acronyms (a *memory* of what the projection throws away)

The registry isn't a smarter function; it's a side-table of the casing the kebab form can't carry.

- **Forward** (`pascal->kebab`) handles acronyms *heuristically* — a run of capitals is one word:
  `WebACL` → `web-acl`, `WebACLRule` → `web-acl-rule`. No registry needed forward.
- **Reverse** (`kebab->pascal`) is where the info was lost — consult the acronym set: `web-acl` →
  `WebACL` iff `ACL` is registered (else `WebAcl`).
- **Round-trip restored** for any registered acronym: `WebACL ⇄ web-acl ⇄ WebACL`.

## Scoped (builder: *"a registry for correction in a scope"*)

Not a global hardcoded list — a set the **scope/domain owns**: AWS code registers `{ACL, HTTP, URL,
ARN, IAM, ID, …}`; another domain registers its own. The converter is parameterized by the registry
the caller supplies. This is the honest version (no universal acronym list could be right).

## Prior art (independent arrival — this is the established solution)

This is the canonical, solved approach: Rails `ActiveSupport::Inflector.acronym`, Go golint's
`initialisms` set, Python `inflection`. We reached it by chasing the WebACL problem to ground, not by
copying — recorded as a prior-art collision in arc-232 REALIZATIONS ("we land on the greats without
replicating them").

## Open design questions (a proper design pass, not a guess)

1. **Expand-time vs runtime.** The macro-callable `pascal->kebab` runs at *expand* time; runtime
   name-munging of external strings runs at *eval* time. A registry consulted at expand time vs eval
   time are different mechanisms. defservice op names are disciplined (no acronyms), so the basic
   macro path likely needs NO registry — the acronym-aware converter is probably a *runtime* variant
   (or a distinct acronym-aware fn). Decide whether it's one registry or two, and whether the
   acronym-aware reverse is even macro-reachable (likely not — so per OP-PLACEMENT it can be a wat
   helper, not an intrinsic).
2. **Scoping mechanism.** Global config vs a program-declared acronym set (the user.program/config
   pattern?) vs a registry value passed into the converter.
3. **Revisit `PASCAL-KEBAB-CONVERSION.md`** — its "discipline the namespace" bijection contract gets
   an escape hatch: the registry makes the round-trip total for registered acronyms even on
   un-disciplined external names.

## Out of scope until picked up
- The forward capital-run heuristic refinement of `pascal->kebab` (currently boundary-before-each-
   uppercase → `WebACL` becomes `web-a-c-l`; acronym-aware forward would make it `web-acl`). Decide
   at draw whether to refine the intrinsic or build the run-heuristic into the acronym-aware variant.
