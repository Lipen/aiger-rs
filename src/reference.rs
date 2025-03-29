use std::fmt::{Display, Formatter};
use std::ops::Neg;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Ref(u32);

impl Ref {
    pub const FALSE: Self = Self(0);
    pub const TRUE: Self = Self(1);

    pub const fn new(id: u32, negated: bool) -> Self {
        debug_assert!(
            id != 0,
            "Ref id must be non-zero. Use Ref::FALSE or Ref::TRUE instead."
        );
        Self((id << 1) + negated as u32)
    }
    pub const fn positive(id: u32) -> Self {
        Self::new(id, false)
    }
    pub const fn negative(id: u32) -> Self {
        Self::new(id, true)
    }

    pub const fn from_raw(raw: u32) -> Self {
        Self(raw)
    }

    pub const fn raw(self) -> u32 {
        self.0
    }
    pub const fn id(self) -> u32 {
        self.0 >> 1
    }
    pub const fn is_negated(self) -> bool {
        self.0 & 1 != 0
    }
    pub const fn get(self) -> i32 {
        let id = self.id() as i32;
        if self.is_negated() {
            -id
        } else {
            id
        }
    }

    pub const fn is_const(self) -> bool {
        self.id() == 0
    }
    pub const fn is_false(self) -> bool {
        self.0 == 0
    }
    pub const fn is_true(self) -> bool {
        self.0 == 1
    }
    pub const fn get_const(self) -> Option<bool> {
        if self.is_false() {
            Some(false)
        } else if self.is_true() {
            Some(true)
        } else {
            None
        }
    }
}

impl Neg for Ref {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self(self.0 ^ 1)
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
