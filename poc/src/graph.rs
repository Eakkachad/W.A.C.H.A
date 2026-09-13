//! Knowledge Graph: stores entities, relations, and triples for AXIOM-Gen traversal.
//!
//! The graph uses integer IDs for efficient traversal and lookup.
//! Entity/relation names are stored separately and mapped via indices.

use std::collections::{HashMap, VecDeque};

/// A triple (subject, relation, object) in the knowledge graph.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Triple {
    pub subject_id: usize,
    pub relation_id: usize,
    pub object_id: usize,
}

/// A pair of facts that disagree on the object for the same subject/relation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Contradiction {
    pub subject: String,
    pub relation: String,
    pub first_object: String,
    pub second_object: String,
}

/// A knowledge graph storing entities, relations, and their connections as triples.
#[derive(Debug, Clone)]
pub struct KnowledgeGraph {
    /// All entity names, indexed by ID.
    pub entities: Vec<String>,
    /// All relation names, indexed by ID.
    pub relations: Vec<String>,
    /// All triples in the graph.
    pub triples: Vec<Triple>,
    /// Reverse mapping from entity name to ID.
    pub entity_index: HashMap<String, usize>,
    /// Reverse mapping from relation name to ID.
    relation_index: HashMap<String, usize>,
    /// Adjacency: entity ID → indices of triples it participates in.
    /// Enables O(degree) neighbor expansion instead of O(all triples) scans.
    pub adjacency: Vec<Vec<usize>>,
}

impl KnowledgeGraph {
    /// Create a new empty knowledge graph.
    pub fn new() -> Self {
        Self {
            entities: Vec::new(),
            relations: Vec::new(),
            triples: Vec::new(),
            entity_index: HashMap::new(),
            relation_index: HashMap::new(),
            adjacency: Vec::new(),
        }
    }

    /// Add an entity to the graph, returning its ID.
    /// If the entity already exists, returns the existing ID.
    pub fn add_entity(&mut self, name: &str) -> usize {
        if let Some(&id) = self.entity_index.get(name) {
            return id;
        }
        let id = self.entities.len();
        self.entities.push(name.to_string());
        self.entity_index.insert(name.to_string(), id);
        id
    }

    /// Add a relation to the graph, returning its ID.
    /// If the relation already exists, returns the existing ID.
    pub fn add_relation(&mut self, name: &str) -> usize {
        if let Some(&id) = self.relation_index.get(name) {
            return id;
        }
        let id = self.relations.len();
        self.relations.push(name.to_string());
        self.relation_index.insert(name.to_string(), id);
        id
    }

    /// Add a triple to the graph by entity/relation names.
    /// Automatically adds entities and relations if they don't exist.
    /// Returns the index of the added triple.
    pub fn add_triple(&mut self, subject: &str, relation: &str, object: &str) -> usize {
        let subj_id = self.add_entity(subject);
        let rel_id = self.add_relation(relation);
        let obj_id = self.add_entity(object);
        let triple = Triple {
            subject_id: subj_id,
            relation_id: rel_id,
            object_id: obj_id,
        };
        let idx = self.triples.len();
        self.triples.push(triple);
        // Maintain adjacency lists. Ensure vectors are sized to the new
        // entity count.
        while self.adjacency.len() <= subj_id.max(obj_id) {
            self.adjacency.push(Vec::new());
        }
        self.adjacency[subj_id].push(idx);
        self.adjacency[obj_id].push(idx);
        idx
    }

    /// Get the indices of all triples an entity participates in (as subject
    /// or object). O(degree), the fast adjacency path for beam expansion.
    pub fn adjacency_of(&self, entity_id: usize) -> &[usize] {
        self.adjacency
            .get(entity_id)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    /// Get all triples where the given entity is the subject.
    pub fn get_triples_from(&self, entity_id: usize) -> Vec<&Triple> {
        self.triples
            .iter()
            .filter(|t| t.subject_id == entity_id)
            .collect()
    }

    /// Export all triples as (subject, relation, object) name triples.
    ///
    /// This is the bridge to downstream consumers (e.g. the VSA-LM knowledge
    /// prior) that want the graph facts as plain strings.
    pub fn export_triples(&self) -> Vec<[String; 3]> {
        self.triples
            .iter()
            .map(|t| {
                [
                    self.entity_name(t.subject_id).to_string(),
                    self.relation_name(t.relation_id).to_string(),
                    self.entity_name(t.object_id).to_string(),
                ]
            })
            .collect()
    }

    /// Merge comma-suffixed entity variants into their clean pre-comma head.
    /// "Chicago, Illinois, 17 mi" → redirect triples to "Chicago".
    pub fn consolidate_comma_entities(&mut self) {
        self.consolidate_with(|name| {
            name.find(',').map(|p| name[..p].trim().to_string())
        });
    }

    /// Merge word-order permutations: "Hingis, Martina" → "Martina Hingis".
    /// Only merges when the reversed word set matches exactly (no false
    /// positives).  Applied after comma consolidation.
    pub fn consolidate_permutation_entities(&mut self) {
        // First, canonicalize names that are "Last, First" → "First Last".
        let n = self.entities.len();
        let mut rename: Vec<Option<String>> = vec![None; n];
        for i in 0..n {
            let name = &self.entities[i];
            if name.contains(',') {
                let parts: Vec<&str> = name.split(',').map(|s| s.trim()).collect();
                if parts.len() == 2 {
                    let both_caps = parts.iter().all(|p| {
                        p.split_whitespace().next().map(|w| w.chars().next().map(|c| c.is_uppercase()).unwrap_or(false)).unwrap_or(false)
                    });
                    if both_caps && !parts[0].is_empty() && !parts[1].is_empty() {
                        let canonical = format!("{} {}", parts[1], parts[0]);
                        // Only rename if canonical already exists OR looks like a person name.
                        if self.entity_index.contains_key(&canonical) {
                            rename[i] = Some(canonical);
                        }
                    }
                }
            }
        }
        // Apply renames: update entity strings + index.
        for i in 0..n {
            if let Some(new_name) = rename[i].clone() {
                let old = self.entities[i].clone();
                if let Some(&target) = self.entity_index.get(&new_name) {
                    if target != i {
                        // Redirect triples from old id to target.
                        for t in &mut self.triples {
                            if t.subject_id == i { t.subject_id = target; }
                            if t.object_id == i { t.object_id = target; }
                        }
                        self.entity_index.remove(&old);
                    }
                } else {
                    // Rename in place (no existing canonical entity).
                    self.entity_index.remove(&old);
                    self.entities[i] = new_name;
                    self.entity_index.insert(self.entities[i].clone(), i);
                }
            }
        }
        self.rebuild_adjacency();
    }

    fn rebuild_adjacency(&mut self) {
        self.adjacency = vec![Vec::new(); self.entities.len()];
        for (idx, t) in self.triples.iter().enumerate() {
            if t.subject_id < self.entities.len() { self.adjacency[t.subject_id].push(idx); }
            if t.object_id < self.entities.len() { self.adjacency[t.object_id].push(idx); }
        }
    }

    fn consolidate_with(&mut self, extract_head: impl Fn(&str) -> Option<String>) {
        let n = self.entities.len();
        let mut merge_target: Vec<Option<usize>> = vec![None; n];
        for i in 0..n {
            if let Some(head) = extract_head(&self.entities[i]) {
                if let Some(&target) = self.entity_index.get(&head) {
                    if target != i { merge_target[i] = Some(target); }
                }
            }
        }
        if merge_target.iter().filter(|m| m.is_some()).count() == 0 { return; }
        for t in &mut self.triples {
            if let Some(dst) = merge_target[t.subject_id] { t.subject_id = dst; }
            if let Some(dst) = merge_target[t.object_id] { t.object_id = dst; }
        }
        self.rebuild_adjacency();
        for i in 0..n {
            if merge_target[i].is_some() { self.entity_index.remove(&self.entities[i]); }
        }
    }

    /// Get all triples where the given entity is the object.
    pub fn get_triples_to(&self, entity_id: usize) -> Vec<&Triple> {
        self.triples
            .iter()
            .filter(|t| t.object_id == entity_id)
            .collect()
    }

    /// Get the ID of an entity by name, if it exists.
    pub fn entity_id(&self, name: &str) -> Option<usize> {
        self.entity_index.get(name).copied()
    }

    /// Get the name of an entity by ID.
    pub fn entity_name(&self, id: usize) -> &str {
        &self.entities[id]
    }

    /// Get total number of entities in the knowledge graph.
    pub fn entity_count(&self) -> usize {
        self.entities.len()
    }

    /// Personalized PageRank with restart, hub-corrected (T1.9c).
    ///
    /// π_q solves π_q = (1-c)·v + c·Pᵀ·π_q via power iteration, where v is a
    /// teleport distribution over `seeds` and P is the degree-normalized
    /// transition matrix. The returned score subtracts global PageRank π from
    /// personalized π_q — the FolkRank idea (Hotho et al. 2006; Jäschke et al.
    /// 2007). The `log π_q − log π` (log-ratio) form is our own variant, not the
    /// published FolkRank difference. This cancels hub popularity: an entity 1-2
    /// hops from every query gets a large π_q *and* a large π, so the correction
    /// neutralizes it. (Earlier comment mis-attributed this to Milne & Witten,
    /// whose measure has no PageRank content — corrected 2026-09-13.)
    ///
    /// Deterministic: fixed iteration count, no tolerance-based termination.
    /// Returns one score per entity (index-aligned with `self.entities`).
    pub fn personalized_pagerank(&self, seeds: &[usize], iterations: usize) -> Vec<f32> {
        let n = self.entities.len();
        if n == 0 || seeds.is_empty() {
            return vec![0.0; n];
        }
        let c = 0.85f32;
        // Degree-normalized out-transition weights. Use a uniform edge weight
        // (every triple edge counts once in each direction).
        let mut out_degree = vec![0usize; n];
        for t in &self.triples {
            out_degree[t.subject_id] += 1;
            out_degree[t.object_id] += 1;
        }
        // Transition P[u][v] = 1/out_degree(u) for each neighbor v of u.
        // Precompute neighbor lists once.
        let mut neighbors: Vec<Vec<usize>> = (0..n).map(|_| Vec::new()).collect();
        for t in &self.triples {
            neighbors[t.subject_id].push(t.object_id);
            neighbors[t.object_id].push(t.subject_id);
        }
        let step = |pi: &[f32]| -> Vec<f32> {
            let mut next = vec![0.0f32; n];
            for u in 0..n {
                let deg = out_degree[u].max(1);
                let share = pi[u] / deg as f32;
                for &v in &neighbors[u] {
                    next[v] += share;
                }
            }
            next
        };

        // Global PageRank (uniform teleport).
        let mut pi_global = vec![1.0 / n as f32; n];
        for _ in 0..iterations {
            let walked = step(&pi_global);
            for i in 0..n {
                pi_global[i] = (1.0 - c) / n as f32 + c * walked[i];
            }
        }

        // Personalized PageRank (teleport to seeds).
        let mut pi_q = vec![0.0f32; n];
        let seed_mass = 1.0 / seeds.len() as f32;
        for &s in seeds {
            pi_q[s] = seed_mass;
        }
        for _ in 0..iterations {
            let walked = step(&pi_q);
            for i in 0..n {
                let teleport = if seeds.contains(&i) { seed_mass } else { 0.0 };
                pi_q[i] = (1.0 - c) * teleport + c * walked[i];
            }
        }

        // Relative PPR with hub correction. Guard against log(0).
        let mut scores = vec![0.0f32; n];
        let min_p = 1e-6f32;
        for i in 0..n {
            let a = pi_q[i].max(min_p);
            let b = pi_global[i].max(min_p);
            scores[i] = (a / b).ln();
        }
        scores
    }

    /// Query-aware personalized PageRank (T1.18g, QASA-style gate).
    ///
    /// Same relative-PPR hub-corrected signal as [`Self::personalized_pagerank`],
    /// but the PERSONALIZED walk's mass into each node `v` is gated by
    /// `gate[v].powf(gamma)` (QASA arXiv:2606.30133: `σ(v)=max(cos(e_v,e_q),0)`
    /// — here a deterministic LEXICAL proxy, never VSA cosine). The global
    /// PageRank baseline is ungated (so hub correction still holds). The gate
    /// makes the walk drift toward nodes whose evidence discusses the question,
    /// instead of diffusing into unrelated hubs.
    pub fn personalized_pagerank_query_aware(
        &self,
        seeds: &[usize],
        iterations: usize,
        gate: Option<&[f32]>,
        gamma: f32,
    ) -> Vec<f32> {
        let n = self.entities.len();
        if n == 0 || seeds.is_empty() {
            return vec![0.0; n];
        }
        let c = 0.85f32;
        let mut out_degree = vec![0usize; n];
        for t in &self.triples {
            out_degree[t.subject_id] += 1;
            out_degree[t.object_id] += 1;
        }
        let mut neighbors: Vec<Vec<usize>> = (0..n).map(|_| Vec::new()).collect();
        for t in &self.triples {
            neighbors[t.subject_id].push(t.object_id);
            neighbors[t.object_id].push(t.subject_id);
        }
        let step = |pi: &[f32]| -> Vec<f32> {
            let mut next = vec![0.0f32; n];
            for u in 0..n {
                let deg = out_degree[u].max(1);
                let share = pi[u] / deg as f32;
                for &v in &neighbors[u] {
                    next[v] += share;
                }
            }
            next
        };
        // Global PageRank (ungated baseline for hub correction).
        let mut pi_global = vec![1.0 / n as f32; n];
        for _ in 0..iterations {
            let walked = step(&pi_global);
            for i in 0..n {
                pi_global[i] = (1.0 - c) / n as f32 + c * walked[i];
            }
        }
        // Personalized PageRank with the query-aware gate on the walk.
        let mut pi_q = vec![0.0f32; n];
        let seed_mass = 1.0 / seeds.len() as f32;
        for &s in seeds {
            pi_q[s] = seed_mass;
        }
        for _ in 0..iterations {
            let mut next = vec![0.0f32; n];
            for u in 0..n {
                let deg = out_degree[u].max(1);
                let share = pi_q[u] / deg as f32;
                for &v in &neighbors[u] {
                    let w = match gate {
                        Some(g) => g[v].max(0.0).powf(gamma),
                        None => 1.0,
                    };
                    next[v] += share * w;
                }
            }
            for i in 0..n {
                let teleport = if seeds.contains(&i) { seed_mass } else { 0.0 };
                pi_q[i] = (1.0 - c) * teleport + c * next[i];
            }
        }
        let mut scores = vec![0.0f32; n];
        let min_p = 1e-6f32;
        for i in 0..n {
            let a = pi_q[i].max(min_p);
            let b = pi_global[i].max(min_p);
            scores[i] = (a / b).ln();
        }
        scores
    }

    /// Get the name of a relation by ID.
    pub fn relation_name(&self, id: usize) -> &str {
        &self.relations[id]
    }

    /// Heuristic confidence score for a triple (EGA-style gate proxy).
    ///
    /// Low-confidence triples have garbage subjects ("Together they") or
    /// verbose objects ("a Swiss professional tennis player who won the
    /// Australian Open in 1997").  Answer selection weights these scores so
    /// the beam's energy function implicitly favours reliable facts.
    pub fn triple_confidence(&self, triple_idx: usize) -> f32 {
        let t = &self.triples[triple_idx];
        let subj_words = self.entity_name(t.subject_id).split_whitespace().count();
        let obj_words = self.entity_name(t.object_id).split_whitespace().count();
        // Short entities are more reliable (long phrases are decomposition
        // noise).  Use a sigmoid-like ramp: 1.0 for ≤2 words, decaying to 0.
        let len_score = (0.8f32).powi(subj_words.saturating_sub(2) as i32)
            .min((0.8f32).powi(obj_words.saturating_sub(1) as i32));
        // Penalise relations that are bare copulas — they match too easily.
        let rel_name = self.relation_name(t.relation_id);
        let rel_penalty = if rel_name == "is" || rel_name == "are" || rel_name == "was" || rel_name == "were" {
            0.6
        } else {
            1.0
        };
        len_score * rel_penalty
    }

    /// BFS subgraph extraction: starting from a set of entities,
    /// explore up to `max_hops` hops and return all reachable triples.
    pub fn bfs_subgraph(&self, start_entities: &[usize], max_hops: usize) -> Vec<Triple> {
        let mut visited = vec![false; self.entities.len()];
        let mut result_triples = Vec::new();
        let mut seen_triples = vec![false; self.triples.len()];
        let mut queue: VecDeque<(usize, usize)> = VecDeque::new();

        for &entity_id in start_entities {
            if entity_id < self.entities.len() {
                visited[entity_id] = true;
                queue.push_back((entity_id, 0));
            }
        }

        while let Some((current, depth)) = queue.pop_front() {
            if depth >= max_hops {
                continue;
            }

            // Explore triples adjacent to the current entity via the adjacency
            // index (O(degree)) instead of scanning all triples.
            for &idx in self.adjacency_of(current) {
                if seen_triples[idx] {
                    continue;
                }
                let triple = &self.triples[idx];
                seen_triples[idx] = true;
                result_triples.push(*triple);
                if !visited[triple.object_id] {
                    visited[triple.object_id] = true;
                    queue.push_back((triple.object_id, depth + 1));
                }
                if !visited[triple.subject_id] {
                    visited[triple.subject_id] = true;
                    queue.push_back((triple.subject_id, depth + 1));
                }
            }
        }

        result_triples
    }

    /// Find deterministic subject/relation conflicts in insertion order.
    pub fn contradictions(&self) -> Vec<Contradiction> {
        let mut seen: HashMap<(usize, usize), usize> = HashMap::new();
        let mut conflicts = Vec::new();
        for triple in &self.triples {
            let key = (triple.subject_id, triple.relation_id);
            if let Some(&first_object) = seen.get(&key) {
                if first_object != triple.object_id {
                    let conflict = Contradiction {
                        subject: self.entity_name(triple.subject_id).to_string(),
                        relation: self.relation_name(triple.relation_id).to_string(),
                        first_object: self.entity_name(first_object).to_string(),
                        second_object: self.entity_name(triple.object_id).to_string(),
                    };
                    if !conflicts.contains(&conflict) {
                        conflicts.push(conflict);
                    }
                }
            } else {
                seen.insert(key, triple.object_id);
            }
        }
        conflicts
    }

    /// Remove earlier values for a subject/relation before inserting a replacement.
    pub fn remove_conflicting_triples(&mut self, subject: &str, relation: &str) -> usize {
        let Some(&subject_id) = self.entity_index.get(subject) else { return 0; };
        let Some(&relation_id) = self.relation_index.get(relation) else { return 0; };
        let before = self.triples.len();
        self.triples.retain(|triple| {
            triple.subject_id != subject_id || triple.relation_id != relation_id
        });
        before - self.triples.len()
    }
}

impl Default for KnowledgeGraph {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_entities_and_relations() {
        let mut kg = KnowledgeGraph::new();
        let sky = kg.add_entity("sky");
        let blue = kg.add_entity("blue");
        assert_eq!(sky, 0);
        assert_eq!(blue, 1);
        assert_eq!(kg.entity_id("sky"), Some(0));
        assert_eq!(kg.entity_name(0), "sky");
    }

    #[test]
    fn test_add_triple() {
        let mut kg = KnowledgeGraph::new();
        kg.add_triple("sky", "is", "blue");
        assert_eq!(kg.triples.len(), 1);
        assert_eq!(kg.entities.len(), 2);
        assert_eq!(kg.relations.len(), 1);
    }

    #[test]
    fn test_get_triples_from() {
        let mut kg = KnowledgeGraph::new();
        kg.add_triple("sky", "is", "blue");
        kg.add_triple("sky", "has", "clouds");
        kg.add_triple("ocean", "is", "blue");

        let sky_id = kg.entity_id("sky").unwrap();
        let from_sky = kg.get_triples_from(sky_id);
        assert_eq!(from_sky.len(), 2);
    }

    #[test]
    fn test_get_triples_to() {
        let mut kg = KnowledgeGraph::new();
        kg.add_triple("sky", "is", "blue");
        kg.add_triple("ocean", "is", "blue");

        let blue_id = kg.entity_id("blue").unwrap();
        let to_blue = kg.get_triples_to(blue_id);
        assert_eq!(to_blue.len(), 2);
    }

    #[test]
    fn test_duplicate_entity() {
        let mut kg = KnowledgeGraph::new();
        let id1 = kg.add_entity("sky");
        let id2 = kg.add_entity("sky");
        assert_eq!(id1, id2);
        assert_eq!(kg.entities.len(), 1);
    }

    #[test]
    fn test_contradictions_ignore_duplicates() {
        let mut kg = KnowledgeGraph::new();
        kg.add_triple("sky", "color", "blue");
        kg.add_triple("sky", "color", "blue");
        kg.add_triple("sky", "color", "green");
        let conflicts = kg.contradictions();
        assert_eq!(conflicts.len(), 1);
        assert_eq!(conflicts[0].first_object, "blue");
        assert_eq!(conflicts[0].second_object, "green");
    }

    #[test]
    fn test_export_triples() {
        let mut kg = KnowledgeGraph::new();
        kg.add_triple("sky", "is", "blue");
        kg.add_triple("blue", "has", "short_wavelength");
        let triples = kg.export_triples();
        assert_eq!(triples.len(), 2);
        assert_eq!(triples[0], ["sky", "is", "blue"]);
        assert_eq!(triples[1], ["blue", "has", "short_wavelength"]);
    }

    #[test]
    fn test_bfs_subgraph() {
        let mut kg = KnowledgeGraph::new();
        kg.add_triple("sky", "is", "blue");
        kg.add_triple("blue", "relates_to", "ocean");
        kg.add_triple("ocean", "contains", "fish");
        kg.add_triple("unrelated", "is", "separate");

        let sky_id = kg.entity_id("sky").unwrap();
        let subgraph = kg.bfs_subgraph(&[sky_id], 2);
        // Should reach sky->blue->ocean but not "unrelated"
        assert!(subgraph.len() >= 2);
        // The unrelated triple should not be in the subgraph
        let unrelated_id = kg.entity_id("unrelated").unwrap();
        assert!(!subgraph.iter().any(|t| t.subject_id == unrelated_id));
    }

    #[test]
    fn test_bfs_subgraph_max_hops() {
        let mut kg = KnowledgeGraph::new();
        kg.add_triple("a", "to", "b");
        kg.add_triple("b", "to", "c");
        kg.add_triple("c", "to", "d");

        let a_id = kg.entity_id("a").unwrap();

        // With 1 hop, should only get a->b
        let sub1 = kg.bfs_subgraph(&[a_id], 1);
        assert_eq!(sub1.len(), 1);

        // With 3 hops, should get all
        let sub3 = kg.bfs_subgraph(&[a_id], 3);
        assert_eq!(sub3.len(), 3);
    }

    #[test]
    fn test_personalized_pagerank_hub_corrected() {
        let mut kg = KnowledgeGraph::new();
        // Chain: seed -> hub -> leaf. The hub connects to many leaves, so its
        // global PageRank is high; the relative PPR (π_q/π) must not let the
        // hub dominate the seed's actual neighbor.
        kg.add_triple("seed", "links", "hub");
        kg.add_triple("hub", "links", "leaf1");
        kg.add_triple("hub", "links", "leaf2");
        kg.add_triple("hub", "links", "leaf3");
        kg.add_triple("hub", "links", "leaf4");
        kg.add_triple("hub", "links", "leaf5");

        let seed = kg.entity_id("seed").unwrap();
        let hub = kg.entity_id("hub").unwrap();
        let scores = kg.personalized_pagerank(&[seed], 60);

        assert_eq!(scores.len(), kg.entities.len());
        // The seed (teleport) must have the highest relative PPR.
        assert!(
            scores[seed] > scores[hub],
            "seed {:.3} should outrank hub {:.3} (hub-corrected)",
            scores[seed], scores[hub]
        );
        // Every score is finite (log of positive ratio).
        for &s in &scores {
            assert!(s.is_finite());
        }
    }
}
