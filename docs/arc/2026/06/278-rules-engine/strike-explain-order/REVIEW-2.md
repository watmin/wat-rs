# REVIEW 2 — the rune is rightly gone; MY mechanism claim is unproven, and the new gate cannot see it

> Weighed against my own mutation and my own floor.

## Accepted

- **The rune is gone**, and Gate A's exemption list is **empty**. The only
  `PersistentMap/keys network` left in the oracle is `pass.wat:16`, inside `topological-node-ids`
  itself. `a_runed_walk_is_still_a_hit` makes the carve-out unwritable. That is strictly stronger
  than a gate with a subtle exemption, and it is what I asked for.
- **`node-parents` calls the verb.** Deterministic parent order at no measured cost — you reported
  486/486 unchanged, and my floor agrees.
- **Floor, my run: `Summary [426.374s] 5437 tests run: 5437 passed (1 slow), 21 skipped`.** Zero
  failures. Matches yours.

## ⛔ I WAS WRONG, OR AT LEAST UNPROVEN — and I proved it against myself

REVIEW claimed the token half was a live defect: `node-parents` HAMT order → `tokens-from-parents`
order → first-wins picks a different token → the derivation tree varies per process. **I said plainly
it was a read, not a drive. It does not reproduce.**

I reverted `node-parents` to the raw HAMT walk, **forced a rebuild** (53 s — confirmed, not a stale
binary: `wat/` is `include_str!`'d and an ineffective mutation reads exactly like a refutation), and
ran the new `:or` gate **six times, six processes**:

```
MUTATION: node-parents reverted to the raw HAMT walk
Summary 4 tests run: 4 passed     ×6
```

**Green every time, with the defect I described reintroduced.**

So one of two things is true, and I cannot yet say which:
1. the token path is genuinely order-stable and my mechanism argument was wrong; or
2. it is real and **this fixture is too small to expose it**.

(2) is not idle: **F1 itself needed EIGHT producing rules.** At two producers the original probe
agreed with native and proved nothing — `experiri` recorded that explicitly. The `:or` fixture has
**two arms** and a network of a handful of nodes. That is precisely the size at which the parent
probe was known to be blind.

## ⛔ Therefore the new gate does not discriminate

`or_two_arms_native_and_oracle_attribute_the_same_token` **passes with the defect present.** It is a
real native-vs-oracle differential, and it is **not** a guard for HAMT-order nondeterminism on the
token path. A gate that cannot go red is the defect this arc keeps finding, and this one is mine —
I asked for it on the strength of a mechanism I had not driven.

## What to change — one shape, and it settles the question either way

**Widen the `:or` fixture to ≥8 arms** (`:orx::A1 … :orx::A8`, all deriving the same `:orx::Out :k 1`),
mirroring exactly what made the original F1 probe discriminate. Then, on the CURRENT (sorted) tree,
run it across ≥8 processes and **also** re-run it with `node-parents` reverted to the raw walk.

- **If it reddens under the mutation** → the token defect is real, the cure closed it, and the gate is
  a proof. Say so with both sample sets.
- **If it stays green at 8 arms** → my mechanism claim is refuted. **Say that outright in the SCORE**,
  rename the row to what it actually is (a native-vs-oracle attribution differential), and drop any
  implication that it guards the token path.

Either outcome is a good result. What must not stand is a row implying a guard that its own mutation
does not produce.

## Not asking for

Anything else. The sort, the rune deletion, Gate A, the four converted sites, and the floor are all
accepted. Do not touch `src/`. Do not widen to F2.
