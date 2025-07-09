#[cfg(debug_assertions)]
/// Dump the tree for debugging
pub fn dump_tree(node: tree_sitter::Node, source: &str) {
    fn _dump_tree(node: tree_sitter::Node, source: &str, indent: usize) {
        let indent_str = "  ".repeat(indent);
        let kind = node.kind();
        let span = &source[node.start_byte()..node.end_byte()];
        println!("{indent_str}{kind}: {:?}", span.trim());

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            _dump_tree(child, source, indent + 1);
        }
    }
    _dump_tree(node, source, 0);
}

#[cfg(not(debug_assertions))]
#[inline(always)]
/// No-op for release builds
pub fn dump_tree(_: tree_sitter::Node, _: &str) {}
