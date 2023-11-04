use enum_map::Enum;

pub fn enum_iter<E>() -> Vec<E>
where
    E: Enum,
{
    // not actually an iterator. i'm not sure how to make it one
    (0..E::LENGTH).map(|i| E::from_usize(i)).collect()
}
