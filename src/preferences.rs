use crate::puzzle::cube::CubeRay;
use crate::puzzle::dodeca::DodecaRay;
use crate::puzzle::octa::OctaRay;
use crate::util::color;
use crate::util::color::Color;
use enum_map::enum_map;
use enum_map::EnumMap;
use serde::Deserialize;
use serde::Serialize;

const PREFS_PATH: &'static str = "./preferences.json";

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
    use crate::puzzle::octa::*;

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

fn color_dodeca_default() -> EnumMap<DodecaRay, Color> {
    use crate::puzzle::dodeca::*;

    enum_map! {
        DodecaRay(Basis::X, Sign::Pos, Sign::Pos) => color::ORANGE, // "PB",
        DodecaRay(Basis::X, Sign::Pos, Sign::Neg) => color::GRAY, // "PD",
        DodecaRay(Basis::X, Sign::Neg, Sign::Pos) => color::WHITE, // "U",
        DodecaRay(Basis::X, Sign::Neg, Sign::Neg) => color::RED, // "F",
        DodecaRay(Basis::Y, Sign::Pos, Sign::Pos) => color::BROWN, // "BL",
        DodecaRay(Basis::Y, Sign::Pos, Sign::Neg) => color::PURPLE, // "BR",
        DodecaRay(Basis::Y, Sign::Neg, Sign::Pos) => color::PINK, // "DR",
        DodecaRay(Basis::Y, Sign::Neg, Sign::Neg) => color::YELLOW, // "DL",
        DodecaRay(Basis::Z, Sign::Pos, Sign::Pos) => color::GREEN, // "PR",
        DodecaRay(Basis::Z, Sign::Pos, Sign::Neg) => color::BLUE, // "R",
        DodecaRay(Basis::Z, Sign::Neg, Sign::Pos) => color::CYAN, // "PL",
        DodecaRay(Basis::Z, Sign::Neg, Sign::Neg) => color::DARK_GREEN, // "L",
    }
}

#[derive(Serialize, Deserialize)]
pub struct ColorPreferences {
    #[serde(default = "color_cube_default")]
    #[serde(with = "crate::util::enum_map_serde")]
    pub cube: EnumMap<CubeRay, Color>,
    #[serde(default = "color_octa_default")]
    #[serde(with = "crate::util::enum_map_serde")]
    pub octa: EnumMap<OctaRay, Color>,
    #[serde(default = "color_dodeca_default")]
    #[serde(with = "crate::util::enum_map_serde")]
    pub dodeca: EnumMap<DodecaRay, Color>,
}

impl Default for ColorPreferences {
    fn default() -> Self {
        Self {
            cube: color_cube_default(),
            octa: color_octa_default(),
            dodeca: color_dodeca_default(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct ConcretePuzzlePreferences {
    pub octa_extend: bool,
}

impl Default for ConcretePuzzlePreferences {
    fn default() -> Self {
        Self { octa_extend: true }
    }
}

fn default_animation_length() -> f32 {
    150.0
}

#[derive(Serialize, Deserialize)]
pub struct Preferences {
    #[serde(default)]
    pub colors: ColorPreferences,
    #[serde(default)]
    pub viewport_keys: bool,
    #[serde(default)]
    pub concrete: ConcretePuzzlePreferences,
    #[serde(default = "default_animation_length")]
    pub animation_length: f32,
}

impl Default for Preferences {
    fn default() -> Self {
        Preferences {
            colors: Default::default(),
            viewport_keys: false,
            concrete: Default::default(),
            animation_length: 150.0,
        }
    }
}

impl Preferences {
    pub fn save(&self) -> eyre::Result<()> {
        std::fs::write(PREFS_PATH, serde_json::to_string(self)?)?;
        Ok(())
    }

    pub fn load() -> eyre::Result<Self> {
        let path = std::path::PathBuf::from(PREFS_PATH);

        let file = if path.exists() {
            std::fs::File::open(path)?
        } else {
            return Ok(Default::default());
        };
        let reader = std::io::BufReader::new(file);
        Ok(serde_json::from_reader(reader)?)
    }
}
