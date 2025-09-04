// rectiq-cli/src/sketches/circular/detect_cycles.rs

// Detects all circular reference cycles in a list of RefEdges.
// Returns a vector of cycles, where each cycle is a Vec<RefEdge> that form a loop.
use rectiq_types::{JsonPath, JsonPathSegment, RefEdge};
use std::collections::{HashMap, HashSet};

/// Build a stable string key for a `JsonPath`.
///
/// We namespace each segment with a tag to avoid ambiguity and join with a
/// non-printable delimiter unlikely to occur in real data.
fn path_key(path: &JsonPath<'_>) -> String {
    const SEP: &str = "\u{1D}"; // non-printable separator
    let mut out = String::new();
    let mut first = true;
    for seg in &path.0 {
        if first {
            first = false;
        } else {
            out.push_str(SEP);
        }
        match seg {
            JsonPathSegment::Str(s) => {
                out.push_str("s:");
                out.push_str(s);
            }
            JsonPathSegment::Index(i) => {
                out.push_str("i:");
                out.push_str(&i.to_string());
            }
        }
    }
    out
}

/// Detect reference cycles using DFS on the implicit graph induced by edges.
///
/// Input: slice of `RefEdge` (from, to, pos, ...)
/// Output: list of cycles, each as owned `RefEdges` forming the cycle order.
#[must_use]
pub fn detect_ref_cycles<'a>(edges: &[RefEdge<'a>]) -> Vec<Vec<RefEdge<'a>>> {
    // Graph from node key -> Vec<(edge_index, to_key)>
    let mut graph: HashMap<String, Vec<(usize, String)>> = HashMap::new();
    for (idx, edge) in edges.iter().enumerate() {
        let from_k = path_key(&edge.from);
        let to_k = path_key(&edge.to);
        graph.entry(from_k).or_default().push((idx, to_k));
    }

    let mut all_cycles: Vec<Vec<RefEdge<'a>>> = Vec::new();
    let mut visited_nodes: HashSet<String> = HashSet::new();

    for start_key in graph.keys() {
        let mut stack: Vec<usize> = Vec::new(); // stack of edge indices
        let mut path_nodes: HashSet<String> = HashSet::new();
        dfs(
            start_key,
            &graph,
            &mut visited_nodes,
            &mut stack,
            &mut path_nodes,
            edges,
            &mut all_cycles,
        );
    }

    all_cycles
}

fn dfs<'a>(
    current: &str,
    graph: &HashMap<String, Vec<(usize, String)>>,
    visited_nodes: &mut HashSet<String>,
    stack: &mut Vec<usize>,           // edge indices composing current path
    path_nodes: &mut HashSet<String>, // nodes currently on recursion path
    edges: &[RefEdge<'a>],
    all_cycles: &mut Vec<Vec<RefEdge<'a>>>,
) {
    if path_nodes.contains(current) {
        // Find the earliest position in the stack where the cycle starts.
        // We locate the index in the stack whose `from` node equals `current`.
        if let Some(pos) = stack
            .iter()
            .position(|&ei| path_key(&edges[ei].from) == current)
        {
            let cycle: Vec<RefEdge<'a>> =
                stack[pos..].iter().map(|&ei| edges[ei].clone()).collect();
            all_cycles.push(cycle);
        }
        return;
    }

    if visited_nodes.contains(current) {
        return;
    }

    visited_nodes.insert(current.to_string());
    path_nodes.insert(current.to_string());

    if let Some(neighbors) = graph.get(current) {
        for (edge_idx, to_node) in neighbors {
            stack.push(*edge_idx);
            dfs(
                to_node,
                graph,
                visited_nodes,
                stack,
                path_nodes,
                edges,
                all_cycles,
            );
            stack.pop();
        }
    }

    path_nodes.remove(current);
}
