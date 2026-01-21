/*
# @bengineering/http-utilities

Utilities for HTML on the server

License = MIT
*/

// TODO HTML escape
/** Create an HTML text representation of an element */
export function createElement(
    name: string,
    attributes: Record<string, string | true>,
    children: string[]
): string {
    let s = `<${name}`;
    for (const key in attributes) {
        s += " ";
        s += key;
        const value = attributes[key];
        if (typeof value === "string") {
            s += "=";
            s += "\"";
            s += escapeHTML(value);
            s += "\"";
        }
    }
    s += ">";
    // TODO more here
    if (name === "img") return s;
    for (const child of children) s += child;
    s += `</${name}>`;
    return s
}

/** Escapes values in HTML */
export function escapeHTML(text: string) {
    let start = 0;
    let str = "";
    for (let i = 0; i < text.length; i++) {
        const chr = text[i];
        let escape = "";
        switch (chr) {
            case "&": escape = "&amp;"; break;
            case "\"": escape = "&quot;"; break;
            case "<": escape = "&lt;"; break;
            case ">": escape = "&gt;"; break;
            default: break;
        }
        if (escape) {
            str += text.slice(start) + escape;
            start = i + 1;
        }
    }
    if (str) {
        return str + text.slice(start)
    } else {
        return text
    }
}

// TODO
export interface Metadata {

}

export type Rule = { selector: string, declarations: [string, string][] };

export interface HTMLOptions {
    title: string,
    metadata?: Metadata,
    fixed_zoom?: boolean,
    styles?: (string | URL | Rule)[]
    scripts?: (string | URL)[]
}

/** Build a shell around some HTML content. Handle *prelude*, title, metadata, scripts, styles etc */
export function htmlShell(body: string, options: HTMLOptions): string {
    let head = "";

    if (options.styles) {
        // TODO append content and rules
        let styles = "";
        for (const entry of options.styles) {
            if (typeof entry === "string") {
                styles += entry
            } else if (entry instanceof URL) {
                head += `<link type="text/css" rel="stylesheet" href="${entry}">`
            } else {
                const declarations = entry.declarations.map(([name, value]) => `${name}:${value}`);
                styles += `${entry.selector}{${declarations.join(";")}}`
            }
        }
        if (styles) {
            head += `<style>${styles}</style>`;
        }
    }
    if (options.scripts) {
        // TODO append content and rules
        for (const entry of options.scripts) {
            if (typeof entry === "string") {
                head += `<script type="module">${entry}</script>`;
            } else if (entry instanceof URL) {
                head += `<link type="text/css" rel="stylesheet" href="${entry}">`
            }
        }
    }
    return `<!DOCTYPE html><html lang="en"><head><meta charset="UTF-8"><meta name="viewport" content="width=device-width,initial-scale=1.0${options.fixed_zoom ? ", maximum-scale=1.0, user-scalable=0" : ""}"><title>${options.title}</title>${head}</head><body>${body}</body></html>`
}

export type Data = string | number | boolean;
export type RowArray = Data[];
export type RowObject = Record<string, Data>;

/** We assume objects have all keys present and in same order */
export function table(rows: RowArray[] | RowObject[], header?: string[]): string {
    let s = "<table>";
    const keys = header ?? Object.keys(rows[0]);
    {
        s += "<thead>";
        for (const key in keys) {
            s += `<td>${key}</td>`;
        }
        s += "</thead>";
    }
    s += "<tbody>";
    for (const row of rows) {
        s += "<tr>";
        if (Array.isArray(row)) {
            for (const col of row) {
                s += `<td>${col}</td>`;
            }
        } else {
            // TODO
        }
        s += "</tr>";
    }
    s += "</tbody>";
    s += "</table>";
    return s
}

// TODO
export function form(): string {
    let s = "<form>";
    s += "</form>";
    return s
}