pub fn calculate_dst_size(
    src_height: usize,
    src_width: usize,
    height: Option<usize>,
    width: Option<usize>,
    percent: Option<f32>,
) -> (usize, usize) {
    if let (Some(h), Some(w)) = (height, width) {
        return (h, w);
    }

    if let Some(p) = percent {
        let nh = (src_height as f32 * p).round().max(1.0) as usize;
        let nw = (src_width as f32 * p).round().max(1.0) as usize;
        return (nh, nw);
    }

    if let Some(h) = height {
        let w = ((src_width as f32) * (h as f32) / (src_height as f32)).round().max(1.0) as usize;
        return (h, w);
    }

    if let Some(w) = width {
        let h = ((src_height as f32) * (w as f32) / (src_width as f32)).round().max(1.0) as usize;
        return (h, w);
    }

    (src_height, src_width)
}
