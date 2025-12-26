pub const MAX_DEPTH: usize = 8;
const NULL_IDX: u32 = 0;

type NodeIdx = u32;

#[derive(Clone, Debug)]
struct OctreeNode {
    red_sum: u64,
    green_sum: u64,
    blue_sum: u64,
    pixel_count: u64,
    palette_index: u16,
    children: [NodeIdx; 8],
}

impl Default for OctreeNode {
    fn default() -> Self {
        Self {
            red_sum: 0,
            green_sum: 0,
            blue_sum: 0,
            pixel_count: 0,
            palette_index: 0,
            children: [NULL_IDX; 8],
        }
    }
}

impl OctreeNode {
    #[inline]
    fn is_leaf(&self) -> bool {
        self.pixel_count > 0
    }

    #[inline]
    fn get_color(&self) -> [f32; 3] {
        if self.pixel_count == 0 {
            return [0.0, 0.0, 0.0];
        }
        let count = self.pixel_count as f32 * 255.0;
        [
            self.red_sum as f32 / count,
            self.green_sum as f32 / count,
            self.blue_sum as f32 / count,
        ]
    }
}

#[inline]
fn color_index(color: &[u8], level: usize) -> usize {
    let mask = 0x80u8 >> level;
    let mut idx = 0;
    if color[0] & mask != 0 {
        idx |= 4;
    }
    if color[1] & mask != 0 {
        idx |= 2;
    }
    if color[2] & mask != 0 {
        idx |= 1;
    }
    idx
}

pub struct OctreeQuantizer {
    nodes: Vec<OctreeNode>,
    levels: [Vec<NodeIdx>; MAX_DEPTH],
}

impl Default for OctreeQuantizer {
    fn default() -> Self {
        Self::new()
    }
}

impl OctreeQuantizer {
    pub fn new() -> Self {
        let mut nodes = Vec::new();
        nodes.push(OctreeNode::default());
        nodes.push(OctreeNode::default());

        Self {
            nodes,
            levels: Default::default(),
        }
    }

    #[inline]
    fn root_idx(&self) -> NodeIdx {
        1
    }

    #[inline]
    fn alloc_node(&mut self) -> NodeIdx {
        let idx = self.nodes.len() as NodeIdx;
        self.nodes.push(OctreeNode::default());
        idx
    }

    pub fn add_color(&mut self, rgb: &[u8]) {
        let mut node_idx = self.root_idx();

        for level in 0..MAX_DEPTH {
            let child_pos = color_index(rgb, level);
            let child_idx = self.nodes[node_idx as usize].children[child_pos];

            if child_idx == NULL_IDX {
                let new_idx = self.alloc_node();
                self.nodes[node_idx as usize].children[child_pos] = new_idx;

                if level < MAX_DEPTH - 1 {
                    self.levels[level].push(new_idx);
                }
                node_idx = new_idx;
            } else {
                node_idx = child_idx;
            }
        }
        let leaf = &mut self.nodes[node_idx as usize];
        leaf.red_sum += rgb[0] as u64;
        leaf.green_sum += rgb[1] as u64;
        leaf.blue_sum += rgb[2] as u64;
        leaf.pixel_count += 1;
    }
    pub fn count_leaves(&self) -> usize {
        let mut count = 0;
        let mut stack = vec![self.root_idx()];

        while let Some(idx) = stack.pop() {
            if idx == NULL_IDX {
                continue;
            }
            let node = &self.nodes[idx as usize];

            if node.is_leaf() {
                count += 1;
            } else {
                for &child in &node.children {
                    if child != NULL_IDX {
                        stack.push(child);
                    }
                }
            }
        }
        count
    }

    fn get_children_pixel_count(&self, idx: NodeIdx) -> u64 {
        let node = &self.nodes[idx as usize];
        let mut sum = 0;
        for &child_idx in &node.children {
            if child_idx != NULL_IDX {
                let child = &self.nodes[child_idx as usize];
                sum += child.pixel_count;
            }
        }
        sum
    }

    fn remove_leaves(&mut self, idx: NodeIdx) -> usize {
        let mut removed_leaves = 0;

        let mut r_sum = 0;
        let mut g_sum = 0;
        let mut b_sum = 0;
        let mut pix_cnt = 0;

        {
            let node = &self.nodes[idx as usize];
            for &child_idx in &node.children {
                if child_idx != NULL_IDX {
                    let child = &self.nodes[child_idx as usize];
                    r_sum += child.red_sum;
                    g_sum += child.green_sum;
                    b_sum += child.blue_sum;
                    pix_cnt += child.pixel_count;

                    if child.pixel_count > 0 {
                        removed_leaves += 1;
                    }
                }
            }
        }

        let parent = &mut self.nodes[idx as usize];
        parent.children = [NULL_IDX; 8];
        parent.red_sum += r_sum;
        parent.green_sum += g_sum;
        parent.blue_sum += b_sum;
        parent.pixel_count += pix_cnt;
        if removed_leaves > 0 { removed_leaves - 1 } else { 0 }
    }

    pub fn make_palette(&mut self, max_colors: usize) -> Vec<f32> {
        let mut leaf_count = self.count_leaves();
        for level in (0..MAX_DEPTH - 1).rev() {
            if leaf_count <= max_colors {
                break;
            }
            let level_indices = std::mem::take(&mut self.levels[level]);
            let mut candidates = Vec::with_capacity(level_indices.len());
            for &idx in &level_indices {
                let weight = self.get_children_pixel_count(idx);
                if weight > 0 {
                    candidates.push((idx, weight));
                }
            }
            candidates.sort_by(|a, b| a.1.cmp(&b.1));
            for (idx, _) in candidates {
                if leaf_count <= max_colors {
                    break;
                }
                let reduced = self.remove_leaves(idx);
                if reduced > 0 {
                    leaf_count -= reduced;
                }
            }
        }
        let mut palette = Vec::with_capacity(max_colors * 3);
        let mut leaves = Vec::new();
        self.collect_leaves(self.root_idx(), &mut leaves);
        leaves.sort_by(|a, b| {
            let node_a = &self.nodes[*a as usize];
            let node_b = &self.nodes[*b as usize];
            node_b.pixel_count.cmp(&node_a.pixel_count)
        });

        for (i, &leaf_idx) in leaves.iter().take(max_colors).enumerate() {
            let node = &mut self.nodes[leaf_idx as usize];
            palette.extend(node.get_color());
            node.palette_index = i as u16;
        }

        palette
    }

    fn collect_leaves(&self, idx: NodeIdx, out: &mut Vec<NodeIdx>) {
        if idx == NULL_IDX {
            return;
        }
        let node = &self.nodes[idx as usize];
        if node.is_leaf() {
            out.push(idx);
        } else {
            for &c in &node.children {
                self.collect_leaves(c, out);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enums::ImgColor;
    use crate::read::read_in_path;

    #[test]
    fn test_basic_colors() {
        let img = read_in_path(
            "/run/media/umzi/H/nahuy_pixiv/WOSManga_train_test/hq/000012.png",
            ImgColor::RGB,
        )
        .unwrap();
        let data = img.get_data::<u8>().unwrap();
        let mut quantizer = OctreeQuantizer::new();

        // Add some colors

        for c in data.chunks(3) {
            // println!("{}");
            quantizer.add_color(c);
        }

        // Generate palette with max 4 colors
        let palette = quantizer.make_palette(16);
        println!("Palette ({} colors):", palette.len());
        println!("{:?}", palette);
        // for (i, c) in palette.iter().enumerate() {
        //     println!("  {}: RGB({}, {}, {})", i, c.0, c.1, c.2);
        // }
    }
}
