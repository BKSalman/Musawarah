import { error } from '@sveltejs/kit';
import type { PageServerLoad } from './$types';
import type { ComicResponse } from '$app/../../../bindings/ComicResponse';

export const load = (async ({ fetch, params, cookies}) => {
    const { comic_id, username } = params;

    const res = await fetch(`http://localhost:6060/api/v1/comics/${comic_id}`);

    if (res.status != 200) {
        const errorMessage = await res.json();
        throw error(res.status, errorMessage);
    }

    const data: ComicResponse = await res.json();

    return {
        comic: data
    };

}) satisfies PageServerLoad;