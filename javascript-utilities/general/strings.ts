/*
# @bengineering/strings

Extensions to string operations and formatting

- See [Intl](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Intl)

License = MIT
*/

export function pluralise(items: number | Array<any>, word: string): string {
    const length = typeof items === "number" ? items : items.length;
    if (length === 1) {
        return word
    } else {
        // `party => parties`
        if (word.endsWith("y")) {
            return word.slice(-1) + "ies"
        } else {
            return word + "s"
        }
    }
}

export function strip_prefix(word: string, prefix: string): string | null {
    return word.startsWith(prefix) ? word.slice(prefix.length) : null
}

export function strip_suffix(word: string, suffix: string): string | null {
    return word.endsWith(suffix) ? word.slice(0, -suffix.length) : null
}

/** TODO: *, _, $ for markdown? */
export function mapping(a: string): string | null {
    switch (a) {
        case '(': return ')';
        case '[': return ']';
        case '{': return '}';
        case '<': return '>';
        case '"':
        case '\'':
        case '`':
            return a;
        default: return null
    }
}

export function surround(on: string, lhs: string): string {
    const rhs = mapping(lhs);
    if (rhs === null) {
        // TODO error?
        return on
    } else {
        return lhs + on.trim() + rhs;
    }
}

/** TODO brackets, i.e. eg etc */
export function separate_sentences(text: string): string[] {
    const parts: Array<string> = [];
    let start = 0;
    for (let i = 0; i < text.length; i++) {
        const isBreak = "?!".includes(text[i]) || (text[i] === "." && (text[i + 1] ?? " ") === " ");
        if (isBreak) {
            parts.push(text.slice(start, i).trim());
            start = i + 1;
        }
    }
    const rest = text.slice(start).trim();
    if (rest.length) parts.push(rest);
    return parts
}

// TODO isTitleCase, toCamelCase
export const cases = {
    isTitleCase(s: string): boolean { throw Error("TODO!") }
}