import { error } from '@sveltejs/kit';
import type { PageServerLoad } from './$types';
import type { ComicResponse } from 'bindings/ComicResponse';
import type { UserResponse } from 'bindings/UserResponse';

export const load = (async ({ fetch, params }) => {

  const { username } = params;

  const comics_res = await fetch(`http://localhost:6060/api/v1/users/comics/${username}`, {
    credentials: "include",
  });
  if (comics_res.status != 200) {
    const res_error = await comics_res.json().catch(() => ({ error: comics_res.statusText }));
    throw error(comics_res.status, res_error);
  }
  const comics: ComicResponse[] = await comics_res.json();

  const user_res = await fetch(`http://localhost:6060/api/v1/users/${username}`, {
    credentials: "include",
  });
  if (user_res.status != 200) {
    const res_error = await user_res.json().catch(() => ({ error: user_res.statusText }));
    throw error(user_res.status, res_error);
  }
  const user: UserResponse = await user_res.json();

  return {
    comics,
    user,
  };
}) satisfies PageServerLoad;
