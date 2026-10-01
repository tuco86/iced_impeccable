//! RGBA frames: PNG files, crop and zoom, the annotation overlay, the
//! recording pointer, and contact sheets.

use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::Path;

use font8x8::UnicodeFonts;
use iced::{Point, Rectangle, Size};

/// An RGBA8 frame, rows top to bottom.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Frame {
    pub(crate) rgba: Vec<u8>,
    pub(crate) size: Size<u32>,
}

impl Frame {
    pub(crate) fn new(size: Size<u32>, fill: [u8; 4]) -> Self {
        let pixels = size.width as usize * size.height as usize;
        Self {
            rgba: fill.repeat(pixels),
            size,
        }
    }

    fn put(&mut self, x: i64, y: i64, color: [u8; 4]) {
        if x < 0 || y < 0 || x >= i64::from(self.size.width) || y >= i64::from(self.size.height) {
            return;
        }
        let i = (y as usize * self.size.width as usize + x as usize) * 4;
        self.rgba[i..i + 4].copy_from_slice(&color);
    }

    fn fill_rect(&mut self, x: i64, y: i64, w: i64, h: i64, color: [u8; 4]) {
        for py in y..y + h {
            for px in x..x + w {
                self.put(px, py, color);
            }
        }
    }

    /// Copies `other` with its top-left corner at `(x, y)`.
    fn blit(&mut self, other: &Frame, x: u32, y: u32) {
        let w = other.size.width.min(self.size.width.saturating_sub(x)) as usize;
        for row in 0..other.size.height.min(self.size.height.saturating_sub(y)) as usize {
            let src = row * other.size.width as usize * 4;
            let dst = ((y as usize + row) * self.size.width as usize + x as usize) * 4;
            self.rgba[dst..dst + w * 4].copy_from_slice(&other.rgba[src..src + w * 4]);
        }
    }
}

pub(crate) fn write_png(path: &Path, frame: &Frame) -> Result<(), String> {
    let file = File::create(path).map_err(|e| e.to_string())?;
    let mut encoder = png::Encoder::new(BufWriter::new(file), frame.size.width, frame.size.height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header().map_err(|e| e.to_string())?;
    writer
        .write_image_data(&frame.rgba)
        .map_err(|e| e.to_string())?;
    writer.finish().map_err(|e| e.to_string())
}

/// Reads an 8-bit RGB or RGBA PNG (palette images are expanded) as RGBA.
pub(crate) fn read_png(path: &Path) -> Result<Frame, String> {
    let file = File::open(path).map_err(|e| e.to_string())?;
    let mut decoder = png::Decoder::new(BufReader::new(file));
    decoder.set_transformations(png::Transformations::EXPAND);
    let mut reader = decoder.read_info().map_err(|e| e.to_string())?;
    let mut buffer = vec![0; reader.output_buffer_size().ok_or("image too large")?];
    let info = reader.next_frame(&mut buffer).map_err(|e| e.to_string())?;
    buffer.truncate(info.buffer_size());
    if info.bit_depth != png::BitDepth::Eight {
        return Err(format!("{:?}-bit PNG, expected 8-bit", info.bit_depth));
    }
    let rgba = match info.color_type {
        png::ColorType::Rgba => buffer,
        png::ColorType::Rgb => buffer
            .as_chunks::<3>()
            .0
            .iter()
            .flat_map(|p| [p[0], p[1], p[2], 255])
            .collect(),
        other => return Err(format!("{other:?} PNG, expected RGB or RGBA")),
    };
    Ok(Frame {
        rgba,
        size: Size::new(info.width, info.height),
    })
}

/// Cuts out the logical rectangle `rect`, scaled to pixels and clamped to
/// the frame. Errors without the `err ` prefix.
pub(crate) fn crop(frame: &Frame, rect: Rectangle, scale: f32) -> Result<Frame, String> {
    let (fw, fh) = (frame.size.width as f32, frame.size.height as f32);
    let x0 = (rect.x * scale).round().clamp(0.0, fw);
    let y0 = (rect.y * scale).round().clamp(0.0, fh);
    let x1 = ((rect.x + rect.width) * scale).round().clamp(0.0, fw);
    let y1 = ((rect.y + rect.height) * scale).round().clamp(0.0, fh);
    if x1 <= x0 || y1 <= y0 {
        return Err(format!(
            "crop outside {}x{}",
            frame.size.width, frame.size.height
        ));
    }
    let (x0, y0, w, h) = (
        x0 as usize,
        y0 as usize,
        (x1 - x0) as usize,
        (y1 - y0) as usize,
    );
    let stride = frame.size.width as usize * 4;
    let mut rgba = Vec::with_capacity(w * h * 4);
    for row in y0..y0 + h {
        let start = row * stride + x0 * 4;
        rgba.extend_from_slice(&frame.rgba[start..start + w * 4]);
    }
    Ok(Frame {
        rgba,
        size: Size::new(w as u32, h as u32),
    })
}

/// Nearest-neighbour enlargement by `n`.
pub(crate) fn zoom(frame: &Frame, n: u32) -> Frame {
    if n == 1 {
        return frame.clone();
    }
    let (w, h) = (frame.size.width as usize, frame.size.height as usize);
    let n = n as usize;
    let mut rgba = Vec::with_capacity(w * h * n * n * 4);
    for y in 0..h {
        let mut row = Vec::with_capacity(w * n * 4);
        for x in 0..w {
            let pixel = &frame.rgba[(y * w + x) * 4..(y * w + x) * 4 + 4];
            for _ in 0..n {
                row.extend_from_slice(pixel);
            }
        }
        for _ in 0..n {
            rgba.extend_from_slice(&row);
        }
    }
    Frame {
        rgba,
        size: Size::new((w * n) as u32, (h * n) as u32),
    }
}

/// Halves both sides with a 2x2 box filter.
pub(crate) fn halve(frame: &Frame) -> Frame {
    let (w, h) = (frame.size.width as usize, frame.size.height as usize);
    let (hw, hh) = ((w / 2).max(1), (h / 2).max(1));
    let mut rgba = Vec::with_capacity(hw * hh * 4);
    for y in 0..hh {
        for x in 0..hw {
            for c in 0..4 {
                let mut sum = 0u32;
                for (dx, dy) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
                    let sx = (x * 2 + dx).min(w - 1);
                    let sy = (y * 2 + dy).min(h - 1);
                    sum += u32::from(frame.rgba[(sy * w + sx) * 4 + c]);
                }
                rgba.push(((sum + 2) / 4) as u8);
            }
        }
    }
    Frame {
        rgba,
        size: Size::new(hw as u32, hh as u32),
    }
}

/// Draws `text` with the 8x8 font, each font pixel a `block`-sized square,
/// top-left corner at `(x, y)`. Characters without a glyph draw as `?`.
fn draw_text(frame: &mut Frame, x: i64, y: i64, text: &str, block: i64, color: [u8; 4]) {
    for (i, c) in (0i64..).zip(text.chars()) {
        let glyph = font8x8::BASIC_FONTS
            .get(c)
            .or_else(|| font8x8::BASIC_FONTS.get('?'))
            .unwrap_or([0; 8]);
        for (row, bits) in (0i64..).zip(glyph) {
            for col in 0..8i64 {
                if bits & (1 << col) != 0 {
                    frame.fill_rect(
                        x + (i * 8 + col) * block,
                        y + row * block,
                        block,
                        block,
                        color,
                    );
                }
            }
        }
    }
}

/// Outlines each box (logical coordinates) in its colour and labels it with
/// its index at the top-left corner.
pub(crate) fn annotate(frame: &mut Frame, boxes: &[(usize, Rectangle, [u8; 4])], scale: f32) {
    const LINE: i64 = 2;
    let block: i64 = if scale < 1.5 { 1 } else { 2 };
    for &(_, rect, color) in boxes {
        let x = (rect.x * scale).round() as i64;
        let y = (rect.y * scale).round() as i64;
        let w = ((rect.width * scale).round() as i64).max(1);
        let h = ((rect.height * scale).round() as i64).max(1);
        frame.fill_rect(x, y, w, LINE, color);
        frame.fill_rect(x, y + h - LINE, w, LINE, color);
        frame.fill_rect(x, y, LINE, h, color);
        frame.fill_rect(x + w - LINE, y, LINE, h, color);
    }
    // Labels after all outlines, so no outline crosses a label.
    for &(index, rect, _) in boxes {
        let label = index.to_string();
        let x = (rect.x * scale).round() as i64;
        let y = (rect.y * scale).round() as i64;
        let w = (label.len() as i64 * 8 + 2) * block;
        let h = 10 * block;
        frame.fill_rect(x, y, w, h, [0, 0, 0, 255]);
        draw_text(
            frame,
            x + block,
            y + block,
            &label,
            block,
            [255, 255, 255, 255],
        );
    }
}

/// The arrow a recording draws, tip at the top left: `X` outline, `.` fill,
/// space transparent. One character is one logical pixel.
const POINTER: [&str; 17] = [
    "X",
    "XX",
    "X.X",
    "X..X",
    "X...X",
    "X....X",
    "X.....X",
    "X......X",
    "X.......X",
    "X........X",
    "X.....XXXXX",
    "X..X..X",
    "X.X X..X",
    "XX  X..X",
    "X    X..X",
    "     X..X",
    "      XX",
];

/// Paints the pointer with its tip at the logical point `at`; `pressed`
/// adds a ring around the tip, so a drag reads as a drag.
pub(crate) fn draw_pointer(frame: &mut Frame, at: Point, scale: f32, pressed: bool) {
    const OUTLINE: [u8; 4] = [0, 0, 0, 255];
    const FILL: [u8; 4] = [255, 255, 255, 255];
    const RING: [u8; 4] = [255, 196, 0, 255];
    let tip = (at.x * scale, at.y * scale);
    if pressed {
        let radius = 9.0 * scale;
        let half = scale;
        let reach = (radius + half).ceil() as i64;
        let (cx, cy) = (tip.0.round() as i64, tip.1.round() as i64);
        for dy in -reach..=reach {
            for dx in -reach..=reach {
                let distance = ((dx * dx + dy * dy) as f32).sqrt();
                if (distance - radius).abs() <= half {
                    frame.put(cx + dx, cy + dy, RING);
                }
            }
        }
    }
    let block = scale.ceil() as i64;
    for (row, line) in (0i64..).zip(POINTER) {
        for (col, c) in (0i64..).zip(line.chars()) {
            let color = match c {
                'X' => OUTLINE,
                '.' => FILL,
                _ => continue,
            };
            let x0 = (tip.0 + col as f32 * scale).floor() as i64;
            let y0 = (tip.1 + row as f32 * scale).floor() as i64;
            frame.fill_rect(x0, y0, block, block, color);
        }
    }
}

const GUTTER: u32 = 16;
const LABEL: u32 = 20;

/// Lays images out in a grid of equal cells, each under a label strip. The
/// images are halved until the sheet is at most `max_width` wide; labels and
/// gutters keep their size, so the labels stay readable.
pub(crate) fn sheet(
    images: &[(String, Frame)],
    cols: Option<usize>,
    max_width: u32,
) -> Result<Frame, String> {
    if images.is_empty() {
        return Err("no images".to_owned());
    }
    let n = images.len();
    let cols = cols
        .unwrap_or_else(|| (n as f64).sqrt().ceil() as usize)
        .clamp(1, n);
    let rows = n.div_ceil(cols);
    let layout_width = |cell_w: u32| GUTTER + cols as u32 * (cell_w + GUTTER);
    let mut cell_w = images.iter().map(|(_, f)| f.size.width).max().unwrap_or(1);
    let mut halvings = 0;
    while layout_width(cell_w) > max_width && cell_w > 1 {
        cell_w = (cell_w / 2).max(1);
        halvings += 1;
    }
    let images: Vec<(&str, Frame)> = images
        .iter()
        .map(|(label, frame)| {
            let mut frame = frame.clone();
            for _ in 0..halvings {
                frame = halve(&frame);
            }
            (label.as_str(), frame)
        })
        .collect();
    let cell_w = images.iter().map(|(_, f)| f.size.width).max().unwrap_or(1);
    let cell_h = images.iter().map(|(_, f)| f.size.height).max().unwrap_or(1);
    let width = layout_width(cell_w);
    let height = GUTTER + rows as u32 * (LABEL + cell_h + GUTTER);
    let mut out = Frame::new(Size::new(width, height), [16, 16, 16, 255]);
    for (i, (label, image)) in images.iter().enumerate() {
        let x = GUTTER + (i % cols) as u32 * (cell_w + GUTTER);
        let y = GUTTER + (i / cols) as u32 * (LABEL + cell_h + GUTTER);
        out.fill_rect(
            i64::from(x),
            i64::from(y),
            i64::from(cell_w),
            i64::from(LABEL),
            [32, 32, 32, 255],
        );
        let fits = (cell_w.saturating_sub(8) / 8) as usize;
        let text: String = label.chars().take(fits).collect();
        draw_text(
            &mut out,
            i64::from(x) + 4,
            i64::from(y) + 6,
            &text,
            1,
            [255, 255, 255, 255],
        );
        out.blit(image, x, y + LABEL);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn frame(w: u32, h: u32) -> Frame {
        let mut f = Frame::new(Size::new(w, h), [0, 0, 0, 255]);
        for (i, p) in f.rgba.as_chunks_mut::<4>().0.iter_mut().enumerate() {
            p[0] = i as u8;
        }
        f
    }

    #[test]
    fn crop_scales_clamps_and_rejects_outside() {
        let f = frame(20, 10);
        let c = crop(
            &f,
            Rectangle::new(Point::new(2.0, 1.0), Size::new(3.0, 2.0)),
            2.0,
        )
        .unwrap();
        assert_eq!(c.size, Size::new(6, 4));
        assert_eq!(c.rgba[0], (2 * 20 + 4) as u8);
        let c = crop(
            &f,
            Rectangle::new(Point::new(15.0, 5.0), Size::new(100.0, 100.0)),
            1.0,
        )
        .unwrap();
        assert_eq!(c.size, Size::new(5, 5));
        assert_eq!(
            crop(
                &f,
                Rectangle::new(Point::new(30.0, 0.0), Size::new(5.0, 5.0)),
                1.0
            )
            .unwrap_err(),
            "crop outside 20x10"
        );
    }

    #[test]
    fn zoom_repeats_pixels() {
        let f = frame(2, 1);
        let z = zoom(&f, 3);
        assert_eq!(z.size, Size::new(6, 3));
        let reds: Vec<u8> = z.rgba.as_chunks::<4>().0.iter().map(|p| p[0]).collect();
        assert_eq!(&reds[..6], &[0, 0, 0, 1, 1, 1]);
        assert_eq!(&reds[12..], &[0, 0, 0, 1, 1, 1]);
    }

    #[test]
    fn sheet_dimensions_and_halving() {
        let images: Vec<(String, Frame)> =
            (0..3).map(|i| (format!("{i}"), frame(100, 50))).collect();
        let s = sheet(&images, None, 10_000).unwrap();
        // Two columns, two rows.
        assert_eq!(s.size, Size::new(16 + 2 * 116, 16 + 2 * (20 + 50 + 16)));
        let s = sheet(&images, Some(3), 10_000).unwrap();
        assert_eq!(s.size.width, 16 + 3 * 116);
        // 100 px cells halve twice: 16 + 3 * (50 + 16) = 214 is still wider than 200.
        let s = sheet(&images, Some(3), 200).unwrap();
        assert_eq!(s.size, Size::new(16 + 3 * (25 + 16), 16 + (20 + 12 + 16)));
        assert!(sheet(&[], None, 100).is_err());
    }

    #[test]
    fn halve_averages_two_by_two() {
        let mut f = Frame::new(Size::new(2, 2), [0, 0, 0, 255]);
        f.rgba[0] = 100;
        f.rgba[4] = 200;
        let h = halve(&f);
        assert_eq!(h.size, Size::new(1, 1));
        assert_eq!(h.rgba, vec![75, 0, 0, 255]);
    }
}
