//! S1 spike (Round 6, re-investigated Round 8) — dense-alphabet double-array
//! trie. **NOT INTEGRATED.**
//!
//! ## Status: STOPPED per the S1/S1b stop rule — kept as a documented spike.
//!
//! The build speedup target (≥3×) is **exceeded** but the non-negotiable
//! byte-identical correctness gate **still fails**, so this is deliberately NOT
//! wired into the segmenter (the byte-keyed `datrie.rs` remains the production
//! path). Measured Round 8 on the full 72,175-word merged vocab
//! (`examples/s1b_time.rs`, `examples/s1b_diff.rs`):
//!   - **cold build: byte 57.1 s → symbol 8.2 s = 6.96×** (speed gate PASSES)
//!   - trie arrays: 16.78 MB → 8.39 MB; alphabet = 136 symbols
//!   - **differential test: FAILS** — the byte and symbol tries disagree on the
//!     `longest_prefix` end offset for many vocab words.
//!
//! ## Round 8 re-investigation (what was fixed, what remains)
//!
//! Two real defects were found and fixed here, and the true root cause was
//! isolated — but the invariant fix needed to pass the gate is a non-trivial
//! rework, so the spike stays stopped (same discipline as R6):
//!
//! 1. **Non-determinism (fixed).** `Alphabet::build` assigned symbol ids in
//!    `HashMap` iteration order, so the trie shape — and the number of
//!    mismatches — changed run to run (observed 31 / 33 / 80 on identical
//!    input). Now ids are assigned in sorted codepoint order, so the build is
//!    reproducible. Revealingly, the *sorted* (contiguous-id) layout is far
//!    MORE pathological (~8,990 mismatches) than the accidental sparse layouts —
//!    a direct pointer at the root cause below.
//!
//! 2. **In-place relocation overlap (fixed).** `resolve_collision` moved a
//!    node's children in place; with a dense alphabet `find_new_base` routinely
//!    returns a `new_base` whose destination region OVERLAPS the `old_base`
//!    region, so an early child's write could clobber a later sibling's
//!    not-yet-read old slot. The move now snapshots all moving children first,
//!    clears old slots, then writes — decoupling reads from writes. (The byte
//!    trie shares this loop but its sparse 256-wide layout almost never overlaps,
//!    which is why it never tripped.) This is strictly more correct but did NOT
//!    change the mismatch count, so it was not the dominant cause.
//!
//! 3. **Root cause (NOT yet fixed) — a base-region invariant violation.**
//!    Tracing a failing word (`กะพรูดกะพราด`) shows two DISTINCT parent nodes
//!    whose `base` values differ by 1 both mapping a child into the SAME slot
//!    (e.g. parent A base 10112 + sym 90 and parent B base 10113 + sym 89 both
//!    target slot 10202). That violates the fundamental double-array invariant
//!    (one slot ↔ one parent). With sparse byte ids this near-adjacency is
//!    astronomically unlikely; with 136 contiguous symbol ids it is common, so
//!    `find_new_base`'s occupancy check is not sufficient to keep sibling
//!    subtrees' base windows disjoint under heavy relocation. Fixing it means
//!    reworking base allocation (e.g. a free-slot linked list / disjoint-window
//!    guarantee, à la Aoe/darts-clone) — more than a one-line patch, and out of
//!    scope for one unattended night.
//!
//! The 6.96× is real and the approach is sound; the remaining work is the
//! base-allocation rework. This module is compiled and unit-tested (small-scale,
//! green) and exposes `differential()` + the `s1b_*` examples so the gate can be
//! re-checked, but it is referenced by nothing in the engine, so it cannot
//! affect the demo. See `VERIFY_R8.md` / `BENCHMARKS.md`.
//!
//! ---
//!
//! The vendored byte-keyed `datrie.rs` scans `0..256` on every collision
//! resolution and is pathological for Thai UTF-8 (every char is 3 bytes, all
//! starting 0xE0, so trie depth = 3×char-count and branching is ~2 at shallow
//! nodes → collision cascades). This spike remaps each distinct `char` to a
//! dense `u16` symbol before insertion, so:
//!   - Thai text is 1 symbol/char instead of 3 bytes/char (depth ÷3),
//!   - collision scans are `0..alphabet_len` (~136) not `0..256`.
//!
//! `char 0` / symbol id 0 is reserved as "unseen" and never matches, so an OOV
//! char cannot alias into a real symbol.

use std::collections::HashMap;

const UNDEF: u32 = u32::MAX;

/// Symbol alphabet: distinct chars → dense ids (1-based; 0 = unseen/never-match).
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Alphabet {
    to_id: HashMap<char, u16>,
    len: usize, // number of ids INCLUDING the reserved 0
}

impl Alphabet {
    /// Build from all chars appearing in the vocabulary. Deterministic: chars
    /// are assigned ids in sorted (codepoint) order so the trie shape does not
    /// depend on `HashMap` iteration order (S1b — the layout-dependence that
    /// masked the relocation behaviour).
    pub fn build<'a, I: IntoIterator<Item = &'a str>>(words: I) -> Self {
        let mut chars: std::collections::BTreeSet<char> = std::collections::BTreeSet::new();
        for w in words {
            for c in w.chars() {
                chars.insert(c);
            }
        }
        let mut to_id = HashMap::new();
        let mut next: u16 = 1; // 0 reserved
        for c in chars {
            to_id.insert(c, next);
            next += 1;
        }
        Self { to_id, len: next as usize }
    }

    #[inline]
    pub fn alphabet_len(&self) -> usize {
        self.len
    }

    /// Encode a string into (symbols, byte_offsets) where byte_offsets[i] is the
    /// byte offset in `s` at which symbol i starts, plus a final entry = s.len().
    /// An unseen char maps to symbol 0 (never matches any transition).
    #[inline]
    pub fn encode(&self, s: &str) -> (Vec<u16>, Vec<u32>) {
        let mut syms = Vec::with_capacity(s.len());
        let mut offs = Vec::with_capacity(s.len() + 1);
        for (bi, c) in s.char_indices() {
            offs.push(bi as u32);
            syms.push(self.to_id.get(&c).copied().unwrap_or(0));
        }
        offs.push(s.len() as u32);
        (syms, offs)
    }
}

/// Double-array trie keyed by dense `u16` symbols.
#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct SymbolDatrie {
    base: Vec<i32>,
    check: Vec<u32>,
    value: Vec<Option<u32>>,
    alphabet_len: usize,
}

impl SymbolDatrie {
    fn with_capacity(cap: usize, alphabet_len: usize) -> Self {
        let mut base = vec![0i32; cap];
        let check = vec![UNDEF; cap];
        let value = vec![None; cap];
        base[0] = 1;
        Self { base, check, value, alphabet_len }
    }

    fn grow_to(&mut self, min_len: usize) {
        if self.base.len() >= min_len {
            return;
        }
        let new_len = min_len.next_power_of_two();
        self.base.resize(new_len, 0);
        self.check.resize(new_len, UNDEF);
        self.value.resize(new_len, None);
    }

    fn insert(&mut self, key: &[u16], val: u32) {
        let mut state: usize = 0;
        for &sym in key {
            let b = sym as usize;
            let child = (self.base[state] as usize).checked_add(b).expect("overflow");
            self.grow_to(child + 1);
            if self.check[child] == UNDEF {
                self.check[child] = state as u32;
                self.base[child] = 0;
                state = child;
            } else if self.check[child] == state as u32 {
                state = child;
            } else {
                self.resolve_collision(state, b);
                let new_child = (self.base[state] as usize) + b;
                self.grow_to(new_child + 1);
                self.check[new_child] = state as u32;
                self.base[new_child] = 0;
                state = new_child;
            }
        }
        self.value[state] = Some(val);
    }

    fn resolve_collision(&mut self, parent: usize, sym: usize) {
        let child = (self.base[parent] as usize) + sym;
        let occupant = self.check[child] as usize;
        let parent_children = self.children_at(parent);
        let occ_children = self.children_at(occupant);
        let (loser, loser_children) = if parent_children.len() <= occ_children.len() {
            (parent, &parent_children)
        } else {
            (occupant, &occ_children)
        };
        let new_base = self.find_new_base(loser_children, if loser == parent { Some(sym) } else { None });
        let old_base = self.base[loser] as usize;
        // Snapshot every moving child's slot BEFORE mutating anything. With the
        // dense symbol alphabet, `find_new_base` frequently returns a `new_base`
        // whose destination region OVERLAPS the `old_base` region; moving in
        // place would let an early child's write clobber a later child's
        // not-yet-read old slot (the scale-triggered "collision-relocation" bug).
        // Snapshotting decouples reads from writes. (The byte trie shares this
        // loop but its sparse 256-wide layout almost never overlaps, so it never
        // tripped — S1b.)
        struct Moved {
            new_idx: usize,
            old_idx: usize,
            check: u32,
            base: i32,
            value: Option<u32>,
        }
        let moved: Vec<Moved> = loser_children
            .iter()
            .map(|&c| {
                let old_idx = old_base + c;
                Moved {
                    new_idx: new_base + c,
                    old_idx,
                    check: self.check[old_idx],
                    base: self.base[old_idx],
                    value: self.value[old_idx],
                }
            })
            .collect();
        // Clear all old slots first (from the snapshot), so overlapping writes
        // below cannot resurrect a stale occupant.
        for m in &moved {
            self.check[m.old_idx] = UNDEF;
            self.base[m.old_idx] = 0;
            self.value[m.old_idx] = None;
        }
        // Now place each child at its new slot from the snapshot, and reparent
        // its grandchildren (whose `check` still points at the OLD index).
        for m in &moved {
            self.grow_to(m.new_idx + 1);
            self.check[m.new_idx] = m.check;
            self.base[m.new_idx] = m.base;
            self.value[m.new_idx] = m.value;
            self.reparent_children(m.new_idx, m.old_idx);
        }
        self.base[loser] = new_base as i32;
    }

    /// Scan only `0..alphabet_len` (dense) instead of `0..256`.
    fn children_at(&self, s: usize) -> Vec<usize> {
        let b = self.base[s] as usize;
        let mut out = Vec::new();
        for sym in 0..self.alphabet_len {
            let child = b + sym;
            if child < self.check.len() && self.check[child] == s as u32 {
                out.push(sym);
            }
        }
        out
    }

    fn reparent_children(&mut self, new_idx: usize, old_idx: usize) {
        let b = self.base[new_idx] as usize;
        for sym in 0..self.alphabet_len {
            let child = b + sym;
            if child < self.check.len() && self.check[child] == old_idx as u32 {
                self.check[child] = new_idx as u32;
            }
        }
    }

    fn find_new_base(&self, children: &[usize], extra: Option<usize>) -> usize {
        let collides = |cand: usize| -> bool {
            if let Some(b) = extra {
                let idx = cand + b;
                if idx < self.check.len() && self.check[idx] != UNDEF {
                    return true;
                }
            }
            for &b in children {
                let idx = cand + b;
                if idx < self.check.len() && self.check[idx] != UNDEF {
                    return true;
                }
            }
            false
        };
        let mut cand = 1usize;
        while collides(cand) {
            cand += 1;
        }
        cand
    }

    #[inline]
    fn longest_prefix(&self, syms: &[u16], start: usize) -> Option<(u32, usize)> {
        let mut state: usize = 0;
        let mut best: Option<(u32, usize)> = None;
        for (i, &sym) in syms.iter().enumerate().skip(start) {
            if sym == 0 {
                break; // unseen char — never matches
            }
            let child = (self.base[state] as usize).wrapping_add(sym as usize);
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

/// Symbol-keyed vocab: alphabet + trie. Parallels `DatrieVocab` but keyed by
/// dense symbols. Longest-prefix returns a symbol end-index; callers map it back
/// to a byte offset via the encoded `byte_offsets`.
#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct SymbolVocab {
    alphabet: Alphabet,
    trie: SymbolDatrie,
}

impl SymbolVocab {
    /// Build from `HashMap<Vec<u8>, id>` (same input shape as `DatrieVocab`).
    /// Keys must be valid UTF-8 (they are — they come from `&str` words).
    pub fn build(vocab: &HashMap<Vec<u8>, usize>) -> Self {
        // Alphabet over all chars in all keys.
        let words: Vec<&str> = vocab.keys().filter_map(|k| std::str::from_utf8(k).ok()).collect();
        let alphabet = Alphabet::build(words.iter().copied());
        let cap = (vocab.len() * 2).max(256);
        let mut trie = SymbolDatrie::with_capacity(cap, alphabet.alphabet_len());
        // Deterministic insertion order (sort by symbol-encoded key).
        let mut entries: Vec<(Vec<u16>, u32)> = vocab
            .iter()
            .filter_map(|(k, &id)| std::str::from_utf8(k).ok().map(|s| (alphabet.encode(s).0, id as u32)))
            .collect();
        entries.sort();
        for (syms, id) in &entries {
            trie.insert(syms, *id);
        }
        Self { alphabet, trie }
    }

    /// Longest-prefix match from byte offset `start` in `input`. Returns
    /// `(token_id, end_byte_offset)` — byte offsets identical to the byte trie.
    #[inline]
    pub fn longest_prefix_bytes(&self, input: &str, start_byte: usize) -> Option<(usize, usize)> {
        let (syms, offs) = self.alphabet.encode(input);
        // Find the symbol index whose byte offset == start_byte.
        let start_sym = match offs.binary_search(&(start_byte as u32)) {
            Ok(i) => i,
            Err(_) => return None, // start not on a char boundary
        };
        self.trie.longest_prefix(&syms, start_sym).map(|(v, end_sym)| {
            (v as usize, offs[end_sym] as usize)
        })
    }

    pub fn alphabet_len(&self) -> usize {
        self.alphabet.alphabet_len()
    }

    pub fn inner_bytes(&self) -> usize {
        let n = self.trie.base.len();
        n * 4 + n * 4 + n * std::mem::size_of::<Option<u32>>()
    }
}

/// One differential mismatch between the byte trie and the symbol trie.
#[derive(Debug, Clone)]
pub struct SegDiff {
    pub word: String,
    /// `longest_prefix` end byte offset from the byte trie (the ground truth).
    pub byte_end: Option<usize>,
    /// `longest_prefix` end byte offset from the symbol trie.
    pub sym_end: Option<usize>,
}

/// S1b differential: build BOTH the byte `DatrieVocab` and the symbol
/// `SymbolVocab` from the *identical* deduplicated vocab (same construction as
/// `Segmenter::from_words`), then for every distinct word compare
/// `longest_prefix` starting at offset 0. Returns every word where the two
/// tries disagree on the matched end offset. An empty result == byte-identical.
pub fn differential<I, S>(words: I) -> Vec<SegDiff>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    use crate::datrie::DatrieVocab;
    // Same dedup + id assignment as Segmenter::from_words.
    let mut vocab_map: HashMap<Vec<u8>, usize> = HashMap::new();
    let mut idx = 0usize;
    let mut distinct: Vec<String> = Vec::new();
    for w in words {
        let w = w.as_ref().trim();
        if w.is_empty() {
            continue;
        }
        vocab_map.entry(w.as_bytes().to_vec()).or_insert_with(|| {
            let i = idx;
            idx += 1;
            distinct.push(w.to_string());
            i
        });
    }
    let byte = DatrieVocab::build(&vocab_map);
    let sym = SymbolVocab::build(&vocab_map);
    let mut diffs = Vec::new();
    for w in &distinct {
        let b = byte.longest_prefix(w.as_bytes(), 0).map(|(_, end)| end);
        let s = sym.longest_prefix_bytes(w, 0).map(|(_, end)| end);
        if b != s {
            diffs.push(SegDiff { word: w.clone(), byte_end: b, sym_end: s });
        }
    }
    diffs
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_vocab(pairs: &[(&str, usize)]) -> HashMap<Vec<u8>, usize> {
        pairs.iter().map(|&(k, v)| (k.as_bytes().to_vec(), v)).collect()
    }

    #[test]
    fn symbol_longest_prefix_matches_bytes_semantics() {
        let vocab = make_vocab(&[("บ้าน", 0), ("บ้านเรือน", 1), ("เรือน", 2)]);
        let sv = SymbolVocab::build(&vocab);
        // "บ้านเรือน" longest prefix from 0 = บ้านเรือน (id 1), end = full byte len.
        let full = "บ้านเรือน";
        assert_eq!(sv.longest_prefix_bytes(full, 0), Some((1, full.len())));
        // From the byte offset of เรือน within บ้านเรือน:
        let ban = "บ้าน";
        assert_eq!(sv.longest_prefix_bytes(full, ban.len()), Some((2, full.len())));
        // OOV
        assert_eq!(sv.longest_prefix_bytes("xyz", 0), None);
    }

    #[test]
    fn unseen_char_never_matches() {
        let vocab = make_vocab(&[("cat", 0)]);
        let sv = SymbolVocab::build(&vocab);
        // 'z' unseen -> symbol 0 -> no match starting there
        assert_eq!(sv.longest_prefix_bytes("zcat", 0), None);
        // but "cat" after it matches
        assert_eq!(sv.longest_prefix_bytes("zcat", 1), Some((0, 4)));
    }
}
