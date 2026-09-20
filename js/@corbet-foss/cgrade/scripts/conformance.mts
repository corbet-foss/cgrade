/**
 * TypeScript side of the cross-language conformance gate: runs every
 * `tests/vectors/*.json` vector through `src/index.ts` and compares with
 * `expected` exactly. Exits non-zero with the first mismatch.
 *
 * Run from the package root:
 *   bun ./scripts/conformance.mts
 */
import { readdirSync, readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { dirname, join, resolve } from 'node:path';
import {
    availableScales,
    bavarianFromDe,
    bavarianToDe,
    convert,
    formatGrade,
    fromDe,
    isPass,
    isSupported,
    parseGrade,
    scale,
    toDe,
} from '../src/index.js';

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), '../../../..');

interface Vector {
    name: string;
    fn: string;
    sys?: string;
    from?: string;
    to?: string;
    value?: number;
    nmax?: number;
    nmin?: number;
    text?: string;
    decimals?: number;
    expected: unknown;
}

function needNumber(vector: Vector, field: 'value' | 'nmax' | 'nmin', what: string): number {
    const got = vector[field];
    if (typeof got !== 'number') throw new Error(`${what}: vector needs ${field}`);
    return got;
}

function needSys(vector: Vector, field: 'sys' | 'from' | 'to', what: string): string {
    const got = vector[field];
    if (typeof got !== 'string') throw new Error(`${what}: vector needs ${field}`);
    return got;
}

function needText(vector: Vector, what: string): string {
    if (typeof vector.text !== 'string') throw new Error(`${what}: vector needs text`);
    return vector.text;
}

function needDecimals(vector: Vector, what: string): number {
    if (typeof vector.decimals !== 'number') throw new Error(`${what}: vector needs decimals`);
    return vector.decimals;
}

function runVector(file: string, vector: Vector): void {
    const what = `${file} :: ${vector.name}`;
    let actual: unknown;
    switch (vector.fn) {
        case 'available_scales':
            actual = availableScales();
            break;
        case 'is_supported':
            actual = isSupported(needSys(vector, 'sys', what));
            break;
        case 'scale':
            actual = scale(needSys(vector, 'sys', what));
            break;
        case 'is_pass':
            actual = isPass(needSys(vector, 'sys', what), needNumber(vector, 'value', what));
            break;
        case 'to_de':
            actual = toDe(needSys(vector, 'sys', what), needNumber(vector, 'value', what));
            break;
        case 'from_de':
            actual = fromDe(needSys(vector, 'sys', what), needNumber(vector, 'value', what));
            break;
        case 'convert':
            actual = convert(
                needSys(vector, 'from', what),
                needSys(vector, 'to', what),
                needNumber(vector, 'value', what),
            );
            break;
        case 'bavarian_to_de':
            actual = bavarianToDe(
                needNumber(vector, 'nmax', what),
                needNumber(vector, 'nmin', what),
                needNumber(vector, 'value', what),
            );
            break;
        case 'bavarian_from_de':
            actual = bavarianFromDe(
                needNumber(vector, 'nmax', what),
                needNumber(vector, 'nmin', what),
                needNumber(vector, 'value', what),
            );
            break;
        case 'parse_grade':
            actual = parseGrade(needText(vector, what));
            break;
        case 'format_grade':
            actual = formatGrade(needNumber(vector, 'value', what), needDecimals(vector, what));
            break;
        default:
            throw new Error(`${what}: unknown fn ${vector.fn}`);
    }
    const got = JSON.stringify(actual) ?? 'undefined';
    const want = JSON.stringify(vector.expected) ?? 'undefined';
    if (got !== want) throw new Error(`${what}: ${got} vs ${want}`);
}

const dir = join(ROOT, 'tests/vectors');
const files = readdirSync(dir)
    .filter((file) => file.endsWith('.json'))
    .sort();
if (files.length === 0) throw new Error('no vector files in tests/vectors');
let count = 0;
for (const file of files) {
    const vectors = JSON.parse(readFileSync(join(dir, file), 'utf8')) as Vector[];
    for (const vector of vectors) {
        runVector(file, vector);
        count += 1;
    }
}
console.log(`TS conformance green: ${count} vectors across ${files.length} files`);
