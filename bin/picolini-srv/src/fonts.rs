use std::fs;

use skia_safe::{
    Font, FontArguments, FontMgr, FourByteTag,
    font::Edging,
    font_arguments::{VariationPosition, variation_position::Coordinate},
};

pub struct Fonts {
    pub big_bold: Font,
    pub small: Font,
    pub medium: Font,
}

impl Fonts {
    pub fn new(path: &str) -> Self {
        let font_mgr = FontMgr::new();
        let typeface = font_mgr
            .new_from_data(&fs::read(path).unwrap(), None)
            .unwrap();
        let w400 = typeface
            .clone_with_arguments(&FontArguments::new().set_variation_design_position(
                VariationPosition {
                    coordinates: &[Coordinate {
                        axis: FourByteTag::from_chars('w', 'g', 'h', 't'),
                        value: 400.0,
                    }],
                },
            ))
            .unwrap();
        let w700 = typeface
            .clone_with_arguments(&FontArguments::new().set_variation_design_position(
                VariationPosition {
                    coordinates: &[Coordinate {
                        axis: FourByteTag::from_chars('w', 'g', 'h', 't'),
                        value: 700.0,
                    }],
                },
            ))
            .unwrap();

        let mut f = Self {
            big_bold: Font::from_typeface(w700, 27.0),
            small: Font::from_typeface(w400.clone(), 13.0),
            medium: Font::from_typeface(w400, 16.0),
        };
        f.big_bold.set_edging(Edging::Alias);
        f.small.set_edging(Edging::Alias);
        f.medium.set_edging(Edging::Alias);
        f
    }
}
