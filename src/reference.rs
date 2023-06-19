use std::fmt::{Display, Formatter};
use std::ops::Neg;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Ref(pub(crate) u32);

impl Ref {
    pub const fn new(id: u32, negated: bool) -> Self {
        Self((id << 1) + negated as u32)
    }
    pub const fn positive(id: u32) -> Self {
        Self::new(id, false)
    }
    pub const fn negative(id: u32) -> Self {
        Self::new(id, true)
    }
    pub const fn from_i32(value: i32) -> Self {
        Self::new(value.unsigned_abs(), value < 0)
    }
    pub const fn from_u32(value: u32) -> Self {
        Self(value)
    }

    pub const fn id(self) -> u32 {
        self.0 >> 1
    }

    pub const fn is_negated(self) -> bool {
        self.0 & 1 == 1
    }

    pub const fn negate(self) -> Self {
        Self(self.0 ^ 1)
    }

    pub const fn get(self) -> i32 {
        let id = self.id() as i32;
        if self.is_negated() {
            -id
        } else {
            id
        }
    }
}

impl Neg for Ref {
    type Output = Self;

    fn neg(self) -> Self::Output {
        self.negate()
    }
}

impl Display for Ref {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}@{}",
            if self.is_negated() { "~" } else { "" },
            self.id()
        )
    }
}
