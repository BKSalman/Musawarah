import { error } from '@sveltejs/kit';
import type { PageLoad } from './$types';
import type { ChapterResponse } from 'bindings/ChapterResponse';

export const load = (async ({ fetch, params }) => {
    const { username, comic_id, chapter_id } = params;
    
    const res = await fetch(`http://localhost:6060/api/v1/comics/chapters/${chapter_id}/s`);

    if (res.status !== 200) {
        const errorMessage = await res.json();
        throw error(res.status, errorMessage.error);
    }

    const chapter: ChapterResponse = await res.json();

    return {
        chapter,
    }
}) satisfies PageLoad;
