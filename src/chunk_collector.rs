use tree_sitter::Node;

use crate::heuristics::Heuristics;

pub(crate) struct ChunkCollector;

impl ChunkCollector {
    pub fn collect(root: tree_sitter::Node, source: &str) -> Vec<(usize, usize)> {
        let mut spans = Self::collect_valid_chunks(root, source);
        Self::merge_adjacent(&mut spans);
        spans
    }

    /// Collect valid chunks using heuristics to pre-filter nodes
    fn collect_valid_chunks(node: Node, source: &str) -> Vec<(usize, usize)> {
        let flat_nodes = Heuristics::filter_children(node, source);

        if flat_nodes.is_empty() {
            return vec![(node.start_byte(), node.end_byte())];
        }

        let mut spans = Vec::new();

        flat_nodes
            .iter()
            .for_each(|child| spans.push((child.start_byte(), child.end_byte())));

        spans
    }

    fn merge_adjacent(spans: &mut Vec<(usize, usize)>) {
        if spans.is_empty() {
            return;
        }

        spans.sort_by_key(|&(start, _)| start);
        let mut merged = Vec::with_capacity(spans.len());
        let mut current = spans[0];

        for &(start, end) in &spans[1..] {
            if start <= current.1 + 1 {
                current.1 = current.1.max(end);
            } else {
                merged.push(current);
                current = (start, end);
            }
        }

        merged.push(current);
        *spans = merged;
    }
}
