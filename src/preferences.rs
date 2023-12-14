use crate::puzzle::cube::CubeRay;
use crate::puzzle::octa::OctaRay;
use crate::util::color;
use crate::util::color::Color;
use enum_map::enum_map;
use enum_map::EnumMap;
use serde::Deserialize;
use serde::Serialize;

fn color_cube_default() -> EnumMap<CubeRay, Color> {
    use crate::puzzle::cube::*;

    enum_map! {
        CubeRay(Basis::Y, Sign::Pos) => color::ORANGE,
        CubeRay(Basis::Z, Sign::Pos) => color::WHITE,
        CubeRay(Basis::X, Sign::Pos) => color::BLUE,
        CubeRay(Basis::Z, Sign::Neg) => color::YELLOW,
        CubeRay(Basis::X, Sign::Neg) => color::GREEN,
        CubeRay(Basis::Y, Sign::Neg) => color::RED,
    }
}

fn color_octa_default() -> EnumMap<OctaRay, Color> {
    use crate::puzzle::cube::*;

    enum_map! {
        OctaRay(Sign::Pos, Sign::Neg, Sign::Pos) => color::WHITE,
        OctaRay(Sign::Pos, Sign::Neg, Sign::Neg) => color::GREEN,
        OctaRay(Sign::Neg, Sign::Neg, Sign::Pos) => color::RED,
        OctaRay(Sign::Neg, Sign::Neg, Sign::Neg) => color::DARK_GREEN,
        OctaRay(Sign::Pos, Sign::Pos, Sign::Pos) => color::BLUE,
        OctaRay(Sign::Pos, Sign::Pos, Sign::Neg) => color::ORANGE,
        OctaRay(Sign::Neg, Sign::Pos, Sign::Pos) => color::PURPLE,
        OctaRay(Sign::Neg, Sign::Pos, Sign::Neg) => color::YELLOW,
    }
}

#[derive(Serialize, Deserialize)]
pub struct ColorPreferences {
    #[serde(default = "color_cube_default")]
    #[serde(with = "crate::util::enum_map_serde")]
    cube_: EnumMap<CubeRay, Color>,
    #[serde(default = "color_octa_default")]
    #[serde(with = "crate::util::enum_map_serde")]
    octa_: EnumMap<OctaRay, Color>,
}

#[derive(Serialize, Deserialize)]
pub struct Preferences {
    pub colors: ColorPreferences,
}
