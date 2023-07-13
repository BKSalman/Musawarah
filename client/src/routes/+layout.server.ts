import type { UserResponseBrief } from "bindings/UserResponseBrief";
import type { LayoutServerLoad } from "./$types";

export const load = (async ({ fetch }) => {
  const res = await fetch(`http://localhost:6060/api/v1/users/me`, {
    credentials: "include",
  });
  const user: UserResponseBrief | undefined = res.status !== 200 ? undefined : await res.json();
  return { user };
}) satisfies LayoutServerLoad;
