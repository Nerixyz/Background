use skia_safe::{
    FontMgr, Rect,
    resources::{LocalResourceProvider, NativeResourceProvider},
    svg::{Dom, Length, LengthUnit},
};
use std::{collections::HashMap, fs, marker::PhantomData, path::PathBuf};

use crate::{
    dwd::{Datapoint, icons::IconSet},
    paint::SvgItem,
};

pub struct IconRenderer<I> {
    cache: HashMap<PathBuf, Option<Dom>>,
    nrp: NativeResourceProvider,
    _phantom: PhantomData<I>,
}

impl<I> IconRenderer<I> {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
            nrp: LocalResourceProvider::new(FontMgr::new()).into(),
            _phantom: PhantomData,
        }
    }
}

impl<I: IconSet> IconRenderer<I> {
    pub fn layout(&mut self, d: &Datapoint, rect: Rect) -> Option<SvgItem> {
        let path = d.icon::<I>()?;
        let dom = self
            .cache
            .entry(path)
            .or_insert_with_key(|path| {
                Dom::read(
                    fs::OpenOptions::new().read(true).open(path).ok()?,
                    self.nrp.clone(),
                )
                .ok()
            })
            .clone()?;
        let mut root = dom.root();
        // This isn't ideal. If the aspect ratio isn't 1, then the SVG will show stuff from outside the viewbox.
        root.set_width(Length::new(100.0, LengthUnit::Percentage));
        root.set_height(Length::new(100.0, LengthUnit::Percentage));
        Some(SvgItem { dom, rect })
    }
}
