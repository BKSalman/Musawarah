import { error } from '@sveltejs/kit';
import type { PageServerLoad } from './$types';
import type { ComicResponseBrief } from 'bindings/ComicResponseBrief';
import type { UserResponse } from 'bindings/UserResponse';

export const load = (async ({ fetch, params }) => {

  const { username } = params;

  const userRes = await fetch(`http://localhost:6060/api/v1/users/${username}`, {
    credentials: "include",
  });
  if (userRes.status != 200) {
    const resError = await userRes.json().catch(() => ({ error: userRes.statusText }));
    throw error(userRes.status, resError);
  }
  const user: UserResponse = await userRes.json();

  const comicsRes = await fetch(`http://localhost:6060/api/v1/users/comics/${user.id}`, {
    credentials: "include",
  });
  if (comicsRes.status != 200) {
    const resError = await comicsRes.json().catch(() => ({ error: comicsRes.statusText }));
    throw error(comicsRes.status, resError);
  }
  const comics: ComicResponseBrief[] = await comicsRes.json();


  return {
    comics,
    user,
  };
}) satisfies PageServerLoad;
