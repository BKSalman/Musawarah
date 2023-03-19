import { error, redirect } from '@sveltejs/kit';
import type { Actions } from './$types';
import type { UserToken } from '../../../../bindings/UserToken';
 
export const actions = {
  default: async ({ request, fetch, cookies }) => {
    const formData = (await request.formData());
    const email = formData.get("email");
    const password = formData.get("password");

    const res = await fetch("http://127.0.0.1:6060/api/users/login", {
        method: "POST",
        headers: {
            "Content-Type": "application/json",
        },
        body: JSON.stringify({
            email,
            password,
        })
    });

    if (res.status != 200) {
        const errorMessage = await res.json();
        throw error(res.status, errorMessage);
    }

    const data: UserToken = await res.json();

    cookies.set("auth_key", data.access_token);

    // TODO: add redirectTo stuff
    throw redirect(303, "/");
  }
} satisfies Actions;