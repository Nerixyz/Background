use skia_safe::{
    Font, FontStyle, FontStyleSet,
    font_style::{Slant, Weight, Width},
};

pub struct Fonts {
    pub small: Font,
    pub medium: Font,
    pub medium_light: Font,
    pub large: Font,
}

impl Fonts {
    pub fn new(style_set: &mut FontStyleSet) -> Self {
        let small = query_font(style_set, Weight::THIN, 10.0);
        let medium = query_font(style_set, Weight::NORMAL, 16.0);
        let medium_light = query_font(style_set, Weight::LIGHT, 14.0);
        let large = query_font(style_set, Weight::BOLD, 28.0);

        Self {
            small,
            medium,
            medium_light,
            large,
        }
    }
}

fn query_font(style_set: &mut FontStyleSet, weight: Weight, size: f32) -> Font {
    let tf = style_set
        .match_style(FontStyle::new(weight, Width::NORMAL, Slant::Upright))
        .unwrap();
    let mut font = Font::from_typeface(tf, size);
    font.set_edging(skia_safe::font::Edging::SubpixelAntiAlias);
    font
}
