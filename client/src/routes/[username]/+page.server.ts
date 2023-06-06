import { error } from '@sveltejs/kit';
import type { PageServerLoad } from './$types';
import type { ComicResponse } from 'bindings/ComicResponse';

export const load = (async ({ fetch, params }) => {

  const { username } = params;

  const res = await fetch(`http://localhost:6060/api/v1/users/comics/${username}`, {
            credentials: "include",
          });

  if (res.status != 200) {
    const errorMessage = await res.json();
    throw error(res.status, errorMessage.error);
  }

  const data: ComicResponse[] = await res.json();

  return {
    comics: data
  };
}) satisfies PageServerLoad;
