/**
 * {
 *     "name": "@bengineering/http-authentication",
 *     "description": "Basic wrapper of HTTP authentication",
 *     "version": "0.0.2",
 *     "exports": "./mod.ts",
 *     "license": "MIT"
 * }
 */

/** A basic user object */
export type User = { username: string, password: string };

/** A predicate of a {@link User} */
export type AuthPredicate = (username: string, password: string) => boolean;
/** Either a single allowed user, list of allowed user or a predicate */
export type Auth = User | User[] | AuthPredicate;
/** Handler for the {@link accept} function */
export type AuthCallback = {
  valid: (req: Request, user: User) => MaybePromise<Response>,
  invalid: (req: Request) => MaybePromise<Response>
}

/** Sync or Promise */
export type MaybePromise<T> = T | Promise<T>;

/** Take a request and run the callback depending on whether this passes the `guard` condition */
export async function accept(
  request: Request,
  guard: Auth,
  cb: AuthCallback
): Promise<Response> {
  const headers = new Headers({
    "WWW-Authenticate": "Basic realm=\"Access to the staging site\", charset=\"UTF-8\"",
  });
  const requestAuthHeader = request.headers.get("Authorization");
  if (requestAuthHeader?.startsWith("Basic")) {
    const [username, password] = atob(requestAuthHeader!.slice("Basic".length)).split(":");

    let matches = false;
    if (Array.isArray(guard)) {
      matches = guard.some(guard => guard.username == username && guard.password == password);
    } else if (typeof guard === "function") {
      matches = guard(username, password);
    } else {
      matches = guard.username == username && guard.password == password;
    }

    if (matches) {
      return cb.valid(request, { username, password })
    } else {
      // const response = cb.invalid(request);
      // response.headers.set
      return new Response("Incorrect credentials", { status: 401, headers });
    }
  } else {
    return new Response("Forbidden", { status: 401, headers });
  }
}