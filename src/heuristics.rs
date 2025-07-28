use std::vec;

use tree_sitter::Node;

pub(crate) struct Heuristics;

impl Heuristics {
    pub fn filter_children<'a>(node: Node<'a>, source: &str) -> Vec<Node<'a>> {
        if !Self::current_node_valid(node) {
            return vec![];
        }

        let mut cursor = node.walk();
        let children: Vec<_> = node.children(&mut cursor).collect();

        if children.is_empty() {
            return vec![node];
        }

        Self::filter_identifier_blocks(children.first().unwrap(), source)
    }

    fn filter_identifier_blocks<'a>(start: &Node<'a>, source: &str) -> Vec<Node<'a>> {
        let mut filtered = Vec::new();
        let mut current = Some(*start);

        let mut block_start: Option<Node<'a>> = None;
        let mut block_len = 0;

        while let Some(node) = current {
            if node.child_count() == 0 {
                if node.kind() == "identifier"
                    || node.kind() == "type_identifier"
                    || node.kind() == "number_literal"
                    || node.kind() == "ERROR"
                {
                    if block_start.is_none() {
                        block_start = Some(node);
                    }
                    block_len += 1;
                } else {
                    // Flush any current block
                    if let Some(id_node) = block_start.take() {
                        if block_len == 1 {
                            filtered.push(id_node);
                        }
                    }
                    block_len = 0;

                    filtered.push(node);
                }
            } else {
                // Flush block when encountering complex node
                if let Some(id_node) = block_start.take() {
                    if block_len == 1 {
                        filtered.push(id_node);
                    }
                }
                block_len = 0;

                filtered.extend(Self::filter_children(node, source));
            }

            current = node.next_sibling();
        }

        // Flush last block at end
        if let Some(id_node) = block_start {
            if block_len == 1 {
                filtered.push(id_node);
            }
        }

        filtered
    }

    fn current_node_valid(node: tree_sitter::Node) -> bool {
        if node.kind() != "declaration" {
            return true;
        }

        let mut cursor = node.walk();
        node.children(&mut cursor)
            .any(|child| child.kind() == ";" && !child.is_missing())
    }
}
