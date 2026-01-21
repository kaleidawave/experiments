export function ResponseHTML(body: string) {
    const headers = new Headers;
    headers.set("Content-Type", "text/html");
    return new Response(body, { headers });
}

export interface Headers {
    /** mimetype */
    "Content-Type": string,
    Date: string,
}