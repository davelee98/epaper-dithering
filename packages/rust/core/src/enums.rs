use crate::algorithms::{
    Kernel, ATKINSON, BURKES, FLOYD_STEINBERG, JARVIS_JUDICE_NINKE, SIERRA, SIERRA_LITE, STUCKI,
};

/// Dithering algorithm. Integer values match OpenDisplay firmware conventions.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DitherMode {
    None           = 0,
    Ordered        = 1,
    FloydSteinberg = 2,
    #[default]
    Burkes         = 3,
    Atkinson       = 4,
    Stucki         = 5,
    Sierra         = 6,
    SierraLite     = 7,
    JarvisJudiceNinke = 8,
}

impl DitherMode {
    pub fn kernel(self) -> Option<&'static Kernel> {
        match self {
            DitherMode::None | DitherMode::Ordered => None,
            DitherMode::FloydSteinberg => Some(&FLOYD_STEINBERG),
            DitherMode::Burkes => Some(&BURKES),
            DitherMode::Atkinson => Some(&ATKINSON),
            DitherMode::Stucki => Some(&STUCKI),
            DitherMode::Sierra => Some(&SIERRA),
            DitherMode::SierraLite => Some(&SIERRA_LITE),
            DitherMode::JarvisJudiceNinke => Some(&JARVIS_JUDICE_NINKE),
        }
    }
}
