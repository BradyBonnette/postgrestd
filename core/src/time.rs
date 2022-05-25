#![stable(feature = "duration_core", since = "1.25.0")]

//! Temporal quantification.
//!
//! # Examples:
//!
//! There are multiple ways to create a new [`Duration`]:
//!
//! ```
//! # use std::time::Duration;
//! let five_seconds = Duration::from_secs(5);
//! assert_eq!(five_seconds, Duration::from_millis(5_000));
//! assert_eq!(five_seconds, Duration::from_micros(5_000_000));
//! assert_eq!(five_seconds, Duration::from_nanos(5_000_000_000));
//!
//! let ten_seconds = Duration::from_secs(10);
//! let seven_nanos = Duration::from_nanos(7);
//! let total = ten_seconds + seven_nanos;
//! assert_eq!(total, Duration::new(10, 7));
//! ```

use crate::fmt;
use crate::iter::Sum;
use crate::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};

const NANOS_PER_SEC: u32 = 1_000_000_000;
const NANOS_PER_MILLI: u32 = 1_000_000;
const NANOS_PER_MICRO: u32 = 1_000;
const MILLIS_PER_SEC: u64 = 1_000;
const MICROS_PER_SEC: u64 = 1_000_000;

/// A `Duration` type to represent a span of time, typically used for system
/// timeouts.
///
/// Each `Duration` is composed of a whole number of seconds and a fractional part
/// represented in nanoseconds. If the underlying system does not support
/// nanosecond-level precision, APIs binding a system timeout will typically round up
/// the number of nanoseconds.
///
/// [`Duration`]s implement many common traits, including [`Add`], [`Sub`], and other
/// [`ops`] traits. It implements [`Default`] by returning a zero-length `Duration`.
///
/// [`ops`]: crate::ops
///
/// # Examples
///
/// ```
/// use std::time::Duration;
///
/// let five_seconds = Duration::new(5, 0);
/// let five_seconds_and_five_nanos = five_seconds + Duration::new(0, 5);
///
/// assert_eq!(five_seconds_and_five_nanos.as_secs(), 5);
/// assert_eq!(five_seconds_and_five_nanos.subsec_nanos(), 5);
///
/// let ten_millis = Duration::from_millis(10);
/// ```
///
/// # Formatting `Duration` values
///
/// `Duration` intentionally does not have a `Display` impl, as there are a
/// variety of ways to format spans of time for human readability. `Duration`
/// provides a `Debug` impl that shows the full precision of the value.
///
/// The `Debug` output uses the non-ASCII "µs" suffix for microseconds. If your
/// program output may appear in contexts that cannot rely on full Unicode
/// compatibility, you may wish to format `Duration` objects yourself or use a
/// crate to do so.
#[stable(feature = "duration", since = "1.3.0")]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[cfg_attr(not(test), rustc_diagnostic_item = "Duration")]
pub struct Duration {
    secs: u64,
    nanos: u32, // Always 0 <= nanos < NANOS_PER_SEC
}

impl Duration {
    /// The duration of one second.
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(duration_constants)]
    /// use std::time::Duration;
    ///
    /// assert_eq!(Duration::SECOND, Duration::from_secs(1));
    /// ```
    #[unstable(feature = "duration_constants", issue = "57391")]
    pub const SECOND: Duration = Duration::from_secs(1);

    /// The duration of one millisecond.
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(duration_constants)]
    /// use std::time::Duration;
    ///
    /// assert_eq!(Duration::MILLISECOND, Duration::from_millis(1));
    /// ```
    #[unstable(feature = "duration_constants", issue = "57391")]
    pub const MILLISECOND: Duration = Duration::from_millis(1);

    /// The duration of one microsecond.
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(duration_constants)]
    /// use std::time::Duration;
    ///
    /// assert_eq!(Duration::MICROSECOND, Duration::from_micros(1));
    /// ```
    #[unstable(feature = "duration_constants", issue = "57391")]
    pub const MICROSECOND: Duration = Duration::from_micros(1);

    /// The duration of one nanosecond.
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(duration_constants)]
    /// use std::time::Duration;
    ///
    /// assert_eq!(Duration::NANOSECOND, Duration::from_nanos(1));
    /// ```
    #[unstable(feature = "duration_constants", issue = "57391")]
    pub const NANOSECOND: Duration = Duration::from_nanos(1);

    /// A duration of zero time.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::time::Duration;
    ///
    /// let duration = Duration::ZERO;
    /// assert!(duration.is_zero());
    /// assert_eq!(duration.as_nanos(), 0);
    /// ```
    #[stable(feature = "duration_zero", since = "1.53.0")]
    pub const ZERO: Duration = Duration::from_nanos(0);

    /// The maximum duration.
    ///
    /// May vary by platform as necessary. Must be able to contain the difference between
    /// two instances of [`Instant`] or two instances of [`SystemTime`].
    /// This constraint gives it a value of about 584,942,417,355 years in practice,
    /// which is currently used on all platforms.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::time::Duration;
    ///
    /// assert_eq!(Duration::MAX, Duration::new(u64::MAX, 1_000_000_000 - 1));
    /// ```
    /// [`Instant`]: ../../std/time/struct.Instant.html
    /// [`SystemTime`]: ../../std/time/struct.SystemTime.html
    #[stable(feature = "duration_saturating_ops", since = "1.53.0")]
    pub const MAX: Duration = Duration::new(u64::MAX, NANOS_PER_SEC - 1);

    /// Creates a new `Duration` from the specified number of whole seconds and
    /// additional nanoseconds.
    ///
    /// If the number of nanoseconds is greater than 1 billion (the number of
    /// nanoseconds in a second), then it will carry over into the seconds provided.
    ///
    /// # Panics
    ///
    /// This constructor will panic if the carry from the nanoseconds overflows
    /// the seconds counter.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::time::Duration;
    ///
    /// let five_seconds = Duration::new(5, 0);
    /// ```
    #[stable(feature = "duration", since = "1.3.0")]
    #[inline]
    #[must_use]
    #[rustc_const_stable(feature = "duration_consts_2", since = "1.58.0")]
    pub const fn new(secs: u64, nanos: u32) -> Duration {
        let secs = match secs.checked_add((nanos / NANOS_PER_SEC) as u64) {
            Some(secs) => secs,
            None => panic!("overflow in Duration::new"),
        };
        let nanos = nanos % NANOS_PER_SEC;
        Duration { secs, nanos }
    }

    /// Creates a new `Duration` from the specified number of whole seconds.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::time::Duration;
    ///
    /// let duration = Duration::from_secs(5);
    ///
    /// assert_eq!(5, duration.as_secs());
    /// assert_eq!(0, duration.subsec_nanos());
    /// ```
    #[stable(feature = "duration", since = "1.3.0")]
    #[must_use]
    #[inline]
    #[rustc_const_stable(feature = "duration_consts", since = "1.32.0")]
    pub const fn from_secs(secs: u64) -> Duration {
        Duration { secs, nanos: 0 }
    }

    /// Creates a new `Duration` from the specified number of milliseconds.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::time::Duration;
    ///
    /// let duration = Duration::from_millis(2569);
    ///
    /// assert_eq!(2, duration.as_secs());
    /// assert_eq!(569_000_000, duration.subsec_nanos());
    /// ```
    #[stable(feature = "duration", since = "1.3.0")]
    #[must_use]
    #[inline]
    #[rustc_const_stable(feature = "duration_consts", since = "1.32.0")]
    pub const fn from_millis(millis: u64) -> Duration {
        Duration {
            secs: millis / MILLIS_PER_SEC,
            nanos: ((millis % MILLIS_PER_SEC) as u32) * NANOS_PER_MILLI,
        }
    }

    /// Creates a new `Duration` from the specified number of microseconds.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::time::Duration;
    ///
    /// let duration = Duration::from_micros(1_000_002);
    ///
    /// assert_eq!(1, duration.as_secs());
    /// assert_eq!(2000, duration.subsec_nanos());
    /// ```
    #[stable(feature = "duration_from_micros", since = "1.27.0")]
    #[must_use]
    #[inline]
    #[rustc_const_stable(feature = "duration_consts", since = "1.32.0")]
    pub const fn from_micros(micros: u64) -> Duration {
        Duration {
            secs: micros / MICROS_PER_SEC,
            nanos: ((micros % MICROS_PER_SEC) as u32) * NANOS_PER_MICRO,
        }
    }

    /// Creates a new `Duration` from the specified number of nanoseconds.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::time::Duration;
    ///
    /// let duration = Duration::from_nanos(1_000_000_123);
    ///
    /// assert_eq!(1, duration.as_secs());
    /// assert_eq!(123, duration.subsec_nanos());
    /// ```
    #[stable(feature = "duration_extras", since = "1.27.0")]
    #[must_use]
    #[inline]
    #[rustc_const_stable(feature = "duration_consts", since = "1.32.0")]
    pub const fn from_nanos(nanos: u64) -> Duration {
        Duration {
            secs: nanos / (NANOS_PER_SEC as u64),
            nanos: (nanos % (NANOS_PER_SEC as u64)) as u32,
        }
    }

    /// Returns true if this `Duration` spans no time.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::time::Duration;
    ///
    /// assert!(Duration::ZERO.is_zero());
    /// assert!(Duration::new(0, 0).is_zero());
    /// assert!(Duration::from_nanos(0).is_zero());
    /// assert!(Duration::from_secs(0).is_zero());
    ///
    /// assert!(!Duration::new(1, 1).is_zero());
    /// assert!(!Duration::from_nanos(1).is_zero());
    /// assert!(!Duration::from_secs(1).is_zero());
    /// ```
    #[must_use]
    #[stable(feature = "duration_zero", since = "1.53.0")]
    #[rustc_const_stable(feature = "duration_zero", since = "1.53.0")]
    #[inline]
    pub const fn is_zero(&self) -> bool {
        self.secs == 0 && self.nanos == 0
    }

    /// Returns the number of _whole_ seconds contained by this `Duration`.
    ///
    /// The returned value does not include the fractional (nanosecond) part of the
    /// duration, which can be obtained using [`subsec_nanos`].
    ///
    /// # Examples
    ///
    /// ```
    /// use std::time::Duration;
    ///
    /// let duration = Duration::new(5, 730023852);
    /// assert_eq!(duration.as_secs(), 5);
    /// ```
    ///
    /// To determine the total number of seconds represented by the `Duration`,
    /// use `as_secs` in combination with [`subsec_nanos`]:
    ///
    /// ```
    /// use std::time::Duration;
    ///
    /// let duration = Duration::new(5, 730023852);
    ///
    /// assert_eq!(5.730023852,
    ///            duration.as_secs() as f64
    ///            + duration.subsec_nanos() as f64 * 1e-9);
    /// ```
    ///
    /// [`subsec_nanos`]: Duration::subsec_nanos
    #[stable(feature = "duration", since = "1.3.0")]
    #[rustc_const_stable(feature = "duration", since = "1.32.0")]
    #[must_use]
    #[inline]
    pub const fn as_secs(&self) -> u64 {
        self.secs
    }

    /// Returns the fractional part of this `Duration`, in whole milliseconds.
    ///
    /// This method does **not** return the length of the duration when
    /// represented by milliseconds. The returned number always represents a
    /// fractional portion of a second (i.e., it is less than one thousand).
    ///
    /// # Examples
    ///
    /// ```
    /// use std::time::Duration;
    ///
    /// let duration = Duration::from_millis(5432);
    /// assert_eq!(duration.as_secs(), 5);
    /// assert_eq!(duration.subsec_millis(), 432);
    /// ```
    #[stable(feature = "duration_extras", since = "1.27.0")]
    #[rustc_const_stable(feature = "duration_extras", since = "1.32.0")]
    #[must_use]
    #[inline]
    pub const fn subsec_millis(&self) -> u32 {
        self.nanos / NANOS_PER_MILLI
    }

    /// Returns the fractional part of this `Duration`, in whole microseconds.
    ///
    /// This method does **not** return the length of the duration when
    /// represented by microseconds. The returned number always represents a
    /// fractional portion of a second (i.e., it is less than one million).
    ///
    /// # Examples
    ///
    /// ```
    /// use std::time::Duration;
    ///
    /// let duration = Duration::from_micros(1_234_567);
    /// assert_eq!(duration.as_secs(), 1);
    /// assert_eq!(duration.subsec_micros(), 234_567);
    /// ```
    #[stable(feature = "duration_extras", since = "1.27.0")]
    #[rustc_const_stable(feature = "duration_extras", since = "1.32.0")]
    #[must_use]
    #[inline]
    pub const fn subsec_micros(&self) -> u32 {
        self.nanos / NANOS_PER_MICRO
    }

    /// Returns the fractional part of this `Duration`, in nanoseconds.
    ///
    /// This method does **not** return the length of the duration when
    /// represented by nanoseconds. The returned number always represents a
    /// fractional portion of a second (i.e., it is less than one billion).
    ///
    /// # Examples
    ///
    /// ```
    /// use std::time::Duration;
    ///
    /// let duration = Duration::from_millis(5010);
    /// assert_eq!(duration.as_secs(), 5);
    /// assert_eq!(duration.subsec_nanos(), 10_000_000);
    /// ```
    #[stable(feature = "duration", since = "1.3.0")]
    #[rustc_const_stable(feature = "duration", since = "1.32.0")]
    #[must_use]
    #[inline]
    pub const fn subsec_nanos(&self) -> u32 {
        self.nanos
    }

    /// Returns the total number of whole milliseconds contained by this `Duration`.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::time::Duration;
    ///
    /// let duration = Duration::new(5, 730023852);
    /// assert_eq!(duration.as_millis(), 5730);
    /// ```
    #[stable(feature = "duration_as_u128", since = "1.33.0")]
    #[rustc_const_stable(feature = "duration_as_u128", since = "1.33.0")]
    #[must_use]
    #[inline]
    pub const fn as_millis(&self) -> u128 {
        self.secs as u128 * MILLIS_PER_SEC as u128 + (self.nanos / NANOS_PER_MILLI) as u128
    }

    /// Returns the total number of whole microseconds contained by this `Duration`.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::time::Duration;
    ///
    /// let duration = Duration::new(5, 730023852);
    /// assert_eq!(duration.as_micros(), 5730023);
    /// ```
    #[stable(feature = "duration_as_u128", since = "1.33.0")]
    #[rustc_const_stable(feature = "duration_as_u128", since = "1.33.0")]
    #[must_use]
    #[inline]
    pub const fn as_micros(&self) -> u128 {
        self.secs as u128 * MICROS_PER_SEC as u128 + (self.nanos / NANOS_PER_MICRO) as u128
    }

    /// Returns the total number of nanoseconds contained by this `Duration`.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::time::Duration;
    ///
    /// let duration = Duration::new(5, 730023852);
    /// assert_eq!(duration.as_nanos(), 5730023852);
    /// ```
    #[stable(feature = "duration_as_u128", since = "1.33.0")]
    #[rustc_const_stable(feature = "duration_as_u128", since = "1.33.0")]
    #[must_use]
    #[inline]
    pub const fn as_nanos(&self) -> u128 {
        self.secs as u128 * NANOS_PER_SEC as u128 + self.nanos as u128
    }

    /// Checked `Duration` addition. Computes `self + other`, returning [`None`]
    /// if overflow occurred.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use std::time::Duration;
    ///
    /// assert_eq!(Duration::new(0, 0).checked_add(Duration::new(0, 1)), Some(Duration::new(0, 1)));
    /// assert_eq!(Duration::new(1, 0).checked_add(Duration::new(u64::MAX, 0)), None);
    /// ```
    #[stable(feature = "duration_checked_ops", since = "1.16.0")]
    #[must_use = "this returns the result of the operation, \
                  without modifying the original"]
    #[inline]
    #[rustc_const_stable(feature = "duration_consts_2", since = "1.58.0")]
    pub const fn checked_add(self, rhs: Duration) -> Option<Duration> {
        if let Some(mut secs) = self.secs.checked_add(rhs.secs) {
            let mut nanos = self.nanos + rhs.nanos;
            if nanos >= NANOS_PER_SEC {
                nanos -= NANOS_PER_SEC;
                if let Some(new_secs) = secs.checked_add(1) {
                    secs = new_secs;
                } else {
                    return None;
                }
            }
            debug_assert!(nanos < NANOS_PER_SEC);
            Some(Duration { secs, nanos })
        } else {
            None
        }
    }

    /// Saturating `Duration` addition. Computes `self + other`, returning [`Duration::MAX`]
    /// if overflow occurred.
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(duration_constants)]
    /// use std::time::Duration;
    ///
    /// assert_eq!(Duration::new(0, 0).saturating_add(Duration::new(0, 1)), Duration::new(0, 1));
    /// assert_eq!(Duration::new(1, 0).saturating_add(Duration::new(u64::MAX, 0)), Duration::MAX);
    /// ```
    #[stable(feature = "duration_saturating_ops", since = "1.53.0")]
    #[must_use = "this returns the result of the operation, \
                  without modifying the original"]
    #[inline]
    #[rustc_const_stable(feature = "duration_consts_2", since = "1.58.0")]
    pub const fn saturating_add(self, rhs: Duration) -> Duration {
        match self.checked_add(rhs) {
            Some(res) => res,
            None => Duration::MAX,
        }
    }

    /// Checked `Duration` subtraction. Computes `self - other`, returning [`None`]
    /// if the result would be negative or if overflow occurred.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use std::time::Duration;
    ///
    /// assert_eq!(Duration::new(0, 1).checked_sub(Duration::new(0, 0)), Some(Duration::new(0, 1)));
    /// assert_eq!(Duration::new(0, 0).checked_sub(Duration::new(0, 1)), None);
    /// ```
    #[stable(feature = "duration_checked_ops", since = "1.16.0")]
    #[must_use = "this returns the result of the operation, \
                  without modifying the original"]
    #[inline]
    #[rustc_const_stable(feature = "duration_consts_2", since = "1.58.0")]
    pub const fn checked_sub(self, rhs: Duration) -> Option<Duration> {
        if let Some(mut secs) = self.secs.checked_sub(rhs.secs) {
            let nanos = if self.nanos >= rhs.nanos {
                self.nanos - rhs.nanos
            } else if let Some(sub_secs) = secs.checked_sub(1) {
                secs = sub_secs;
                self.nanos + NANOS_PER_SEC - rhs.nanos
            } else {
                return None;
            };
            debug_assert!(nanos < NANOS_PER_SEC);
            Some(Duration { secs, nanos })
        } else {
            None
        }
    }

    /// Saturating `Duration` subtraction. Computes `self - other`, returning [`Duration::ZERO`]
    /// if the result would be negative or if overflow occurred.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::time::Duration;
    ///
    /// assert_eq!(Duration::new(0, 1).saturating_sub(Duration::new(0, 0)), Duration::new(0, 1));
    /// assert_eq!(Duration::new(0, 0).saturating_sub(Duration::new(0, 1)), Duration::ZERO);
    /// ```
    #[stable(feature = "duration_saturating_ops", since = "1.53.0")]
    #[must_use = "this returns the result of the operation, \
                  without modifying the original"]
    #[inline]
    #[rustc_const_stable(feature = "duration_consts_2", since = "1.58.0")]
    pub const fn saturating_sub(self, rhs: Duration) -> Duration {
        match self.checked_sub(rhs) {
            Some(res) => res,
            None => Duration::ZERO,
        }
    }

    /// Checked `Duration` multiplication. Computes `self * other`, returning
    /// [`None`] if overflow occurred.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use std::time::Duration;
    ///
    /// assert_eq!(Duration::new(0, 500_000_001).checked_mul(2), Some(Duration::new(1, 2)));
    /// assert_eq!(Duration::new(u64::MAX - 1, 0).checked_mul(2), None);
    /// ```
    #[stable(feature = "duration_checked_ops", since = "1.16.0")]
    #[must_use = "this returns the result of the operation, \
                  without modifying the original"]
    #[inline]
    #[rustc_const_stable(feature = "duration_consts_2", since = "1.58.0")]
    pub const fn checked_mul(self, rhs: u32) -> Option<Duration> {
        // Multiply nanoseconds as u64, because it cannot overflow that way.
        let total_nanos = self.nanos as u64 * rhs as u64;
        let extra_secs = total_nanos / (NANOS_PER_SEC as u64);
        let nanos = (total_nanos % (NANOS_PER_SEC as u64)) as u32;
        if let Some(s) = self.secs.checked_mul(rhs as u64) {
            if let Some(secs) = s.checked_add(extra_secs) {
                debug_assert!(nanos < NANOS_PER_SEC);
                return Some(Duration { secs, nanos });
            }
        }
        None
    }

    /// Saturating `Duration` multiplication. Computes `self * other`, returning
    /// [`Duration::MAX`] if overflow occurred.
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(duration_constants)]
    /// use std::time::Duration;
    ///
    /// assert_eq!(Duration::new(0, 500_000_001).saturating_mul(2), Duration::new(1, 2));
    /// assert_eq!(Duration::new(u64::MAX - 1, 0).saturating_mul(2), Duration::MAX);
    /// ```
    #[stable(feature = "duration_saturating_ops", since = "1.53.0")]
    #[must_use = "this returns the result of the operation, \
                  without modifying the original"]
    #[inline]
    #[rustc_const_stable(feature = "duration_consts_2", since = "1.58.0")]
    pub const fn saturating_mul(self, rhs: u32) -> Duration {
        match self.checked_mul(rhs) {
            Some(res) => res,
            None => Duration::MAX,
        }
    }

    /// Checked `Duration` division. Computes `self / other`, returning [`None`]
    /// if `other == 0`.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use std::time::Duration;
    ///
    /// assert_eq!(Duration::new(2, 0).checked_div(2), Some(Duration::new(1, 0)));
    /// assert_eq!(Duration::new(1, 0).checked_div(2), Some(Duration::new(0, 500_000_000)));
    /// assert_eq!(Duration::new(2, 0).checked_div(0), None);
    /// ```
    #[stable(feature = "duration_checked_ops", since = "1.16.0")]
    #[must_use = "this returns the result of the operation, \
                  without modifying the original"]
    #[inline]
    #[rustc_const_stable(feature = "duration_consts_2", since = "1.58.0")]
    pub const fn checked_div(self, rhs: u32) -> Option<Duration> {
        if rhs != 0 {
            let secs = self.secs / (rhs as u64);
            let carry = self.secs - secs * (rhs as u64);
            let extra_nanos = carry * (NANOS_PER_SEC as u64) / (rhs as u64);
            let nanos = self.nanos / rhs + (extra_nanos as u32);
            debug_assert!(nanos < NANOS_PER_SEC);
            Some(Duration { secs, nanos })
        } else {
            None
        }
    }

    /// Returns the number of seconds contained by this `Duration` as `f64`.
    ///
    /// The returned value does include the fractional (nanosecond) part of the duration.
    ///
    /// # Examples
    /// ```
    /// use std::time::Duration;
    ///
    /// let dur = Duration::new(2, 700_000_000);
    /// assert_eq!(dur.as_secs_f64(), 2.7);
    /// ```
    #[stable(feature = "duration_float", since = "1.38.0")]
    #[must_use]
    #[inline]
    #[rustc_const_unstable(feature = "duration_consts_float", issue = "72440")]
    pub const fn as_secs_f64(&self) -> f64 {
        (self.secs as f64) + (self.nanos as f64) / (NANOS_PER_SEC as f64)
    }

    /// Returns the number of seconds contained by this `Duration` as `f32`.
    ///
    /// The returned value does include the fractional (nanosecond) part of the duration.
    ///
    /// # Examples
    /// ```
    /// use std::time::Duration;
    ///
    /// let dur = Duration::new(2, 700_000_000);
    /// assert_eq!(dur.as_secs_f32(), 2.7);
    /// ```
    #[stable(feature = "duration_float", since = "1.38.0")]
    #[must_use]
    #[inline]
    #[rustc_const_unstable(feature = "duration_consts_float", issue = "72440")]
    pub const fn as_secs_f32(&self) -> f32 {
        (self.secs as f32) + (self.nanos as f32) / (NANOS_PER_SEC as f32)
    }

    /// Creates a new `Duration` from the specified number of seconds represented
    /// as `f64`.
    ///
    /// # Panics
    /// This constructor will panic if `secs` is negative, overflows `Duration` or not finite.
    ///
    /// # Examples
    /// ```
    /// use std::time::Duration;
    ///
    /// let res = Duration::from_secs_f64(0.0);
    /// assert_eq!(res, Duration::new(0, 0));
    /// let res = Duration::from_secs_f64(1e-20);
    /// assert_eq!(res, Duration::new(0, 0));
    /// let res = Duration::from_secs_f64(4.2e-7);
    /// assert_eq!(res, Duration::new(0, 420));
    /// let res = Duration::from_secs_f64(2.7);
    /// assert_eq!(res, Duration::new(2, 700_000_000));
    /// let res = Duration::from_secs_f64(3e10);
    /// assert_eq!(res, Duration::new(30_000_000_000, 0));
    /// // subnormal float
    /// let res = Duration::from_secs_f64(f64::from_bits(1));
    /// assert_eq!(res, Duration::new(0, 0));
    /// // conversion uses rounding
    /// let res = Duration::from_secs_f64(0.999e-9);
    /// assert_eq!(res, Duration::new(0, 1));
    /// ```
    #[stable(feature = "duration_float", since = "1.38.0")]
    #[must_use]
    #[inline]
    #[rustc_const_unstable(feature = "duration_consts_float", issue = "72440")]
    pub const fn from_secs_f64(secs: f64) -> Duration {
        match Duration::try_from_secs_f64(secs) {
            Ok(v) => v,
            Err(e) => panic!("{}", e.description()),
        }
    }

    /// Creates a new `Duration` from the specified number of seconds represented
    /// as `f32`.
    ///
    /// # Panics
    /// This constructor will panic if `secs` is negative, overflows `Duration` or not finite.
    ///
    /// # Examples
    /// ```
    /// use std::time::Duration;
    ///
    /// let res = Duration::from_secs_f32(0.0);
    /// assert_eq!(res, Duration::new(0, 0));
    /// let res = Duration::from_secs_f32(1e-20);
    /// assert_eq!(res, Duration::new(0, 0));
    /// let res = Duration::from_secs_f32(4.2e-7);
    /// assert_eq!(res, Duration::new(0, 420));
    /// let res = Duration::from_secs_f32(2.7);
    /// assert_eq!(res, Duration::new(2, 700_000_048));
    /// let res = Duration::from_secs_f32(3e10);
    /// assert_eq!(res, Duration::new(30_000_001_024, 0));
    /// // subnormal float
    /// let res = Duration::from_secs_f32(f32::from_bits(1));
    /// assert_eq!(res, Duration::new(0, 0));
    /// // conversion uses rounding
    /// let res = Duration::from_secs_f32(0.999e-9);
    /// assert_eq!(res, Duration::new(0, 1));
    /// ```
    #[stable(feature = "duration_float", since = "1.38.0")]
    #[must_use]
    #[inline]
    #[rustc_const_unstable(feature = "duration_consts_float", issue = "72440")]
    pub const fn from_secs_f32(secs: f32) -> Duration {
        match Duration::try_from_secs_f32(secs) {
            Ok(v) => v,
            Err(e) => panic!("{}", e.description()),
        }
    }

    /// Multiplies `Duration` by `f64`.
    ///
    /// # Panics
    /// This method will panic if result is negative, overflows `Duration` or not finite.
    ///
    /// # Examples
    /// ```
    /// use std::time::Duration;
    ///
    /// let dur = Duration::new(2, 700_000_000);
    /// assert_eq!(dur.mul_f64(3.14), Duration::new(8, 478_000_000));
    /// assert_eq!(dur.mul_f64(3.14e5), Duration::new(847_800, 0));
    /// ```
    #[stable(feature = "duration_float", since = "1.38.0")]
    #[must_use = "this returns the result of the operation, \
                  without modifying the original"]
    #[inline]
    #[rustc_const_unstable(feature = "duration_consts_float", issue = "72440")]
    pub const fn mul_f64(self, rhs: f64) -> Duration {
        Duration::from_secs_f64(rhs * self.as_secs_f64())
    }

    /// Multiplies `Duration` by `f32`.
    ///
    /// # Panics
    /// This method will panic if result is negative, overflows `Duration` or not finite.
    ///
    /// # Examples
    /// ```
    /// use std::time::Duration;
    ///
    /// let dur = Duration::new(2, 700_000_000);
    /// assert_eq!(dur.mul_f32(3.14), Duration::new(8, 478_000_641));
    /// assert_eq!(dur.mul_f32(3.14e5), Duration::new(847800, 0));
    /// ```
    #[stable(feature = "duration_float", since = "1.38.0")]
    #[must_use = "this returns the result of the operation, \
                  without modifying the original"]
    #[inline]
    #[rustc_const_unstable(feature = "duration_consts_float", issue = "72440")]
    pub const fn mul_f32(self, rhs: f32) -> Duration {
        Duration::from_secs_f32(rhs * self.as_secs_f32())
    }

    /// Divide `Duration` by `f64`.
    ///
    /// # Panics
    /// This method will panic if result is negative, overflows `Duration` or not finite.
    ///
    /// # Examples
    /// ```
    /// use std::time::Duration;
    ///
    /// let dur = Duration::new(2, 700_000_000);
    /// assert_eq!(dur.div_f64(3.14), Duration::new(0, 859_872_611));
    /// assert_eq!(dur.div_f64(3.14e5), Duration::new(0, 8_599));
    /// ```
    #[stable(feature = "duration_float", since = "1.38.0")]
    #[must_use = "this returns the result of the operation, \
                  without modifying the original"]
    #[inline]
    #[rustc_const_unstable(feature = "duration_consts_float", issue = "72440")]
    pub const fn div_f64(self, rhs: f64) -> Duration {
        Duration::from_secs_f64(self.as_secs_f64() / rhs)
    }

    /// Divide `Duration` by `f32`.
    ///
    /// # Panics
    /// This method will panic if result is negative, overflows `Duration` or not finite.
    ///
    /// # Examples
    /// ```
    /// use std::time::Duration;
    ///
    /// let dur = Duration::new(2, 700_000_000);
    /// // note that due to rounding errors result is slightly
    /// // different from 0.859_872_611
    /// assert_eq!(dur.div_f32(3.14), Duration::new(0, 859_872_580));
    /// assert_eq!(dur.div_f32(3.14e5), Duration::new(0, 8_599));
    /// ```
    #[stable(feature = "duration_float", since = "1.38.0")]
    #[must_use = "this returns the result of the operation, \
                  without modifying the original"]
    #[inline]
    #[rustc_const_unstable(feature = "duration_consts_float", issue = "72440")]
    pub const fn div_f32(self, rhs: f32) -> Duration {
        Duration::from_secs_f32(self.as_secs_f32() / rhs)
    }

    /// Divide `Duration` by `Duration` and return `f64`.
    ///
    /// # Examples
    /// ```
    /// #![feature(div_duration)]
    /// use std::time::Duration;
    ///
    /// let dur1 = Duration::new(2, 700_000_000);
    /// let dur2 = Duration::new(5, 400_000_000);
    /// assert_eq!(dur1.div_duration_f64(dur2), 0.5);
    /// ```
    #[unstable(feature = "div_duration", issue = "63139")]
    #[must_use = "this returns the result of the operation, \
                  without modifying the original"]
    #[inline]
    #[rustc_const_unstable(feature = "duration_consts_float", issue = "72440")]
    pub const fn div_duration_f64(self, rhs: Duration) -> f64 {
        self.as_secs_f64() / rhs.as_secs_f64()
    }

    /// Divide `Duration` by `Duration` and return `f32`.
    ///
    /// # Examples
    /// ```
    /// #![feature(div_duration)]
    /// use std::time::Duration;
    ///
    /// let dur1 = Duration::new(2, 700_000_000);
    /// let dur2 = Duration::new(5, 400_000_000);
    /// assert_eq!(dur1.div_duration_f32(dur2), 0.5);
    /// ```
    #[unstable(feature = "div_duration", issue = "63139")]
    #[must_use = "this returns the result of the operation, \
                  without modifying the original"]
    #[inline]
    #[rustc_const_unstable(feature = "duration_consts_float", issue = "72440")]
    pub const fn div_duration_f32(self, rhs: Duration) -> f32 {
        self.as_secs_f32() / rhs.as_secs_f32()
    }
}

#[stable(feature = "duration", since = "1.3.0")]
impl Add for Duration {
    type Output = Duration;

    fn add(self, rhs: Duration) -> Duration {
        self.checked_add(rhs).expect("overflow when adding durations")
    }
}

#[stable(feature = "time_augmented_assignment", since = "1.9.0")]
impl AddAssign for Duration {
    fn add_assign(&mut self, rhs: Duration) {
        *self = *self + rhs;
    }
}

#[stable(feature = "duration", since = "1.3.0")]
impl Sub for Duration {
    type Output = Duration;

    fn sub(self, rhs: Duration) -> Duration {
        self.checked_sub(rhs).expect("overflow when subtracting durations")
    }
}

#[stable(feature = "time_augmented_assignment", since = "1.9.0")]
impl SubAssign for Duration {
    fn sub_assign(&mut self, rhs: Duration) {
        *self = *self - rhs;
    }
}

#[stable(feature = "duration", since = "1.3.0")]
impl Mul<u32> for Duration {
    type Output = Duration;

    fn mul(self, rhs: u32) -> Duration {
        self.checked_mul(rhs).expect("overflow when multiplying duration by scalar")
    }
}

#[stable(feature = "symmetric_u32_duration_mul", since = "1.31.0")]
impl Mul<Duration> for u32 {
    type Output = Duration;

    fn mul(self, rhs: Duration) -> Duration {
        rhs * self
    }
}

#[stable(feature = "time_augmented_assignment", since = "1.9.0")]
impl MulAssign<u32> for Duration {
    fn mul_assign(&mut self, rhs: u32) {
        *self = *self * rhs;
    }
}

#[stable(feature = "duration", since = "1.3.0")]
impl Div<u32> for Duration {
    type Output = Duration;

    fn div(self, rhs: u32) -> Duration {
        self.checked_div(rhs).expect("divide by zero error when dividing duration by scalar")
    }
}

#[stable(feature = "time_augmented_assignment", since = "1.9.0")]
impl DivAssign<u32> for Duration {
    fn div_assign(&mut self, rhs: u32) {
        *self = *self / rhs;
    }
}

macro_rules! sum_durations {
    ($iter:expr) => {{
        let mut total_secs: u64 = 0;
        let mut total_nanos: u64 = 0;

        for entry in $iter {
            total_secs =
                total_secs.checked_add(entry.secs).expect("overflow in iter::sum over durations");
            total_nanos = match total_nanos.checked_add(entry.nanos as u64) {
                Some(n) => n,
                None => {
                    total_secs = total_secs
                        .checked_add(total_nanos / NANOS_PER_SEC as u64)
                        .expect("overflow in iter::sum over durations");
                    (total_nanos % NANOS_PER_SEC as u64) + entry.nanos as u64
                }
            };
        }
        total_secs = total_secs
            .checked_add(total_nanos / NANOS_PER_SEC as u64)
            .expect("overflow in iter::sum over durations");
        total_nanos = total_nanos % NANOS_PER_SEC as u64;
        Duration { secs: total_secs, nanos: total_nanos as u32 }
    }};
}

#[stable(feature = "duration_sum", since = "1.16.0")]
impl Sum for Duration {
    fn sum<I: Iterator<Item = Duration>>(iter: I) -> Duration {
        sum_durations!(iter)
    }
}

#[stable(feature = "duration_sum", since = "1.16.0")]
impl<'a> Sum<&'a Duration> for Duration {
    fn sum<I: Iterator<Item = &'a Duration>>(iter: I) -> Duration {
        sum_durations!(iter)
    }
}

#[stable(feature = "duration_debug_impl", since = "1.27.0")]
impl fmt::Debug for Duration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        /// Formats a floating point number in decimal notation.
        ///
        /// The number is given as the `integer_part` and a fractional part.
        /// The value of the fractional part is `fractional_part / divisor`. So
        /// `integer_part` = 3, `fractional_part` = 12 and `divisor` = 100
        /// represents the number `3.012`. Trailing zeros are omitted.
        ///
        /// `divisor` must not be above 100_000_000. It also should be a power
        /// of 10, everything else doesn't make sense. `fractional_part` has
        /// to be less than `10 * divisor`!
        ///
        /// A prefix and postfix may be added. The whole thing is padded
        /// to the formatter's `width`, if specified.
        fn fmt_decimal(
            f: &mut fmt::Formatter<'_>,
            mut integer_part: u64,
            mut fractional_part: u32,
            mut divisor: u32,
            prefix: &str,
            postfix: &str,
        ) -> fmt::Result {
            // Encode the fractional part into a temporary buffer. The buffer
            // only need to hold 9 elements, because `fractional_part` has to
            // be smaller than 10^9. The buffer is prefilled with '0' digits
            // to simplify the code below.
            let mut buf = [b'0'; 9];

            // The next digit is written at this position
            let mut pos = 0;

            // We keep writing digits into the buffer while there are non-zero
            // digits left and we haven't written enough digits yet.
            while fractional_part > 0 && pos < f.precision().unwrap_or(9) {
                // Write new digit into the buffer
                buf[pos] = b'0' + (fractional_part / divisor) as u8;

                fractional_part %= divisor;
                divisor /= 10;
                pos += 1;
            }

            // If a precision < 9 was specified, there may be some non-zero
            // digits left that weren't written into the buffer. In that case we
            // need to perform rounding to match the semantics of printing
            // normal floating point numbers. However, we only need to do work
            // when rounding up. This happens if the first digit of the
            // remaining ones is >= 5.
            if fractional_part > 0 && fractional_part >= divisor * 5 {
                // Round up the number contained in the buffer. We go through
                // the buffer backwards and keep track of the carry.
                let mut rev_pos = pos;
                let mut carry = true;
                while carry && rev_pos > 0 {
                    rev_pos -= 1;

                    // If the digit in the buffer is not '9', we just need to
                    // increment it and can stop then (since we don't have a
                    // carry anymore). Otherwise, we set it to '0' (overflow)
                    // and continue.
                    if buf[rev_pos] < b'9' {
                        buf[rev_pos] += 1;
                        carry = false;
                    } else {
                        buf[rev_pos] = b'0';
                    }
                }

                // If we still have the carry bit set, that means that we set
                // the whole buffer to '0's and need to increment the integer
                // part.
                if carry {
                    integer_part += 1;
                }
            }

            // Determine the end of the buffer: if precision is set, we just
            // use as many digits from the buffer (capped to 9). If it isn't
            // set, we only use all digits up to the last non-zero one.
            let end = f.precision().map(|p| crate::cmp::min(p, 9)).unwrap_or(pos);

            // This closure emits the formatted duration without emitting any
            // padding (padding is calculated below).
            let emit_without_padding = |f: &mut fmt::Formatter<'_>| {
                write!(f, "{}{}", prefix, integer_part)?;

                // Write the decimal point and the fractional part (if any).
                if end > 0 {
                    // SAFETY: We are only writing ASCII digits into the buffer and
                    // it was initialized with '0's, so it contains valid UTF8.
                    let s = unsafe { crate::str::from_utf8_unchecked(&buf[..end]) };

                    // If the user request a precision > 9, we pad '0's at the end.
                    let w = f.precision().unwrap_or(pos);
                    write!(f, ".{:0<width$}", s, width = w)?;
                }

                write!(f, "{}", postfix)
            };

            match f.width() {
                None => {
                    // No `width` specified. There's no need to calculate the
                    // length of the output in this case, just emit it.
                    emit_without_padding(f)
                }
                Some(requested_w) => {
                    // A `width` was specified. Calculate the actual width of
                    // the output in order to calculate the required padding.
                    // It consists of 4 parts:
                    // 1. The prefix: is either "+" or "", so we can just use len().
                    // 2. The postfix: can be "µs" so we have to count UTF8 characters.
                    let mut actual_w = prefix.len() + postfix.chars().count();
                    // 3. The integer part:
                    if let Some(log) = integer_part.checked_log10() {
                        // integer_part is > 0, so has length log10(x)+1
                        actual_w += 1 + log as usize;
                    } else {
                        // integer_part is 0, so has length 1.
                        actual_w += 1;
                    }
                    // 4. The fractional part (if any):
                    if end > 0 {
                        let frac_part_w = f.precision().unwrap_or(pos);
                        actual_w += 1 + frac_part_w;
                    }

                    if requested_w <= actual_w {
                        // Output is already longer than `width`, so don't pad.
                        emit_without_padding(f)
                    } else {
                        // We need to add padding. Use the `Formatter::padding` helper function.
                        let default_align = crate::fmt::rt::v1::Alignment::Left;
                        let post_padding = f.padding(requested_w - actual_w, default_align)?;
                        emit_without_padding(f)?;
                        post_padding.write(f)
                    }
                }
            }
        }

        // Print leading '+' sign if requested
        let prefix = if f.sign_plus() { "+" } else { "" };

        if self.secs > 0 {
            fmt_decimal(f, self.secs, self.nanos, NANOS_PER_SEC / 10, prefix, "s")
        } else if self.nanos >= NANOS_PER_MILLI {
            fmt_decimal(
                f,
                (self.nanos / NANOS_PER_MILLI) as u64,
                self.nanos % NANOS_PER_MILLI,
                NANOS_PER_MILLI / 10,
                prefix,
                "ms",
            )
        } else if self.nanos >= NANOS_PER_MICRO {
            fmt_decimal(
                f,
                (self.nanos / NANOS_PER_MICRO) as u64,
                self.nanos % NANOS_PER_MICRO,
                NANOS_PER_MICRO / 10,
                prefix,
                "µs",
            )
        } else {
            fmt_decimal(f, self.nanos as u64, 0, 1, prefix, "ns")
        }
    }
}

/// An error which can be returned when converting a floating-point value of seconds
/// into a [`Duration`].
///
/// This error is used as the error type for [`Duration::try_from_secs_f32`] and
/// [`Duration::try_from_secs_f64`].
///
/// # Example
///
/// ```
/// #![feature(duration_checked_float)]
/// use std::time::Duration;
///
/// if let Err(e) = Duration::try_from_secs_f32(-1.0) {
///     println!("Failed conversion to Duration: {}", e);
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
#[unstable(feature = "duration_checked_float", issue = "83400")]
pub struct FromFloatSecsError {
    kind: FromFloatSecsErrorKind,
}

impl FromFloatSecsError {
    const fn description(&self) -> &'static str {
        match self.kind {
            FromFloatSecsErrorKind::Negative => {
                "can not convert float seconds to Duration: value is negative"
            }
            FromFloatSecsErrorKind::OverflowOrNan => {
                "can not convert float seconds to Duration: value is either too big or NaN"
            }
        }
    }
}

#[unstable(feature = "duration_checked_float", issue = "83400")]
impl fmt::Display for FromFloatSecsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.description().fmt(f)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum FromFloatSecsErrorKind {
    // Value is negative.
    Negative,
    // Value is either too big to be represented as `Duration` or `NaN`.
    OverflowOrNan,
}

macro_rules! try_from_secs {
    (
        secs = $secs: expr,
        mantissa_bits = $mant_bits: literal,
        exponent_bits = $exp_bits: literal,
        offset = $offset: literal,
        bits_ty = $bits_ty:ty,
        double_ty = $double_ty:ty,
    ) => {{
        const MIN_EXP: i16 = 1 - (1i16 << $exp_bits) / 2;
        const MANT_MASK: $bits_ty = (1 << $mant_bits) - 1;
        const EXP_MASK: $bits_ty = (1 << $exp_bits) - 1;

        if $secs.is_sign_negative() {
            return Err(FromFloatSecsError { kind: FromFloatSecsErrorKind::Negative });
        }

        let bits = $secs.to_bits();
        let mant = (bits & MANT_MASK) | (MANT_MASK + 1);
        let exp = ((bits >> $mant_bits) & EXP_MASK) as i16 + MIN_EXP;

        let (secs, nanos) = if exp < -31 {
            // the input represents less than 1ns and can not be rounded to it
            (0u64, 0u32)
        } else if exp < 0 {
            // the input is less than 1 second
            let t = <$double_ty>::from(mant) << ($offset + exp);
            let nanos_offset = $mant_bits + $offset;
            let nanos_tmp = u128::from(NANOS_PER_SEC) * u128::from(t);
            let nanos = (nanos_tmp >> nanos_offset) as u32;

            let rem_mask = (1 << nanos_offset) - 1;
            let rem_msb_mask = 1 << (nanos_offset - 1);
            let rem = nanos_tmp & rem_mask;
            let is_tie = rem == rem_msb_mask;
            let is_even = (nanos & 1) == 0;
            let rem_msb = nanos_tmp & rem_msb_mask == 0;
            let add_ns = !(rem_msb || (is_even && is_tie));

            // note that neither `f32`, nor `f64` can represent
            // 0.999_999_999_5 exactly, so the nanos part
            // never will be equal to 10^9.
            (0, nanos + add_ns as u32)
        } else if exp < $mant_bits {
            let secs = u64::from(mant >> ($mant_bits - exp));
            let t = <$double_ty>::from((mant << exp) & MANT_MASK);
            let nanos_offset = $mant_bits;
            let nanos_tmp = <$double_ty>::from(NANOS_PER_SEC) * t;
            let nanos = (nanos_tmp >> nanos_offset) as u32;

            let rem_mask = (1 << nanos_offset) - 1;
            let rem_msb_mask = 1 << (nanos_offset - 1);
            let rem = nanos_tmp & rem_mask;
            let is_tie = rem == rem_msb_mask;
            let is_even = (nanos & 1) == 0;
            let rem_msb = nanos_tmp & rem_msb_mask == 0;
            let add_ns = !(rem_msb || (is_even && is_tie));

            // neither `f32`, nor `f64` can represent x.999_999_999_5 exactly,
            // so the nanos part never will be equal to 10^9
            (secs, nanos + add_ns as u32)
        } else if exp < 64 {
            // the input has no fractional part
            let secs = u64::from(mant) << (exp - $mant_bits);
            (secs, 0)
        } else {
            return Err(FromFloatSecsError { kind: FromFloatSecsErrorKind::OverflowOrNan });
        };

        Ok(Duration { secs, nanos })
    }};
}

impl Duration {
    /// The checked version of [`from_secs_f32`].
    ///
    /// [`from_secs_f32`]: Duration::from_secs_f32
    ///
    /// This constructor will return an `Err` if `secs` is negative, overflows `Duration` or not finite.
    ///
    /// # Examples
    /// ```
    /// #![feature(duration_checked_float)]
    ///
    /// use std::time::Duration;
    ///
    /// let res = Duration::try_from_secs_f32(0.0);
    /// assert_eq!(res, Ok(Duration::new(0, 0)));
    /// let res = Duration::try_from_secs_f32(1e-20);
    /// assert_eq!(res, Ok(Duration::new(0, 0)));
    /// let res = Duration::try_from_secs_f32(4.2e-7);
    /// assert_eq!(res, Ok(Duration::new(0, 420)));
    /// let res = Duration::try_from_secs_f32(2.7);
    /// assert_eq!(res, Ok(Duration::new(2, 700_000_048)));
    /// let res = Duration::try_from_secs_f32(3e10);
    /// assert_eq!(res, Ok(Duration::new(30_000_001_024, 0)));
    /// // subnormal float:
    /// let res = Duration::try_from_secs_f32(f32::from_bits(1));
    /// assert_eq!(res, Ok(Duration::new(0, 0)));
    /// // conversion uses rounding
    /// let res = Duration::try_from_secs_f32(0.999e-9);
    /// assert_eq!(res, Ok(Duration::new(0, 1)));
    ///
    /// let res = Duration::try_from_secs_f32(-5.0);
    /// assert!(res.is_err());
    /// let res = Duration::try_from_secs_f32(f32::NAN);
    /// assert!(res.is_err());
    /// let res = Duration::try_from_secs_f32(2e19);
    /// assert!(res.is_err());
    ///
    /// // this method uses round to nearest, ties to even
    ///
    /// // this float represents exactly 976562.5e-9
    /// let val = f32::from_bits(0x3A80_0000);
    /// let res = Duration::try_from_secs_f32(val);
    /// assert_eq!(res, Ok(Duration::new(0, 976_562)));
    ///
    /// // this float represents exactly 2929687.5e-9
    /// let val = f32::from_bits(0x3B40_0000);
    /// let res = Duration::try_from_secs_f32(val);
    /// assert_eq!(res, Ok(Duration::new(0, 2_929_688)));
    ///
    /// // this float represents exactly 1.000_976_562_5
    /// let val = f32::from_bits(0x3F802000);
    /// let res = Duration::try_from_secs_f32(val);
    /// assert_eq!(res, Ok(Duration::new(1, 976_562)));
    ///
    /// // this float represents exactly 1.002_929_687_5
    /// let val = f32::from_bits(0x3F806000);
    /// let res = Duration::try_from_secs_f32(val);
    /// assert_eq!(res, Ok(Duration::new(1, 2_929_688)));
    /// ```
    #[unstable(feature = "duration_checked_float", issue = "83400")]
    #[inline]
    pub const fn try_from_secs_f32(secs: f32) -> Result<Duration, FromFloatSecsError> {
        try_from_secs!(
            secs = secs,
            mantissa_bits = 23,
            exponent_bits = 8,
            offset = 41,
            bits_ty = u32,
            double_ty = u64,
        )
    }

    /// The checked version of [`from_secs_f64`].
    ///
    /// [`from_secs_f64`]: Duration::from_secs_f64
    ///
    /// This constructor will return an `Err` if `secs` is negative, overflows `Duration` or not finite.
    ///
    /// # Examples
    /// ```
    /// #![feature(duration_checked_float)]
    ///
    /// use std::time::Duration;
    ///
    /// let res = Duration::try_from_secs_f64(0.0);
    /// assert_eq!(res, Ok(Duration::new(0, 0)));
    /// let res = Duration::try_from_secs_f64(1e-20);
    /// assert_eq!(res, Ok(Duration::new(0, 0)));
    /// let res = Duration::try_from_secs_f64(4.2e-7);
    /// assert_eq!(res, Ok(Duration::new(0, 420)));
    /// let res = Duration::try_from_secs_f64(2.7);
    /// assert_eq!(res, Ok(Duration::new(2, 700_000_000)));
    /// let res = Duration::try_from_secs_f64(3e10);
    /// assert_eq!(res, Ok(Duration::new(30_000_000_000, 0)));
    /// // subnormal float
    /// let res = Duration::try_from_secs_f64(f64::from_bits(1));
    /// assert_eq!(res, Ok(Duration::new(0, 0)));
    /// // conversion uses rounding
    /// let res = Duration::try_from_secs_f32(0.999e-9);
    /// assert_eq!(res, Ok(Duration::new(0, 1)));
    ///
    /// let res = Duration::try_from_secs_f64(-5.0);
    /// assert!(res.is_err());
    /// let res = Duration::try_from_secs_f64(f64::NAN);
    /// assert!(res.is_err());
    /// let res = Duration::try_from_secs_f64(2e19);
    /// assert!(res.is_err());
    ///
    /// // this method uses round to nearest, ties to even
    ///
    /// // this float represents exactly 976562.5e-9
    /// let val = f64::from_bits(0x3F50_0000_0000_0000);
    /// let res = Duration::try_from_secs_f64(val);
    /// assert_eq!(res, Ok(Duration::new(0, 976_562)));
    ///
    /// // this float represents exactly 2929687.5e-9
    /// let val = f64::from_bits(0x3F68_0000_0000_0000);
    /// let res = Duration::try_from_secs_f64(val);
    /// assert_eq!(res, Ok(Duration::new(0, 2_929_688)));
    ///
    /// // this float represents exactly 1.000_976_562_5
    /// let val = f64::from_bits(0x3FF0_0400_0000_0000);
    /// let res = Duration::try_from_secs_f64(val);
    /// assert_eq!(res, Ok(Duration::new(1, 976_562)));
    ///
    /// // this float represents exactly 1.002_929_687_5
    /// let val = f64::from_bits(0x3_FF00_C000_0000_000);
    /// let res = Duration::try_from_secs_f64(val);
    /// assert_eq!(res, Ok(Duration::new(1, 2_929_688)));
    /// ```
    #[unstable(feature = "duration_checked_float", issue = "83400")]
    #[inline]
    pub const fn try_from_secs_f64(secs: f64) -> Result<Duration, FromFloatSecsError> {
        try_from_secs!(
            secs = secs,
            mantissa_bits = 52,
            exponent_bits = 11,
            offset = 44,
            bits_ty = u64,
            double_ty = u128,
        )
    }
}
