use enum_map::Enum;
use enum_map::EnumArray;
use enum_map::EnumMap;

pub type Vec3 = cgmath::Vector3<f32>;
pub type Mat3 = cgmath::Matrix3<f32>;
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

pub mod enum_map_serde {
    use enum_map::EnumArray;
    use enum_map::EnumMap;
    use serde::ser::SerializeTuple;
    use serde::Deserialize;
    use serde::Deserializer;
    use serde::Serialize;
    use serde::Serializer;

    pub fn serialize<S, K, V>(map: &EnumMap<K, V>, ser: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        K: EnumArray<V>,
        V: Serialize,
    {
        let mut tup = ser.serialize_tuple(K::LENGTH)?;
        for el in map.values() {
            tup.serialize_element(el)?;
        }
        tup.end()
    }

    pub fn deserialize<'de, D, K, V>(de: D) -> Result<EnumMap<K, V>, D::Error>
    where
        D: Deserializer<'de>,
        K: enum_map::EnumArray<V>,
        V: serde::Deserialize<'de> + Clone,
    {
        //let arr: [V; K::LENGTH] = Deserialize::deserialize(de)?;
        let arr: Vec<V> = Deserialize::deserialize(de)?;

        /*arr.try_into()
        .map_err(|_e| D::Error::custom("bad enum map"))
        .map(EnumMap::from_array)*/
        Ok(EnumMap::from_fn(|i| arr[K::into_usize(i)].clone()))
    }
}

pub mod color {
    use serde::Deserialize;
    use serde::Serialize;
    use three_d::Srgba;

    #[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq)]
    pub struct Color {
        pub r: u8,
        pub g: u8,
        pub b: u8,
    }

    impl Color {
        pub const fn hex(color: u32) -> Color {
            Color {
                r: (color >> 16) as u8,
                g: (color >> 8) as u8,
                b: color as u8,
            }
        }

        pub fn to_srgba(&self) -> Srgba {
            Srgba::new_opaque(self.r, self.g, self.b)
        }

        pub fn as_array(&self) -> [u8; 3] {
            [self.r, self.g, self.b]
        }

        pub fn as_mut_array(&mut self) -> [&mut u8; 3] {
            [&mut self.r, &mut self.g, &mut self.b]
        }
    }

    impl From<[u8; 3]> for Color {
        fn from(val: [u8; 3]) -> Color {
            Color {
                r: val[0],
                g: val[1],
                b: val[2],
            }
        }
    }

    pub const WHITE: Color = Color::hex(0xffffff);
    pub const GREEN: Color = Color::hex(0x1eef1e);
    pub const RED: Color = Color::hex(0xed1b1b);
    pub const BLUE: Color = Color::hex(0x387eff);
    pub const ORANGE: Color = Color::hex(0xff821c);
    pub const YELLOW: Color = Color::hex(0xffd414);
    pub const PURPLE: Color = Color::hex(0x663399);
    pub const DARK_GREEN: Color = Color::hex(0x1a891a);
    pub const BROWN: Color = Color::hex(0x884d0f);
    pub const GRAY: Color = Color::hex(0x808080);
    pub const PINK: Color = Color::hex(0xff8fff);
    pub const CYAN: Color = Color::hex(0x1ec4ff);
}
