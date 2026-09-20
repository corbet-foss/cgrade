/// Cross-language conformance gate for the Typst port: runs every
/// `tests/vectors/*.json` vector through `typst/grade.typ` and compares
/// with `expected` exactly. Any mismatch aborts compilation with the
/// vector name; reaching the summary line means the gate is green.
///
/// Run from the repository root:
///   typst compile --root . tests/conformance.typ /tmp/opencode/conformance.pdf
#import "../typst/grade.typ" as api

#let vectors = json("vectors/grades.json")

#let run-vector(file, vector) = {
  let what = file + " :: " + vector.at("name", default: "<unnamed>")
  let expected = vector.at("expected", default: none)
  let actual = if vector.at("fn") == "available_scales" {
    api.available-scales()
  } else if vector.at("fn") == "is_supported" {
    api.is-supported(vector.at("sys"))
  } else if vector.at("fn") == "scale" {
    api.scale(vector.at("sys"))
  } else if vector.at("fn") == "is_pass" {
    api.is-pass(vector.at("sys"), vector.at("value"))
  } else if vector.at("fn") == "to_de" {
    api.to-de(vector.at("sys"), vector.at("value"))
  } else if vector.at("fn") == "from_de" {
    api.from-de(vector.at("sys"), vector.at("value"))
  } else if vector.at("fn") == "convert" {
    api.convert(vector.at("from"), vector.at("to"), vector.at("value"))
  } else if vector.at("fn") == "bavarian_to_de" {
    api.bavarian-to-de(vector.at("nmax"), vector.at("nmin"), vector.at("value"))
  } else if vector.at("fn") == "bavarian_from_de" {
    api.bavarian-from-de(vector.at("nmax"), vector.at("nmin"), vector.at("value"))
  } else if vector.at("fn") == "parse_grade" {
    api.parse-grade(vector.at("text"))
  } else if vector.at("fn") == "format_grade" {
    api.format-grade(vector.at("value"), vector.at("decimals"))
  } else {
    panic("unknown fn " + repr(vector.at("fn")))
  }
  assert(actual == expected, message: what + ": " + repr(actual) + " vs " + repr(expected))
}

#for vector in vectors {
  run-vector("grades.json", vector)
}

Typst conformance green: #vectors.len() vectors across 1 file.
