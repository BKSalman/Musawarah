import { error } from '@sveltejs/kit';
import type { PageServerLoad } from './$types';
import type { ChapterResponse } from '$app/../../../bindings/ChapterResponse';

export const load = (async ({ fetch, params, cookies}) => {
    const { comic_id, username, chapter_id } = params;

    const res = await fetch(`http://localhost:6060/api/v1/chapters/s/${chapter_id}`);

    if (res.status != 200) {
        const errorMessage = await res.json();
        throw error(res.status, errorMessage);
    }

    const data: ChapterResponse = await res.json();

    return {
        chapter: data
    };

}) satisfies PageServerLoad;
