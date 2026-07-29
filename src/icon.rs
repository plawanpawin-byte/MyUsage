/// Procedurally draws a simple filled-circle app icon so the project needs
/// no external image assets (keeps the binary small and the repo dependency-free).
pub fn generate_rgba(size: u32, rgb: [u8; 3]) -> Vec<u8> {
    let mut buf = vec![0u8; (size * size * 4) as usize];
    let center = size as f32 / 2.0;
    let radius = center - 1.0;

    for y in 0..size {
        for x in 0..size {
            let dx = x as f32 + 0.5 - center;
            let dy = y as f32 + 0.5 - center;
            let dist = (dx * dx + dy * dy).sqrt();
            let idx = ((y * size + x) * 4) as usize;
            if dist <= radius {
                // Soft anti-aliased edge over the last 1.5px.
                let edge = (radius - dist).min(1.5) / 1.5;
                let alpha = (edge.clamp(0.0, 1.0) * 255.0) as u8;
                buf[idx] = rgb[0];
                buf[idx + 1] = rgb[1];
                buf[idx + 2] = rgb[2];
                buf[idx + 3] = alpha;
            }
        }
    }
    buf
}
