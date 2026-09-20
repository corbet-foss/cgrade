/**
 * Deterministic school grades across scales, plus the modified Bavarian
 * formula for foreign-to-German conversion.
 *
 * Pure TypeScript port of the cgrade Rust crate: zero dependencies, zero
 * Node APIs, synchronous, no I/O. Behavior is defined by `tables/*.json`
 * at the repository root; `tests/vectors/*.json` is the shared conformance
 * suite. No words or labels are produced; tone belongs to applications.
 */
import { GRADES_TABLE } from './generated/tables.ts';

export interface Scale {
    id: string;
    best: number;
    worst: number;
    pass: number;
    step: string;
    higher_is_better: boolean;
}

interface ScaleEntry {
    best: number;
    worst: number;
    pass: number;
    step: string;
    higher_is_better: boolean;
}

interface GradesTable {
    scales: Record<string, ScaleEntry>;
    supported: string[];
}

const table = GRADES_TABLE as GradesTable;

function lookup(sys: string): { id: string; entry: ScaleEntry } | null {
    const lower = sys.toLowerCase();
    if (Object.hasOwn(table.scales, lower)) {
        const entry = table.scales[lower] as ScaleEntry;
        return { id: lower, entry };
    }
    return null;
}

/**
 * Quantize a grade to integer thousandths (half away from zero), so the
 * division and truncation below run on integers and binary-float drift
 * can never flip a truncation boundary (see `tables/README.md`).
 * Operands stay far below 2^53, so every intermediate integer is exact
 * and `Math.floor` on the single correctly-rounded quotient truncates
 * exactly (any non-integral quotient stays >= 1/denominator away from
 * the next integer, far above float error).
 */
function thousandths(value: number): number {
    return Math.sign(value) * Math.round(Math.abs(value) * 1000);
}

/**
 * Raw modified Bavarian formula, foreign → German: `nmax` is the best
 * achievable foreign grade, `nmin` the lowest passing one, `nd` the
 * achieved grade. Returns `null` for non-finite inputs, a degenerate
 * scale (`nmin === nmax`), or `nd` outside the passing interval between
 * `nmin` and `nmax` (never extrapolates).
 */
export function bavarianToDe(nmax: number, nmin: number, nd: number): number | null {
    if (!Number.isFinite(nmax) || !Number.isFinite(nmin) || !Number.isFinite(nd)) return null;
    if (nmax === nmin) return null;
    if (nd < Math.min(nmax, nmin) || nd > Math.max(nmax, nmin)) return null;
    const nmaxT = thousandths(nmax);
    const nminT = thousandths(nmin);
    const ndT = thousandths(nd);
    const denominator = nmaxT - nminT;
    if (denominator === 0) return null;
    const numerator = denominator + 3 * (nmaxT - ndT);
    return Math.floor((10 * numerator) / denominator) / 10;
}

/**
 * Raw inverse Bavarian formula, German → foreign: `x` is a German grade
 * in `[1.0, 4.0]`. The result is truncated to 2 decimals. Returns `null`
 * for non-finite inputs, a degenerate scale, or `x` outside the German
 * passing interval.
 */
export function bavarianFromDe(nmax: number, nmin: number, x: number): number | null {
    if (!Number.isFinite(nmax) || !Number.isFinite(nmin) || !Number.isFinite(x)) return null;
    if (nmax === nmin) return null;
    if (x < 1 || x > 4) return null;
    const nmaxT = thousandths(nmax);
    const delta = nmaxT - thousandths(nmin);
    if (delta === 0) return null;
    const numerator = nmaxT * 3000 - (thousandths(x) - 1000) * delta;
    return Math.floor(numerator / 30000) / 100;
}

/** Grading scale for a (case-insensitive) ID, or `null` when unknown. */
export function scale(sys: string): Scale | null {
    const found = lookup(sys);
    if (found === null) return null;
    return {
        id: found.id,
        best: found.entry.best,
        worst: found.entry.worst,
        pass: found.entry.pass,
        step: found.entry.step,
        higher_is_better: found.entry.higher_is_better,
    };
}

/** Lowercase scale IDs with a table entry, sorted. */
export function availableScales(): string[] {
    return Object.keys(table.scales).sort();
}

/** Whether a scale ID is supported (present in the supported set). */
export function isSupported(sys: string): boolean {
    return table.supported.includes(sys.toLowerCase());
}

function inFullRange(entry: ScaleEntry, value: number): boolean {
    return value >= Math.min(entry.best, entry.worst) && value <= Math.max(entry.best, entry.worst);
}

/**
 * Whether a grade passes in its system: `value >= pass` where higher is
 * better, `value <= pass` where lower is better. Returns `null` for
 * unknown systems, non-finite values, and grades outside the full
 * `[worst .. best]` interval; existing but failing grades yield `false`.
 */
export function isPass(sys: string, value: number): boolean | null {
    const found = lookup(sys);
    if (found === null) return null;
    if (!Number.isFinite(value) || !inFullRange(found.entry, value)) return null;
    return found.entry.higher_is_better ? value >= found.entry.pass : value <= found.entry.pass;
}

/**
 * Convert a passing foreign grade to the German scale with the modified
 * Bavarian formula (truncated to 1 decimal). Returns `null` for unknown
 * systems, non-finite values, and grades outside `[pass .. best]`.
 */
export function toDe(sys: string, value: number): number | null {
    const found = lookup(sys);
    if (found === null) return null;
    return bavarianToDe(found.entry.best, found.entry.pass, value);
}

/**
 * Convert a German grade in `[1.0, 4.0]` to a foreign scale with the
 * inverse Bavarian formula (truncated to 2 decimals). Returns `null` for
 * unknown systems and out-of-range German grades.
 */
export function fromDe(sys: string, xDe: number): number | null {
    const found = lookup(sys);
    if (found === null) return null;
    return bavarianFromDe(found.entry.best, found.entry.pass, xDe);
}

/**
 * Convert a grade from one system to another by pivoting through the
 * German scale. This is a documented approximation: the intermediate
 * 1-decimal truncation loses information, so same-system conversion is
 * not the identity. Returns `null` whenever either leg is invalid.
 */
export function convert(fromSys: string, toSys: string, value: number): number | null {
    const xDe = toDe(fromSys, value);
    if (xDe === null) return null;
    return fromDe(toSys, xDe);
}

/** ASCII whitespace per Rust `char::is_ascii_whitespace` (U+0009–U+000D, U+0020). */
const ASCII_WS = '[\\t\\n\\f\\r \\v]';
const TRIM_ASCII_RE = new RegExp(`^${ASCII_WS}+|${ASCII_WS}+$`, 'g');

/**
 * Parse a grade numeral: ASCII-trimmed digits with a single dot OR comma
 * decimal separator, so `"6,0"` and `"6.0"` both yield `6`. Strict:
 * rejects empty input, multiple or mixed separators, inner whitespace,
 * trailing text and non-finite results. Scale-free (no range check).
 */
export function parseGrade(text: string): number | null {
    if (typeof text !== 'string') return null;
    const trimmed = text.replace(TRIM_ASCII_RE, '');
    let negative = false;
    let unsigned = trimmed;
    if (unsigned.startsWith('-')) {
        negative = true;
        unsigned = unsigned.slice(1);
    } else if (unsigned.startsWith('+')) {
        unsigned = unsigned.slice(1);
    }
    if (unsigned.length === 0) return null;
    let dots = 0;
    let commas = 0;
    for (const ch of unsigned) {
        if (ch === '.') dots += 1;
        else if (ch === ',') commas += 1;
        else if (ch < '0' || ch > '9') return null;
    }
    if (dots + commas > 1) return null;
    const canonical = unsigned.replace(',', '.');
    if (canonical === '.') return null;
    const value = Number(canonical);
    if (!Number.isFinite(value)) return null;
    return negative ? -value : value;
}

/**
 * Render a grade with an explicit decimal count, ALWAYS with a dot
 * decimal separator regardless of locale (`"6.0"`, never `"6,0"`).
 * Integer-exact like the formula paths: the value is quantized to
 * thousandths (half away from zero), then the requested places round
 * half away from zero on integers. `decimals` must be `0..=3`.
 * Returns `null` for non-finite values, out-of-range precision, and
 * magnitudes `>= 1e15`. A negative-zero quantization (e.g. `-0.0004`)
 * renders WITHOUT a minus sign, mirroring the Rust reference.
 */
export function formatGrade(value: number, decimals: number): string | null {
    if (!Number.isFinite(value)) return null;
    if (!Number.isInteger(decimals) || decimals < 0 || decimals > 3) return null;
    if (Math.abs(value) >= 1e15) return null;
    // Half away from zero via the shared helper (sign handled explicitly:
    // Math.round alone is half-up toward +inf, wrong for negatives).
    const quant = thousandths(value);
    const negative = quant < 0;
    // Exact integer arithmetic from here on (|quant| < 1e18 < 2^63).
    const magnitude = BigInt(Math.abs(quant));
    const scaleF = 10n ** BigInt(3 - decimals);
    const base = magnitude / scaleF;
    const rest = magnitude % scaleF;
    const rounded = rest * 2n >= scaleF ? base + 1n : base;
    let rendered: string;
    if (decimals === 0) {
        rendered = rounded.toString();
    } else {
        const factor = 10n ** BigInt(decimals);
        const frac = (rounded % factor).toString().padStart(decimals, '0');
        rendered = `${(rounded / factor).toString()}.${frac}`;
    }
    return negative ? `-${rendered}` : rendered;
}
