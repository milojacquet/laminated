use enum_map::Enum;
use enum_map::EnumArray;
use enum_map::EnumMap;

pub type Vec3 = cgmath::Vector3<f32>;
pub type Mat4 = cgmath::Matrix4<f32>;

pub fn enum_iter<E>() -> impl Iterator<Item = E>
where
    E: Enum,
{
    (0..E::LENGTH).map(|i| E::from_usize(i))
}

// ad hoc clone
pub fn enum_map_clone<K: EnumArray<V>, V: Clone>(enum_map: &EnumMap<K, V>) -> EnumMap<K, V> {
    EnumMap::from_fn(|key: K| enum_map[key].clone())
}

pub mod color {
    use three_d::Srgba;

    pub type Color = Srgba;

    const fn hex(color: u32) -> Color {
        Srgba::new_opaque((color >> 16) as u8, (color >> 8) as u8, color as u8)
    }

    pub const WHITE: Color = hex(0xffffff);
    pub const GREEN: Color = hex(0x1eef1e);
    pub const RED: Color = hex(0xed1b1b);
    pub const DARK_GREEN: Color = hex(0x1a891a);
    pub const BLUE: Color = hex(0x387eff);
    pub const ORANGE: Color = hex(0xff821c);
    pub const PURPLE: Color = hex(0x663399);
    pub const YELLOW: Color = hex(0xffd414);
}
