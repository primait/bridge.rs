use std::ops::RangeInclusive;
use std::sync::RwLock;

#[derive(Clone)]
pub struct StalenessCheckPercentage(RangeInclusive<f64>);

impl StalenessCheckPercentage {
    pub fn new(min: f64, max: f64) -> Self {
        assert!((0.0..=1.0).contains(&min));
        assert!((0.0..=1.0).contains(&max));
        assert!(min <= max);

        Self(min..=max)
    }

    pub fn random_value_between(&self) -> f64 {
        use rand::Rng;
        rand::thread_rng().gen_range(self.0.clone())
    }
}

impl Default for StalenessCheckPercentage {
    fn default() -> Self {
        Self(0.6..=0.9)
    }
}

impl From<RangeInclusive<f64>> for StalenessCheckPercentage {
    fn from(range: RangeInclusive<f64>) -> Self {
        Self::new(*range.start(), *range.end())
    }
}

/// A wrapper around RwLock that cannot be poisoned,
/// by requiring the value inside to be completely replaced,
/// ensuring a panic can never occur while the lock is held
#[derive(Debug)]
pub struct UnpoisonableRwLock<T>(RwLock<T>);

impl<T> UnpoisonableRwLock<T> {
    pub fn new(v: T) -> Self {
        Self(RwLock::new(v))
    }

    pub fn read(&self) -> std::sync::RwLockReadGuard<T> {
        // Unwrapping here is fine since a panic can never occur while we hold the lock
        self.0.read().unwrap()
    }

    pub fn write(&self, v: T) {
        // Unwrapping here is fine since a panic can never occur while we hold the lock
        *self.0.write().unwrap() = v;
    }
}
