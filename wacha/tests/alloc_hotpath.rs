//! E1 (Round 8) — measure the ACTUAL allocation count on the hot query paths
//! with a global `CountingAllocator`, so any "low/zero-alloc" statement is
//! backed by a counter reading, not by inspection.
//!
//! Pattern copied (not imported) from
//! `katgpt-rs/crates/katgpt-dec/tests/common/counting_allocator.rs` — this test
//! is self-contained and does NOT touch `katgpt-rs`.
//!
//! Hot paths measured on a seed-only engine (no external data files needed):
//!   - `Segmenter::segment` — the greedy longest-match trie walk (the tokenizer
//!     hot loop; the datrie `longest_prefix` is documented "zero allocations").
//!   - `Engine::related_ranked` — the ranked relation lookup.
//!
//! We report the measured deltas rather than asserting a hard zero. The
//! segmenter path is small and linear in tokens; the relation path turns out to
//! be **allocation-heavy** — measured tens of thousands of allocs — because
//! `related()` runs a per-query personalized PageRank over the whole graph. E1
//! reports the real counter values and only claims what they support.

use std::sync::atomic::{AtomicUsize, Ordering};

struct CountingAllocator;
static ALLOC_COUNT: AtomicUsize = AtomicUsize::new(0);
static DEALLOC_COUNT: AtomicUsize = AtomicUsize::new(0);

unsafe impl std::alloc::GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: std::alloc::Layout) -> *mut u8 {
        ALLOC_COUNT.fetch_add(1, Ordering::Relaxed);
        unsafe { std::alloc::System.alloc(layout) }
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: std::alloc::Layout) {
        DEALLOC_COUNT.fetch_add(1, Ordering::Relaxed);
        unsafe { std::alloc::System.dealloc(ptr, layout) }
    }
}

#[global_allocator]
static A: CountingAllocator = CountingAllocator;

#[inline]
fn alloc_delta<R>(f: impl FnOnce() -> R) -> (R, usize) {
    let before = ALLOC_COUNT.load(Ordering::Relaxed);
    let r = f();
    let after = ALLOC_COUNT.load(Ordering::Relaxed);
    (r, after - before)
}

#[test]
fn hot_path_allocation_counts_are_bounded_and_reported() {
    let engine = wacha::Engine::seed_only();

    // Warm any lazy statics so we measure the steady-state path, not one-time init.
    let _ = engine.segment("แมวกินปลา");
    let _ = engine.related_ranked("แมว", 5);

    // ── segment() hot path ──
    // Single in-vocab word: minimal work (one trie walk + one output token).
    let (_toks, a_one) = alloc_delta(|| engine.segment("แมว"));
    // A short multi-word Thai sentence.
    let (toks, a_sentence) = alloc_delta(|| engine.segment("แมวกินปลาที่บ้าน"));
    let n_tokens = toks.len();

    // ── related_ranked() hot path ──
    let (rel, a_related) = alloc_delta(|| engine.related_ranked("แมว", 5));
    let n_related = rel.len();

    // ── datrie longest_prefix (the documented "zero allocations" claim) ──
    // Segmenting an OOV/ASCII string exercises the trie walk + TCC fallback.
    let (_t, a_lookup_miss) = alloc_delta(|| engine.segment("x"));

    println!("=== E1 hot-path allocation counts (measured, CountingAllocator) ===");
    println!("segment(\"แมว\")            allocs = {a_one}");
    println!("segment(16-char sentence) allocs = {a_sentence}  ({n_tokens} tokens)");
    println!("related_ranked(\"แมว\", 5)  allocs = {a_related}  ({n_related} related)");
    println!("segment(\"x\") [OOV 1-char] allocs = {a_lookup_miss}");
    println!("total allocs so far = {}", ALLOC_COUNT.load(Ordering::Relaxed));

    // MEASURED FINDINGS (honest, from the counter — see VERIFY_R9 §E1/Q1):
    //
    // * `segment` (default = maximal-matching DP since R9 Q1) is still cheap and
    //   O(tokens), just a higher constant than greedy: a single in-vocab word is
    //   ~14 allocs (the DP's cost/prev/ends vectors + owned token Strings), a
    //   10-token sentence ~132. Bounded and linear, NOT zero-alloc.
    // * `related_ranked` is ALLOC-HEAVY: tens of thousands of allocs, because
    //   `related()` runs a per-query personalized PageRank over the whole graph.
    //   No zero-alloc claim is made for the relation path.
    //
    // Assertions pin the segment path's small linear bound; the relation path is
    // asserted only to be finite (documented as heavy, not claimed light).
    assert!(a_one <= 32, "single-word segment should be a bounded handful of allocs, got {a_one}");
    assert!(
        a_sentence <= 24 * (n_tokens + 1),
        "segment allocs ({a_sentence}) must be O(tokens={n_tokens})"
    );
    assert!(a_related > 0, "related_ranked ran"); // heavy; count reported, not bounded-claimed
    let _ = n_related;
}
