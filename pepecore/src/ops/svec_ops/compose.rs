//! Compose two `SVec` images onto one canvas and export a layered PSD.
//!
//! The first image is the background (defines canvas size and bit depth),
//! the second is placed on top with configurable size, position and scaling.
//! The result keeps both images as PSD layers plus a flattened merged
//! composite, so it opens in Photoshop with editable layers.
//!
//! # Examples
//!
//! ```no_run
//! use pepecore::compose::{PsdComposeOptions, compose_to_psd};
//! use pepecore::enums::ImgColor;
//! use pepecore::read::read_in_path;
//!
//! let bg = read_in_path("bg.png", ImgColor::RGB).unwrap();
//! let fg = read_in_path("logo.png", ImgColor::RGBA).unwrap();
//! let opts = PsdComposeOptions::fit(800, 600);
//! compose_to_psd(bg, fg, &opts, "out.psd").unwrap();
//! ```

use std::path::Path;

use fast_image_resize::{FilterType, ResizeAlg};
use pepecore_array::{ImgData, PixelType, SVec, Shape};
use photocraft_psd::{ColorMode, Compression, LayerSpec, PixelData, PsdBuilder};

use crate::errors::SaveError;
use crate::errors::SaveError::{RGBSaveError, UnsupportedChannelSaveError};
use crate::ops::svec_ops::resize::fir::ResizeSVec;

/// How the foreground image is scaled onto the canvas.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OverlayFit {
    /// Stretch to exactly `width × height`, ignoring aspect ratio.
    Stretch { width: usize, height: usize },
    /// Fit inside `width × height`, keeping aspect ratio.
    Fit { width: usize, height: usize },
    /// Fill `width × height` (cover), keeping aspect ratio; excess is cropped.
    Cover { width: usize, height: usize },
    /// Keep original size, no scaling.
    Original,
}

/// Anchor of the overlay box on the canvas.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OverlayAnchor {
    TopLeft,
    TopCenter,
    TopRight,
    CenterLeft,
    Center,
    CenterRight,
    BottomLeft,
    BottomCenter,
    BottomRight,
}

/// PSD channel compression.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PsdCompression {
    Raw,
    #[default]
    Rle,
    Zip,
    ZipPrediction,
}

impl From<PsdCompression> for Compression {
    fn from(value: PsdCompression) -> Self {
        match value {
            PsdCompression::Raw => Compression::Raw,
            PsdCompression::Rle => Compression::Rle,
            PsdCompression::Zip => Compression::Zip,
            PsdCompression::ZipPrediction => Compression::ZipPrediction,
        }
    }
}

/// Options for [`compose_to_psd`].
#[derive(Debug, Clone, Copy)]
pub struct PsdComposeOptions {
    /// How the foreground is scaled. Target box is relative to the canvas.
    pub fit: OverlayFit,
    /// Offset of the overlay box from the anchor point, in pixels.
    /// Positive `x` moves right, positive `y` moves down; may push the
    /// overlay partially (or fully) outside the canvas — outside parts clip.
    pub offset: (i32, i32),
    /// Anchor of the overlay box on the canvas.
    pub anchor: OverlayAnchor,
    /// Resampling used when the foreground must be scaled.
    pub resize_alg: ResizeAlg,
    /// Passed to `fast_image_resize` as premultiplied-alpha hint.
    pub premultiplied_alpha: bool,
    /// Layer name for the background.
    pub background_name: &'static str,
    /// Layer name for the overlay.
    pub overlay_name: &'static str,
    /// Channel compression for layers and merged image.
    pub compression: PsdCompression,
}

impl Default for PsdComposeOptions {
    fn default() -> Self {
        Self {
            fit: OverlayFit::Original,
            offset: (0, 0),
            anchor: OverlayAnchor::Center,
            resize_alg: ResizeAlg::Convolution(FilterType::Lanczos3),
            premultiplied_alpha: false,
            background_name: "Background",
            overlay_name: "Overlay",
            compression: PsdCompression::Rle,
        }
    }
}

impl PsdComposeOptions {
    /// Centered overlay stretched to `width × height`.
    pub fn stretch(width: usize, height: usize) -> Self {
        Self {
            fit: OverlayFit::Stretch { width, height },
            ..Self::default()
        }
    }

    /// Centered overlay fitted inside `width × height`, aspect kept.
    pub fn fit(width: usize, height: usize) -> Self {
        Self {
            fit: OverlayFit::Fit { width, height },
            ..Self::default()
        }
    }

    /// Centered overlay covering `width × height`, aspect kept, excess cropped.
    pub fn cover(width: usize, height: usize) -> Self {
        Self {
            fit: OverlayFit::Cover { width, height },
            ..Self::default()
        }
    }
}

/// Compose `background` + `foreground` and save a layered PSD to `path`.
///
/// Canvas size and bit depth come from `background` (u8 / u16 only);
/// `foreground` is converted to the same depth and color layout before
/// placement. Returns the number of overlay pixels actually drawn
/// (0 = fully outside the canvas).
///
/// # Parameters
///
/// - `background`: bottom layer; defines canvas size, depth, color layout.
/// - `foreground`: top layer; scaled per `opts.fit`, anchored per
///   `opts.anchor` + `opts.offset`, alpha-blended over the background.
/// - `opts`: placement, scaling and PSD settings.
/// - `path`: output `.psd` file.
///
/// # Errors
///
/// Returns `SaveError` on unsupported pixel types/layouts or PSD build/IO
/// failures.
pub fn compose_to_psd<P: AsRef<Path> + ?Sized>(
    background: SVec,
    foreground: SVec,
    opts: &PsdComposeOptions,
    path: &P,
) -> Result<usize, SaveError> {
    let canvas = compose_layers(background, foreground, opts)?;
    let depth = if canvas.depth_is_u16 { 16 } else { 8 };
    let mut builder = PsdBuilder::new(canvas.width as u32, canvas.height as u32)
        .color_mode(if canvas.gray() { ColorMode::Grayscale } else { ColorMode::Rgb })
        .depth(depth)
        .compression(opts.compression.into());

    builder.push_layer(canvas.background_spec(opts.background_name)?);
    let drawn = if canvas.over_w == 0 || canvas.over_h == 0 {
        0
    } else {
        builder.push_layer(canvas.overlay_spec(opts.overlay_name)?);
        canvas.over_w * canvas.over_h
    };
    builder.composite(canvas.composite_pixels()?);

    let bytes = builder.to_bytes().map_err(|e| RGBSaveError(format!("{:?}", e)))?;
    std::fs::write(path, bytes).map_err(|e| RGBSaveError(format!("{:?}", e)))?;
    Ok(drawn)
}

// ---------------------------------------------------------------------------
// Internal pipeline: normalize -> scale -> place -> blend.
// ---------------------------------------------------------------------------

/// Foreground converted to canvas depth/layout, scaled and alpha-blended over
/// the background. Holds the flattened canvas (merged composite), the full
/// background pixels (bottom layer) and the clipped overlay (top layer).
struct ComposedCanvas {
    width: usize,
    height: usize,
    /// 2 = gray(+alpha), 4 = rgb(+alpha). Background without alpha is stored
    /// opaque; layers always carry an alpha plane.
    channels: usize,
    depth_is_u16: bool,
    /// Flattened canvas, pixel order.
    flat_u8: Vec<u8>,
    flat_u16: Vec<u16>,
    /// Background layer pixels (full canvas, with alpha).
    bg_u8: Vec<u8>,
    bg_u16: Vec<u16>,
    /// Overlay pixels clipped to canvas.
    over_u8: Vec<u8>,
    over_u16: Vec<u16>,
    over_left: i32,
    over_top: i32,
    over_w: usize,
    over_h: usize,
}

fn compose_layers(mut background: SVec, mut foreground: SVec, opts: &PsdComposeOptions) -> Result<ComposedCanvas, SaveError> {
    let (_, bw, bc_opt) = background.shape();
    let bg_channels = bc_opt.unwrap_or(1);
    if !matches!(bg_channels, 1 | 3 | 4) {
        return Err(UnsupportedChannelSaveError(format!(
            "background has {} channels",
            bg_channels
        )));
    }
    if bw == 0 || background.shape().0 == 0 {
        return Err(UnsupportedChannelSaveError("background image is empty".to_string()));
    }
    let depth_is_u16 = match (background.pixel_type(), foreground.pixel_type()) {
        (PixelType::F32, _) | (_, PixelType::F32) => {
            return Err(UnsupportedChannelSaveError(
                "F32 images are not supported for PSD export".to_string(),
            ));
        }
        (PixelType::U16, _) | (_, PixelType::U16) => true,
        _ => false,
    };
    if depth_is_u16 {
        background.as_u16();
        foreground.as_u16();
    } else {
        background.as_u8();
        foreground.as_u8();
    }

    // Foreground always carries alpha internally (opaque if it had none).
    let gray_canvas = bg_channels == 1;
    to_layer_layout(&mut foreground, gray_canvas)?;
    let (fh, fw, _) = foreground.shape();
    if fw == 0 || fh == 0 {
        return Err(UnsupportedChannelSaveError("foreground image is empty".to_string()));
    }

    let (box_w, box_h) = overlay_box(fw, fh, opts.fit);
    if (box_w, box_h) != (fw, fh) {
        foreground.resize(box_h, box_w, opts.resize_alg, opts.premultiplied_alpha);
    }
    let (th, tw, _) = foreground.shape();
    let (left, top) = anchor_pos(tw, th, bw, background.shape().0, opts.anchor, opts.offset);

    // Clip overlay to canvas; fg origin of the clipped box.
    let cx0 = left.max(0) as usize;
    let cy0 = top.max(0) as usize;
    let cx1 = (left + tw as i32).min(bw as i32).max(0) as usize;
    let cy1 = (top + th as i32).min(background.shape().0 as i32).max(0) as usize;
    let over_w = cx1.saturating_sub(cx0);
    let over_h = cy1.saturating_sub(cy0);
    let fg_ox = (cx0 as i32 - left).max(0) as usize;
    let fg_oy = (cy0 as i32 - top).max(0) as usize;

    let cc = if gray_canvas { 2 } else { 4 };
    let mut canvas = ComposedCanvas {
        width: bw,
        height: background.shape().0,
        channels: cc,
        depth_is_u16,
        flat_u8: Vec::new(),
        flat_u16: Vec::new(),
        bg_u8: Vec::new(),
        bg_u16: Vec::new(),
        over_u8: Vec::new(),
        over_u16: Vec::new(),
        over_left: cx0 as i32,
        over_top: cy0 as i32,
        over_w,
        over_h,
    };
    if depth_is_u16 {
        let bg = background
            .get_data::<u16>()
            .map_err(|e| RGBSaveError(format!("{:?}", e)))?
            .to_vec();
        let fg = foreground
            .get_data::<u16>()
            .map_err(|e| RGBSaveError(format!("{:?}", e)))?
            .to_vec();
        let (flat, bg_full, over) = blend_u16(
            &bg,
            bw,
            canvas.height,
            bg_channels,
            &fg,
            tw,
            cc,
            cx0,
            cy0,
            fg_ox,
            fg_oy,
            over_w,
            over_h,
        );
        canvas.flat_u16 = flat;
        canvas.bg_u16 = bg_full;
        canvas.over_u16 = over;
    } else {
        let bg = background
            .get_data::<u8>()
            .map_err(|e| RGBSaveError(format!("{:?}", e)))?
            .to_vec();
        let fg = foreground
            .get_data::<u8>()
            .map_err(|e| RGBSaveError(format!("{:?}", e)))?
            .to_vec();
        let (flat, bg_full, over) = blend_u8(
            &bg,
            bw,
            canvas.height,
            bg_channels,
            &fg,
            tw,
            cc,
            cx0,
            cy0,
            fg_ox,
            fg_oy,
            over_w,
            over_h,
        );
        canvas.flat_u8 = flat;
        canvas.bg_u8 = bg_full;
        canvas.over_u8 = over;
    }
    Ok(canvas)
}

/// Target overlay size for the requested fit mode (aspect kept except Stretch).
fn overlay_box(fw: usize, fh: usize, fit: OverlayFit) -> (usize, usize) {
    match fit {
        OverlayFit::Original => (fw, fh),
        OverlayFit::Stretch { width, height } => (width.max(1), height.max(1)),
        OverlayFit::Fit { width, height } => {
            let s = (width.max(1) as f64 / fw as f64).min(height.max(1) as f64 / fh as f64);
            (
                ((fw as f64 * s).round() as usize).max(1),
                ((fh as f64 * s).round() as usize).max(1),
            )
        }
        OverlayFit::Cover { width, height } => {
            let s = (width.max(1) as f64 / fw as f64).max(height.max(1) as f64 / fh as f64);
            (
                ((fw as f64 * s).round() as usize).max(1),
                ((fh as f64 * s).round() as usize).max(1),
            )
        }
    }
}

/// Top-left position of a `tw × th` overlay on a `bw × bh` canvas.
fn anchor_pos(tw: usize, th: usize, bw: usize, bh: usize, anchor: OverlayAnchor, offset: (i32, i32)) -> (i32, i32) {
    let (bw, bh, tw, th) = (bw as i32, bh as i32, tw as i32, th as i32);
    let (x, y) = match anchor {
        OverlayAnchor::TopLeft => (0, 0),
        OverlayAnchor::TopCenter => ((bw - tw) / 2, 0),
        OverlayAnchor::TopRight => (bw - tw, 0),
        OverlayAnchor::CenterLeft => (0, (bh - th) / 2),
        OverlayAnchor::Center => ((bw - tw) / 2, (bh - th) / 2),
        OverlayAnchor::CenterRight => (bw - tw, (bh - th) / 2),
        OverlayAnchor::BottomLeft => (0, bh - th),
        OverlayAnchor::BottomCenter => ((bw - tw) / 2, bh - th),
        OverlayAnchor::BottomRight => (bw - tw, bh - th),
    };
    (x + offset.0, y + offset.1)
}

/// Convert `img` to GrayA (gray canvas) or RGBA (color canvas).
/// Gray→color replicates channels; color→gray uses BT.709; missing alpha is opaque.
fn to_layer_layout(img: &mut SVec, gray_canvas: bool) -> Result<(), SaveError> {
    let (h, w, c_opt) = img.shape();
    let c = c_opt.unwrap_or(1);
    let (target_c, is_gray) = if gray_canvas { (2, true) } else { (4, false) };
    if !is_gray && c == 4 || is_gray && c == 2 {
        if is_gray {
            img.shape = Shape::new(h, w, Some(2));
        }
        return Ok(());
    }
    if c == 1 && !is_gray {
        // Fast path without float math: replicate + opaque alpha.
    }
    match img.pixel_type() {
        PixelType::U8 => {
            let src = img.get_data::<u8>().map_err(|e| RGBSaveError(format!("{:?}", e)))?.to_vec();
            img.data = ImgData::U8(if is_gray {
                any_to_graya_u8(&src, c)
            } else {
                any_to_rgba_u8(&src, c)
            });
        }
        PixelType::U16 => {
            let src = img.get_data::<u16>().map_err(|e| RGBSaveError(format!("{:?}", e)))?.to_vec();
            img.data = ImgData::U16(if is_gray {
                any_to_graya_u16(&src, c)
            } else {
                any_to_rgba_u16(&src, c)
            });
        }
        PixelType::F32 => {
            return Err(UnsupportedChannelSaveError(
                "F32 images are not supported for PSD export".to_string(),
            ));
        }
    }
    img.shape = Shape::new(h, w, Some(target_c));
    Ok(())
}

fn any_to_rgba_u8(src: &[u8], c: usize) -> Vec<u8> {
    let pixels = src.len() / c.max(1);
    let mut out = Vec::with_capacity(pixels * 4);
    match c {
        1 => {
            for &g in src {
                out.extend_from_slice(&[g, g, g, u8::MAX]);
            }
        }
        2 => {
            for ga in src.chunks_exact(2) {
                out.extend_from_slice(&[ga[0], ga[0], ga[0], ga[1]]);
            }
        }
        3 => {
            for rgb in src.chunks_exact(3) {
                out.extend_from_slice(&[rgb[0], rgb[1], rgb[2], u8::MAX]);
            }
        }
        _ => {
            for rgba in src.chunks_exact(4).take(pixels) {
                out.extend_from_slice(&rgba[..4.min(rgba.len())]);
            }
        }
    }
    out
}

fn any_to_rgba_u16(src: &[u16], c: usize) -> Vec<u16> {
    let pixels = src.len() / c.max(1);
    let mut out = Vec::with_capacity(pixels * 4);
    match c {
        1 => {
            for &g in src {
                out.extend_from_slice(&[g, g, g, u16::MAX]);
            }
        }
        2 => {
            for ga in src.chunks_exact(2) {
                out.extend_from_slice(&[ga[0], ga[0], ga[0], ga[1]]);
            }
        }
        3 => {
            for rgb in src.chunks_exact(3) {
                out.extend_from_slice(&[rgb[0], rgb[1], rgb[2], u16::MAX]);
            }
        }
        _ => {
            for rgba in src.chunks_exact(4).take(pixels) {
                out.extend_from_slice(&rgba[..4.min(rgba.len())]);
            }
        }
    }
    out
}

fn luma_u8(r: u8, g: u8, b: u8) -> u8 {
    (r as f32 * 0.2126 + g as f32 * 0.7152 + b as f32 * 0.0722) as u8
}

fn luma_u16(r: u16, g: u16, b: u16) -> u16 {
    (0.2126 * r as f32 + 0.7152 * g as f32 + 0.0722 * b as f32) as u16
}

fn any_to_graya_u8(src: &[u8], c: usize) -> Vec<u8> {
    let pixels = src.len() / c.max(1);
    let mut out = Vec::with_capacity(pixels * 2);
    match c {
        1 => {
            for &g in src {
                out.extend_from_slice(&[g, u8::MAX]);
            }
        }
        2 => out.extend_from_slice(src),
        3 => {
            for rgb in src.chunks_exact(3) {
                out.push(luma_u8(rgb[0], rgb[1], rgb[2]));
                out.push(u8::MAX);
            }
        }
        _ => {
            for rgba in src.chunks_exact(4) {
                out.push(luma_u8(rgba[0], rgba[1], rgba[2]));
                out.push(rgba[3]);
            }
        }
    }
    out
}

fn any_to_graya_u16(src: &[u16], c: usize) -> Vec<u16> {
    let pixels = src.len() / c.max(1);
    let mut out = Vec::with_capacity(pixels * 2);
    match c {
        1 => {
            for &g in src {
                out.extend_from_slice(&[g, u16::MAX]);
            }
        }
        2 => out.extend_from_slice(src),
        3 => {
            for rgb in src.chunks_exact(3) {
                out.push(luma_u16(rgb[0], rgb[1], rgb[2]));
                out.push(u16::MAX);
            }
        }
        _ => {
            for rgba in src.chunks_exact(4) {
                out.push(luma_u16(rgba[0], rgba[1], rgba[2]));
                out.push(rgba[3]);
            }
        }
    }
    out
}

// ---------------------------------------------------------------------------
// Blending: background -> opaque canvas, foreground src-over.
// Returns (flattened, background_with_alpha, clipped_overlay).
// ---------------------------------------------------------------------------

#[allow(clippy::too_many_arguments)]
fn blend_u8(
    bg: &[u8],
    bw: usize,
    bh: usize,
    bg_c: usize,
    fg: &[u8],
    tw: usize,
    cc: usize,
    cx0: usize,
    cy0: usize,
    fg_ox: usize,
    fg_oy: usize,
    over_w: usize,
    over_h: usize,
) -> (Vec<u8>, Vec<u8>, Vec<u8>) {
    let mut flat = vec![0u8; bw * bh * cc];
    let mut bg_full = vec![0u8; bw * bh * cc];
    for y in 0..bh {
        for x in 0..bw {
            let s = (y * bw + x) * bg_c;
            let d = (y * bw + x) * cc;
            bg_px_u8(&mut bg_full[d..d + cc], &bg[s..s + bg_c], bg_c, cc);
            flat[d..d + cc].copy_from_slice(&bg_full[d..d + cc]);
        }
    }
    let mut over = vec![0u8; over_w * over_h * cc];
    for oy in 0..over_h {
        for ox in 0..over_w {
            let s = ((fg_oy + oy) * tw + (fg_ox + ox)) * cc;
            let d = ((cy0 + oy) * bw + (cx0 + ox)) * cc;
            let o = (oy * over_w + ox) * cc;
            over[o..o + cc].copy_from_slice(&fg[s..s + cc]);
            blend_px_u8(&mut flat[d..d + cc], &fg[s..s + cc]);
        }
    }
    (flat, bg_full, over)
}

/// Expand one background pixel to the canvas layout (opaque unless gray+alpha input).
fn bg_px_u8(dst: &mut [u8], src: &[u8], bg_c: usize, cc: usize) {
    if cc == 2 {
        dst[0] = src[0];
        dst[1] = if bg_c == 2 { src[1] } else { u8::MAX };
    } else if bg_c == 4 {
        dst[..4].copy_from_slice(&src[..4]);
    } else if bg_c == 3 {
        dst[..3].copy_from_slice(&src[..3]);
        dst[3] = u8::MAX;
    } else {
        dst[0] = src[0];
        dst[1] = src[0];
        dst[2] = src[0];
        dst[3] = u8::MAX;
    }
}

/// Src-over blend of one pixel (last channel is alpha).
fn blend_px_u8(dst: &mut [u8], src: &[u8]) {
    let cc = dst.len();
    let a = src[cc - 1] as f32 / 255.0;
    let inv = 1.0 - a;
    for i in 0..cc - 1 {
        dst[i] = (src[i] as f32 * a + dst[i] as f32 * inv).round().clamp(0.0, 255.0) as u8;
    }
    let da = dst[cc - 1] as f32 / 255.0;
    dst[cc - 1] = ((a + da * inv) * 255.0).round().clamp(0.0, 255.0) as u8;
}

#[allow(clippy::too_many_arguments)]
fn blend_u16(
    bg: &[u16],
    bw: usize,
    bh: usize,
    bg_c: usize,
    fg: &[u16],
    tw: usize,
    cc: usize,
    cx0: usize,
    cy0: usize,
    fg_ox: usize,
    fg_oy: usize,
    over_w: usize,
    over_h: usize,
) -> (Vec<u16>, Vec<u16>, Vec<u16>) {
    let mut flat = vec![0u16; bw * bh * cc];
    let mut bg_full = vec![0u16; bw * bh * cc];
    for y in 0..bh {
        for x in 0..bw {
            let s = (y * bw + x) * bg_c;
            let d = (y * bw + x) * cc;
            bg_px_u16(&mut bg_full[d..d + cc], &bg[s..s + bg_c], bg_c, cc);
            flat[d..d + cc].copy_from_slice(&bg_full[d..d + cc]);
        }
    }
    let mut over = vec![0u16; over_w * over_h * cc];
    for oy in 0..over_h {
        for ox in 0..over_w {
            let s = ((fg_oy + oy) * tw + (fg_ox + ox)) * cc;
            let d = ((cy0 + oy) * bw + (cx0 + ox)) * cc;
            let o = (oy * over_w + ox) * cc;
            over[o..o + cc].copy_from_slice(&fg[s..s + cc]);
            blend_px_u16(&mut flat[d..d + cc], &fg[s..s + cc]);
        }
    }
    (flat, bg_full, over)
}

fn bg_px_u16(dst: &mut [u16], src: &[u16], bg_c: usize, cc: usize) {
    if cc == 2 {
        dst[0] = src[0];
        dst[1] = if bg_c == 2 { src[1] } else { u16::MAX };
    } else if bg_c == 4 {
        dst[..4].copy_from_slice(&src[..4]);
    } else if bg_c == 3 {
        dst[..3].copy_from_slice(&src[..3]);
        dst[3] = u16::MAX;
    } else {
        dst[0] = src[0];
        dst[1] = src[0];
        dst[2] = src[0];
        dst[3] = u16::MAX;
    }
}

fn blend_px_u16(dst: &mut [u16], src: &[u16]) {
    let cc = dst.len();
    let a = src[cc - 1] as f32 / 65535.0;
    let inv = 1.0 - a;
    for i in 0..cc - 1 {
        dst[i] = (src[i] as f32 * a + dst[i] as f32 * inv).round().clamp(0.0, 65535.0) as u16;
    }
    let da = dst[cc - 1] as f32 / 65535.0;
    dst[cc - 1] = ((a + da * inv) * 65535.0).round().clamp(0.0, 65535.0) as u16;
}

// ---------------------------------------------------------------------------
// PSD layers.
// ---------------------------------------------------------------------------

impl ComposedCanvas {
    fn gray(&self) -> bool {
        self.channels == 2
    }

    fn composite_pixels(&self) -> Result<PixelData, SaveError> {
        if self.gray() {
            if self.depth_is_u16 {
                Ok(PixelData::GrayA16(self.flat_u16.clone()))
            } else {
                Ok(PixelData::GrayA8(self.flat_u8.clone()))
            }
        } else if self.depth_is_u16 {
            Ok(PixelData::Rgba16(self.flat_u16.clone()))
        } else {
            Ok(PixelData::Rgba8(self.flat_u8.clone()))
        }
    }

    fn layer_pixels(&self, pixels_u8: &[u8], pixels_u16: &[u16], w: usize, h: usize) -> Result<PixelData, SaveError> {
        if pixels_u8.len() != w * h * self.channels && pixels_u16.len() != w * h * self.channels {
            return Err(RGBSaveError(format!("layer buffer size mismatch: {}x{}", w, h)));
        }
        if self.gray() {
            if self.depth_is_u16 {
                Ok(PixelData::GrayA16(pixels_u16.to_vec()))
            } else {
                Ok(PixelData::GrayA8(pixels_u8.to_vec()))
            }
        } else if self.depth_is_u16 {
            Ok(PixelData::Rgba16(pixels_u16.to_vec()))
        } else {
            Ok(PixelData::Rgba8(pixels_u8.to_vec()))
        }
    }

    fn background_spec(&self, name: &'static str) -> Result<LayerSpec, SaveError> {
        let pixels = self.layer_pixels(&self.bg_u8, &self.bg_u16, self.width, self.height)?;
        Ok(LayerSpec::new(name, 0, 0, self.width as u32, self.height as u32, pixels))
    }

    fn overlay_spec(&self, name: &'static str) -> Result<LayerSpec, SaveError> {
        let pixels = self.layer_pixels(&self.over_u8, &self.over_u16, self.over_w, self.over_h)?;
        Ok(LayerSpec::new(
            name,
            self.over_left,
            self.over_top,
            self.over_w as u32,
            self.over_h as u32,
            pixels,
        ))
    }
}
