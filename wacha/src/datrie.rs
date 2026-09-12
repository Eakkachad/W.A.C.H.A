//! Double-array trie for zero-alloc byte-key lookups (Research 137).
//!
//! Vendored (2026-09-12) from `katgpt-rs/crates/katgpt-tokenizer/src/datrie.rs`
//! (MIT-licensed, same repo `katgpt-rs`'s tokenizer primitives this project's
//! segmentation layer is built on) — verbatim except: (1) only the `Datrie`
//! core and `DatrieVocab` (T1) are kept, `DatrieTreeIndex` (T2, used only by
//! `katgpt-tokenizer`'s unrelated ToaST split-tree feature) is dropped, it was
//! never used here; (2) includes two fixes made in this project and not yet
//! committed upstream in `katgpt-rs` (which is not this project's repository
//! to commit to) — see `../PROGRESS.md` 2026-09-04 and 2026-09-12:
//!   - `insert`'s collision path calls `grow_to` instead of asserting the
//!     arrays are already large enough (they aren't, always — the assert was
//!     a real panic bug at real-dictionary scale).
//!   - `Datrie`/`DatrieVocab` derive `Serialize`/`Deserialize` so the built
//!     segmenter can be cached to disk (see `segmenter.rs`'s cache methods).
//! Vendoring this (instead of a path dependency on `katgpt-rs`) makes `wacha`
//! fully self-contained: nothing outside this repo can break it.
//!
//! **Source:** Aoe, J. (1989). An efficient digital search algorithm by using
//! a double-array structure. IEEE TSE.

use std::collections::HashMap;

// ── Core double-array trie ──────────────────────────────────────────────────

/// Double-array trie storing an optional `u32` value at each node.
///
/// Invariant: for a transition on byte `b` from state `s`:
///   child = base[s] + b  (as usize, base is always ≥ 0 after build)
///   check[child] == s     (ownership guard)
///   value[child] = Some(v) if this node is a terminal
#[derive(serde::Serialize, serde::Deserialize)]
struct Datrie {
    base: Vec<i32>,
    check: Vec<u32>,
    value: Vec<Option<u32>>,
}

impl Clone for Datrie {
    fn clone(&self) -> Self {
        Self {
            base: self.base.clone(),
            check: self.check.clone(),
            value: self.value.clone(),
        }
    }
}

impl std::fmt::Debug for Datrie {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Datrie")
            .field("slots", &self.base.len())
            .finish()
    }
}

/// Sentinel value: check[s] == UNDEF means slot `s` is unoccupied.
const UNDEF: u32 = u32::MAX;

impl Datrie {
    /// Build an empty trie with a pre-allocated capacity.
    fn with_capacity(cap: usize) -> Self {
        let mut base = vec![0i32; cap];
        let check = vec![UNDEF; cap];
        let value = vec![None; cap];
        // Slot 0 is the root; base[0] will be assigned during build.
        base[0] = 1; // root starts pointing past itself
        Self { base, check, value }
    }

    /// Ensure the arrays are at least `min_len` elements.
    fn grow_to(&mut self, min_len: usize) {
        if self.base.len() >= min_len {
            return;
        }
        let new_len = min_len.next_power_of_two();
        self.base.resize(new_len, 0);
        self.check.resize(new_len, UNDEF);
        self.value.resize(new_len, None);
    }

    /// Insert `key → val`. Panics on duplicate keys.
    fn insert(&mut self, key: &[u8], val: u32) {
        let mut state: usize = 0;

        for &byte in key {
            let b = byte as usize;
            let child = (self.base[state] as usize)
                .checked_add(b)
                .expect("base + byte overflow");

            self.grow_to(child + 1);

            if self.check[child] == UNDEF {
                // Free slot — claim it.
                self.check[child] = state as u32;
                // base[child] starts at 0; will be fixed if it gets children.
                self.base[child] = 0;
                state = child;
            } else if self.check[child] == state as u32 {
                // Already our child — walk down.
                state = child;
            } else {
                // Collision — the slot is owned by another parent.
                // Relocate the current node's existing children.
                self.resolve_collision(state, b, child);
                // After resolution, `base[state] + b` is now ours.
                let new_child = (self.base[state] as usize) + b;
                // `find_new_base` may choose a base whose slot lies past the
                // current array end (out-of-range slots are treated as free);
                // ensure the arrays are large enough before claiming the slot.
                self.grow_to(new_child + 1);
                self.check[new_child] = state as u32;
                self.base[new_child] = 0;
                state = new_child;
            }
        }

        assert!(
            self.value[state].is_none(),
            "duplicate key in datrie (slot {state})"
        );
        self.value[state] = Some(val);
    }

    /// Resolve a collision at `child` when trying to insert byte `b` from `parent`.
    ///
    /// Strategy: relocate whichever node has fewer children (the loser). This
    /// is the classic Aoe approach.
    fn resolve_collision(&mut self, parent: usize, byte: usize, child: usize) {
        let occupant = self.check[child] as usize;

        // Collect children of parent and occupant at current base positions.
        let parent_children = self.children_at(parent);
        let occ_children = self.children_at(occupant);

        // Pick the node with fewer children to relocate.
        let (loser, loser_children) = if parent_children.len() <= occ_children.len() {
            (parent, &parent_children)
        } else {
            (occupant, &occ_children)
        };

        // Find a new base for the loser that accommodates all its children
        // AND the new byte (if the loser is `parent`).
        let new_base = self.find_new_base(
            loser,
            loser_children,
            if loser == parent { Some(byte) } else { None },
        );

        // Relocate loser's children to new_base.
        let old_base = self.base[loser] as usize;
        for &c in loser_children {
            let old_idx = old_base + c;
            let new_idx = new_base + c;

            self.grow_to(new_idx + 1);
            // Move slot.
            self.check[new_idx] = self.check[old_idx];
            self.base[new_idx] = self.base[old_idx];
            self.value[new_idx] = self.value[old_idx].take();

            // Patch children of the moved node to point back to new_idx.
            self.reparent_children(new_idx, old_idx);

            // Free old slot.
            self.check[old_idx] = UNDEF;
            self.base[old_idx] = 0;
        }

        self.base[loser] = new_base as i32;
    }

    /// Collect byte values of children of node `s` at its current base.
    fn children_at(&self, s: usize) -> Vec<usize> {
        let b = self.base[s] as usize;
        // A node can have at most 256 children (one per byte value).
        let mut out = Vec::with_capacity(256);
        // Scan all 256 possible byte offsets.
        for byte in 0..256 {
            let child = b + byte;
            if child < self.check.len() && self.check[child] == s as u32 {
                out.push(byte);
            }
        }
        out
    }

    /// Reparent: after moving a node from `old_idx` to `new_idx`, all its
    /// children have `check[child] == old_idx`; update them to `new_idx`.
    fn reparent_children(&mut self, new_idx: usize, old_idx: usize) {
        let b = self.base[new_idx] as usize;
        for byte in 0..256u32 {
            let child = b + byte as usize;
            if child < self.check.len() && self.check[child] == old_idx as u32 {
                self.check[child] = new_idx as u32;
            }
        }
    }

    /// Find a new base for `loser` that can host all `children` bytes
    /// (and optionally `extra_byte`) without collisions.
    //
    // Zero-allocation: iterates over `children` and optionally checks the
    // `extra_byte` separately instead of building a merged Vec.
    fn find_new_base(&self, _loser: usize, children: &[usize], extra_byte: Option<usize>) -> usize {
        // Collision predicate at a given candidate base: true if any byte in
        // the required set (children ∪ {extra_byte}) collides with an occupied slot.
        let collides = |candidate: usize, eb: Option<usize>| -> bool {
            // Check `extra_byte` first when present: it's the byte that triggered
            // the collision, so it's the most likely to keep colliding.
            if let Some(b) = eb {
                let idx = candidate + b;
                if idx < self.check.len() && self.check[idx] != UNDEF {
                    return true;
                }
            }
            for &b in children {
                let idx = candidate + b;
                if idx < self.check.len() && self.check[idx] != UNDEF {
                    return true;
                }
            }
            false
        };

        // Try successive offsets until we find a collision-free base.
        let mut candidate = 1usize;
        while collides(candidate, extra_byte) {
            candidate += 1;
        }
        candidate
    }

    /// Look up `key` in the trie. Returns `Some(value)` if found, `None` otherwise.
    /// Zero allocations.
    #[inline]
    fn lookup(&self, key: &[u8]) -> Option<u32> {
        let mut state: usize = 0;
        for &byte in key {
            let child = (self.base[state] as usize).wrapping_add(byte as usize);
            if child >= self.check.len() || self.check[child] != state as u32 {
                return None;
            }
            state = child;
        }
        self.value[state]
    }

    /// Longest-prefix match: walk `input` from `start`, returning
    /// `(value, end_offset)` of the longest matching prefix.
    /// Returns `None` if no prefix matches.
    #[inline]
    fn longest_prefix(&self, input: &[u8], start: usize) -> Option<(u32, usize)> {
        let mut state: usize = 0;
        let mut best: Option<(u32, usize)> = None;

        for (i, &byte) in input.iter().enumerate().skip(start) {
            let child = (self.base[state] as usize).wrapping_add(byte as usize);
            if child >= self.check.len() || self.check[child] != state as u32 {
                break;
            }
            state = child;
            if let Some(v) = self.value[state] {
                best = Some((v, i + 1));
            }
        }
        best
    }
}

// ── DatrieVocab ──────────────────────────────────────────────────────────────

/// Double-array trie replacing `HashMap<Vec<u8>, usize>` for token vocab lookup.
///
/// Build once from the tokenizer vocabulary, then use `lookup()` during encode.
/// Zero allocations on the hot path.
#[derive(Clone, Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct DatrieVocab {
    inner: Datrie,
}

impl DatrieVocab {
    /// Build a `DatrieVocab` from a `HashMap<Vec<u8>, usize>`.
    ///
    /// Keys are token byte sequences, values are token IDs.
    /// The input HashMap is not consumed (it may be needed for decode).
    pub fn build(vocab: &HashMap<Vec<u8>, usize>) -> Self {
        // Pre-allocate ~2× the vocab size (heuristic for trie density).
        let cap = (vocab.len() * 2).max(256);
        let mut trie = Datrie::with_capacity(cap);

        // Sort keys for better locality during insertion (fewer collisions).
        let mut entries: Vec<_> = vocab.iter().collect();
        entries.sort_by(|a, b| a.0.cmp(b.0));

        for (key, id) in &entries {
            trie.insert(key, **id as u32);
        }

        Self { inner: trie }
    }

    /// Look up a token by its byte sequence. Returns the token ID or `None`.
    #[inline]
    pub fn lookup(&self, key: &[u8]) -> Option<usize> {
        self.inner.lookup(key).map(|v| v as usize)
    }

    /// Longest-prefix match from `start` in `input`. Returns `(token_id, end_offset)`.
    #[inline]
    pub fn longest_prefix(&self, input: &[u8], start: usize) -> Option<(usize, usize)> {
        self.inner
            .longest_prefix(input, start)
            .map(|(v, end)| (v as usize, end))
    }

    /// Total bytes used by the internal arrays (base + check + value).
    pub fn inner_bytes(&self) -> usize {
        let n = self.inner.base.len();
        n * 4 + // base: Vec<i32>
        n * 4 + // check: Vec<u32>
        n * std::mem::size_of::<Option<u32>>() // value: Vec<Option<u32>>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_vocab(pairs: &[(&[u8], usize)]) -> HashMap<Vec<u8>, usize> {
        pairs.iter().map(|&(k, v)| (k.to_vec(), v)).collect()
    }

    #[test]
    fn datrie_basic_lookup() {
        let vocab = make_vocab(&[
            (b"hello", 0),
            (b"hell", 1),
            (b"he", 2),
            (b"cat", 3),
            (b"cats", 4),
        ]);
        let trie = DatrieVocab::build(&vocab);

        assert_eq!(trie.lookup(b"hello"), Some(0));
        assert_eq!(trie.lookup(b"hell"), Some(1));
        assert_eq!(trie.lookup(b"he"), Some(2));
        assert_eq!(trie.lookup(b"cat"), Some(3));
        assert_eq!(trie.lookup(b"cats"), Some(4));
        assert_eq!(trie.lookup(b"h"), None);
        assert_eq!(trie.lookup(b"hello!"), None);
        assert_eq!(trie.lookup(b"dog"), None);
    }

    #[test]
    fn datrie_longest_prefix() {
        let vocab = make_vocab(&[(b"ab", 0), (b"abcd", 1), (b"abcdef", 2)]);
        let trie = DatrieVocab::build(&vocab);

        assert_eq!(trie.longest_prefix(b"abcdef", 0), Some((2, 6)));
        assert_eq!(trie.longest_prefix(b"abcdefg", 0), Some((2, 6)));
        assert_eq!(trie.longest_prefix(b"abcdxz", 0), Some((1, 4)));
        assert_eq!(trie.longest_prefix(b"abzz", 0), Some((0, 2)));
        assert_eq!(trie.longest_prefix(b"xyz", 0), None);
    }

    #[test]
    fn datrie_single_byte_tokens() {
        let vocab: HashMap<Vec<u8>, usize> = (0u8..=255).map(|b| (vec![b], b as usize)).collect();
        let trie = DatrieVocab::build(&vocab);

        for b in 0u8..=255 {
            assert_eq!(trie.lookup(&[b]), Some(b as usize));
        }
        assert_eq!(trie.lookup(b"ab"), None);
    }

    #[test]
    fn datrie_empty_key() {
        let vocab = make_vocab(&[(b"", 42), (b"a", 1)]);
        let trie = DatrieVocab::build(&vocab);
        assert_eq!(trie.lookup(b""), Some(42));
        assert_eq!(trie.lookup(b"a"), Some(1));
    }

    #[test]
    fn datrie_large_vocab() {
        // Simulate a realistic vocab with varied-length tokens.
        let mut vocab = HashMap::new();
        for i in 0..5000u32 {
            let key = format!("token_{i}").into_bytes();
            vocab.insert(key, i as usize);
        }
        // Add single-byte tokens.
        for b in 0u8..255 {
            vocab.insert(vec![b], b as usize + 10000);
        }

        let trie = DatrieVocab::build(&vocab);

        // Spot-check.
        assert_eq!(trie.lookup(b"token_0"), Some(0));
        assert_eq!(trie.lookup(b"token_4999"), Some(4999));
        assert_eq!(trie.lookup(b"token_5000"), None);
        assert_eq!(trie.lookup(&[0u8]), Some(10000));
        assert_eq!(trie.lookup(&[254u8]), Some(10254));
    }

    /// Regression test for the vendored `grow_to` fix (see module docs): this
    /// vocab size was exactly what tripped the old `assert!` panic on the real
    /// 62k-word Thai list, which the tiny hand-written vocabs above never hit.
    #[test]
    fn datrie_handles_collision_growth_past_array_end() {
        // A vocab dense enough, with enough shared-prefix collisions, to force
        // `find_new_base` to pick a base past the current array end during
        // `insert`'s collision path — exactly the case the old assert missed.
        let mut vocab = HashMap::new();
        for i in 0..20_000u32 {
            // Shared multi-byte UTF-8-like prefix on every key (mimics Thai's
            // narrow leading-byte range) to force heavy collisions.
            let key = format!("\u{0e01}{i}").into_bytes();
            vocab.insert(key, i as usize);
        }
        let trie = DatrieVocab::build(&vocab); // must not panic
        assert_eq!(trie.lookup("\u{0e01}0".as_bytes()), Some(0));
        assert_eq!(trie.lookup("\u{0e01}19999".as_bytes()), Some(19999));
    }
}
