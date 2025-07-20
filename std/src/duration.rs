//! Duration information

/// A span of time, represented by femtoseconds (to be in line with HPET)
pub struct Duration {
    /// Number of femtoseconds
    femtoseconds: usize,
}

impl Duration {
    /// Constructs a new duration from the given number of femtoseconds
    pub const fn from_femtoseconds(femtoseconds: usize) -> Self {
        Self { femtoseconds }
    }

    /// Returns the stored duration in femtoseconds
    pub const fn as_femtoseconds(&self) -> usize {
        self.femtoseconds
    }

    /// Constructs a new duration from the given number of picoseconds
    pub const fn from_picoseconds(picoseconds: usize) -> Self {
        Self::from_femtoseconds(picoseconds * 1_000)
    }

    /// Returns the stored duration in picoseconds, discarding any extra precision
    pub const fn as_picoseconds(&self) -> usize {
        self.femtoseconds / 1_000
    }

    /// Constructs a new duration from the given number of nanoseconds
    pub const fn from_nanoseconds(nanoseconds: usize) -> Self {
        Self::from_femtoseconds(nanoseconds * 1_000_000)
    }

    /// Returns the stored duration in nanoseconds, discarding any extra precision
    pub const fn as_nanoseconds(&self) -> usize {
        self.femtoseconds / 1_000_000
    }

    /// Constructs a new duration from the given number of microseconds
    pub const fn from_microseconds(microseconds: usize) -> Self {
        Self::from_femtoseconds(microseconds * 1_000_000_000)
    }

    /// Returns the stored duration in microseconds, discarding any extra precision
    pub const fn as_microseconds(&self) -> usize {
        self.femtoseconds / 1_000_000_000
    }

    /// Constructs a new duration from the given number of milliseconds
    pub const fn from_milliseconds(milliseconds: usize) -> Self {
        Self::from_femtoseconds(milliseconds * 1_000_000_000_000)
    }

    /// Returns the stored duration in milliseconds, discarding any extra precision
    pub const fn as_milliseconds(&self) -> usize {
        self.femtoseconds / 1_000_000_000_000
    }

    /// Constructs a new duration from the given number of seconds
    pub const fn from_seconds(seconds: usize) -> Self {
        Self::from_femtoseconds(seconds * 1_000_000_000_000_000)
    }

    /// Returns the stored duration in seconds, discarding any extra precision
    pub const fn as_seconds(&self) -> usize {
        self.femtoseconds / 1_000_000_000_000_000
    }
}
