/// Procedurally draws a small badge resembling the Codex CLI mark: a
/// purple-to-blue gradient circle with a white ">" prompt chevron and dash,
/// so the widget doesn't need to ship/redistribute the real logo as an
/// external image asset.
pub fn codex_badge_rgba(size: u32) -> Vec<u8> {
    let mut buf = vec![0u8; (size * size * 4) as usize];
    let center = size as f32 / 2.0;
    let radius = center - 1.0;
    let s = size as f32;

    // #B9AEFB -> #3B4CF5 (hex literals can't carry a float suffix — 'f' is a
    // valid hex digit, so it'd just extend the integer — hence decimal here).
    let top = [185.0_f32, 174.0_f32, 251.0_f32];
    let bottom = [59.0_f32, 76.0_f32, 245.0_f32];

    let stroke_w = (s * 0.12).max(1.4);
    let cx = center - s * 0.05;
    let chevron_top = (cx - s * 0.09, center - s * 0.17);
    let chevron_mid = (cx + s * 0.11, center);
    let chevron_bot = (cx - s * 0.09, center + s * 0.17);
    let dash_from = (center + s * 0.03, center + s * 0.17);
    let dash_to = (center + s * 0.22, center + s * 0.17);
    let segments = [
        (chevron_top, chevron_mid),
        (chevron_mid, chevron_bot),
        (dash_from, dash_to),
    ];

    for y in 0..size {
        for x in 0..size {
            let dx = x as f32 + 0.5 - center;
            let dy = y as f32 + 0.5 - center;
            let dist = (dx * dx + dy * dy).sqrt();
            let idx = ((y * size + x) * 4) as usize;
            if dist > radius {
                continue;
            }
            let edge = (radius - dist).min(1.5) / 1.5;
            let alpha = edge.clamp(0.0, 1.0);

            let t = (y as f32 / s).clamp(0.0, 1.0);
            let r = top[0] + (bottom[0] - top[0]) * t;
            let g = top[1] + (bottom[1] - top[1]) * t;
            let b = top[2] + (bottom[2] - top[2]) * t;

            let px = x as f32 + 0.5;
            let py = y as f32 + 0.5;
            let line_dist = segments
                .iter()
                .map(|(a, b)| dist_to_segment((px, py), *a, *b))
                .fold(f32::MAX, f32::min);
            let line_mix = ((stroke_w / 2.0 + 0.6) - line_dist).clamp(0.0, 1.0);

            let r = r + (255.0 - r) * line_mix;
            let g = g + (255.0 - g) * line_mix;
            let b = b + (255.0 - b) * line_mix;

            buf[idx] = r as u8;
            buf[idx + 1] = g as u8;
            buf[idx + 2] = b as u8;
            buf[idx + 3] = (alpha * 255.0) as u8;
        }
    }
    buf
}

fn dist_to_segment(p: (f32, f32), a: (f32, f32), b: (f32, f32)) -> f32 {
    let ab = (b.0 - a.0, b.1 - a.1);
    let ap = (p.0 - a.0, p.1 - a.1);
    let len_sq = ab.0 * ab.0 + ab.1 * ab.1;
    let t = if len_sq > 0.0 {
        ((ap.0 * ab.0 + ap.1 * ab.1) / len_sq).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let closest = (a.0 + ab.0 * t, a.1 + ab.1 * t);
    let d = (p.0 - closest.0, p.1 - closest.1);
    (d.0 * d.0 + d.1 * d.1).sqrt()
}
