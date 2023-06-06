import { error } from '@sveltejs/kit';
import type { PageServerLoad } from './$types';
import type { ChapterResponse } from 'bindings/ChapterResponse';

export const load = (async ({ fetch, params, cookies}) => {
    const { comic_id, username, chapter_id } = params;

    const res = await fetch(`http://localhost:6060/api/v1/comics/chapters/${chapter_id}/s`);

    if (res.status != 200) {
        const errorMessage = await res.json();
        throw error(res.status, errorMessage);
    }

    const data: ChapterResponse = await res.json();

    return {
        chapter: data
    };

}) satisfies PageServerLoad;
