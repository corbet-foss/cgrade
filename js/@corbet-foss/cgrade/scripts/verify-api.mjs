// This module also runs in a real browser with no Node shims.
export function verify(api) {
    const check = (actual, expected) => {
        if (JSON.stringify(actual) !== JSON.stringify(expected)) {
            throw new Error(`${JSON.stringify(actual)} !== ${JSON.stringify(expected)}`);
        }
    };
    check(api.toDe('ch', 5.5), 1.7);
    check(api.fromDe('ch', 1.7), 5.53);
    check(api.toDe('ch', 3.5), null);
    check(api.convert('ch', 'fr', 5.5), 17.66);
    check(api.parseGrade('6,0'), 6);
    check(api.parseGrade(' 5,5 '), 5.5);
    check(api.parseGrade('6,0!'), null);
    check(api.formatGrade(5.25, 1), '5.3');
    check(api.formatGrade(-0.0004, 1), '0.0');
    check(api.formatGrade(6, 4), null);
}
