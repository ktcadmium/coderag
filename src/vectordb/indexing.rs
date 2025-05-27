// HNSW (Hierarchical Navigable Small World) implementation for fast approximate nearest neighbor search

use crate::vectordb::types::{Vector, VectorId};
use anyhow::Result;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::time::Instant;
use tracing::debug;

/// Entry in priority queue for HNSW search
#[derive(Debug, Clone)]
struct HnswEntry {
    id: VectorId,
    distance: f32,
}

impl PartialEq for HnswEntry {
    fn eq(&self, other: &Self) -> bool {
        self.distance.eq(&other.distance)
    }
}

impl Eq for HnswEntry {}

impl PartialOrd for HnswEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for HnswEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse order for max heap - we want smallest distances first
        other
            .distance
            .partial_cmp(&self.distance)
            .unwrap_or(Ordering::Equal)
    }
}

/// HNSW index parameters
#[derive(Debug, Clone)]
pub struct HnswParams {
    /// M parameter - max neighbors per node
    pub max_connections: usize,
    /// Connections at level 0 (more connections at ground level)
    pub max_connections_level0: usize,
    /// ef during construction (search width during build)
    pub ef_construction: usize,
    /// Default ef during search (search width during query)
    pub ef_search: usize,
    /// Cosine similarity (true) or L2 distance (false)
    pub use_cosine: bool,
}

impl Default for HnswParams {
    fn default() -> Self {
        Self {
            max_connections: 16,        // M parameter
            max_connections_level0: 32, // M0 parameter
            ef_construction: 100,       // Default construction beam width
            ef_search: 50,              // Default search beam width
            use_cosine: true,           // Use cosine similarity by default
        }
    }
}

/// Connection structure for HNSW nodes
#[derive(Debug, Clone)]
struct HnswConnections {
    /// Connections at each layer - Vec<layer -> Vec<connected nodes>>
    connections: Vec<Vec<VectorId>>,
}

impl HnswConnections {
    /// Create new empty connections
    fn new(max_level: usize) -> Self {
        let mut connections = Vec::with_capacity(max_level + 1);
        for _ in 0..=max_level {
            connections.push(Vec::new());
        }
        Self { connections }
    }

    /// Get connections at a specific layer
    fn get_layer_connections(&self, layer: usize) -> Option<&Vec<VectorId>> {
        self.connections.get(layer)
    }

    /// Get mutable connections at a specific layer
    fn get_layer_connections_mut(&mut self, layer: usize) -> Option<&mut Vec<VectorId>> {
        self.connections.get_mut(layer)
    }

    /// Add a connection at a specific layer
    fn add_connection(&mut self, layer: usize, id: VectorId) -> Result<()> {
        if layer >= self.connections.len() {
            anyhow::bail!("Layer index out of bounds: {}", layer);
        }

        if !self.connections[layer].contains(&id) {
            self.connections[layer].push(id);
        }

        Ok(())
    }
}

/// Node in HNSW graph
struct HnswNode {
    /// Unique identifier
    id: VectorId,
    /// Vector embedding
    vector: Vector,
    /// Connections to other nodes at different layers
    connections: HnswConnections,
    /// Maximum layer this node appears in
    max_level: usize,
}

/// HNSW index implementation
pub struct HnswIndex {
    /// All nodes in the graph
    nodes: HashMap<VectorId, HnswNode>,
    /// Entry point (highest level node)
    entry_point: Option<VectorId>,
    /// HNSW parameters
    params: HnswParams,
    /// Maximum level in the graph
    max_level: usize,
    /// Vector dimension
    dimension: usize,
    /// Random level generator
    level_generator: fn() -> usize,
}

impl HnswIndex {
    /// Create a new empty HNSW index
    pub fn new(dimension: usize, params: HnswParams) -> Self {
        Self {
            nodes: HashMap::new(),
            entry_point: None,
            params,
            max_level: 0,
            dimension,
            level_generator: || {
                // Generate random level with exponential distribution
                // -ln(rand(0..1)) * scale_factor
                // Higher scale_factor = fewer levels
                let scale_factor = 1.0 / 0.5; // Adjustable scale factor
                let r = rand::random::<f32>();
                if r == 0.0 {
                    return 0;
                } // Avoid ln(0)

                let level = (-r.ln() * scale_factor).floor() as usize;
                std::cmp::min(level, 10) // Cap at 10 levels for sanity
            },
        }
    }

    /// Calculate distance between vectors based on the index's distance metric
    fn distance(&self, a: &[f32], b: &[f32]) -> f32 {
        if self.params.use_cosine {
            // For cosine similarity, we want 1 - similarity for a distance
            // (0 = identical, 2 = opposite)
            1.0 - self.cosine_similarity(a, b)
        } else {
            // L2 squared distance
            a.iter().zip(b.iter()).map(|(x, y)| (x - y).powi(2)).sum()
        }
    }

    /// Calculate cosine similarity between two vectors
    fn cosine_similarity(&self, a: &[f32], b: &[f32]) -> f32 {
        if a.len() != b.len() {
            return 0.0;
        }

        let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

        if norm_a == 0.0 || norm_b == 0.0 {
            return 0.0;
        }

        dot_product / (norm_a * norm_b)
    }

    /// Add a vector to the index
    pub fn add(&mut self, id: VectorId, vector: Vector) -> Result<()> {
        // Validate vector dimension
        if vector.dimension() != self.dimension {
            anyhow::bail!(
                "Vector dimension mismatch: expected {}, got {}",
                self.dimension,
                vector.dimension()
            );
        }

        // Check if ID already exists
        if self.nodes.contains_key(&id) {
            anyhow::bail!("Node with ID {} already exists", id);
        }

        // Generate random level for the node
        let level = (self.level_generator)();

        // Update max level if needed
        let is_first_node = self.nodes.is_empty();
        if level > self.max_level && !is_first_node {
            self.max_level = level;
        }

        // Create node with connections up to its level
        let node = HnswNode {
            id: id.clone(),
            vector: vector.clone(),
            connections: HnswConnections::new(level),
            max_level: level,
        };

        // Insert into graph
        self.nodes.insert(id.clone(), node);

        // If this is the first node, set it as entry point and return
        if is_first_node {
            self.entry_point = Some(id);
            self.max_level = level;
            return Ok(());
        }

        // Connect the new node to the graph
        self.connect_node(&id, level)?;

        Ok(())
    }

    /// Connect a new node to the graph
    fn connect_node(&mut self, id: &VectorId, level: usize) -> Result<()> {
        let entry_id = match &self.entry_point {
            Some(ep) => ep.clone(),
            None => anyhow::bail!("No entry point found"),
        };

        // Get vector of the new node
        let vector = match self.nodes.get(id) {
            Some(node) => node.vector.values.clone(),
            None => anyhow::bail!("Node with ID {} not found", id),
        };

        // Start from entry point at the highest level
        let mut cur_node_id = entry_id;
        let mut cur_dist = self.distance(
            &self.nodes.get(&cur_node_id).unwrap().vector.values,
            &vector,
        );

        // Search from top level down to the node's level
        for l in (level + 1..=self.max_level).rev() {
            let changed = self.search_layer(&vector, &mut cur_node_id, &mut cur_dist, l)?;
            if !changed {
                debug!("No change at level {}", l);
            }
        }

        // For each level from the node's level down to 0
        for l in (0..=level).rev() {
            // Find nearest neighbors at this level
            let ef = if l == 0 {
                self.params.ef_construction * 2
            } else {
                self.params.ef_construction
            };

            let nearest = self.search_neighbors(&vector, cur_node_id.clone(), ef, l)?;

            // Calculate max connections for this level
            let max_conn = if l == 0 {
                self.params.max_connections_level0
            } else {
                self.params.max_connections
            };

            // Connect to nearest neighbors (bidirectional)
            self.connect_neighbors(id, &nearest, l, max_conn)?;

            // Update entry point for next level
            if !nearest.is_empty() {
                cur_node_id = nearest[0].id.clone();
                // cur_dist = nearest[0].distance; // Not used after this point
            }
        }

        // Update entry point if new node is at a higher level
        if level > self.max_level {
            self.entry_point = Some(id.clone());
            self.max_level = level;
        }

        Ok(())
    }

    /// Search for neighbors at a specific layer, moving to a better node if found
    fn search_layer(
        &self,
        query: &[f32],
        best_id: &mut VectorId,
        best_dist: &mut f32,
        level: usize,
    ) -> Result<bool> {
        let mut changed = false;
        let mut visited = HashSet::new();

        // Get the current best node
        let cur_node = match self.nodes.get(best_id) {
            Some(node) => node,
            None => anyhow::bail!("Node with ID {} not found", best_id),
        };

        // Check all connections at this level
        let connections = match cur_node.connections.get_layer_connections(level) {
            Some(conn) => conn,
            None => anyhow::bail!("No connections at level {}", level),
        };

        for conn_id in connections {
            // Skip if already visited
            if visited.contains(conn_id) {
                continue;
            }
            visited.insert(conn_id.clone());

            // Get connected node
            let conn_node = match self.nodes.get(conn_id) {
                Some(node) => node,
                None => continue, // Skip if node doesn't exist
            };

            // Calculate distance
            let dist = self.distance(&conn_node.vector.values, query);

            // Update if better
            if dist < *best_dist {
                *best_id = conn_id.clone();
                *best_dist = dist;
                changed = true;
            }
        }

        Ok(changed)
    }

    /// Search for ef nearest neighbors at a specific layer
    fn search_neighbors(
        &self,
        query: &[f32],
        entry_id: VectorId,
        ef: usize,
        level: usize,
    ) -> Result<Vec<HnswEntry>> {
        // Priority queues for candidates and results
        let mut candidates = BinaryHeap::new();
        let mut results = BinaryHeap::new();
        let mut visited = HashSet::new();

        // Get distance to entry point
        let entry_node = match self.nodes.get(&entry_id) {
            Some(node) => node,
            None => anyhow::bail!("Entry node with ID {} not found", entry_id),
        };

        let entry_dist = self.distance(&entry_node.vector.values, query);

        // Initialize with entry point
        candidates.push(HnswEntry {
            id: entry_id.clone(),
            distance: entry_dist,
        });

        results.push(HnswEntry {
            id: entry_id.clone(),
            distance: entry_dist,
        });

        visited.insert(entry_id);

        // Process candidates
        while !candidates.is_empty() {
            // Get closest candidate
            let current = match candidates.pop() {
                Some(c) => c,
                None => break,
            };

            // If we have results and the current candidate is worse than the worst result, we're done
            if !results.is_empty() && current.distance > results.peek().unwrap().distance {
                break;
            }

            // Get current node
            let current_node = match self.nodes.get(&current.id) {
                Some(node) => node,
                None => continue,
            };

            // Check all connections at this level
            let connections = match current_node.connections.get_layer_connections(level) {
                Some(conn) => conn,
                None => anyhow::bail!("No connections at level {}", level),
            };

            for conn_id in connections {
                // Skip if already visited
                if visited.contains(conn_id) {
                    continue;
                }
                visited.insert(conn_id.clone());

                // Get connected node
                let conn_node = match self.nodes.get(conn_id) {
                    Some(node) => node,
                    None => continue,
                };

                // Calculate distance
                let dist = self.distance(&conn_node.vector.values, query);

                // Check if we should add to results
                let should_add = if results.len() < ef {
                    // Always add if we have fewer than ef results
                    true
                } else {
                    // Add if better than the worst result
                    let worst_dist = results.peek().unwrap().distance;
                    dist < worst_dist
                };

                if should_add {
                    // Add to candidates for further exploration
                    candidates.push(HnswEntry {
                        id: conn_id.clone(),
                        distance: dist,
                    });

                    // Add to results
                    results.push(HnswEntry {
                        id: conn_id.clone(),
                        distance: dist,
                    });

                    // Keep only the best ef results
                    if results.len() > ef {
                        results.pop();
                    }
                }
            }
        }

        // Convert results heap to sorted vector
        let mut sorted_results: Vec<_> = results.into_iter().collect();
        sorted_results.sort_by(|a, b| {
            a.distance
                .partial_cmp(&b.distance)
                .unwrap_or(Ordering::Equal)
        });

        Ok(sorted_results)
    }

    /// Connect a node to its neighbors bidirectionally, using heuristic to limit connections
    fn connect_neighbors(
        &mut self,
        id: &VectorId,
        neighbors: &[HnswEntry],
        level: usize,
        max_connections: usize,
    ) -> Result<()> {
        // Connect the node to its neighbors
        let node = match self.nodes.get_mut(id) {
            Some(node) => node,
            None => anyhow::bail!("Node with ID {} not found", id),
        };

        // Connect up to max_connections
        let connections = match node.connections.get_layer_connections_mut(level) {
            Some(conn) => conn,
            None => anyhow::bail!("No connections at level {}", level),
        };

        connections.clear();
        for neighbor in neighbors.iter().take(max_connections) {
            connections.push(neighbor.id.clone());
        }

        // Connect neighbors to the node
        for neighbor in neighbors.iter().take(max_connections) {
            // Check if neighbor exists and if already connected
            let (should_update, neighbor_vector, existing_connections) = {
                match self.nodes.get(&neighbor.id) {
                    Some(node) => match node.connections.get_layer_connections(level) {
                        Some(conn) => {
                            let already_connected = conn.contains(id);
                            let connections = conn.clone();
                            (!already_connected, node.vector.values.clone(), connections)
                        }
                        None => continue,
                    },
                    None => continue,
                }
            };

            if !should_update {
                continue;
            }

            // Check if we need to use the heuristic
            let needs_heuristic = {
                let neighbor_node = self.nodes.get(&neighbor.id).unwrap();
                let neighbor_connections = neighbor_node
                    .connections
                    .get_layer_connections(level)
                    .unwrap();
                neighbor_connections.len() >= max_connections
            };

            if needs_heuristic {
                // Calculate distances to all current connections before mutating
                let mut all_connections = Vec::with_capacity(existing_connections.len() + 1);

                // Add the new node
                all_connections.push(HnswEntry {
                    id: id.clone(),
                    distance: neighbor.distance,
                });

                // Add existing connections and calculate distances
                let distances: Vec<(String, f32)> = existing_connections
                    .iter()
                    .filter_map(|conn_id| {
                        self.nodes.get(conn_id).map(|conn_node| {
                            let dist = self.distance(&conn_node.vector.values, &neighbor_vector);
                            (conn_id.clone(), dist)
                        })
                    })
                    .collect();

                for (conn_id, dist) in distances {
                    all_connections.push(HnswEntry {
                        id: conn_id,
                        distance: dist,
                    });
                }

                // Sort by distance
                all_connections.sort_by(|a, b| {
                    a.distance
                        .partial_cmp(&b.distance)
                        .unwrap_or(Ordering::Equal)
                });

                // Now update the connections
                let neighbor_node = self.nodes.get_mut(&neighbor.id).unwrap();
                let neighbor_connections = neighbor_node
                    .connections
                    .get_layer_connections_mut(level)
                    .unwrap();
                neighbor_connections.clear();
                for conn in all_connections.iter().take(max_connections) {
                    neighbor_connections.push(conn.id.clone());
                }
            } else {
                // Just add the connection
                let neighbor_node = self.nodes.get_mut(&neighbor.id).unwrap();
                let neighbor_connections = neighbor_node
                    .connections
                    .get_layer_connections_mut(level)
                    .unwrap();
                neighbor_connections.push(id.clone());
            }
        }

        Ok(())
    }

    /// Search for k nearest neighbors
    pub fn search(&self, query: &[f32], k: usize) -> Result<Vec<(VectorId, f32)>> {
        // Time the search
        let start = Instant::now();

        // Validate query dimension
        if query.len() != self.dimension {
            anyhow::bail!(
                "Query dimension mismatch: expected {}, got {}",
                self.dimension,
                query.len()
            );
        }

        // Return empty result if index is empty
        if self.nodes.is_empty() {
            return Ok(Vec::new());
        }

        // Get entry point
        let entry_id = match &self.entry_point {
            Some(ep) => ep.clone(),
            None => anyhow::bail!("No entry point found"),
        };

        // Start from entry point
        let mut cur_node_id = entry_id;
        let mut cur_dist =
            self.distance(&self.nodes.get(&cur_node_id).unwrap().vector.values, query);

        // Search from top level down
        for l in (1..=self.max_level).rev() {
            let changed = self.search_layer(query, &mut cur_node_id, &mut cur_dist, l)?;
            if !changed {
                debug!("No change at level {}", l);
            }
        }

        // Find ef_search nearest neighbors at level 0
        let ef_search = self.params.ef_search.max(k);
        let nearest = self.search_neighbors(query, cur_node_id, ef_search, 0)?;

        // Convert to result format
        let mut results = Vec::with_capacity(k.min(nearest.len()));
        for neighbor in nearest.iter().take(k) {
            // Convert distance to similarity score (0.0 to 1.0) if using cosine
            let score = if self.params.use_cosine {
                1.0 - neighbor.distance // Convert back to similarity
            } else {
                1.0 / (1.0 + neighbor.distance) // Convert L2 to similarity-like score
            };

            results.push((neighbor.id.clone(), score));
        }

        // Log search stats
        let duration = start.elapsed();
        debug!(
            "HNSW search: found {} results in {:?} (ef_search={})",
            results.len(),
            duration,
            ef_search
        );

        Ok(results)
    }

    /// Get number of nodes in the index
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Check if index is empty
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Get node IDs in the index
    pub fn node_ids(&self) -> Vec<VectorId> {
        self.nodes.keys().cloned().collect()
    }

    /// Get index stats for debugging
    pub fn stats(&self) -> HnswStats {
        let mut connections_per_level = vec![0; self.max_level + 1];
        let mut max_connections_per_level = vec![0; self.max_level + 1];

        for node in self.nodes.values() {
            for l in 0..=node.max_level {
                if let Some(conns) = node.connections.get_layer_connections(l) {
                    connections_per_level[l] += conns.len();
                    max_connections_per_level[l] = max_connections_per_level[l].max(conns.len());
                }
            }
        }

        let avg_connections_per_level = connections_per_level
            .iter()
            .enumerate()
            .map(|(i, &count)| {
                let nodes_at_level = self.nodes.values().filter(|n| n.max_level >= i).count();
                if nodes_at_level > 0 {
                    count as f32 / nodes_at_level as f32
                } else {
                    0.0
                }
            })
            .collect();

        HnswStats {
            node_count: self.nodes.len(),
            max_level: self.max_level,
            dimension: self.dimension,
            entry_point: self.entry_point.clone(),
            connections_per_level,
            max_connections_per_level,
            avg_connections_per_level,
        }
    }
}

/// Statistics about the HNSW index
#[derive(Debug, Clone)]
pub struct HnswStats {
    /// Number of nodes in the index
    pub node_count: usize,
    /// Maximum level in the graph
    pub max_level: usize,
    /// Vector dimension
    pub dimension: usize,
    /// Entry point node ID
    pub entry_point: Option<VectorId>,
    /// Total connections at each level
    pub connections_per_level: Vec<usize>,
    /// Maximum connections per node at each level
    pub max_connections_per_level: Vec<usize>,
    /// Average connections per node at each level
    pub avg_connections_per_level: Vec<f32>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vectordb::types::Vector;

    #[test]
    fn test_hnsw_basic() -> Result<()> {
        // Create index
        let mut index = HnswIndex::new(3, HnswParams::default());

        // Add some vectors
        index.add("1".to_string(), Vector::new(vec![1.0, 0.0, 0.0]))?;
        index.add("2".to_string(), Vector::new(vec![0.0, 1.0, 0.0]))?;
        index.add("3".to_string(), Vector::new(vec![0.0, 0.0, 1.0]))?;

        // Search
        let results = index.search(&[1.0, 0.1, 0.1], 2)?;

        // Should find closest vectors
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].0, "1");

        Ok(())
    }

    #[test]
    fn test_hnsw_large() -> Result<()> {
        // Create index
        let mut index = HnswIndex::new(3, HnswParams::default());

        // Add many vectors
        for i in 0..100 {
            let x = (i % 10) as f32 / 10.0;
            let y = (i / 10) as f32 / 10.0;
            let z = 0.0;

            index.add(i.to_string(), Vector::new(vec![x, y, z]))?;
        }

        // Search
        let results = index.search(&[0.25, 0.25, 0.0], 5)?;

        // Should find closest vectors
        assert_eq!(results.len(), 5);

        // Check stats
        let stats = index.stats();
        assert_eq!(stats.node_count, 100);

        Ok(())
    }
}
