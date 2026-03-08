//! SVG dependency graph formatter for `mille analyze`.
//!
//! Design system:
//!   Background: #0F172A  (slate-900)
//!   Node fill:  #1E293B  (slate-800)
//!   Node border: #22C55E (green-500)
//!   Text:       #F8FAFC  (slate-50)
//!   Edge:       #22C55E  (green-500)
//!   Font:       monospace

use std::collections::{HashMap, HashSet};

use crate::usecase::analyze::{AnalyzeResult, LayerNode};

// Layout constants
const NODE_W: i32 = 180;
const NODE_H: i32 = 52;
const H_GAP: i32 = 40; // horizontal gap between nodes in the same rank
const V_GAP: i32 = 80; // vertical gap between ranks
const MARGIN: i32 = 40;

const BG: &str = "#0F172A";
const NODE_FILL: &str = "#1E293B";
const BORDER: &str = "#22C55E";
const TEXT: &str = "#F8FAFC";
const MUTED: &str = "#94A3B8";
const EDGE_COLOR: &str = "#22C55E";

pub fn format_svg(result: &AnalyzeResult) -> String {
    if result.nodes.is_empty() {
        return empty_svg();
    }

    // Build adjacency for topological sort (from → set of to)
    let mut adj: HashMap<&str, HashSet<&str>> = HashMap::new();
    let mut in_degree: HashMap<&str, usize> = HashMap::new();
    for node in &result.nodes {
        adj.entry(node.name.as_str()).or_default();
        in_degree.entry(node.name.as_str()).or_insert(0);
    }
    for edge in &result.edges {
        adj.entry(edge.from.as_str())
            .or_default()
            .insert(edge.to.as_str());
        *in_degree.entry(edge.to.as_str()).or_insert(0) += 1;
    }

    // Assign ranks (topological levels) using Kahn's algorithm
    let mut rank: HashMap<&str, usize> = HashMap::new();
    let mut queue: std::collections::VecDeque<&str> = in_degree
        .iter()
        .filter(|(_, &d)| d == 0)
        .map(|(&n, _)| n)
        .collect();
    // Sort queue for determinism
    let mut q_sorted: Vec<&str> = queue.drain(..).collect();
    q_sorted.sort_unstable();
    queue.extend(q_sorted);

    while let Some(node) = queue.pop_front() {
        let cur_rank = *rank.get(node).unwrap_or(&0);
        if let Some(neighbors) = adj.get(node) {
            let mut ns: Vec<&str> = neighbors.iter().copied().collect();
            ns.sort_unstable();
            for next in ns {
                let next_rank = rank.entry(next).or_insert(0);
                if *next_rank <= cur_rank {
                    *next_rank = cur_rank + 1;
                }
                *in_degree.get_mut(next).unwrap() -= 1;
                if in_degree[next] == 0 {
                    queue.push_back(next);
                }
            }
        }
    }

    // Ensure all nodes have a rank (disconnected nodes → rank 0)
    for node in &result.nodes {
        rank.entry(node.name.as_str()).or_insert(0);
    }

    // Group nodes by rank
    let max_rank = rank.values().copied().max().unwrap_or(0);
    let mut ranks: Vec<Vec<&LayerNode>> = vec![vec![]; max_rank + 1];
    for node in &result.nodes {
        let r = rank[node.name.as_str()];
        ranks[r].push(node);
    }
    // Sort each rank group by name for determinism
    for group in &mut ranks {
        group.sort_by_key(|n| n.name.as_str());
    }

    // Compute positions
    let max_cols = ranks.iter().map(|g| g.len()).max().unwrap_or(1) as i32;
    let canvas_w = MARGIN * 2 + max_cols * NODE_W + (max_cols - 1) * H_GAP;
    let canvas_h = MARGIN * 2 + (max_rank as i32 + 1) * NODE_H + max_rank as i32 * V_GAP;

    let mut positions: HashMap<&str, (i32, i32)> = HashMap::new();
    for (r, group) in ranks.iter().enumerate() {
        let cols = group.len() as i32;
        let total_w = cols * NODE_W + (cols - 1) * H_GAP;
        let start_x = (canvas_w - total_w) / 2;
        let y = MARGIN + r as i32 * (NODE_H + V_GAP);
        for (c, node) in group.iter().enumerate() {
            let x = start_x + c as i32 * (NODE_W + H_GAP);
            positions.insert(node.name.as_str(), (x, y));
        }
    }

    let mut buf = String::new();

    // SVG header
    buf.push_str(&format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{}" height="{}" viewBox="0 0 {} {}">"#,
        canvas_w, canvas_h, canvas_w, canvas_h
    ));
    buf.push('\n');

    // Defs: arrowhead marker
    buf.push_str(&format!(
        r#"  <defs>
    <marker id="arrow" markerWidth="10" markerHeight="7" refX="9" refY="3.5" orient="auto">
      <polygon points="0 0, 10 3.5, 0 7" fill="{}" />
    </marker>
  </defs>
"#,
        EDGE_COLOR
    ));

    // Background
    buf.push_str(&format!(
        r#"  <rect width="100%" height="100%" fill="{}" />
"#,
        BG
    ));

    // Edges (drawn first so nodes appear on top)
    for edge in &result.edges {
        let Some(&(fx, fy)) = positions.get(edge.from.as_str()) else {
            continue;
        };
        let Some(&(tx, ty)) = positions.get(edge.to.as_str()) else {
            continue;
        };

        // Connect bottom-center of from-node to top-center of to-node
        let x1 = fx + NODE_W / 2;
        let y1 = fy + NODE_H;
        let x2 = tx + NODE_W / 2;
        let y2 = ty;

        // Bezier control points
        let cy1 = y1 + V_GAP / 3;
        let cy2 = y2 - V_GAP / 3;

        buf.push_str(&format!(
            r#"  <path d="M {x1} {y1} C {x1} {cy1}, {x2} {cy2}, {x2} {y2}" fill="none" stroke="{EDGE_COLOR}" stroke-width="1.5" marker-end="url(#arrow)" />
"#
        ));

        // Label: import count at midpoint
        let lx = (x1 + x2) / 2 + 6;
        let ly = (y1 + y2) / 2;
        buf.push_str(&format!(
            r#"  <text x="{lx}" y="{ly}" fill="{MUTED}" font-family="monospace" font-size="11" dominant-baseline="middle">{}</text>
"#,
            edge.import_count
        ));
    }

    // Nodes
    for node in &result.nodes {
        let Some(&(x, y)) = positions.get(node.name.as_str()) else {
            continue;
        };
        let rx = 6;

        buf.push_str(&format!(
            r#"  <rect x="{x}" y="{y}" width="{NODE_W}" height="{NODE_H}" rx="{rx}" fill="{NODE_FILL}" stroke="{BORDER}" stroke-width="1.5" />
"#
        ));

        // Layer name
        let cx = x + NODE_W / 2;
        let ty = y + NODE_H / 2 - 6;
        buf.push_str(&format!(
            r#"  <text x="{cx}" y="{ty}" fill="{TEXT}" font-family="monospace" font-size="13" font-weight="600" text-anchor="middle" dominant-baseline="middle">{}</text>
"#,
            node.name
        ));

        // File count
        let file_label = if node.file_count == 1 {
            "1 file".to_string()
        } else {
            format!("{} files", node.file_count)
        };
        let cy = y + NODE_H / 2 + 10;
        buf.push_str(&format!(
            r#"  <text x="{cx}" y="{cy}" fill="{MUTED}" font-family="monospace" font-size="10" text-anchor="middle" dominant-baseline="middle">{file_label}</text>
"#
        ));
    }

    buf.push_str("</svg>\n");
    buf
}

fn empty_svg() -> String {
    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="200" height="60">
  <rect width="100%" height="100%" fill="{BG}" />
  <text x="100" y="30" fill="{MUTED}" font-family="monospace" font-size="13" text-anchor="middle" dominant-baseline="middle">No layers defined</text>
</svg>
"#
    )
}
