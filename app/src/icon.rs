//! The Bplus mark, drawn in code: a dark rounded tile with a violet "B" and a "+",
//! in the same style as the sibling Wplus/Rplus icons. Used by build.rs (the .exe icon)
//! and at run time (window icon and banner badge).

const TILE: [f32; 3] = [0x1F as f32, 0x20 as f32, 0x24 as f32];
const BORDER: [f32; 3] = [0x2C as f32, 0x2E as f32, 0x33 as f32];
const INK: [f32; 3] = [0x8B as f32, 0x5C as f32, 0xF6 as f32];

fn seg(px: f32, py: f32, ax: f32, ay: f32, bx: f32, by: f32) -> f32 {
    let (dx, dy) = (bx - ax, by - ay);
    let t = (((px - ax) * dx + (py - ay) * dy) / (dx * dx + dy * dy)).clamp(0.0, 1.0);
    ((px - ax - t * dx).powi(2) + (py - ay - t * dy).powi(2)).sqrt()
}

/// Distance to the right half of a circle (the bowls of the B).
fn right_arc(px: f32, py: f32, cx: f32, cy: f32, r: f32) -> f32 {
    if px >= cx {
        (((px - cx).powi(2) + (py - cy).powi(2)).sqrt() - r).abs()
    } else {
        let top = ((px - cx).powi(2) + (py - (cy - r)).powi(2)).sqrt();
        let bot = ((px - cx).powi(2) + (py - (cy + r)).powi(2)).sqrt();
        top.min(bot)
    }
}

/// Distance from the edge of a rounded rectangle (negative inside).
fn round_rect(px: f32, py: f32, x0: f32, y0: f32, x1: f32, y1: f32, r: f32) -> f32 {
    let qx = (px - (x0 + x1) / 2.0).abs() - ((x1 - x0) / 2.0 - r);
    let qy = (py - (y0 + y1) / 2.0).abs() - ((y1 - y0) / 2.0 - r);
    let out = (qx.max(0.0).powi(2) + qy.max(0.0).powi(2)).sqrt();
    out + qx.max(qy).min(0.0) - r
}

/// Colour at a point in a 256-unit design square: (rgb, alpha).
fn sample(x: f32, y: f32) -> ([f32; 3], f32) {
    let edge = round_rect(x, y, 2.0, 2.0, 254.0, 254.0, 30.0);
    if edge > 0.0 {
        return ([0.0; 3], 0.0);
    }
    let half = 12.0; // letter stroke half-width
    let b = [
        seg(x, y, 66.0, 62.0, 66.0, 196.0),
        seg(x, y, 66.0, 62.0, 112.0, 62.0),
        right_arc(x, y, 112.0, 95.0, 33.0),
        seg(x, y, 66.0, 128.0, 116.0, 128.0),
        right_arc(x, y, 116.0, 162.0, 34.0),
        seg(x, y, 66.0, 196.0, 116.0, 196.0),
    ]
    .into_iter()
    .fold(f32::MAX, f32::min);
    let plus = seg(x, y, 172.0, 88.0, 212.0, 88.0).min(seg(x, y, 192.0, 68.0, 192.0, 108.0));
    if b <= half || plus <= 6.0 {
        (INK, 1.0)
    } else if edge > -4.0 {
        (BORDER, 1.0)
    } else {
        (TILE, 1.0)
    }
}

/// Straight (not premultiplied) RGBA pixels of the mark at `size` × `size`.
pub fn rgba(size: u32) -> Vec<u8> {
    let n = 4; // supersampling per axis
    let scale = 256.0 / size as f32;
    let mut out = Vec::with_capacity((size * size * 4) as usize);
    for py in 0..size {
        for px in 0..size {
            let (mut r, mut g, mut b, mut a) = (0.0, 0.0, 0.0, 0.0);
            for sy in 0..n {
                for sx in 0..n {
                    let x = (px as f32 + (sx as f32 + 0.5) / n as f32) * scale;
                    let y = (py as f32 + (sy as f32 + 0.5) / n as f32) * scale;
                    let (c, al) = sample(x, y);
                    r += c[0] * al;
                    g += c[1] * al;
                    b += c[2] * al;
                    a += al;
                }
            }
            let k = (n * n) as f32;
            if a > 0.0 {
                out.extend([(r / a).round() as u8, (g / a).round() as u8, (b / a).round() as u8, (a / k * 255.0).round() as u8]);
            } else {
                out.extend([0, 0, 0, 0]);
            }
        }
    }
    out
}

/// A Windows .ico file holding the mark at the usual sizes (32-bit images).
#[allow(dead_code)]
pub fn ico() -> Vec<u8> {
    let sizes = [16u32, 24, 32, 48, 64, 128, 256];
    let images: Vec<Vec<u8>> = sizes
        .iter()
        .map(|&s| {
            let px = rgba(s);
            let mut bmp = Vec::new();
            // BITMAPINFOHEADER: height counts the colour rows and the mask rows.
            bmp.extend(40u32.to_le_bytes());
            bmp.extend((s as i32).to_le_bytes());
            bmp.extend((2 * s as i32).to_le_bytes());
            bmp.extend(1u16.to_le_bytes());
            bmp.extend(32u16.to_le_bytes());
            bmp.extend([0u8; 24]);
            for row in (0..s).rev() {
                for col in 0..s {
                    let i = ((row * s + col) * 4) as usize;
                    bmp.extend([px[i + 2], px[i + 1], px[i], px[i + 3]]);
                }
            }
            let mask_row = (s.div_ceil(32) * 4) as usize;
            bmp.extend(vec![0u8; mask_row * s as usize]);
            bmp
        })
        .collect();
    let mut out = Vec::new();
    out.extend(0u16.to_le_bytes());
    out.extend(1u16.to_le_bytes());
    out.extend((sizes.len() as u16).to_le_bytes());
    let mut offset = 6 + 16 * sizes.len() as u32;
    for (s, img) in sizes.iter().zip(&images) {
        let d = if *s >= 256 { 0u8 } else { *s as u8 };
        out.extend([d, d, 0, 0]);
        out.extend(1u16.to_le_bytes());
        out.extend(32u16.to_le_bytes());
        out.extend((img.len() as u32).to_le_bytes());
        out.extend(offset.to_le_bytes());
        offset += img.len() as u32;
    }
    for img in images {
        out.extend(img);
    }
    out
}
