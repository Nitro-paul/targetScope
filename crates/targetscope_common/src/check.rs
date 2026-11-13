// doc/shared_libraries.md#value-checks

use std::fmt::Display;

use serde::de::Error;
use serde::{Deserialize, Deserializer};

fn checked<'de, D, T>(d: D, ok: impl Fn(&T) -> bool, rule: &str) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de> + Display,
{
    let v = T::deserialize(d)?;
    if ok(&v) {
        Ok(v)
    } else {
        Err(D::Error::custom(format!("{v} {rule}")))
    }
}

fn checked_opt<'de, D, T>(d: D, ok: impl Fn(&T) -> bool, rule: &str) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de> + Display,
{
    match Option::<T>::deserialize(d)? {
        Some(v) if !ok(&v) => Err(D::Error::custom(format!("{v} {rule}"))),
        v => Ok(v),
    }
}

// doc/interfaces.md#shot
pub fn angle_turns<'de, D: Deserializer<'de>>(d: D) -> Result<f64, D::Error> {
    checked(d, |v: &f64| (0.0..1.0).contains(v), "is not in 0 ≤ angle_turns < 1")
}

pub fn non_negative<'de, D: Deserializer<'de>>(d: D) -> Result<f64, D::Error> {
    checked(d, |v: &f64| *v >= 0.0, "is below 0")
}

pub fn positive<'de, D: Deserializer<'de>>(d: D) -> Result<f64, D::Error> {
    checked(d, |v: &f64| *v > 0.0, "is not above 0")
}

pub fn unit<'de, D: Deserializer<'de>>(d: D) -> Result<f64, D::Error> {
    checked(d, |v: &f64| (0.0..=1.0).contains(v), "is not in 0..1")
}

pub fn opt_unit<'de, D: Deserializer<'de>>(d: D) -> Result<Option<f64>, D::Error> {
    checked_opt(d, |v: &f64| (0.0..=1.0).contains(v), "is not in 0..1")
}

pub fn count<'de, D: Deserializer<'de>>(d: D) -> Result<i64, D::Error> {
    checked(d, |v: &i64| *v >= 0, "is below 0")
}

pub fn opt_count<'de, D: Deserializer<'de>>(d: D) -> Result<Option<i64>, D::Error> {
    checked_opt(d, |v: &i64| *v >= 0, "is below 0")
}

/// `null` is kept as `Some(None)`, a missing field (with `default`) is `None`.
pub fn opt_opt_count<'de, D: Deserializer<'de>>(d: D) -> Result<Option<Option<i64>>, D::Error> {
    opt_count(d).map(Some)
}

// doc/interfaces.md#sessionshot
pub fn number<'de, D: Deserializer<'de>>(d: D) -> Result<i64, D::Error> {
    checked(d, |v: &i64| *v >= 1, "is below 1")
}

pub fn opt_number<'de, D: Deserializer<'de>>(d: D) -> Result<Option<i64>, D::Error> {
    checked_opt(d, |v: &i64| *v >= 1, "is below 1")
}

pub fn name<'de, D: Deserializer<'de>>(d: D) -> Result<String, D::Error> {
    let v = String::deserialize(d)?;
    if v.is_empty() {
        return Err(D::Error::custom("name is empty"));
    }
    Ok(v)
}

// doc/interfaces.md#session
pub const NOTES_MAX_CHARS: usize = 10_000;

pub fn notes<'de, D: Deserializer<'de>>(d: D) -> Result<Option<String>, D::Error> {
    let v = Option::<String>::deserialize(d)?;
    if v.as_ref().is_some_and(|n| n.chars().count() > NOTES_MAX_CHARS) {
        return Err(D::Error::custom(format!("notes are longer than {NOTES_MAX_CHARS} characters")));
    }
    Ok(v)
}

pub fn non_empty<'de, D, T>(d: D) -> Result<Vec<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    let v = Vec::<T>::deserialize(d)?;
    if v.is_empty() {
        return Err(D::Error::custom("list is empty"));
    }
    Ok(v)
}
