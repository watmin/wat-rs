//! Fire loop: alpha/root/hash/production passes, leftover rematch, delta fixpoint.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use rustc_hash::FxHashMap;

use crate::ast::WatAST;
use crate::rete::compiled_cond::{BindIntern, ValIntern};
use crate::rete::matcher::Bindings;
use crate::runtime::{EvalBreak, RuntimeError, RuntimeErrorKind, SymbolTable, Value};
use crate::span::Span;
use crate::types::Nature;
use crate::value::value::AggregateValue;

use super::*;

/// Split-borrow of `FireSession`. Token/Element are Copy; we cannot
/// hold `&mut FireSession` while walking beta/alpha. Facts stay out of
/// the bind intern (`DESIGN-STONE-fact-as-index`).
pub(crate) struct FireCtx<'a> {
    pub(crate) compiled_conds: &'a HashMap<i64, crate::rete::compiled_cond::CompiledCond>,
    pub(crate) scratch: &'a mut SlotFrame,
    pub(crate) pool: &'a mut Vec<(u32, u32)>,
    pub(crate) match_pool: &'a mut Vec<(u32, i64)>,
    pub(crate) keys: &'a [Value],
    pub(crate) vals: &'a [Value],
    pub(crate) val_ids: &'a crate::rete::compiled_cond::ValIntern,
    pub(crate) facts: &'a Value,
    pub(crate) derived: &'a [Value],
    pub(crate) n_input: u32,
    pub(crate) i64_by_fact: &'a [Option<I64Row>],
    pub(crate) bind_only: &'a HashMap<i64, Vec<u8>>,
    pub(crate) cond_key_ids: &'a CondKeyIds,
}

// ── Pass 1: Alpha pass ────────────────────────────────────────────────────────

/// `activate-alpha` + `activate-fact` — type-index each fact,
/// `exec_compiled_with_key_ids` against that type's alphas. Mirrors
/// `wat/rete/oracle/pass.wat`. A missing compiled cond refuses — do not walk
/// `alpha_match_inner`.
#[cfg(test)]
pub(crate) fn alpha_pass(wm: &mut FireSession, arm: &InternedNetwork) -> Result<(), EvalBreak> {
    let mut match_scratch: SlotFrame = Vec::with_capacity(arm.compiled_max_slots);
    let mut cand_scratch: Vec<i64> = Vec::new();
    let mut cond_key_ids: CondKeyIds = HashMap::new();
    for (&id, c) in &arm.compiled_conds {
        cond_key_ids.insert(
            id,
            crate::rete::compiled_cond::intern_cond_keys(c, &mut wm.bind_keys),
        );
    }

    let input = match &wm.facts {
        Value::wat__core__PersistentVector(pv) => pv.clone(),
        _ => return Ok(()),
    };
    wm.n_input = input.len() as u32;

    for (i, fact) in input.iter().enumerate() {
        let (fact_class, fact_fields) = match fact {
            Value::Aggregate(a) if a.nature != Nature::Struct => {
                (a.class.as_ref(), a.fields.as_slice())
            }
            _ => continue,
        };
        arm.alpha_tree
            .candidates_into(fact_class, fact_fields, &mut cand_scratch);
        for aid in cand_scratch.iter().copied() {
            let compiled = rematch_compiled(&arm.compiled_conds, aid)?;
            let Some((off, len)) = crate::rete::compiled_cond::exec_compiled_with_key_ids(
                compiled,
                fact_fields,
                &mut match_scratch,
                &mut crate::rete::compiled_cond::BindIntern {
                    keys: &mut wm.bind_keys,
                    vals: &mut wm.bind_vals,
                    ids: &mut wm.bind_val_ids,
                    pool: &mut wm.bind_pool,
                },
                fact,
                cond_key_ids.get(&aid).map(|v| v.as_slice()),
            ) else {
                continue;
            };
            let el = make_element(i as u32, off, len);
            Arc::make_mut(wm.alpha.entry(aid).or_default()).push(el);
        }
    }
    Ok(())
}

// ── Pass 2: Root-join pass ────────────────────────────────────────────────────

/// `root-join-pass` / `seed-root-join-children` / `seed-token` / `append-token` —
/// for each AlphaNode with Elements, seed one Token per Element into each RootJoinNode child's beta.
/// Mirrors `wat/rete/oracle/pass.wat`.
#[cfg(test)]
pub(crate) fn root_join_pass(wm: &mut FireSession) {
    let node_ids = sorted_node_ids(&wm.network);

    for node_id in &node_ids {
        // Group C: use &Value ref from wm.network; borrow ends after node_children (NLL).
        let node = match get_node(&wm.network, *node_id) {
            Some(n) => n,
            None => continue,
        };
        if kind_of(node) != NodeKind::Alpha {
            continue;
        }
        let child_ids = node_children(node);
        // node's last use is node_children above; wm.network borrow ends here (NLL).

        // Group C: borrow elements from wm.alpha — wm.beta mutations below are on a different field.
        let elements = match wm.alpha.get(node_id) {
            Some(els) => els.as_slice(),
            None => continue, // no elements → skip
        };

        for child_id in &child_ids {
            // Group C: child_node ref — only used for kind_of; borrow ends before wm.beta mutation.
            let child_node = match get_node(&wm.network, *child_id) {
                Some(n) => n,
                None => continue,
            };
            if kind_of(child_node) != NodeKind::RootJoin {
                continue;
            }
            // Seed one native Token per Element into beta[child_id].
            for el in elements {
                // Support edge: (fact idx, alpha-id). Mirrors seed-token (wat:544-551).
                // Share the Element span — no PMap (`DESIGN-STONE-token-bind-pool`).
                let tok = Token {
                    matches: push_match(&mut wm.match_pool, el.fact, *node_id),
                    binds: seed_token_binds(el),
                };
                wm.beta.entry(*child_id).or_default().push(tok);
            }
        }
    }
}

pub(crate) fn driver_of(
    drivers: &HashMap<i64, CondDriver>,
    alpha_id: i64,
) -> Result<&CondDriver, EvalBreak> {
    drivers.get(&alpha_id).ok_or_else(|| {
        RuntimeError::new(
            crate::rust_caller_span!(),
            RuntimeErrorKind::MalformedForm {
                head: ":wat::rete::fire-rules".into(),
                reason: format!(
                    "alpha {alpha_id} has no compiled driver — setup should have compiled every alpha"
                ),
            },
        )
        .into()
    })
}

/// Binding maps that satisfy a compiled combinator driver under `seed`.
fn binding_extensions(
    driver: &CondDriver,
    wm: &FireSession,
    seed: &crate::value::pmap::PMap,
    compiled_conds: &HashMap<i64, crate::rete::compiled_cond::CompiledCond>,
    scratch: &mut SlotFrame,
    sym: &SymbolTable,
    gather_cache: &mut GatherCache,
) -> Result<Vec<crate::value::pmap::PMap>, EvalBreak> {
    match driver {
        // rune:temperare(simplicity-win) — And is sequential join of kid extensions;
        // empty short-circuits. A specialized 2-kid path would duplicate the fold.
        CondDriver::And(kids) => {
            let mut exts = vec![seed.clone()];
            for kid in kids {
                let mut next = Vec::new();
                for ext in &exts {
                    next.extend(binding_extensions(
                        kid,
                        wm,
                        ext,
                        compiled_conds,
                        scratch,
                        sym,
                        gather_cache,
                    )?);
                }
                exts = next;
                if exts.is_empty() {
                    break;
                }
            }
            Ok(exts)
        }
        CondDriver::Or(kids) => {
            let mut out = Vec::new();
            for kid in kids {
                out.extend(binding_extensions(
                    kid,
                    wm,
                    seed,
                    compiled_conds,
                    scratch,
                    sym,
                    gather_cache,
                )?);
            }
            Ok(out)
        }
        CondDriver::Where(program) => {
            match crate::rete::expr_ir::exec_where(program, seed, sym, &program.span)? {
                true => Ok(vec![seed.clone()]),
                false => Ok(vec![]),
            }
        }
        CondDriver::Not(inner) => {
            if exists_cond_under(inner, wm, seed, compiled_conds, scratch, sym, gather_cache)? {
                Ok(vec![])
            } else {
                Ok(vec![seed.clone()])
            }
        }
        CondDriver::Exists(inner) => {
            if exists_cond_under(inner, wm, seed, compiled_conds, scratch, sym, gather_cache)? {
                Ok(vec![seed.clone()])
            } else {
                Ok(vec![])
            }
        }
        CondDriver::Leaf(alpha_id) => {
            let compiled = rematch_compiled(compiled_conds, *alpha_id)?;
            Ok(seeded_bindings_keyed(
                gather_cache,
                wm,
                *alpha_id,
                seed,
                compiled,
                scratch,
            ))
        }
    }
}

pub(crate) fn rematch_compiled(
    compiled_conds: &HashMap<i64, crate::rete::compiled_cond::CompiledCond>,
    alpha_id: i64,
) -> Result<&crate::rete::compiled_cond::CompiledCond, EvalBreak> {
    compiled_conds.get(&alpha_id).ok_or_else(|| {
        RuntimeError::new(
            crate::rust_caller_span!(),
            RuntimeErrorKind::MalformedForm {
                head: ":wat::rete::fire-rules".into(),
                reason: format!(
                    "alpha {alpha_id} has no compiled cond — setup should have compiled every fact-shaped alpha"
                ),
            },
        )
        .into()
    })
}

fn fact_holds_under<B: Bindings + ?Sized>(
    fact: &Value,
    seed: &B,
    compiled: &crate::rete::compiled_cond::CompiledCond,
    scratch: &mut SlotFrame,
) -> bool {
    let fact_fields = match fact {
        Value::Aggregate(a) if a.nature != Nature::Struct => a.fields.as_slice(),
        _ => return false,
    };
    crate::rete::compiled_cond::exec_compiled_under_holds(compiled, fact_fields, scratch, seed)
}

fn fact_bindings_under<B: Bindings + ?Sized>(
    fact: &Value,
    seed: &B,
    compiled: &crate::rete::compiled_cond::CompiledCond,
    scratch: &mut SlotFrame,
) -> Option<crate::value::pmap::PMap> {
    let fact_fields = match fact {
        Value::Aggregate(a) if a.nature != Nature::Struct => a.fields.as_slice(),
        _ => return None,
    };
    let pairs =
        crate::rete::compiled_cond::exec_compiled_under(compiled, fact_fields, scratch, seed)?;
    let pairs = crate::rete::compiled_cond::attach_fact(compiled, fact, pairs);
    // Unify with the seed. A Bind of `?c` compiles as a first-write (the cond
    // in isolation does not know the left token already bound `?c`). Conflict
    // is no match — same as `alpha_match_inner_seeded` / Clara join. Overwrite
    // is how `where-not-and-bound` row 3 accepted Temp.c ≠ Cold.c.
    // Leftover rematch only: materialize a PMap. The fire path does not
    // keep it (`DESIGN-STONE-token-bind-pool`).
    let mut pm =
        crate::value::pmap::PMap::from_pairs(seed.iter().map(|(k, v)| (k.clone(), v.clone())));
    for (k, v) in pairs.iter() {
        match pm.get(k) {
            Some(existing) if existing != v => return None,
            Some(_) => {}
            None => pm = pm.assoc(k.clone(), v.clone()),
        }
    }
    Some(pm)
}

fn token_exists_under<B: Bindings + ?Sized>(
    driver: &CondDriver,
    tok: &B,
    wm: &FireSession,
    compiled_conds: &HashMap<i64, crate::rete::compiled_cond::CompiledCond>,
    scratch: &mut SlotFrame,
    sym: &SymbolTable,
    gather_cache: &mut GatherCache,
) -> Result<bool, EvalBreak> {
    match driver {
        CondDriver::Leaf(alpha_id) => {
            let compiled = rematch_compiled(compiled_conds, *alpha_id)?;
            Ok(any_seeded_keyed(
                gather_cache,
                wm,
                *alpha_id,
                tok,
                compiled,
                scratch,
            ))
        }
        // rune:temperare(simplicity-win) — combinator :not/:exists still PMap::from_pairs;
        // leaf already uses BindView. n tokens with combinator inners is the rare path.
        other => {
            let seed = crate::value::pmap::PMap::from_pairs(
                tok.iter().map(|(k, v)| (k.clone(), v.clone())),
            );
            exists_cond_under(other, wm, &seed, compiled_conds, scratch, sym, gather_cache)
        }
    }
}

fn exists_cond_under(
    driver: &CondDriver,
    wm: &FireSession,
    seed: &crate::value::pmap::PMap,
    compiled_conds: &HashMap<i64, crate::rete::compiled_cond::CompiledCond>,
    scratch: &mut SlotFrame,
    sym: &SymbolTable,
    gather_cache: &mut GatherCache,
) -> Result<bool, EvalBreak> {
    match driver {
        CondDriver::And(_) => {
            Ok(
                !binding_extensions(driver, wm, seed, compiled_conds, scratch, sym, gather_cache)?
                    .is_empty(),
            )
        }
        CondDriver::Or(kids) => {
            for k in kids {
                if exists_cond_under(k, wm, seed, compiled_conds, scratch, sym, gather_cache)? {
                    return Ok(true);
                }
            }
            Ok(false)
        }
        CondDriver::Where(program) => {
            crate::rete::expr_ir::exec_where(program, seed, sym, &program.span)
        }
        CondDriver::Not(inner) => Ok(!exists_cond_under(
            inner,
            wm,
            seed,
            compiled_conds,
            scratch,
            sym,
            gather_cache,
        )?),
        CondDriver::Exists(inner) => {
            exists_cond_under(inner, wm, seed, compiled_conds, scratch, sym, gather_cache)
        }
        CondDriver::Leaf(leaf_id) => {
            let compiled = rematch_compiled(compiled_conds, *leaf_id)?;
            Ok(any_seeded_keyed(
                gather_cache,
                wm,
                *leaf_id,
                seed,
                compiled,
                scratch,
            ))
        }
    }
}

/// Merge an Element's fact and bindings into a native `Token`.
/// Concat left match-span + `(el_fact, alpha_id)` into `match_pool`; concat left
/// bind-span + right-only pairs into `bind_pool`. Duplicate keys on the right are
/// skipped (a repeated `?v` bind within one condition is an equality CONSTRAINT
/// in `eval_clause`, never a second pair).
pub(crate) fn extend_token(
    tok: &Token,
    el_fact: u32,
    el_span: BindSpan,
    alpha_id: i64,
    pool: &mut Vec<(u32, u32)>,
    match_pool: &mut Vec<(u32, i64)>,
) -> Token {
    // Concat left edges + the new fact idx. Do not hold a slice across push
    // (`DESIGN-STONE-match-pool`, `DESIGN-STONE-match-pool-fact-as-index`).
    let mo = tok.matches.off as usize;
    let mn = tok.matches.len as usize;
    let mstart = match_pool.len();
    for i in 0..mn {
        let e = match_pool[mo + i];
        match_pool.push(e);
    }
    match_pool.push((el_fact, alpha_id));
    // Concat left + right-only into the bind pool. Do not hold a slice across push
    // (`DESIGN-STONE-token-bind-pool`, `DESIGN-STONE-bind-value-intern`).
    let lo = tok.binds.off as usize;
    let ln = tok.binds.len as usize;
    let eo = el_span.off as usize;
    let en = el_span.len as usize;
    let start = pool.len();
    for i in 0..ln {
        let p = pool[lo + i];
        pool.push(p);
    }
    for i in 0..en {
        let (k, v) = pool[eo + i];
        let already = (start..start + ln).any(|j| pool[j].0 == k);
        if !already {
            pool.push((k, v));
        }
    }
    Token {
        matches: BindSpan {
            off: mstart as u32,
            len: (match_pool.len() - mstart) as u16,
        },
        binds: BindSpan {
            off: start as u32,
            len: (pool.len() - start) as u16,
        },
    }
}

/// Share the Element's bind-span as the new Token's binds.
/// Root-join is the one place a Token is born from an Element; every other
/// Token is produced by `extend_token`, which concats spans in `bind_pool`.
fn seed_token_binds(el: &Element) -> BindSpan {
    el.binds
}

/// Write a Token/join right span from packed vids when the Element
/// skipped BindSpan (`DESIGN-STONE-column-gather-fold`).
fn span_from_row(
    pool: &mut Vec<(u32, u32)>,
    el: &Element,
    alpha_id: i64,
    i64_by_fact: &[Option<I64Row>],
    bind_only: &HashMap<i64, Vec<u8>>,
    cond_key_ids: &CondKeyIds,
) -> BindSpan {
    let Some(fields) = bind_only.get(&alpha_id) else {
        return empty_span();
    };
    let Some(kids) = cond_key_ids.get(&alpha_id) else {
        return empty_span();
    };
    let Some(row) = i64_by_fact.get(el.fact as usize).and_then(|o| o.as_ref()) else {
        return empty_span();
    };
    let skip = kids.len().saturating_sub(fields.len());
    let off = pool.len();
    for (i, &fi) in fields.iter().enumerate() {
        if skip + i >= kids.len() || (fi as usize) >= row.n as usize {
            pool.truncate(off);
            return empty_span();
        }
        pool.push((kids[skip + i], row.vids[fi as usize]));
    }
    BindSpan {
        off: off as u32,
        len: (pool.len() - off) as u16,
    }
}

/// Occupancy stays empty. The join-index copy gets a BindSpan once
/// (`DESIGN-STONE-join-index-span`). `join_extend` then shares two words.
fn element_with_row_span(
    el: Element,
    pool: &mut Vec<(u32, u32)>,
    alpha_id: i64,
    i64_by_fact: &[Option<I64Row>],
    bind_only: &HashMap<i64, Vec<u8>>,
    cond_key_ids: &CondKeyIds,
) -> Element {
    if el.binds.len > 0 {
        return el;
    }
    let binds = span_from_row(pool, &el, alpha_id, i64_by_fact, bind_only, cond_key_ids);
    Element {
        fact: el.fact,
        binds,
    }
}

fn token_assoc(tok: &Token, k: Value, v: Value, intern: &mut BindIntern<'_>) -> Token {
    let key_id = intern_key(intern.keys, &k);
    let vid = intern_val(intern.vals, intern.ids, v);
    let pairs: Vec<(u32, u32)> = pool_slice(intern.pool, tok.binds).to_vec();
    let start = intern.pool.len();
    let mut found = false;
    for (ek, ev) in pairs {
        if ek == key_id {
            intern.pool.push((ek, vid));
            found = true;
        } else {
            intern.pool.push((ek, ev));
        }
    }
    if !found {
        intern.pool.push((key_id, vid));
    }
    Token {
        matches: tok.matches,
        binds: BindSpan {
            off: start as u32,
            len: (intern.pool.len() - start) as u16,
        },
    }
}

/// Rematch one token against one alpha element and extend the support chain.
/// Returns `None` when a leftover `SeedCmp` rejects the pair. `alpha_id` is
/// recorded on the new token's matches span.
pub(crate) fn join_extend(
    tok: &Token,
    el: &Element,
    alpha_id: i64,
    ctx: &mut FireCtx<'_>,
) -> Result<Option<Token>, EvalBreak> {
    let compiled = rematch_compiled(ctx.compiled_conds, alpha_id)?;
    // No leftover SeedCmp: the keyed bucket is the join (same contract as
    // fold-the-wall). Rematch cannot reject a member (`DESIGN-STONE-join-extend-no-leftover`).
    if compiled.has_seed_cmp()
        && !fact_holds_under(
            fact_at(ctx.facts, ctx.derived, ctx.n_input, el.fact),
            &bind_view(ctx.keys, ctx.vals, ctx.pool, tok.binds),
            compiled,
            ctx.scratch,
        )
    {
        return Ok(None);
    }
    let right = if el.binds.len > 0 {
        el.binds
    } else {
        span_from_row(
            ctx.pool,
            el,
            alpha_id,
            ctx.i64_by_fact,
            ctx.bind_only,
            ctx.cond_key_ids,
        )
    };
    Ok(Some(extend_token(
        tok,
        el.fact,
        right,
        alpha_id,
        ctx.pool,
        ctx.match_pool,
    )))
}

#[cfg(test)]
fn keyed_join(
    left_tokens: &[Token],
    right_elements: &[Element],
    alpha_id: i64,
    ctx: &mut FireCtx<'_>,
) -> Result<Vec<Token>, EvalBreak> {
    if left_tokens.is_empty() || right_elements.is_empty() {
        return Ok(vec![]);
    }

    // Step 1: compute join_keys = sorted shared variable names (intersection of binding key-sets).
    let join_keys: Vec<Value> = gather_join_keys(
        &bind_view(ctx.keys, ctx.vals, ctx.pool, left_tokens[0].binds),
        right_elements,
        GatherIntern::from_ctx(ctx, alpha_id),
    );

    // Step 2: index RIGHT (elements) by join-key-value tuple.
    let mut index: JoinKeyMap<usize> = HashMap::new();
    let intern = GatherIntern::from_ctx(ctx, alpha_id);
    for (i, el) in right_elements.iter().enumerate() {
        let key = key_of_el(el, &join_keys, &intern);
        index.entry(key).or_default().push(i);
    }

    // Step 3: probe with each LEFT (token).
    let mut out: Vec<Token> = Vec::new();
    for tok in left_tokens {
        let probe_key = key_of(
            &bind_view(ctx.keys, ctx.vals, ctx.pool, tok.binds),
            &join_keys,
            ctx.val_ids,
        );
        if let Some(bucket) = index.get(&probe_key) {
            for &el_idx in bucket {
                if let Some(new_tok) = join_extend(tok, &right_elements[el_idx], alpha_id, ctx)? {
                    out.push(new_tok);
                }
            }
        }
    }
    Ok(out)
}

/// Persistent right index for join-after-filter HashJoins.
struct FilterJoinIdx<'a> {
    right_idx: &'a mut JoinRightIndex,
    join_keys_cache: &'a mut JoinKeysCache,
    indexed_n: &'a mut HashMap<i64, usize>,
}

/// Join-after-filter: Δleft ⋈ all_right with a persistent right index (same
/// observable as [`keyed_join`]; first visit indexes, later rounds append).
fn keyed_join_persistent(
    left_tokens: &[Token],
    right_elements: &[Element],
    alpha_id: i64,
    join_id: i64,
    idx: &mut FilterJoinIdx<'_>,
    ctx: &mut FireCtx<'_>,
) -> Result<Vec<Token>, EvalBreak> {
    if left_tokens.is_empty() || right_elements.is_empty() {
        return Ok(vec![]);
    }
    idx.join_keys_cache.entry(join_id).or_insert_with(|| {
        gather_join_keys(
            &bind_view(ctx.keys, ctx.vals, ctx.pool, left_tokens[0].binds),
            right_elements,
            GatherIntern::from_ctx(ctx, alpha_id),
        )
    });
    let already = idx.indexed_n.get(&join_id).copied().unwrap_or(0);
    if already < right_elements.len() {
        let jk = &idx.join_keys_cache[&join_id];
        let ridx = idx.right_idx.entry(join_id).or_default();
        for el in &right_elements[already..] {
            let k = key_of_el(el, jk, &GatherIntern::from_ctx(ctx, alpha_id));
            let el = element_with_row_span(
                *el,
                ctx.pool,
                alpha_id,
                ctx.i64_by_fact,
                ctx.bind_only,
                ctx.cond_key_ids,
            );
            ridx.entry(k).or_default().push(el);
        }
        idx.indexed_n.insert(join_id, right_elements.len());
    }
    let jk = &idx.join_keys_cache[&join_id];
    let Some(ridx) = idx.right_idx.get(&join_id) else {
        return Ok(vec![]);
    };
    let mut out: Vec<Token> = Vec::new();
    for tok in left_tokens {
        let probe_key = key_of(
            &bind_view(ctx.keys, ctx.vals, ctx.pool, tok.binds),
            jk,
            ctx.val_ids,
        );
        if let Some(bucket) = ridx.get(&probe_key) {
            for el in bucket {
                if let Some(new_tok) = join_extend(tok, el, alpha_id, ctx)? {
                    out.push(new_tok);
                }
            }
        }
    }
    Ok(out)
}

/// `hash-join-pass` / `cross-join-node` — propagate tokens from a left-parent to
/// its HashJoinNode children, in ascending node-id order (topological).
/// Left parents: RootJoin / HashJoin / Test / Negation / Exists / Accumulate.
/// Mirrors `wat/rete/oracle/pass.wat` hash-join-pass (A1: a TestNode may parent a HashJoin).
#[cfg(test)]
// ── Pass 3: Hash-join pass ────────────────────────────────────────────────────
pub(crate) fn hash_join_pass(wm: &mut FireSession, arm: &InternedNetwork) -> Result<(), EvalBreak> {
    let node_ids = &arm.node_ids;
    let mut match_scratch: SlotFrame = Vec::with_capacity(arm.compiled_max_slots);
    let mut ctx = FireCtx {
        compiled_conds: &arm.compiled_conds,
        scratch: &mut match_scratch,
        pool: &mut wm.bind_pool,
        match_pool: &mut wm.match_pool,
        keys: &wm.bind_keys,
        vals: &wm.bind_vals,
        val_ids: &wm.bind_val_ids,
        facts: &wm.facts,
        derived: &wm.derived_facts,
        n_input: wm.n_input,
        i64_by_fact: &wm.i64_by_fact,
        bind_only: &wm.bind_only,
        cond_key_ids: &wm.cond_key_ids,
    };

    for node_id in node_ids {
        // Group C: use &Value ref from wm.network; borrow ends after node_children (NLL).
        let node = match get_node(&wm.network, *node_id) {
            Some(n) => n,
            None => continue,
        };
        let kind = kind_of(node);
        if kind != NodeKind::RootJoin
            && kind != NodeKind::HashJoin
            && kind != NodeKind::Test
            && kind != NodeKind::Negation
            && kind != NodeKind::Exists
            && kind != NodeKind::Accumulate
        {
            continue;
        }
        let child_ids = node_children(node);
        // node's last use is node_children above; wm.network borrow for `node` ends here (NLL).

        // tokens must remain a clone: wm.beta[node_id] is read here, wm.beta[child_id] is
        // mutated below — Rust cannot prove key disjointness, so the borrow would conflict.
        // With native Token the clone copies the Vec<Token> (cheap Vec of structs).
        let tokens: Vec<Token> = match wm.beta.get(node_id) {
            Some(ts) => ts.clone(),
            None => continue, // no tokens → skip
        };
        for child_id in &child_ids {
            // Group C: child_node ref — only used for kind_of; borrow ends before wm mutations.
            let child_node = match get_node(&wm.network, *child_id) {
                Some(n) => n,
                None => continue,
            };
            if kind_of(child_node) != NodeKind::HashJoin {
                continue;
            }
            let Some(&alpha_id) = arm.feeding_alpha_of.get(child_id) else {
                continue;
            };
            // Group C: borrow elements from wm.alpha — wm.beta mutations are on a different field.
            let elements = match wm.alpha.get(&alpha_id) {
                Some(els) => els.as_slice(),
                None => continue, // no right-side elements → skip
            };
            let new_tokens = keyed_join(&tokens, elements, alpha_id, &mut ctx)?;
            for new_tok in new_tokens {
                wm.beta.entry(*child_id).or_default().push(new_tok);
            }
        }
    }
    Ok(())
}

/// Delta tokens at every non-alpha parent of `node_id`. Condition `:or` leaves
/// N terminals; a later Test/:not/:exists/accum must see all of them.
fn d_beta_from_parents(parents_of: &ParentsOf, d_beta: &BetaMemory, node_id: i64) -> Vec<Token> {
    let __dbg = phase_start();
    let mut out = Vec::new();
    #[cfg(test)]
    let mut contributing = 0u64;
    if let Some(pids) = parents_of.get(&node_id) {
        for pid in pids {
            if let Some(ts) = d_beta.get(pid) {
                #[cfg(test)]
                if !ts.is_empty() {
                    contributing += 1;
                }
                out.extend(ts.iter().cloned());
            }
        }
    }
    #[cfg(test)]
    {
        census_count_n("dbeta:calls", 1);
        census_count_n("dbeta:tokens", out.len() as u64);
        census_count_n("dbeta:alloc", u64::from(!out.is_empty()));
        census_count_n("dbeta:multi", u64::from(contributing > 1));
    }
    phase_end("  ├ dbeta:gather", __dbg);
    out
}

// ── Pass 4: Production pass ───────────────────────────────────────────────────
/// `production-pass` / `fire-production` — for each ProductionNode, find its parent's beta tokens,
/// for each token × each compiled `:then` form, `exec_compiled_rhs`, push to `production[prod_id]`.
/// Mirrors `wat/rete/oracle/pass.wat`.
#[cfg(test)]
pub(crate) fn production_pass(
    wm: &mut FireSession,
    arm: &InternedNetwork,
    sym: &SymbolTable,
) -> Result<(), EvalBreak> {
    let node_ids = &arm.node_ids;

    for node_id in node_ids {
        // Group C: use &Value ref from wm.network; borrow ends after rule_name extraction (NLL).
        // wm.production mutations below are on a different field — no conflict.
        let node = match get_node(&wm.network, *node_id) {
            Some(n) => n,
            None => continue,
        };
        if kind_of(node) != NodeKind::Production {
            continue;
        }
        let Some(rule_name) = node_named_string(node, "rule-name") else {
            continue;
        };

        let Some(compiled_rhs) = arm.compiled_rhs.get(rule_name) else {
            continue;
        };

        // All non-alpha parents (condition `:or` wires N arm terminals to one production).
        // Slots from the first token of THIS parent — `:or` arms may not share layout
        // (`DESIGN-STONE-rhs-bind-slot`).
        let pids = arm
            .parents_of
            .get(node_id)
            .map(|v| v.as_slice())
            .unwrap_or(&[]);
        for pid in pids {
            let Some(ts) = wm.beta.get(pid) else {
                continue;
            };
            if ts.is_empty() {
                continue;
            }
            let first = bind_view(&wm.bind_keys, &wm.bind_vals, &wm.bind_pool, ts[0].binds);
            let slot_tables: crate::rete::compiled_rhs::RhsSlotTables = compiled_rhs
                .iter()
                .map(|c| crate::rete::compiled_rhs::rhs_bind_slots(c, &first))
                .collect();
            for tok in ts {
                let pairs = bind_view(&wm.bind_keys, &wm.bind_vals, &wm.bind_pool, tok.binds);
                for (compiled, slots) in compiled_rhs.iter().zip(&slot_tables) {
                    let derived = crate::rete::compiled_rhs::exec_compiled_rhs_at(
                        compiled, pairs, slots, sym,
                    )?;
                    wm.production.entry(*node_id).or_default().push(derived);
                }
            }
        }
    }
    Ok(())
}

// ── Pure single-pass fn (extracted for fixpoint reuse) ───────────────────────

/// Public `fire-once`. One round of the delta walk (alpha → join → accum/filter
/// → production), no cascade. Mirrors `fire-once$oracle` (`wat/rete/oracle/fire.wat`):
/// populate-then-emit including Test/Neg/Exists/Accum; keeps alpha and beta.
pub(crate) fn fire_once_session(session: &Value, sym: &SymbolTable) -> Result<Value, EvalBreak> {
    let rules = session_rules(session);
    let rules_empty = matches!(&rules, Value::wat__core__PersistentVector(pv) if pv.is_empty());
    if rules_empty {
        if let Some(net) = session_network(session) {
            if rete_arm_lookup(network_identity(net).unwrap_or(0)).is_none()
                && network_has_production(net)
            {
                return Err(refuse_export_without_arm(
                    ":wat::rete::fire-once",
                    &crate::rust_caller_span!(),
                ));
            }
        }
    }
    fire_fixpoint_delta_armed(session, sym, None, None, FireKind::Once)
}

/// One class-scan query: `{var: fact}` from the closed bag
/// (input ∪ derived), not production-memory
/// (`DESIGN-STONE-query-class-scan-harvest`).
pub(crate) struct QueryClassScan {
    var: Value,
    class: String,
}

/// Alphas that exist only to feed QueryNodes (Alpha → RootJoin → Query).
/// `(?fact <- :Type)` with no field constraints.
pub(crate) fn query_class_scans(arm: &InternedNetwork, network: &Value) -> HashMap<i64, QueryClassScan> {
    let mut q_joins: HashSet<i64> = HashSet::new();
    for &jid in &arm.kind_ids.join_parent {
        let Some(node) = get_node(network, jid) else {
            continue;
        };
        if kind_of(node) != NodeKind::RootJoin {
            continue;
        }
        let Some(kids) = arm.children_of.get(&jid) else {
            continue;
        };
        if kids.is_empty() {
            continue;
        }
        if kids
            .iter()
            .all(|&c| get_node(network, c).is_some_and(|n| kind_of(n) == NodeKind::Query))
        {
            q_joins.insert(jid);
        }
    }
    let mut scans = HashMap::new();
    for &aid in &arm.kind_ids.alpha {
        let Some(kids) = arm.children_of.get(&aid) else {
            continue;
        };
        if kids.is_empty() || !kids.iter().all(|c| q_joins.contains(c)) {
            continue;
        }
        let Some(c) = arm.compiled_conds.get(&aid) else {
            continue;
        };
        let Some(var) = c.fact_bind() else {
            continue;
        };
        if !c.ops().is_empty() {
            continue;
        }
        // Import/Export AlphaNode has no tests AST — class-scan cannot
        // name the type. Leave the chain; beta harvest still works.
        let Some(node) = get_node(network, aid) else {
            continue;
        };
        let Some(cond) = alpha_cond_from_node(node) else {
            continue;
        };
        let Some(pat) = crate::rete::matcher::alpha_pattern(&cond) else {
            continue;
        };
        scans.insert(
            aid,
            QueryClassScan {
                var: var.clone(),
                class: pat.type_head.to_string(),
            },
        );
    }
    scans
}

/// One pass of the closed bag, keyed by class. Only `scan.class`.
/// Skip `wm.facts` when `input_has_scan_class` is false
/// (`DESIGN-STONE-accum-wanted-harvest`).
fn closed_bag_by_class<'a>(
    wm: &'a FireSession,
    wanted: &HashSet<&str>,
) -> HashMap<&'a str, Vec<&'a Value>> {
    let mut idx: HashMap<&str, Vec<&Value>> = HashMap::new();
    let mut push = |f: &'a Value| {
        if let Value::Aggregate(a) = f {
            if a.nature != Nature::Struct && wanted.contains(a.class.as_ref()) {
                idx.entry(a.class.as_ref()).or_default().push(f);
            }
        }
    };
    if wm.input_has_scan_class {
        if let Value::wat__core__PersistentVector(pv) = &wm.facts {
            for f in pv.iter() {
                push(f);
            }
        }
    }
    for f in &wm.derived_facts {
        push(f);
    }
    idx
}

/// Write one-entry `{var: fact}` maps straight into the CALLER'S vec.
///
/// It used to return its own `Vec` and the caller `extend`ed from it — a bag
/// built, then the bag copied (`DESIGN-STONE-harvest-bag-in-place`). `PMap` is
/// 56 B, so at fanout 40k the intermediate was 2.24 MB allocated, filled,
/// memcpy'd and freed on every fire, with the page faults paid twice. `extra`
/// is the upper-bound hint reserved before the walk; the walk itself is WHAT.
fn harvest_class_scan_into<'a, I>(
    out: &mut Vec<crate::value::pmap::PMap>,
    facts: I,
    extra: usize,
    var: &Value,
) where
    I: Iterator<Item = &'a Value>,
{
    out.reserve(extra);
    for f in facts {
        out.push(crate::value::pmap::PMap::from_one(var.clone(), f.clone()));
    }
}

fn compiled_rhs_is_class(arm: &InternedNetwork, class: &str) -> bool {
    !arm.compiled_rhs.is_empty()
        && arm.compiled_rhs.values().all(|forms| {
            !forms.is_empty()
                && forms.iter().all(|f| match f {
                    crate::rete::compiled_rhs::CompiledRhs::Record { class: c, .. } => {
                        c.as_ref() == class
                    }
                    crate::rete::compiled_rhs::CompiledRhs::Call(_) => false,
                })
        })
}

fn harvest_class_scan_filter(
    wm: &FireSession,
    scan: &QueryClassScan,
    derived_is_scan: bool,
) -> Vec<crate::value::pmap::PMap> {
    let matches_class = |f: &Value| match f {
        Value::Aggregate(a) if a.nature != Nature::Struct => a.class.as_ref() == scan.class,
        _ => false,
    };
    let mut maps = Vec::new();
    if wm.input_has_scan_class {
        if let Value::wat__core__PersistentVector(pv) = &wm.facts {
            harvest_class_scan_into(
                &mut maps,
                pv.iter().filter(|f| matches_class(f)),
                pv.len(),
                &scan.var,
            );
        }
    }
    if derived_is_scan {
        harvest_class_scan_into(
            &mut maps,
            wm.derived_facts.iter(),
            wm.derived_facts.len(),
            &scan.var,
        );
    } else {
        harvest_class_scan_into(
            &mut maps,
            wm.derived_facts.iter().filter(|f| matches_class(f)),
            wm.derived_facts.len(),
            &scan.var,
        );
    }
    maps
}

pub(crate) fn harvest_query_memory(
    wm: &mut FireSession,
    arm: &InternedNetwork,
    scans: &HashMap<i64, QueryClassScan>,
) {
    if arm.kind_ids.query.is_empty() {
        wm.query.clear();
        return;
    }
    // Index pays when N queries would rescan the bag. One scan stays the
    // filter path: skip facts when no scan class was packed; wrap derived
    // without class-eq when interned RHS is only that class
    // (`DESIGN-STONE-fanout-identity-filter`).
    let bag = if scans.len() > 1 {
        let wanted: HashSet<&str> = scans.values().map(|s| s.class.as_str()).collect();
        Some(closed_bag_by_class(wm, &wanted))
    } else {
        None
    };
    let derived_is_scan = scans.len() == 1
        && scans
            .values()
            .next()
            .is_some_and(|s| compiled_rhs_is_class(arm, s.class.as_str()));
    let mut harvested: HashMap<String, Vec<crate::value::pmap::PMap>> = HashMap::new();
    for node_id in &arm.kind_ids.query {
        let node = match get_node(&wm.network, *node_id) {
            Some(n) => n,
            None => continue,
        };
        let Some(qname) = node_named_string(node, "query-name").map(str::to_string) else {
            continue;
        };
        let maps = {
            let pids = arm
                .parents_of
                .get(node_id)
                .map(|v| v.as_slice())
                .unwrap_or(&[]);
            let scan = pids.iter().find_map(|pid| {
                let aid = arm.feeding_alpha_of.get(pid)?;
                scans.get(aid)
            });
            if let Some(scan) = scan {
                if let Some(bag) = bag.as_ref() {
                    let facts = bag
                        .get(scan.class.as_str())
                        .map(|v| v.as_slice())
                        .unwrap_or(&[]);
                    let mut maps = Vec::new();
                    harvest_class_scan_into(
                        &mut maps,
                        facts.iter().copied(),
                        facts.len(),
                        &scan.var,
                    );
                    maps
                } else {
                    harvest_class_scan_filter(wm, scan, derived_is_scan)
                }
            } else {
                let mut maps: Vec<crate::value::pmap::PMap> = Vec::new();
                for pid in pids {
                    if let Some(ts) = wm.beta.get(pid) {
                        maps.extend(ts.iter().map(|t| {
                            pmap_from_span(t.binds, &wm.bind_keys, &wm.bind_vals, &wm.bind_pool)
                        }));
                    }
                }
                maps
            }
        };
        harvested.insert(qname, maps);
    }
    drop(bag);
    wm.query = harvested;
}

// ── Public entry: native fire-once ───────────────────────────────────────────

/// `(:wat::rete::fire-once <session>) -> :wat::rete::Session`
///
/// Native Rust single-pass fire cycle: the delta walk, one round, no cascade.
/// Observationally equivalent to the wat oracle's `fire-once$oracle` on AST
/// Sessions (alpha → root-join → accum/filter/hash-join in id order → production).
/// Export is native-only: the oracle refuses an imported Export
/// (`wat/rete.wat`; `wat/rete/oracle/fire.wat` fire-once$oracle).
///
/// Dispatch entry called from `runtime.rs:dispatch_keyword_head_value`.
/// Evaluates the single argument (must be `:wat::rete::Session`) and returns
/// a frozen `Session`.
pub(crate) fn eval_fire_once_native(
    args: &[WatAST],
    list_span: &Span,
    env: &crate::runtime::Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::rete::fire-once";
    if args.len() != 1 {
        return Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::ArityMismatch {
                op: OP.into(),
                expected: 1,
                got: args.len(),
            },
        )
        .into());
    }

    // Evaluate the session argument, then delegate to the pure single-pass fn.
    let session = crate::runtime::eval_inner(&args[0], env, sym)?.value_owned();
    fire_once_session(&session, sym)
}

// ── Cascade fixpoint helpers (P4a) ───────────────────────────────────────────

/// Flatten `production-memory`'s per-node `PV<Record>` values into one `Vec<Value>`.
///
/// `production-memory` is a `PersistentMap<node-id, PV<Record>>`. The outer pass visits
/// each node's PV; the inner pass collects each Record. Mirrors `collect-derived`
/// (`wat/rete/oracle/fire.wat`).
///
/// Used by the 7-strat-native stratified driver (`fire_rules_stratified`) to collect
/// each stratum's derived facts.
pub(crate) fn collect_derived(production_pm: &Value) -> Vec<Value> {
    let mut out: Vec<Value> = Vec::new();
    if let Value::wat__core__PersistentMap(m) = production_pm {
        for (_k, v) in m.iter() {
            if let Value::wat__core__PersistentVector(pv) = v {
                for fact in pv.iter() {
                    out.push(fact.clone());
                }
            }
        }
    }
    out
}

/// Fold `derived` facts into the existing `facts` PersistentVector, conj-ing ONLY facts
/// not already present (structural `==` dedup).
///
/// The dedup is the termination guard: if every derived fact is already in `facts`, the
/// result length equals `facts` length → the fixpoint loop exits. Re-adding a present
/// fact would grow `facts` every round and spin forever. Mirrors `merge-facts`
/// (`wat/rete/oracle/fire.wat`).
///
/// Used by the 7-strat-native stratified driver (`fire_rules_stratified`) — R18: the cross-stratum
/// derived-fact accumulation MUST value-dedup (mirrors the oracle's `merge-facts`,
/// `wat/rete/oracle/fire.wat`), not concat, or a fact produced by more than one stratum's
/// query is double-counted.
///
/// P9 perf, kept for the lineage: membership is a `HashSet` rather than a linear `.any()`
/// scan, which was O(len(pv)) PER derived fact — O(n²) over a stratum-chain run and the exact
/// quadratic blow-up behind the `[7,3000]`-class hang. `Value: Hash + Eq` already (the
/// round-loop's own `seen: HashSet<Value>` dedup, above, uses the same property), so the swap
/// cost nothing in semantics: same value-dedup, same push_back order.
///
/// P9 left the set REBUILT here, once per call, making the per-call cost O(len(pv) +
/// len(derived)). That is no longer what this function does — see the next paragraph. The
/// O(len(pv)) term is now paid ONCE by `facts_membership` outside the loop, and this call is
/// O(len(derived)).
///
/// The membership set is the CALLER'S and is carried across strata — it is not
/// rebuilt here (`DESIGN-STONE-strat-merge-carried-set`). The stratified loop
/// calls this once per stratum with the whole accumulated closure; rebuilding
/// the set each time re-hashed and re-cloned every fact derived so far to
/// re-learn what the previous iteration already held, O(S*N) where the honest
/// cost is O(N). Measured at strat-neg `[6 2000]`: 27000 hashes vs 8000, 2.23 ms
/// of it theater (`strat_merge_present_parts`). Seed the set with
/// `facts_membership` and thread it through the loop.
pub(crate) fn merge_facts(
    facts_pv: &Value,
    present: &mut std::collections::HashSet<Value>,
    derived: &[Value],
) -> Value {
    // Start with a clone of the existing PV.
    let mut pv: crate::value::pvec::PVec = match facts_pv {
        Value::wat__core__PersistentVector(v) => v.clone(),
        _ => crate::value::pvec::PVec::new(),
    };
    #[cfg(test)]
    {
        census_count_n("merge:pv-owners", pv.array_owners() as u64);
        census_count_n("merge:pv-calls", 1);
    }
    for fact in derived {
        // Conj only if not already present (structural equality, now O(1) amortized).
        if present.insert(fact.clone()) {
            pv.push_back_mut(fact.clone());
        }
    }
    Value::wat__core__PersistentVector(pv)
}

/// The membership set `merge_facts` would have collected on its first call —
/// the seed for the carried set. The non-PersistentVector arm mirrors
/// `merge_facts`' own `_ => PVec::new()`, so a `facts` field that is not a
/// vector behaves exactly as before.
pub(crate) fn facts_membership(facts_pv: &Value) -> std::collections::HashSet<Value> {
    match facts_pv {
        Value::wat__core__PersistentVector(v) => v.iter().cloned().collect(),
        _ => std::collections::HashSet::new(),
    }
}

pub(crate) fn network_has_production(network: &Value) -> bool {
    sorted_node_ids(network)
        .iter()
        .any(|&id| get_node(network, id).is_some_and(|n| kind_of(n) == NodeKind::Production))
}

pub(crate) fn refuse_export_without_arm(op: &'static str, span: &Span) -> EvalBreak {
    RuntimeError::new(
        span.clone(),
        RuntimeErrorKind::MalformedForm {
            head: op.into(),
            reason: "cannot consume an Export without interned stratify schedule — empty rules, empty rule_deps, live productions"
                .into(),
        },
    )
    .into()
}

pub(crate) fn rules_lack_ast(rules: &[Value]) -> bool {
    if rules.is_empty() {
        return true;
    }
    rules.iter().all(|r| match rule_named_field(r, "lhs") {
        Some(Value::wat__core__PersistentVector(pv)) => {
            !pv.iter().any(|x| matches!(x, Value::wat__WatAST(_)))
        }
        _ => true,
    })
}

pub(crate) fn synthetic_rule(name: &str) -> Value {
    Value::Aggregate(Arc::new(AggregateValue::record(
        "wat::rete::Rule".into(),
        crate::value::value::names_arc_from_static(RULE_FIELDS),
        Arc::new(vec![
            Value::String(Arc::new(name.to_string())),
            Value::wat__core__PersistentVector(crate::value::pvec::PVec::new()),
            Value::wat__core__PersistentVector(crate::value::pvec::PVec::new()),
        ]),
    )))
}

pub(crate) fn fire_rules_from_deps(
    session: &Value,
    deps: &[RuleDep],
    sym: &SymbolTable,
    support: Option<&mut ExplainSupport>,
) -> Result<Value, EvalBreak> {
    let mut parts: Vec<RuleParts> = Vec::with_capacity(deps.len());
    for d in deps {
        parts.push(RuleParts {
            rule: synthetic_rule(&d.name),
            view: d.view.clone(),
        });
    }
    let pn_only: Vec<StratifyView> = parts.iter().map(|p| p.view.clone()).collect();
    let type_strata = native_stratify(&pn_only)?;
    let mut max_s: i64 = 0;
    let mut rule_strata: Vec<i64> = Vec::with_capacity(parts.len());
    for part in &parts {
        let s = native_rule_stratum(&part.view.produced, &part.view.negated, &type_strata);
        rule_strata.push(s);
        if s > max_s {
            max_s = s;
        }
    }
    if max_s == 0 {
        return fire_fixpoint_delta(session, sym, support);
    }
    fire_rules_stratified(session, &parts, &rule_strata, max_s, sym, support)
}

// ── key_of helper ────────────────────────────────────────────────────────────

/// Extract a join key tuple from a bindings map given the pre-computed `join_keys` list.
///
/// `join_keys` is the sorted list of shared variable names (same tuple `keyed_join` computes).
/// Returns `Vec<Value>` of the bound values in key order. For empty `join_keys` (cartesian
/// product case) returns `vec![]` — all tokens/elements share the single empty-key bucket.
///
/// Panics if a join key is absent from `bindings` (structurally impossible in a well-formed
/// rete network; all shared variables must be bound before this node is reached).
// rune:struere(invariant-coupling) — missing key is a malformed network; Option would
// force every join to invent a fallback the grammar already forbids.
pub(crate) fn key_of<B: Bindings + ?Sized>(
    bindings: &B,
    join_keys: &[Value],
    ids: &crate::rete::compiled_cond::ValIntern,
) -> JoinKey {
    fn vid(ids: &crate::rete::compiled_cond::ValIntern, v: &Value) -> u32 {
        ids.get(v)
            .unwrap_or_else(|| panic!("key_of: filler {v:?} is not interned (bind_val_ids)"))
    }
    match join_keys.len() {
        0 => JoinKey::Empty,
        1 => {
            let v = bindings.get(&join_keys[0]).unwrap_or_else(|| {
                panic!("key_of: join key {:?} missing from bindings", join_keys[0])
            });
            JoinKey::Unary(vid(ids, v))
        }
        _ => {
            let mut out = Vec::with_capacity(join_keys.len());
            for k in join_keys {
                let v = bindings
                    .get(k)
                    .unwrap_or_else(|| panic!("key_of: join key {:?} missing from bindings", k));
                out.push(vid(ids, v));
            }
            JoinKey::Nary(out.into_boxed_slice())
        }
    }
}

/// Derive the join-key tuple shared between `sample_bindings` and `elements` — the cheap half of
/// `gather_index` (step 1 of `keyed_join`): a sorted intersection of
/// `sample_bindings`' keys and a sample element's keys, string-sorted for a stable canonical
/// order, derived from `elements[0]` when non-empty. An empty `elements` slice yields `[]`.
///
/// Split out from the index build so a cache lookup can key on `(alpha_id, join_keys)` *before*
/// paying for the expensive half (`build_gather_index`) — the gather-index cache's ordering
/// constraint (`DESIGN-STONE-gather-index-cache.md`).
pub(crate) fn gather_join_keys<B: Bindings + ?Sized>(
    sample_bindings: &B,
    elements: &[Element],
    intern: GatherIntern<'_>,
) -> Vec<Value> {
    if elements.is_empty() {
        return Vec::new();
    }
    let mut keys: Vec<Value> = if elements[0].binds.len == 0 {
        sample_bindings
            .iter()
            .map(|(k, _)| k)
            .filter(|k| col_field_of(&intern, k).is_some())
            .cloned()
            .collect()
    } else {
        let sample_el_bindings =
            element_fact_bindings(&elements[0], intern.bind_keys, intern.vals, intern.pool);
        sample_bindings
            .iter()
            .map(|(k, _)| k)
            .filter(|k| Bindings::get(&sample_el_bindings, k).is_some())
            .cloned()
            .collect()
    };
    // Binding keys are Value::String (variable names like "?loc").
    // Sort by their string content for a stable canonical order.
    keys.sort_by(|a, b| {
        let a_str = match a {
            Value::String(s) => s.as_str(),
            _ => "",
        };
        let b_str = match b {
            Value::String(s) => s.as_str(),
            _ => "",
        };
        a_str.cmp(b_str)
    });
    keys
}

/// Join-key → element indices (bucket), as built by `build_gather_index`.
/// Unary when `join_keys.len() == 1` — no one-element `Vec` (`DESIGN-STONE-gather-unary-index`).
type GatherUnary = FxHashMap<u32, Vec<usize>>;
type GatherNary = FxHashMap<JoinKey, Vec<usize>>;
/// Bind intern borrowed by gather (avoids too-many-arguments).
pub(crate) struct GatherIntern<'a> {
    bind_keys: &'a [Value],
    vals: &'a [Value],
    pool: &'a [(u32, u32)],
    val_ids: &'a ValIntern,
    i64_by_fact: &'a [Option<I64Row>],
    bind_only: &'a HashMap<i64, Vec<u8>>,
    cond_key_ids: &'a CondKeyIds,
    alpha_id: i64,
}

impl<'a> GatherIntern<'a> {
    fn from_ctx(ctx: &'a FireCtx<'_>, alpha_id: i64) -> Self {
        Self {
            bind_keys: ctx.keys,
            vals: ctx.vals,
            pool: ctx.pool,
            val_ids: ctx.val_ids,
            i64_by_fact: ctx.i64_by_fact,
            bind_only: ctx.bind_only,
            cond_key_ids: ctx.cond_key_ids,
            alpha_id,
        }
    }

    pub(crate) fn from_wm(wm: &'a FireSession, alpha_id: i64) -> Self {
        Self {
            bind_keys: &wm.bind_keys,
            vals: &wm.bind_vals,
            pool: &wm.bind_pool,
            val_ids: &wm.bind_val_ids,
            i64_by_fact: &wm.i64_by_fact,
            bind_only: &wm.bind_only,
            cond_key_ids: &wm.cond_key_ids,
            alpha_id,
        }
    }

    #[cfg(test)]
    pub(crate) fn of(
        bind_keys: &'a [Value],
        vals: &'a [Value],
        pool: &'a [(u32, u32)],
        val_ids: &'a ValIntern,
    ) -> Self {
        Self {
            bind_keys,
            vals,
            pool,
            val_ids,
            i64_by_fact: &[],
            bind_only: {
                static EMPTY: std::sync::OnceLock<HashMap<i64, Vec<u8>>> =
                    std::sync::OnceLock::new();
                EMPTY.get_or_init(HashMap::new)
            },
            cond_key_ids: {
                static EMPTY: std::sync::OnceLock<CondKeyIds> = std::sync::OnceLock::new();
                EMPTY.get_or_init(HashMap::new)
            },
            alpha_id: -1,
        }
    }
}

/// Field index of `join_key` on this intern's alpha, if bind-only packed.
fn col_field_of(intern: &GatherIntern<'_>, join_key: &Value) -> Option<u8> {
    let fields = intern.bind_only.get(&intern.alpha_id)?;
    let kids = intern.cond_key_ids.get(&intern.alpha_id)?;
    let kid = intern
        .bind_keys
        .iter()
        .position(|k| k == join_key)
        .map(|i| i as u32)?;
    let skip = kids.len().saturating_sub(fields.len());
    let pos = kids.iter().skip(skip).position(|&k| k == kid)?;
    fields.get(pos).copied()
}

fn unary_el_vid(
    el: &Element,
    field: Option<u8>,
    key_id: Option<u32>,
    intern: &GatherIntern<'_>,
) -> Option<u32> {
    if let Some(f) = field {
        if let Some(vid) = col_vid(intern, el, f) {
            return Some(vid);
        }
    }
    let key_id = key_id?;
    pool_slice(intern.pool, el.binds)
        .iter()
        .find(|(k, _)| *k == key_id)
        .map(|(_, vid)| *vid)
}

fn col_vid(intern: &GatherIntern<'_>, el: &Element, field: u8) -> Option<u32> {
    intern
        .i64_by_fact
        .get(el.fact as usize)
        .and_then(|o| o.as_ref())
        .filter(|r| (field as usize) < r.n as usize)
        .map(|r| r.vids[field as usize])
}

fn key_of_el(el: &Element, join_keys: &[Value], intern: &GatherIntern<'_>) -> JoinKey {
    if el.binds.len > 0 {
        let el_b = element_fact_bindings(el, intern.bind_keys, intern.vals, intern.pool);
        return key_of(&el_b, join_keys, intern.val_ids);
    }
    match join_keys.len() {
        0 => JoinKey::Empty,
        1 => {
            let field = col_field_of(intern, &join_keys[0]).unwrap_or_else(|| {
                panic!("key_of_el: join key {:?} has no packed field", join_keys[0])
            });
            let vid = col_vid(intern, el, field)
                .unwrap_or_else(|| panic!("key_of_el: packed vid missing for field {field}"));
            JoinKey::Unary(vid)
        }
        _ => {
            let mut out = Vec::with_capacity(join_keys.len());
            for k in join_keys {
                let field = col_field_of(intern, k)
                    .unwrap_or_else(|| panic!("key_of_el: join key {k:?} has no packed field"));
                let vid = col_vid(intern, el, field)
                    .unwrap_or_else(|| panic!("key_of_el: packed vid missing for field {field}"));
                out.push(vid);
            }
            JoinKey::Nary(out.into_boxed_slice())
        }
    }
}

pub(crate) enum GatherIndex {
    /// One join key: interned filler id (`DESIGN-STONE-gather-val-id`).
    UnaryId(GatherUnary),
    Nary(GatherNary),
}

impl GatherIndex {
    fn bucket(&self, key: &JoinKey) -> &[usize] {
        match (self, key) {
            (Self::UnaryId(m), JoinKey::Unary(id)) => m.get(id).map_or(&[], Vec::as_slice),
            (Self::Nary(m), k) => m.get(k).map_or(&[], Vec::as_slice),
            _ => &[],
        }
    }

    /// Push new alpha indices into existing buckets (`DESIGN-STONE-persist-gather-across-rounds`).
    /// New ids are `>=` the previous length (alpha only appends). Foldl order holds.
    fn append(
        &mut self,
        new_idxs: impl IntoIterator<Item = usize>,
        elements: &[Element],
        join_keys: &[Value],
        intern: GatherIntern<'_>,
    ) {
        if elements.is_empty() {
            return;
        }
        if join_keys.is_empty() {
            if let Self::Nary(m) = self {
                m.entry(JoinKey::Empty).or_default().extend(new_idxs);
            }
            return;
        }
        match self {
            Self::UnaryId(m) => {
                let field = col_field_of(&intern, &join_keys[0]);
                let key_id = intern
                    .bind_keys
                    .iter()
                    .position(|k| k == &join_keys[0])
                    .map(|i| i as u32);
                for i in new_idxs {
                    if let Some(vid) = unary_el_vid(&elements[i], field, key_id, &intern) {
                        m.entry(vid).or_default().push(i);
                    }
                }
            }
            Self::Nary(m) => {
                for i in new_idxs {
                    let key = key_of_el(&elements[i], join_keys, &intern);
                    m.entry(key).or_default().push(i);
                }
            }
        }
    }
}

/// Fire-scoped cache: `(alpha_id, join_keys) -> index`. Join keys are the
/// intersection of this sample token's keys with the alpha's elements — they
/// are NOT a property of the alpha alone (query params vs empty parent).
/// Buckets are indices into `wm.alpha[alpha_id]` (`DESIGN-STONE-gather-no-snapshot`).
/// Persists across rounds; `append` takes `d_alpha`
/// (`DESIGN-STONE-persist-gather-across-rounds`). Not a Session field.
// rune:struere(lifetime-coupling) — indices into this fire's `wm.alpha`; a
// GatherIndex must not outlive the alpha vec (`DESIGN-STONE-gather-no-snapshot`,
// `DESIGN-STONE-persist-gather-across-rounds`).
type GatherCache = FxHashMap<(i64, Arc<[Value]>), GatherIndex>;

/// Packed seed occupancy is dirty in full. Walk `0..len`, do not
/// materialize `(0..n).collect()` (`DESIGN-STONE-seed-d-alpha-range`).
#[derive(Clone, Copy)]
pub(crate) struct AlphaNews<'a> {
    inner: AlphaNewsInner<'a>,
}

#[derive(Clone, Copy)]
enum AlphaNewsInner<'a> {
    Range(usize),
    Slots(&'a [usize]),
}

pub(crate) enum AlphaNewsIter<'a> {
    Range(std::ops::Range<usize>),
    Slots(std::slice::Iter<'a, usize>),
}

impl<'a> AlphaNews<'a> {
    pub(crate) fn of(
        d_alpha: &'a AlphaDelta,
        alpha: &'a AlphaMemory,
        aid: i64,
        packed_full: &HashSet<i64>,
    ) -> Self {
        if packed_full.contains(&aid) {
            let n = alpha.get(&aid).map(|v| v.len()).unwrap_or(0);
            Self {
                inner: AlphaNewsInner::Range(n),
            }
        } else {
            match d_alpha.get(&aid) {
                Some(ix) if !ix.is_empty() => Self {
                    inner: AlphaNewsInner::Slots(ix),
                },
                _ => Self {
                    inner: AlphaNewsInner::Range(0),
                },
            }
        }
    }

    pub(crate) fn is_empty(self) -> bool {
        match self.inner {
            AlphaNewsInner::Range(n) => n == 0,
            AlphaNewsInner::Slots(s) => s.is_empty(),
        }
    }

    pub(crate) fn iter(self) -> AlphaNewsIter<'a> {
        match self.inner {
            AlphaNewsInner::Range(n) => AlphaNewsIter::Range(0..n),
            AlphaNewsInner::Slots(s) => AlphaNewsIter::Slots(s.iter()),
        }
    }
}

impl Iterator for AlphaNewsIter<'_> {
    type Item = usize;
    fn next(&mut self) -> Option<usize> {
        match self {
            Self::Range(r) => r.next(),
            Self::Slots(it) => it.next().copied(),
        }
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            Self::Range(r) => r.size_hint(),
            Self::Slots(it) => it.size_hint(),
        }
    }
}

fn append_d_alpha(
    cache: &mut GatherCache,
    d_alpha: &AlphaDelta,
    wm: &FireSession,
    packed_full: &HashSet<i64>,
) {
    for ((aid, join_keys), idx) in cache.iter_mut() {
        let news = AlphaNews::of(d_alpha, &wm.alpha, *aid, packed_full);
        if news.is_empty() {
            continue;
        }
        let els = alpha_elements(&wm.alpha, *aid);
        idx.append(
            news.iter(),
            els,
            join_keys.as_ref(),
            GatherIntern::from_wm(wm, *aid),
        );
    }
}

fn alpha_elements(alpha: &AlphaMemory, alpha_id: i64) -> &[Element] {
    alpha.get(&alpha_id).map(|v| v.as_slice()).unwrap_or(&[])
}

/// Build the bucket index over `elements` for a given `join_keys` tuple — the expensive half of
/// `gather_index` (the full scan). Buckets hold element *indices* in iteration order, matching
/// `keyed_join`'s right-index and the wat oracle's foldl order.
///
/// Panics only via `key_of` if an element's bindings lack a derived join key — structurally
/// impossible for a well-formed network (every element at one alpha node shares a binding
/// key-set, the same guarantee `keyed_join` already rests on).
pub(crate) fn build_gather_index(
    elements: &[Element],
    join_keys: &[Value],
    intern: GatherIntern<'_>,
) -> GatherIndex {
    if join_keys.len() == 1 {
        let mut index: GatherUnary = FxHashMap::default();
        let field = col_field_of(&intern, &join_keys[0]);
        let key_id = intern
            .bind_keys
            .iter()
            .position(|k| k == &join_keys[0])
            .map(|i| i as u32);
        for (i, el) in elements.iter().enumerate() {
            // Packed i64 first; BindSpan if the row is absent (string
            // location, 7b/8b/exists). bind_only + col_field_of without
            // this fallback silently dropped every non-i64 occupant.
            if let Some(vid) = unary_el_vid(el, field, key_id, &intern) {
                index.entry(vid).or_default().push(i);
            }
        }
        GatherIndex::UnaryId(index)
    } else {
        let mut index: GatherNary = FxHashMap::default();
        for (i, el) in elements.iter().enumerate() {
            let key = key_of_el(el, join_keys, &intern);
            index.entry(key).or_default().push(i);
        }
        GatherIndex::Nary(index)
    }
}

/// Get-or-build the fire-scoped gather index for `alpha_id` under `sample`'s shared keys.
/// Acc, Negation, and Exists all miss through here so one pair is built once per fire
/// and appended across rounds (`DESIGN-STONE-persist-gather-across-rounds`).
fn ensure_gather<'a, B: Bindings + ?Sized>(
    cache: &'a mut GatherCache,
    wm: &FireSession,
    alpha_id: i64,
    sample: &B,
) -> Option<(&'a GatherIndex, Arc<[Value]>)> {
    let els = alpha_elements(&wm.alpha, alpha_id);
    if els.is_empty() {
        return None;
    }
    let join_keys: Arc<[Value]> =
        gather_join_keys(sample, els, GatherIntern::from_wm(wm, alpha_id)).into();
    let index = cache
        .entry((alpha_id, Arc::clone(&join_keys)))
        .or_insert_with(|| {
            census_count("accum:index-builds");
            census_count_n("accum:index-elements", els.len() as u64);
            build_gather_index(els, join_keys.as_ref(), GatherIntern::from_wm(wm, alpha_id))
        });
    Some((index, join_keys))
}

/// Exists/Not Leaf: probe the token's bucket. Empty bucket is absence (contract clause 2).
fn any_seeded_keyed<B: Bindings + ?Sized>(
    cache: &mut GatherCache,
    wm: &FireSession,
    alpha_id: i64,
    seed: &B,
    compiled: &crate::rete::compiled_cond::CompiledCond,
    scratch: &mut SlotFrame,
) -> bool {
    let Some((index, join_keys)) = ensure_gather(cache, wm, alpha_id, seed) else {
        return false;
    };
    let key = key_of(seed, join_keys.as_ref(), &wm.bind_val_ids);
    let elements = alpha_elements(&wm.alpha, alpha_id);
    let bucket = index.bucket(&key);
    // No leftover SeedCmp: the keyed bucket is the exists/not (same contract as join_extend).
    if !compiled.has_seed_cmp() {
        return !bucket.is_empty();
    }
    bucket.iter().any(|&i| {
        census_gather_visit();
        fact_holds_under(
            fact_at(&wm.facts, &wm.derived_facts, wm.n_input, elements[i].fact),
            seed,
            compiled,
            scratch,
        )
    })
}

/// Leftover rematch / combinator Leaf: every matching binding in the token's bucket.
fn seeded_bindings_keyed(
    cache: &mut GatherCache,
    wm: &FireSession,
    alpha_id: i64,
    seed: &crate::value::pmap::PMap,
    compiled: &crate::rete::compiled_cond::CompiledCond,
    scratch: &mut SlotFrame,
) -> Vec<crate::value::pmap::PMap> {
    let Some((index, join_keys)) = ensure_gather(cache, wm, alpha_id, seed) else {
        return Vec::new();
    };
    let key = key_of(seed, join_keys.as_ref(), &wm.bind_val_ids);
    let elements = alpha_elements(&wm.alpha, alpha_id);
    let bucket = index.bucket(&key);
    if !compiled.has_seed_cmp() {
        return bucket
            .iter()
            .map(|&i| {
                let el_b = element_fact_bindings(
                    &elements[i],
                    &wm.bind_keys,
                    &wm.bind_vals,
                    &wm.bind_pool,
                );
                let mut pm = seed.clone();
                for (k, v) in el_b.iter() {
                    if pm.get(k).is_none() {
                        pm = pm.assoc(k.clone(), v.clone());
                    }
                }
                pm
            })
            .collect();
    }
    bucket
        .iter()
        .filter_map(|&i| {
            census_gather_visit();
            fact_bindings_under(
                fact_at(&wm.facts, &wm.derived_facts, wm.n_input, elements[i].fact),
                seed,
                compiled,
                scratch,
            )
        })
        .collect()
}

fn exec_stashed_where(
    programs: &HashMap<i64, crate::rete::expr_ir::Program>,
    node_id: i64,
    bindings: &(impl Bindings + ?Sized),
    sym: &SymbolTable,
) -> Result<bool, EvalBreak> {
    let Some(program) = programs.get(&node_id) else {
        return Err(RuntimeError::new(
            crate::rust_caller_span!(),
            RuntimeErrorKind::MalformedForm {
                head: ":wat::rete::fire-rules".into(),
                reason: format!(
                    "TestNode {node_id} has no compiled where — compile-condition should have refused"
                ),
            },
        )
        .into());
    };
    crate::rete::expr_ir::exec_where(program, bindings, sym, &program.span)
}

/// Mut sink for one where-dispatch. Named so the fire loop does not grow
/// an 8-arg helper (validate.rs `ClauseCtx` — a struct, not an allow).
struct WhereSink<'a> {
    where_tree: &'a crate::rete::where_tree::WhereTree,
    compiled_wheres: &'a HashMap<i64, crate::rete::expr_ir::Program>,
    beta_readers: &'a HashSet<i64>,
    wm: &'a mut FireSession,
    d_beta: &'a mut BetaMemory,
    sym: &'a SymbolTable,
}

/// (b) — eval only the TestNodes the where-tree says this token could still pass.
/// `exec_where` stays the authority. Tree miss → skip the eval (over-approx only).
fn dispatch_where_tests(
    tids: &[i64],
    tokens: &[Token],
    sink: &mut WhereSink<'_>,
) -> Result<(), EvalBreak> {
    if tids.is_empty() || tokens.is_empty() {
        return Ok(());
    }
    let use_tree = tids.iter().any(|id| sink.where_tree.covers(*id));
    if use_tree {
        let span = crate::rust_caller_span!();
        for tok in tokens {
            let binds = bind_view(
                &sink.wm.bind_keys,
                &sink.wm.bind_vals,
                &sink.wm.bind_pool,
                tok.binds,
            );
            let cands = sink.where_tree.candidates(&binds, &span);
            let proven: HashSet<i64> = cands.proven.into_iter().collect();
            let maybe: HashSet<i64> = cands.maybe.into_iter().collect();
            for &tid in tids {
                // Uncovered ids are not in the tree — always eval.
                // Covered + miss is a proven fail (or a raise we suppress).
                if sink.where_tree.covers(tid) && !proven.contains(&tid) && !maybe.contains(&tid) {
                    continue;
                }
                if proven.contains(&tid) && sink.where_tree.is_pure_cmp(tid) {
                    census_count("filter:test-reuse");
                    census_count("filter:test-pass");
                    if sink.beta_readers.contains(&tid) {
                        beta_written(tid, 1);
                        sink.wm.beta.entry(tid).or_default().push(*tok);
                    }
                    sink.d_beta.entry(tid).or_default().push(*tok);
                    continue;
                }
                census_count("filter:test-evals");
                if exec_stashed_where(sink.compiled_wheres, tid, &binds, sink.sym)? {
                    census_count("filter:test-pass");
                    if sink.beta_readers.contains(&tid) {
                        beta_written(tid, 1);
                        sink.wm.beta.entry(tid).or_default().push(*tok);
                    }
                    sink.d_beta.entry(tid).or_default().push(*tok);
                }
            }
        }
    } else {
        for &tid in tids {
            for tok in tokens {
                census_count("filter:test-evals");
                if exec_stashed_where(
                    sink.compiled_wheres,
                    tid,
                    &bind_view(
                        &sink.wm.bind_keys,
                        &sink.wm.bind_vals,
                        &sink.wm.bind_pool,
                        tok.binds,
                    ),
                    sink.sym,
                )? {
                    census_count("filter:test-pass");
                    if sink.beta_readers.contains(&tid) {
                        beta_written(tid, 1);
                        sink.wm.beta.entry(tid).or_default().push(*tok);
                    }
                    sink.d_beta.entry(tid).or_default().push(*tok);
                }
            }
        }
    }
    Ok(())
}

mod acc;
use acc::*;
mod delta;
pub(crate) use delta::*;
mod rules;
pub(crate) use rules::*;
