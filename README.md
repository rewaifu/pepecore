# PepeCore

A Rust library for image decoding, encoding, and processing using an efficient `SVec` data structure.

PepeCore provides:

- **SVec** – high-performance, image buffers (`array::svec::SVec`).
- **Image decoding** – read images from file paths or in-memory buffers (`read::read_in_path`, `read::read_in_buffer`).
- **Image saving** – write `SVec` images to disk in common formats (`save::svec_save`).
- **Color conversions** – dynamic RGB ↔ grayscale, YCbCr, CMYK, and channel swaps via `cvt_color`.
- **Halftone effects** – apply dot-based halftoning (`halftone`, `rotate_halftone`).
- **Screentone effects** – comic-style dot screening (`screentone`, `rotate_screentone`).
- **(Experimental)** Color levels adjustment – _coming soon_.

## Installation

Add PepeCore to your `Cargo.toml`:

```toml
[dependencies]
pepecore = "0.2"
```

Then in your crate:

```rust
extern crate pepecore;
use pepecore::array::SVec;
use pepecore::{read::read_in_path, save::svec_save};
use pepecore::{cvt_color, halftone, screentone};
use pepecore::enums::{ImgColor, CVTColor, DotType};
```

## Quick Start

```rust
use pepecore::array::SVec;
use pepecore::{
    read::read_in_path,
    save::svec_save,
    cvt_color,
    screentone,
    enums::{ImgColor, CVTColor, DotType},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Decode an image file as RGB
    let mut img: SVec = read_in_path("input.jpg", ImgColor::RGB)?;
    
    // 2. Convert RGB → Grayscale (BT.709)
    cvt_color(&mut img, CVTColor::RGB2Gray_709);
    
    // 3. Apply a screentone effect (dot pattern 8×8)
    screentone(&mut img, 8, &DotType::Square);

    // 4. Save the processed image
    svec_save(img, "output.png")?;

    Ok(())
}
```

## Features

### Decoding

- **Flexible input**: file paths or raw byte buffers.
- **PSD support**: reads both layered and flattened PSD files.
- **Dynamic types**: returns either `u8`, `u16`, or `f32` data.

### Saving

- Handles `Luma`, `LumaA`, `Rgb`, `Rgba` in `u8`, `u16`, and `f32` formats.
- Converts floating-point to `u8` with simple scaling (`×255`).

### Color Conversion

Use `cvt_color(&mut img, CVTColor::…)` to:

- Convert between RGB, Grayscale (BT.601/709/2020), YCbCr, CMYK.
- Swap channels (RGB ↔ BGR).
- Expand Gray → RGB.

### Halftone & Screentone

- **Halftone** dot screening with configurable dot sizes per channel.
- **Rotate** capability for angled halftones.
- **Screentone** for single-channel comic-style screening, with optional rotation.

### Color Levels (Experimental)

The `color_levels` API is not yet fully implemented. Stay tuned for fine-grained black/white point and gamma adjustments.
