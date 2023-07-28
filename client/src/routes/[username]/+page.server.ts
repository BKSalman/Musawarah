import { error } from '@sveltejs/kit';
import type { PageServerLoad } from './$types';
import type { ComicResponse } from 'bindings/ComicResponse';
import type { UserResponse } from 'bindings/UserResponse';

export const load = (async ({ fetch, params }) => {

  const { username } = params;

  const comicsRes = await fetch(`http://localhost:6060/api/v1/users/comics/${username}`, {
    credentials: "include",
  });
  if (comicsRes.status != 200) {
    const resError = await comicsRes.json().catch(() => ({ error: comicsRes.statusText }));
    throw error(comicsRes.status, resError);
  }
  const comics: ComicResponse[] = await comicsRes.json();

  const userRes = await fetch(`http://localhost:6060/api/v1/users/${username}`, {
    credentials: "include",
  });
  if (userRes.status != 200) {
    const resError = await userRes.json().catch(() => ({ error: userRes.statusText }));
    throw error(userRes.status, resError);
  }
  const user: UserResponse = await userRes.json();

  return {
    comics,
    user,
  };
}) satisfies PageServerLoad;
