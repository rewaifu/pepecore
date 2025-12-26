use crate::enums::PaletteAlg;
use crate::ops::svec_ops::palette_gen::median_cut::{collect_colors, median_cut};
use crate::ops::svec_ops::palette_gen::min_max_uniform::min_max_uniform;
use crate::ops::svec_ops::palette_gen::octree_si::OctreeQuantizer;
use crate::ops::svec_ops::palette_gen::wu_color::{MAXCOLOR, WuQuantizer};
use pepecore_array::SVec;
pub fn svec_to_palette(img: &mut SVec, num_color: usize, p_a: PaletteAlg) -> Vec<f32> {
    img.as_u8();
    let data = img.get_data::<u8>().unwrap();
    match p_a {
        PaletteAlg::OcTree => {
            let mut otq = OctreeQuantizer::new();
            for c in data.chunks(3) {
                otq.add_color(c);
            }
            otq.make_palette(num_color)
        }
        PaletteAlg::MedianCut => {
            let colors = collect_colors(data);
            median_cut(colors, num_color)
        }
        PaletteAlg::Wu => {
            if num_color > MAXCOLOR {
                let mut otq = OctreeQuantizer::new();
                for c in data.chunks(3) {
                    otq.add_color(c);
                }
                return otq.make_palette(num_color);
            }
            let mut quantizer = WuQuantizer::new(data, num_color).unwrap();
            quantizer.quantize().unwrap();
            quantizer.get_palette()
        }
        PaletteAlg::MinMaxUniform => min_max_uniform(data, num_color),
    }
}
