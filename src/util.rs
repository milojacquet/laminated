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
