# Arc 250 — vigilatum-integrity: self-enforcing ward stamps — STUB

**Status:** ⏸ **STUBBED 2026-06-04.** Intent banked; **design OPEN** — the *what* and *how* are not yet settled. This stub captures the problem + the open questions, NOT a plan. (The arc name `vigilatum-integrity` and the check's name are **provisional** — intueri-cast them when the design settles, per `feedback_intueri_names_all_things`.)

## The problem — a `vigilatum` stamp is an unenforced CLAIM

A warded home carries `//! vigilatum: <ts> — vigilia 8-spell L1+L2=0, clippy-clean in-home`. That line is a **claim**, not a guarantee. Nothing re-checks it. It can drift silently: the home's invariants break (a duplicate reappears, clippy regresses, a moved fn comes back), the stamp keeps asserting "warded," and the build stays green. **A stamp that can go false without failing the build is a lie waiting to happen.**

**Worked disaster (2026-06-04, arc 246.2 — the thing that birthed this stub):** `src/collection/` was warded to L1+L2=0 and stamped. Then a `cargo clippy --fix` revert clobbered the razing — the 23 duplicate `*_inner` bodies and the dispatch fork came **back** — the stamp went false, the build stayed green (clippy + suite are blind to a dispatch fork), and the false ward was *committed*. Only a lucky post-commit re-grep of the invariant caught it. The stamp lied; nothing flinched.

**Precedent (reactive, not preventive):** task #168 (ward-integrity clippy sweep — made the 7 stamps TRUE *once*) + #170 (`result_large_err` stamp-drift, fixed *once*). Those fix drift after it's found. This arc is the **structural** version: the stamp re-verifies itself.

## The intent

Make `vigilatum` **self-enforcing**: a gate re-verifies each stamped home's ward-invariants on **every build/test run**, and **drift FAILS LOUD**. Then "warded" stays true by construction — a reappearing duplicate (by *any* cause: agent clobber, future edit, a careless revert) is caught at the next build, not by the orchestrator's luck. This is the layer-4 annihilation in `feedback_commit_milestones_and_invariant_gates` (layers 1–3 are behavioral and already in force).

## What's UNCLEAR — the open questions (settle at design; do NOT resolve here)

1. **The invariant set + how a home declares it (machine-readable).** `clippy-in-home == 0` is clear. "No duplicated bodies" is the load-bearing one — *how is it expressed?* A manifest of the home's `pub(crate) fn` names + an assertion none of them are also `fn`-defined in the flat files (`runtime.rs`/`check.rs`)? A content-hash? Something else? What is the *complete* invariant set a stamp promises?
2. **The enforcement mechanism.** Candidates, each with tradeoffs: a `cargo test` that greps/asserts; a `build.rs`; a custom clippy lint; a CI step; an `xtask`; a wat-substrate self-check. Which? (It must run where a normal `cargo test`/build runs, or it's another un-run gate.)
3. **Where the invariants are declared.** In the stamp comment itself (a parseable form)? A sidecar manifest per home? A central registry of stamped homes?
4. **Per-home vs a global ward-registry.** One self-check per home, or a central enforcer that knows every stamped home and checks them all?
5. **The names.** The check/tool is a new thing → intueri-cast it at open. The arc's real name too (`vigilatum-integrity` is a placeholder).

## Why it matters

User 2026-06-04: *"we do not shy away from hard work — but we dislike having to deal with the same issue twice — how do we prevent this?"* The behavioral layers start now; **this arc is the structural one** — it turns "deal with it twice" into "can't recur silently." A stamp that cannot go false without screaming is the only honest stamp.

## Enabled-by / priority

Not blocked — a meta/tooling arc, independent of the substrate spine. **NOT in the pre-232 resumption gate** (235/245/246/249); a separate banked arc, **priority = builder's call** (open it when a stamp-drift next bites, or when the spine clears). See `feedback_commit_milestones_and_invariant_gates` + `feedback_warded_means_annihilated`.
