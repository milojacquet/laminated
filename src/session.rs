use crate::puzzle::common::*;
use crate::puzzle::cube::CubeRay;
use crate::puzzle::octa::OctaRay;
use crate::render;
use crate::render::common::*;
use crate::render::create::make_concrete_puzzle;
use crate::util::Vec3;
use crate::Preferences;
use crate::VERSION;
use enum_map::EnumMap;
use eyre::eyre;

pub struct Session<Ray: ConcreteRaySystem> {
    pub scramble: Vec<EnumMap<Ray, Ray>>,
    pub concrete_puzzle: ConcretePuzzle<Ray>,
    pub twists: Vec<((Ray, i8), Vec<Vec<i8>>)>,
    pub undid_twists: Vec<((Ray, i8), Vec<Vec<i8>>)>,
    // None: the mouse is not pressed.
    // Some((conj, None)): the mouse is being held from a viewport with conjugation conj, and camera orbiting has started.
    // Some((conj, Some((loc, button)))): the mouse is being held from a viewport with conjugation conj, and camera orbiting has not yet started. the mouse was pressed at loc with button.
    pub mouse_press_location: Option<(
        Ray::Conjugate,
        Option<(three_d::LogicalPoint, three_d::MouseButton)>,
    )>,
    pub save_path: Option<std::path::PathBuf>,
    pub version: String,
    pub camera_facings: EnumMap<Ray::Conjugate, CameraFacing>,
}

fn string_vec_to_enum_map<Ray: ConcreteRaySystem + enum_map::Enum>(
    strs: Vec<String>,
) -> eyre::Result<EnumMap<Ray, Ray>> {
    if strs.len() != Ray::LENGTH {
        return Err(eyre!("Invalid enum length"));
    }
    let rays: Vec<Ray> = strs
        .into_iter()
        .map(|st| Ray::from_name(&st).ok_or_else(|| eyre!("Invalid ray name")))
        .collect::<eyre::Result<_>>()?;
    let map = EnumMap::from_fn(|ray| rays[Ray::into_usize(ray)]);
    Ok(map)
}

impl<'a, Ray: ConcreteRaySystem> Session<Ray> {
    pub fn from_concrete(concrete_puzzle: ConcretePuzzle<Ray>) -> Session<Ray> {
        Session {
            scramble: concrete_puzzle.puzzle.orientations(),
            concrete_puzzle,
            twists: vec![],
            undid_twists: vec![],
            mouse_press_location: None,
            save_path: None,
            version: VERSION.to_string(),
            camera_facings: EnumMap::from_fn(|_| CameraFacing {
                position: Vec3::new(5.0, -10.0, 4.0),
                target: Vec3::new(0.0, 0.0, 0.0),
                up: Vec3::new(0.0, 0.0, 1.0),
            }),
        }
    }

    fn multi_layer_twist(&mut self, tw: (Ray, i8), grips: &Vec<Vec<i8>>) {
        for grip in grips {
            self.concrete_puzzle.twist(tw, &grip[..]);
        }
    }

    pub fn twist(&mut self, tw: (Ray, i8), grips: Vec<Vec<i8>>) {
        self.multi_layer_twist(tw, &grips);
        self.twists.push((tw, grips));
        self.undid_twists = vec![];
    }

    fn scramble_from_concrete(&mut self) {
        self.concrete_puzzle.reset_animations();
        self.scramble = self.concrete_puzzle.puzzle.orientations();
        self.twists = vec![];
        self.undid_twists = vec![];
    }

    pub fn scramble(&mut self) {
        self.concrete_puzzle.puzzle.scramble();
        self.scramble_from_concrete();
    }

    pub fn reset(&mut self) {
        let new_puzzle = Puzzle::make_solved(self.concrete_puzzle.puzzle.grips.clone());
        self.concrete_puzzle.puzzle = new_puzzle;
        self.scramble_from_concrete();
    }

    pub fn undo(&mut self) -> eyre::Result<()> {
        if let Some(((ray, order), grips)) = self.twists.pop() {
            self.undid_twists.push(((ray, order), grips.clone()));
            // we want the animation this time
            self.multi_layer_twist((ray, -order), &grips);
            Ok(())
        } else {
            // no undo left
            Err(eyre!("No undo left"))
        }
    }

    pub fn redo(&mut self) -> eyre::Result<()> {
        if let Some(((ray, order), grips)) = self.undid_twists.pop() {
            self.twists.push(((ray, order), grips.clone()));
            // we want the animation this time
            self.multi_layer_twist((ray, order), &grips);
            Ok(())
        } else {
            // no redo left
            Err(eyre!("No redo left"))
        }
    }

    pub fn do_inverse(&mut self) -> eyre::Result<()> {
        if let Some(((ray, order), grips)) = self.twists.pop() {
            self.twists.push(((ray, -order), grips.clone()));
            self.undid_twists = vec![];
            // we want the animation this time
            self.multi_layer_twist((ray, -order), &grips);
            self.multi_layer_twist((ray, -order), &grips); // do it again
            Ok(())
        } else {
            // no undo left
            Err(eyre!("No undo left"))
        }
    }

    fn set_orientations(&mut self, oris: Vec<Vec<String>>) -> eyre::Result<()> {
        let scramble = oris
            .into_iter()
            .map(|ori| string_vec_to_enum_map(ori))
            .collect::<Result<_, _>>()?;
        self.scramble = scramble;
        self.apply_scramble();
        Ok(())
    }

    fn apply_scramble(&mut self) {
        self.concrete_puzzle.puzzle.set_orientations(&self.scramble);
    }

    fn apply_twists(&mut self) {
        // even though multi_layer_twist only mutates concrete_puzzle,
        // i need to clone twists so i don't double borrow
        for (ray_order, grips) in self.twists.clone() {
            self.multi_layer_twist(ray_order, &grips)
        }
        self.concrete_puzzle.reset_animations();
    }

    fn extract_log(&self) -> (Vec<Vec<String>>, Vec<((String, i8), Vec<Vec<i8>>)>) {
        let scramble_str = self
            .scramble
            .iter()
            .map(|ori| ori.values().map(|ray| ray.name()).collect())
            .collect();

        let twists_str = self
            .twists
            .iter()
            .map(|((ray, order), grips)| ((ray.name(), *order), grips.clone()))
            .collect();

        (scramble_str, twists_str)
    }

    fn process_log(&mut self, log: SessionLog) -> eyre::Result<()> {
        self.version = log.version;
        let suffix = if &self.version == VERSION {
            "".to_string()
        } else {
            format!(" (loading from version {})", self.version)
        };

        self.set_orientations(log.scramble)
            .map_err(|err| eyre!(err.to_string() + &suffix))?;

        for ((st, order), grips) in log.twists {
            self.twist(
                (
                    Ray::from_name(&st).ok_or_else(|| eyre!("Invalid ray name{suffix}"))?,
                    order,
                ),
                grips,
            )
        }

        self.concrete_puzzle.reset_animations();

        Ok(())
    }

    /// Replace the concrete puzzle with a new one.
    /// Only use concrete puzzles which have the same underlying puzzle!
    /// This is not checked!
    pub fn replace_concrete_puzzle(&mut self, new_concrete_puzzle: ConcretePuzzle<Ray>) {
        // this could probably be done better by only replacing self.concrete_puzzle.viewports,
        // but this is easier
        self.concrete_puzzle = new_concrete_puzzle;
        self.apply_scramble();
        self.apply_twists();
    }
}

#[derive(Debug, Copy, Clone, serde::Serialize, serde::Deserialize)]
pub enum CubePuzzle {
    Nnn(i8),
}

#[derive(Debug, Copy, Clone, serde::Serialize, serde::Deserialize)]
pub enum OctaPuzzle {
    //Core,
    Fto(i8),
}

#[derive(Debug, Copy, Clone, serde::Serialize, serde::Deserialize)]
pub enum SessionType {
    Cube(CubePuzzle),
    Octa(OctaPuzzle),
}

pub enum SessionEnum {
    Cube(CubePuzzle, Session<CubeRay>),
    Octa(OctaPuzzle, Session<OctaRay>),
}

impl SessionType {
    pub fn make_session_enum(
        self,
        window_size: (u32, u32),
        context: &three_d::Context,
        prefs: &Preferences,
    ) -> SessionEnum {
        match self {
            SessionType::Cube(ps @ CubePuzzle::Nnn(n)) => SessionEnum::Cube(
                ps,
                Session::from_concrete(make_concrete_puzzle(
                    window_size,
                    &context,
                    render::cube::nnn_seeds(n, &prefs.concrete),
                    prefs,
                )),
            ),
            /*SessionType::Octa(ps @ OctaPuzzle::Core) => SessionEnum::Octa(
                ps,
                Session::from_concrete(make_concrete_puzzle(
                    window_size,
                    &context,
                    render::octa::core_seeds(),
                    prefs,
                )),
            ),*/
            SessionType::Octa(ps @ OctaPuzzle::Fto(n)) => SessionEnum::Octa(
                ps,
                Session::from_concrete(make_concrete_puzzle(
                    window_size,
                    &context,
                    render::octa::fto_seeds(n, &prefs.concrete),
                    prefs,
                )),
            ),
        }
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct SessionLog {
    pub version: String,
    pub session_type: SessionType,
    pub scramble: Vec<Vec<String>>,
    pub twists: Vec<((String, i8), Vec<Vec<i8>>)>,
}

impl SessionEnum {
    pub fn get_type(&self) -> SessionType {
        match self {
            Self::Cube(pz, _) => SessionType::Cube(*pz),
            Self::Octa(pz, _) => SessionType::Octa(*pz),
        }
    }

    pub fn save_path<'a>(&'a self) -> &'a Option<std::path::PathBuf> {
        match self {
            SessionEnum::Cube(_, ref session) => &session.save_path,
            SessionEnum::Octa(_, ref session) => &session.save_path,
        }
    }

    pub fn set_save_path(&mut self, val: Option<std::path::PathBuf>) {
        match self {
            SessionEnum::Cube(_, ref mut session) => session.save_path = val,
            SessionEnum::Octa(_, ref mut session) => session.save_path = val,
        };
    }

    pub fn version<'a>(&'a self) -> &'a String {
        match self {
            SessionEnum::Cube(_, ref session) => &session.version,
            SessionEnum::Octa(_, ref session) => &session.version,
        }
    }

    pub fn to_log(&self) -> SessionLog {
        let (scramble, twists) = match self {
            Self::Cube(_, session) => session.extract_log(),
            Self::Octa(_, session) => session.extract_log(),
        };

        SessionLog {
            version: VERSION.to_string(),
            session_type: self.get_type(),
            scramble,
            twists,
        }
    }

    pub fn save_as(&mut self, path: &std::path::PathBuf) -> eyre::Result<std::path::PathBuf> {
        std::fs::write(path, serde_json::to_string(&self.to_log())?)?;
        self.set_save_path(Some(path.clone()));
        Ok(path.clone())
    }

    fn from_log(
        log: SessionLog,
        window_size: (u32, u32),
        context: &three_d::Context,
        path: std::path::PathBuf,
        prefs: &Preferences,
    ) -> eyre::Result<Self> {
        let mut session = log
            .session_type
            .make_session_enum(window_size, context, prefs);
        match &mut session {
            SessionEnum::Cube(_, ref mut session) => session.process_log(log),
            SessionEnum::Octa(_, ref mut session) => session.process_log(log),
        }?;
        session.set_save_path(Some(path));

        Ok(session)
    }

    pub fn load(
        path: std::path::PathBuf,
        window_size: (u32, u32),
        context: &three_d::Context,
        prefs: &Preferences,
    ) -> eyre::Result<Self> {
        let file = std::fs::File::open(path.clone())?;
        let reader = std::io::BufReader::new(file);
        let session_log: SessionLog = serde_json::from_reader(reader)?;
        Self::from_log(session_log, window_size, context, path, prefs)
    }

    pub fn replace_concrete_puzzle_from(&mut self, other: SessionEnum) {
        // not using _ => () so when i add a new SessionEnum, it errors and i have to add it here
        match (self, other) {
            (SessionEnum::Cube(_, session), SessionEnum::Cube(_, other_s)) => {
                session.replace_concrete_puzzle(other_s.concrete_puzzle)
            }
            (SessionEnum::Cube(_, _), _) => {}
            (SessionEnum::Octa(_, session), SessionEnum::Octa(_, other_s)) => {
                session.replace_concrete_puzzle(other_s.concrete_puzzle)
            }
            (SessionEnum::Octa(_, _), _) => {}
        }
    }
}
