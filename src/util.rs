use enum_map::Enum;
use enum_map::EnumArray;
use enum_map::EnumMap;

pub fn enum_iter<E>() -> Vec<E>
where
    E: Enum,
{
    // not actually an iterator. i'm not sure how to make it one
    (0..E::LENGTH).map(|i| E::from_usize(i)).collect()
}

pub fn enum_index<E>(e: E) -> usize
where
    E: Enum + PartialEq,
{
    enum_iter::<E>()
        .iter()
        .position(|ee| ee == &e)
        .expect("enum elements should always be in enum_iter")
}

// ad hoc clone
pub fn enum_map_clone<K: EnumArray<V>, V: Clone>(enum_map: &EnumMap<K, V>) -> EnumMap<K, V> {
    EnumMap::from_fn(|key: K| enum_map[key].clone())
}
