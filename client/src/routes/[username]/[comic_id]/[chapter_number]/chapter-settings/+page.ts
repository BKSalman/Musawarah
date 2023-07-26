import { error, redirect } from '@sveltejs/kit';
import type { PageLoad } from './$types';
import type { ChapterResponse } from 'bindings/ChapterResponse';

export const load = (async ({ fetch, params, depends }) => {
    const { username, comic_id, chapter_number } = params;

    depends("chapter-info");
    const res = await fetch(`http://localhost:6060/api/v1/comics/chapters/by_number/${comic_id}/${chapter_number}/`
        , {
            credentials: "include",
        });

    if (res.status === 401) {
        throw redirect(307, "/");
    } else if (res.status !== 200) {
        const errorMessage = await res.json();
        throw error(res.status, errorMessage.error);
    }

    const chapter: ChapterResponse = await res.json();

    return {
        comic_id,
        username,
        chapter,
    }
}) satisfies PageLoad;
