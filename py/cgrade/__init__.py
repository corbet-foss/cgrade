"""Deterministic school grades across scales, plus the modified Bavarian
formula for foreign-to-German conversion.

Pure-Python port of the cgrade Rust crate: standard library only, no I/O.
Behavior is defined by ``tables/*.json`` at the repository root;
``tests/vectors/*.json`` is the shared conformance suite
(``py/scripts/conformance.py``). No words or labels are produced; tone
belongs to applications.
"""

import math

from ._tables import TABLES

_RAW = TABLES["grades"]

_SCALES = _RAW["scales"]
_SUPPORTED = _RAW["supported"]

_CANONICAL_BY_LOWER = {key.lower(): key for key in _SCALES}


def _lookup(sys):
    key = _CANONICAL_BY_LOWER.get(sys.lower())
    if key is None:
        return None
    return key, _SCALES[key]


def _thousandths(value):
    """Quantize a grade to integer thousandths (half away from zero).

    All further arithmetic runs on these integers, so binary-float drift
    can never flip a truncation boundary (see ``tables/README.md``).
    """
    return math.copysign(math.floor(abs(value) * 1000 + 0.5), value)


def bavarian_to_de(nmax, nmin, nd):
    """Raw modified Bavarian formula, foreign → German (truncated to 1 decimal).

    Returns ``None`` for non-finite inputs, a degenerate scale
    (``nmin == nmax``), or ``nd`` outside the passing interval between
    ``nmin`` and ``nmax`` (never extrapolates).
    """
    if not all(type(v) in (int, float) and math.isfinite(v) for v in (nmax, nmin, nd)):
        return None
    if nmax == nmin:
        return None
    if nd < min(nmax, nmin) or nd > max(nmax, nmin):
        return None
    nmax_t = int(_thousandths(nmax))
    nmin_t = int(_thousandths(nmin))
    nd_t = int(_thousandths(nd))
    denominator = nmax_t - nmin_t
    if denominator == 0:
        return None
    # x = 1 + 3*(Nmax-Nd)/(Nmax-Nmin) = (den + 3*(Nmax-Nd)) / den.
    # Numerator and denominator share their sign, so the quotient is
    # positive and floored integer division truncates it to tenths.
    numerator = denominator + 3 * (nmax_t - nd_t)
    return (10 * numerator) // denominator / 10


def bavarian_from_de(nmax, nmin, x):
    """Raw inverse Bavarian formula, German → foreign (truncated to 2 decimals).

    Returns ``None`` for non-finite inputs, a degenerate scale, or ``x``
    outside the German passing interval ``[1.0, 4.0]``.
    """
    if not all(type(v) in (int, float) and math.isfinite(v) for v in (nmax, nmin, x)):
        return None
    if nmax == nmin:
        return None
    if x < 1 or x > 4:
        return None
    nmax_t = int(_thousandths(nmax))
    delta = nmax_t - int(_thousandths(nmin))
    if delta == 0:
        return None
    # Nd = Nmax - (x-1)/3*(Nmax-Nmin)
    #    = (Nmax*3000 - (x-1000)*delta) / 3_000_000,
    # truncated to hundredths. Valid inputs keep the numerator positive.
    numerator = nmax_t * 3000 - (int(_thousandths(x)) - 1000) * delta
    return numerator // 30_000 / 100


def scale(sys):
    """Grading scale for a (case-insensitive) ID, or ``None`` when unknown."""
    found = _lookup(sys)
    if found is None:
        return None
    key, entry = found
    return {
        "id": key,
        "best": entry["best"],
        "worst": entry["worst"],
        "pass": entry["pass"],
        "step": entry["step"],
        "higher_is_better": entry["higher_is_better"],
    }


def available_scales():
    """Lowercase scale IDs with a table entry, sorted."""
    return sorted(_SCALES)


def is_supported(sys):
    """Whether a scale ID is supported (present in the supported set)."""
    return sys.lower() in _SUPPORTED


def _in_full_range(entry, value):
    return min(entry["best"], entry["worst"]) <= value <= max(entry["best"], entry["worst"])


def is_pass(sys, value):
    """Whether a grade passes in its system.

    Returns ``None`` for unknown systems, non-finite values, and grades
    outside the full ``[worst .. best]`` interval; existing but failing
    grades yield ``False``.
    """
    found = _lookup(sys)
    if found is None:
        return None
    if type(value) not in (int, float) or not math.isfinite(value):
        return None
    _, entry = found
    if not _in_full_range(entry, value):
        return None
    return value >= entry["pass"] if entry["higher_is_better"] else value <= entry["pass"]


def to_de(sys, value):
    """Convert a passing foreign grade to the German scale (truncated to 1 decimal)."""
    found = _lookup(sys)
    if found is None:
        return None
    _, entry = found
    return bavarian_to_de(entry["best"], entry["pass"], value)


def from_de(sys, x_de):
    """Convert a German grade in ``[1.0, 4.0]`` to a foreign scale (truncated to 2 decimals)."""
    found = _lookup(sys)
    if found is None:
        return None
    _, entry = found
    return bavarian_from_de(entry["best"], entry["pass"], x_de)


def convert(from_sys, to_sys, value):
    """Convert a grade between systems by pivoting through the German scale.

    This is a documented approximation: the intermediate 1-decimal
    truncation loses information, so same-system conversion is not the
    identity. Returns ``None`` whenever either leg is invalid.
    """
    x_de = to_de(from_sys, value)
    if x_de is None:
        return None
    return from_de(to_sys, x_de)


_ASCII_WS = " \t\n\r\x0b\x0c"


def parse_grade(text):
    """Parse a grade numeral: ASCII-trimmed digits with a single dot OR comma
    decimal separator, so ``"6,0"`` and ``"6.0"`` both yield ``6.0``. Strict:
    rejects empty input, multiple or mixed separators, inner whitespace,
    trailing text and non-finite results. Scale-free (no range check).
    """
    if type(text) is not str:
        return None
    trimmed = text.strip(_ASCII_WS)
    if trimmed.startswith("-"):
        negative, unsigned = True, trimmed[1:]
    elif trimmed.startswith("+"):
        negative, unsigned = False, trimmed[1:]
    else:
        negative, unsigned = False, trimmed
    if not unsigned:
        return None
    if unsigned.count(".") + unsigned.count(",") > 1:
        return None
    canonical = unsigned.replace(",", ".")
    if any(char not in "0123456789." for char in canonical):
        return None
    if canonical == ".":
        return None
    try:
        value = float(canonical)
    except ValueError:
        return None
    if not math.isfinite(value):
        return None
    return -value if negative else value


def format_grade(value, decimals):
    """Render a grade with an explicit decimal count, ALWAYS with a dot
    decimal separator regardless of locale (``"6.0"``, never ``"6,0"``).
    Integer-exact like the formula paths: the value is quantized to
    thousandths (half away from zero -- never ``round()``, which is
    half-even), then the requested places round half away from zero on
    integers. ``decimals`` must be ``0..=3``. Returns ``None`` for
    non-finite values, out-of-range precision, and magnitudes ``>= 1e15``.
    A negative-zero quantization (e.g. ``-0.0004``) renders WITHOUT a
    minus sign, mirroring the Rust reference.
    """
    if type(value) not in (int, float) or not math.isfinite(value):
        return None
    if type(decimals) is not int or decimals < 0 or decimals > 3:
        return None
    if abs(value) >= 1e15:
        return None
    # Quantize to integer thousandths, half away from zero, mirroring
    # ``(value * 1000.0).round() as i64`` bit-for-bit. The float product is
    # correctly rounded exactly like Rust's; the half-away step then runs
    # on exact integers. NOTE: not ``_thousandths()`` (``floor(x + 0.5)``):
    # past 2**52 the ``+ 0.5`` itself rounds and flips the boundary
    # (e.g. ``-7839950073357.209`` at ``decimals=3``). The subtractions
    # below are exact by Sterbenz, so the ``>= 0.5`` test never wobbles.
    scaled = float(value) * 1000.0
    if scaled >= 0:
        floored = math.floor(scaled)
        thousandths = floored + 1 if scaled - floored >= 0.5 else floored
    else:
        ceiled = math.ceil(scaled)
        thousandths = ceiled - 1 if ceiled - scaled >= 0.5 else ceiled
    negative = thousandths < 0
    magnitude = abs(thousandths)
    scale = 10 ** (3 - decimals)
    base, rest = divmod(magnitude, scale)
    rounded = base + 1 if rest * 2 >= scale else base
    if decimals == 0:
        rendered = str(rounded)
    else:
        factor = 10 ** decimals
        rendered = f"{rounded // factor}.{rounded % factor:0{decimals}d}"
    return "-" + rendered if negative else rendered


__all__ = ["available_scales", "is_supported", "scale", "is_pass", "to_de", "from_de", "convert", "bavarian_to_de", "bavarian_from_de", "parse_grade", "format_grade"]
