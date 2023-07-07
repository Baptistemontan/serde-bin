use super::{size_hint_caution, Value};
use core::fmt::{self, Debug, Write};

use super::Vec;

#[derive(Clone, PartialEq)]
pub struct ValueEntry<'de> {
    key: Value<'de>,
    value: Value<'de>,
}

impl<'de> Debug for ValueEntry<'de> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}:{:?}", self.key, self.value)
    }
}

#[derive(Clone, PartialEq)]
pub struct ValueMap<'de>(Vec<ValueEntry<'de>>);

impl<'de> Debug for ValueMap<'de> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_char('{')?;
        let len = self.0.len();
        for (i, entry) in self.0.iter().enumerate() {
            Debug::fmt(entry, f)?;
            if i < len - 1 {
                f.write_char(',')?;
            }
        }
        f.write_char('}')
    }
}

impl<'de> ValueMap<'de> {
    pub(crate) fn from_map_access<A>(mut map: A) -> Result<Self, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        let mut buff = Vec::with_capacity(size_hint_caution(map.size_hint()));
        while let Some((key, value)) = map.next_entry()? {
            buff.push(ValueEntry { key, value })
        }
        buff.shrink_to_fit();
        Ok(Self(buff))
    }
}
