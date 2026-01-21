export function createElement(
    name: string,
    attributes: Record<string, string>,
    children: (HTMLElement | string)[]
): HTMLElement {
    const e = document.createElement(name);
    for (const key in attributes) {
        e.setAttribute(key, attributes[key])
    }
    e.append(...children);
    return e
}

// TODO width, height etc
export function createCanvas(): { canvas: HTMLCanvasElement, ctx: CanvasRenderingContext2D } {
    const canvas = document.createElement("canvas");
    // Can be `null` if `getContext` called already
    const ctx = canvas.getContext("2d") as CanvasRenderingContext2D;
    return { canvas, ctx }
}