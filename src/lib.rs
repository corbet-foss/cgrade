//! Deterministic school grades across scales, plus the modified Bavarian
//! formula for foreign-to-German conversion.
//!
//! The scale bounds per country live as data in `tables/grades.json` (see
//! `tables/README.md` for the schema, the KMK sources, and the validity
//! rules); `tests/vectors/*.json` is the executable contract every
//! language port runs. The Typst module in `typst/` derives from the same
//! table.
//!
//! Foreign → German follows the official modified Bavarian formula (KMK
//! Beschluss, "es wird nicht gerundet"):
//! `x = 1 + 3 * (Nmax - Nd) / (Nmax - Nmin)`, truncated to 1 decimal.
//! The inverse (`from_de`) truncates to 2 decimals. `convert` pivots
//! through the German scale and is a documented approximation.
//!
//! Same input always yields the same output: no models, no I/O. No words
//! or labels are produced; tone belongs to applications.

use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;

#[derive(serde::Deserialize)]
struct ScaleEntry {
    best: f64,
    worst: f64,
    pass: f64,
    step: String,
    higher_is_better: bool,
}

#[derive(serde::Deserialize)]
struct GradesFile {
    scales: HashMap<String, ScaleEntry>,
    supported: HashSet<String>,
}

static GRADES: LazyLock<GradesFile> = LazyLock::new(|| {
    serde_json::from_str(include_str!("../tables/grades.json")).expect("tables/grades.json is valid")
});

/// A grading scale with its lowercase ID.
#[derive(Debug, Clone, serde::Serialize, PartialEq)]
pub struct Scale {
    /// Lowercase scale ID, e.g. `ch`.
    pub id: String,
    /// Best achievable grade (`Nmax` in the formula).
    pub best: f64,
    /// Worst possible grade (failing).
    pub worst: f64,
    /// Lowest still-passing grade (`Nmin` in the formula).
    pub pass: f64,
    /// Advisory display hint only, never used in computation.
    pub step: String,
    /// Whether larger numbers are better (`false` for `de`/`at`).
    pub higher_is_better: bool,
}

fn lookup(sys: &str) -> Option<(&'static str, &'static ScaleEntry)> {
    let lower = sys.to_ascii_lowercase();
    GRADES
        .scales
        .get_key_value(lower.as_str())
        .map(|(key, entry)| (key.as_str(), entry))
}

/// Quantize a grade to integer thousandths (half away from zero), so the
/// division and truncation below run on integers and binary-float drift
/// can never flip a truncation boundary (see `tables/README.md`).
/// The `as` cast is exact: callers reject non-finite inputs and grades
/// are range-checked before quantization, so the value always fits.
#[allow(clippy::cast_possible_truncation)]
fn thousandths(value: f64) -> i64 {
    (value * 1000.0).round() as i64
}

/// Raw modified Bavarian formula, foreign → German.
///
/// `nmax` is the best achievable foreign grade, `nmin` the lowest passing
/// one, `nd` the achieved grade. Returns `None` for non-finite inputs, a
/// degenerate scale (`nmin == nmax`), or `nd` outside the passing
/// interval between `nmin` and `nmax` (never extrapolates).
/// Exact float equality is intentional: only a truly degenerate scale
/// divides by zero.
#[allow(clippy::float_cmp)]
#[must_use]
pub fn bavarian_to_de(nmax: f64, nmin: f64, nd: f64) -> Option<f64> {
    if !nmax.is_finite() || !nmin.is_finite() || !nd.is_finite() {
        return None;
    }
    if nmax == nmin {
        return None;
    }
    let lower = nmax.min(nmin);
    let upper = nmax.max(nmin);
    if nd < lower || nd > upper {
        return None;
    }
    let (nmax_t, nmin_t, nd_t) = (thousandths(nmax), thousandths(nmin), thousandths(nd));
    let denominator = nmax_t - nmin_t;
    if denominator == 0 {
        return None;
    }
    // x = 1 + 3*(Nmax-Nd)/(Nmax-Nmin) = (den + 3*(Nmax-Nd)) / den.
    // Numerator and denominator share their sign (both positive for
    // higher-is-better scales, both negative otherwise), so the quotient
    // is positive and floored integer division truncates it to tenths.
    // The `as` cast is exact: tenths always fits in a few dozen.
    let numerator = denominator + 3 * (nmax_t - nd_t);
    let tenths = (10 * numerator).div_euclid(denominator);
    #[allow(clippy::cast_precision_loss)]
    let result = tenths as f64 / 10.0;
    Some(result)
}

/// Raw inverse Bavarian formula, German → foreign.
///
/// `x` is a German grade in `[1.0, 4.0]`; `nmax`/`nmin` bound the foreign
/// scale as in [`bavarian_to_de`]. The result is truncated to 2 decimals.
/// Returns `None` for non-finite inputs, a degenerate scale, or `x`
/// outside the German passing interval.
/// Exact float equality is intentional: only a truly degenerate scale
/// divides by zero.
#[allow(clippy::float_cmp)]
#[must_use]
pub fn bavarian_from_de(nmax: f64, nmin: f64, x: f64) -> Option<f64> {
    if !nmax.is_finite() || !nmin.is_finite() || !x.is_finite() {
        return None;
    }
    if nmax == nmin {
        return None;
    }
    if x < 1.0 || x > 4.0 {
        return None;
    }
    let (nmax_t, nmin_t, x_t) = (thousandths(nmax), thousandths(nmin), thousandths(x));
    let delta = nmax_t - nmin_t;
    if delta == 0 {
        return None;
    }
    // Nd = Nmax - (x-1)/3*(Nmax-Nmin)
    //    = (Nmax*3000 - (x-1000)*delta) / 3_000_000,
    // truncated to hundredths via floored integer division. Valid inputs
    // keep the numerator positive (all tabled scales are non-negative).
    // The `as` cast is exact: hundredths always fits in a few thousand.
    let numerator = nmax_t * 3000 - (x_t - 1000) * delta;
    let hundredths = numerator.div_euclid(30_000);
    #[allow(clippy::cast_precision_loss)]
    let result = hundredths as f64 / 100.0;
    Some(result)
}

fn entry_scale(id: &'static str, entry: &'static ScaleEntry) -> Scale {
    Scale {
        id: id.to_owned(),
        best: entry.best,
        worst: entry.worst,
        pass: entry.pass,
        step: entry.step.clone(),
        higher_is_better: entry.higher_is_better,
    }
}

/// Grading scale for a (case-insensitive) ID, or `None` when unknown.
#[must_use]
pub fn scale(sys: &str) -> Option<Scale> {
    lookup(sys).map(|(id, entry)| entry_scale(id, entry))
}

/// Lowercase scale IDs with a table entry, sorted.
#[must_use]
pub fn available_scales() -> Vec<&'static str> {
    let mut ids: Vec<&'static str> = GRADES.scales.keys().map(String::as_str).collect();
    ids.sort_unstable();
    ids
}

/// Whether a scale ID is supported (present in the supported set).
#[must_use]
pub fn is_supported(sys: &str) -> bool {
    GRADES.supported.contains(sys.to_ascii_lowercase().as_str())
}

fn in_full_range(entry: &ScaleEntry, value: f64) -> bool {
    let lower = entry.best.min(entry.worst);
    let upper = entry.best.max(entry.worst);
    value >= lower && value <= upper
}

/// Whether a grade passes in its system: `value >= pass` where higher is
/// better, `value <= pass` where lower is better. Returns `None` for
/// unknown systems, non-finite values, and grades outside the full
/// `[worst .. best]` interval (grades that do not exist); existing but
/// failing grades yield `false`.
#[must_use]
pub fn is_pass(sys: &str, value: f64) -> Option<bool> {
    let (_, entry) = lookup(sys)?;
    if !value.is_finite() || !in_full_range(entry, value) {
        return None;
    }
    Some(if entry.higher_is_better {
        value >= entry.pass
    } else {
        value <= entry.pass
    })
}

/// Convert a passing foreign grade to the German scale with the modified
/// Bavarian formula (truncated to 1 decimal). Returns `None` for unknown
/// systems, non-finite values, and grades outside `[pass .. best]`.
#[must_use]
pub fn to_de(sys: &str, value: f64) -> Option<f64> {
    let (_, entry) = lookup(sys)?;
    bavarian_to_de(entry.best, entry.pass, value)
}

/// Convert a German grade in `[1.0, 4.0]` to a foreign scale with the
/// inverse Bavarian formula (truncated to 2 decimals). Returns `None` for
/// unknown systems and out-of-range German grades.
#[must_use]
pub fn from_de(sys: &str, x_de: f64) -> Option<f64> {
    let (_, entry) = lookup(sys)?;
    bavarian_from_de(entry.best, entry.pass, x_de)
}

/// Convert a grade from one system to another by pivoting through the
/// German scale. This is a documented approximation: the intermediate
/// 1-decimal truncation loses information, so same-system conversion is
/// not the identity. Returns `None` whenever either leg is invalid.
#[must_use]
pub fn convert(from_sys: &str, to_sys: &str, value: f64) -> Option<f64> {
    let x_de = to_de(from_sys, value)?;
    from_de(to_sys, x_de)
}

/// Parse a grade numeral: ASCII-trimmed digits with a single dot OR comma
/// decimal separator, so `"6,0"` and `"6.0"` both yield `6.0`. Strict:
/// rejects empty input, multiple or mixed separators, inner whitespace,
/// trailing text and non-finite results. Scale-free (no range check);
/// validity belongs to [`is_pass`] and the conversion functions.
#[must_use]
pub fn parse_grade(text: &str) -> Option<f64> {
    let trimmed = text.trim_matches(|c: char| c.is_ascii_whitespace());
    let (negative, unsigned) = match trimmed.strip_prefix('-') {
        Some(rest) => (true, rest),
        None => (false, trimmed.strip_prefix('+').unwrap_or(trimmed)),
    };
    if unsigned.is_empty() {
        return None;
    }
    let dots = unsigned.bytes().filter(|&b| b == b'.').count();
    let commas = unsigned.bytes().filter(|&b| b == b',').count();
    if dots + commas > 1 {
        return None;
    }
    let canonical: String = unsigned
        .bytes()
        .map(|b| {
            if b == b',' {
                Some(b'.')
            } else if b.is_ascii_digit() || b == b'.' {
                Some(b)
            } else {
                None
            }
        })
        .collect::<Option<Vec<u8>>>()?
        .into_iter()
        .map(char::from)
        .collect();
    let value: f64 = canonical.parse().ok()?;
    if !value.is_finite() {
        return None;
    }
    Some(if negative { -value } else { value })
}

/// Render a grade with an explicit decimal count, ALWAYS with a dot
/// decimal separator regardless of locale (Swiss correspondence rule:
/// `"6.0"`, never `"6,0"`). Integer-exact like the formula paths:
/// the value is quantized to thousandths (half away from zero), then the
/// requested places round half away from zero on integers, so no port can
/// diverge on binary-float boundaries. `decimals` must be `0..=3`.
/// Returns `None` for non-finite values and out-of-range precision.
#[must_use]
pub fn format_grade(value: f64, decimals: u32) -> Option<String> {
    if !value.is_finite() || decimals > 3 || value.abs() >= 1e15 {
        return None;
    }
    let thousandths = thousandths(value);
    let negative = thousandths < 0;
    let magnitude = thousandths.unsigned_abs();
    let scale = 10u64.pow(3 - decimals);
    let base = magnitude / scale;
    let rest = magnitude % scale;
    let rounded = if rest * 2 >= scale { base + 1 } else { base };
    let factor = 10u64.pow(decimals);
    let rendered = if decimals == 0 {
        rounded.to_string()
    } else {
        format!(
            "{}.{:0>width$}",
            rounded / factor,
            rounded % factor,
            width = decimals as usize
        )
    };
    Some(if negative {
        format!("-{rendered}")
    } else {
        rendered
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tables_schema() {
        assert_eq!(GRADES.scales.len(), 7, "seven country scales");
        for id in ["at", "ch", "de", "es", "fr", "it", "us"] {
            assert!(GRADES.scales.contains_key(id), "missing scale {id:?}");
            assert!(GRADES.supported.contains(id), "unsupported scale {id:?}");
        }
        assert_eq!(GRADES.supported.len(), GRADES.scales.len());
        for (id, entry) in &GRADES.scales {
            assert_eq!(id, &id.to_ascii_lowercase(), "scale ID must be lowercase: {id:?}");
            assert!(entry.best.is_finite() && entry.worst.is_finite() && entry.pass.is_finite());
            #[allow(clippy::float_cmp)]
            {
                assert_ne!(entry.best, entry.pass, "degenerate scale: {id:?}");
            }
            assert_eq!(
                entry.higher_is_better,
                entry.best > entry.pass,
                "orientation flag contradicts bounds: {id:?}"
            );
            assert!(!entry.step.is_empty(), "step hint must not be empty: {id:?}");
            let lower = entry.best.min(entry.worst);
            let upper = entry.best.max(entry.worst);
            assert!(
                entry.pass >= lower && entry.pass <= upper,
                "pass must lie inside [worst .. best]: {id:?}"
            );
        }
        let ch = &GRADES.scales["ch"];
        assert_eq!((ch.best, ch.worst, ch.pass), (6.0, 1.0, 4.0));
        let de = &GRADES.scales["de"];
        assert_eq!((de.best, de.worst, de.pass), (1.0, 6.0, 4.0));
    }

    #[test]
    fn degenerate_and_non_finite_inputs_yield_nothing() {
        assert_eq!(bavarian_to_de(4.0, 4.0, 4.0), None);
        assert_eq!(bavarian_from_de(4.0, 4.0, 2.0), None);
        for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert_eq!(bavarian_to_de(6.0, 4.0, bad), None);
            assert_eq!(bavarian_to_de(bad, 4.0, 5.0), None);
            assert_eq!(bavarian_from_de(6.0, 4.0, bad), None);
            assert_eq!(to_de("ch", bad), None);
            assert_eq!(from_de("ch", bad), None);
            assert_eq!(is_pass("ch", bad), None);
            assert_eq!(convert("ch", "fr", bad), None);
            assert_eq!(format_grade(bad, 1), None);
        }
        assert_eq!(format_grade(6.0, 4), None);
    }

    #[test]
    fn truncation_never_rounds_up() {
        // Exact 1.75 must stay 1.7, exact 1.29 must stay 1.2.
        assert_eq!(to_de("ch", 5.5), Some(1.7));
        assert_eq!(to_de("us", 3.71), Some(1.2));
        // Exact tenths survive integer truncation.
        assert_eq!(to_de("us", 2.7), Some(2.3));
        assert_eq!(to_de("de", 2.3), Some(2.3));
        assert_eq!(from_de("fr", 3.4), Some(12.0));
    }

    fn number(value: &serde_json::Value, field: &str) -> Option<f64> {
        value.get(field)?.as_f64()
    }

    fn run_vector(file: &std::path::Path, vector: &serde_json::Value) {
        let name = vector["name"].as_str().unwrap_or("<unnamed>");
        let context = format!("{} :: {name}", file.display());
        let expected = vector
            .get("expected")
            .cloned()
            .unwrap_or(serde_json::Value::Null);
        let actual: serde_json::Value = match vector["fn"].as_str().unwrap_or("") {
            "available_scales" => serde_json::to_value(available_scales()).unwrap(),
            "is_supported" => {
                let sys = vector["sys"].as_str().expect("vector needs sys");
                serde_json::Value::Bool(is_supported(sys))
            }
            "scale" => {
                let sys = vector["sys"].as_str().expect("vector needs sys");
                serde_json::to_value(scale(sys)).unwrap()
            }
            "is_pass" => {
                let sys = vector["sys"].as_str().expect("vector needs sys");
                let value = number(vector, "value").expect("vector needs value");
                serde_json::to_value(is_pass(sys, value)).unwrap()
            }
            "to_de" => {
                let sys = vector["sys"].as_str().expect("vector needs sys");
                let value = number(vector, "value").expect("vector needs value");
                serde_json::to_value(to_de(sys, value)).unwrap()
            }
            "from_de" => {
                let sys = vector["sys"].as_str().expect("vector needs sys");
                let value = number(vector, "value").expect("vector needs value");
                serde_json::to_value(from_de(sys, value)).unwrap()
            }
            "convert" => {
                let from = vector["from"].as_str().expect("vector needs from");
                let to = vector["to"].as_str().expect("vector needs to");
                let value = number(vector, "value").expect("vector needs value");
                serde_json::to_value(convert(from, to, value)).unwrap()
            }
            "parse_grade" => {
                let text = vector["text"].as_str().expect("vector needs text");
                serde_json::to_value(parse_grade(text)).unwrap()
            }
            "format_grade" => {
                let value = number(vector, "value").expect("vector needs value");
                let decimals = u32::try_from(
                    vector["decimals"].as_u64().expect("vector needs decimals"),
                )
                .expect("decimals fits u32");
                serde_json::to_value(format_grade(value, decimals)).unwrap()
            }
            "bavarian_to_de" => {
                let (nmax, nmin, value) = (
                    number(vector, "nmax").expect("vector needs nmax"),
                    number(vector, "nmin").expect("vector needs nmin"),
                    number(vector, "value").expect("vector needs value"),
                );
                serde_json::to_value(bavarian_to_de(nmax, nmin, value)).unwrap()
            }
            "bavarian_from_de" => {
                let (nmax, nmin, value) = (
                    number(vector, "nmax").expect("vector needs nmax"),
                    number(vector, "nmin").expect("vector needs nmin"),
                    number(vector, "value").expect("vector needs value"),
                );
                serde_json::to_value(bavarian_from_de(nmax, nmin, value)).unwrap()
            }
            other => panic!("{context}: unknown fn {other:?}"),
        };
        assert_eq!(actual, expected, "{context}");
    }

    #[test]
    fn conformance_vectors() {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/vectors");
        let mut files: Vec<std::path::PathBuf> = std::fs::read_dir(&dir)
            .expect("tests/vectors exists")
            .map(|entry| entry.expect("readable entry").path())
            .collect();
        files.sort();
        assert!(!files.is_empty(), "no vector files in tests/vectors");
        let mut count = 0;
        for file in &files {
            let raw = std::fs::read_to_string(file).expect("vector file is readable");
            let vectors: Vec<serde_json::Value> =
                serde_json::from_str(&raw).expect("vector file is valid JSON");
            for vector in &vectors {
                run_vector(file, vector);
                count += 1;
            }
        }
        assert!(count > 0, "no vectors ran");
    }
}
