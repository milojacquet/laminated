use crate::puzzle::cube::CubeRay;
use crate::puzzle::dodeca::DodecaRay;
use crate::puzzle::octa::OctaRay;
use crate::puzzle::r_dodeca::RDodecaRay;
use crate::render::common::ConcreteRaySystem;
use crate::util::color;
use crate::util::color::Color;
use enum_map::enum_map;
use enum_map::EnumMap;
use serde::Deserialize;
use serde::Serialize;

const PREFS_PATH: &'static str = "./preferences.json";

#[derive(Serialize, Deserialize)]
pub struct ColorPreferences {
    #[serde(default = "CubeRay::default_colors")]
    #[serde(with = "crate::util::enum_map_serde")]
    pub cube: EnumMap<CubeRay, Color>,
    #[serde(default = "OctaRay::default_colors")]
    #[serde(with = "crate::util::enum_map_serde")]
    pub octa: EnumMap<OctaRay, Color>,
    #[serde(default = "DodecaRay::default_colors")]
    #[serde(with = "crate::util::enum_map_serde")]
    pub dodeca: EnumMap<DodecaRay, Color>,
    #[serde(default = "RDodecaRay::default_colors")]
    #[serde(with = "crate::util::enum_map_serde")]
    pub r_dodeca: EnumMap<RDodecaRay, Color>,
}

impl Default for ColorPreferences {
    fn default() -> Self {
        Self {
            cube: CubeRay::default_colors(),
            octa: OctaRay::default_colors(),
            dodeca: DodecaRay::default_colors(),
            r_dodeca: RDodecaRay::default_colors(),
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
