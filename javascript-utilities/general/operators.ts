/*
# @bengineering/operators

...

License = MIT
*/

// TODO otherwise as function ...?
export function map<T, U>(x: T, map: (cb: T) => U, otherwise: T | U = x): T | U {
    try {
        const mapped = map(x);
        return mapped ?? otherwise;
    } catch (_error) {
        return otherwise
    }
}

export function tryFn<T>(fn: () => T): { ok: T } | { err: Error } {
    try {
        const ok = fn();
        return { ok }
    } catch (err) {
        return { err }
    }
}