use crate::make_concrete_puzzle;
use crate::puzzle::common::*;
use crate::render;
use crate::render::common::*;
use crate::CubeRay;
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
        self.concrete_puzzle.puzzle.set_orientations(
            oris.into_iter()
                .map(|ori| string_vec_to_enum_map(ori))
                .collect::<Result<_, _>>()?,
        );
        Ok(())
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
}

#[derive(Debug, Copy, Clone, serde::Serialize, serde::Deserialize)]
pub enum CubePuzzle {
    Nnn(i8),
}

#[derive(Debug, Copy, Clone, serde::Serialize, serde::Deserialize)]
pub enum SessionType {
    Cube(CubePuzzle),
}

pub enum SessionEnum {
    Cube(CubePuzzle, Session<CubeRay>),
}

impl SessionType {
    pub fn make_session_enum(
        self,
        window_size: (u32, u32),
        context: &three_d::Context,
    ) -> SessionEnum {
        match self {
            SessionType::Cube(ps @ CubePuzzle::Nnn(n)) => SessionEnum::Cube(
                ps,
                Session::from_concrete(make_concrete_puzzle(
                    window_size,
                    &context,
                    render::cube::nnn_seeds(n),
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
        }
    }

    pub fn save_path<'a>(&'a self) -> &'a Option<std::path::PathBuf> {
        match self {
            SessionEnum::Cube(_, ref session) => &session.save_path,
        }
    }

    pub fn set_save_path(&mut self, val: Option<std::path::PathBuf>) {
        match self {
            SessionEnum::Cube(_, ref mut session) => session.save_path = val,
        };
    }

    pub fn version<'a>(&'a self) -> &'a String {
        match self {
            SessionEnum::Cube(_, ref session) => &session.version,
        }
    }

    pub fn to_log(&self) -> SessionLog {
        let (scramble, twists) = match self {
            Self::Cube(_, session) => session.extract_log(),
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
    ) -> eyre::Result<Self> {
        let mut session = log.session_type.make_session_enum(window_size, context);
        match &mut session {
            SessionEnum::Cube(_, ref mut session) => session.process_log(log),
        }?;
        session.set_save_path(Some(path));

        Ok(session)
    }

    pub fn load(
        path: std::path::PathBuf,
        window_size: (u32, u32),
        context: &three_d::Context,
    ) -> eyre::Result<Self> {
        let file = std::fs::File::open(path.clone())?;
        let reader = std::io::BufReader::new(file);
        let session_log: SessionLog = serde_json::from_reader(reader)?;
        Self::from_log(session_log, window_size, context, path)
    }
}
