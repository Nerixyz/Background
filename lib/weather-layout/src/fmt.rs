pub fn optional_fract(x: f32) -> String {
    format!("{x:.*}", if x.fract() == 0.0 { 0 } else { 1 })
}
