use super::Value;
use core::fmt::{self, Debug, Write};

use super::Vec;

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct ValueEntry {
    key: Value,
    value: Value,
}

impl Debug for ValueEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}:{:?}", self.key, self.value)
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct ValueMap(Vec<ValueEntry>);

impl Debug for ValueMap {
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
