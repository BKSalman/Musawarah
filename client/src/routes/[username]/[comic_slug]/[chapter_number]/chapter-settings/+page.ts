import { error, redirect } from '@sveltejs/kit';
import type { PageLoad } from './$types';
import type { ChapterResponse } from 'bindings/ChapterResponse';

export const load = (async ({ fetch, params, depends }) => {
    const { username, comic_slug, chapter_number } = params;

    depends("chapter-info");
    const res = await fetch(`http://localhost:6060/api/v1/comics/chapters/by_slug/${username}/${comic_slug}/${chapter_number}/`
        , {
            credentials: "include",
        });

    if (res.status === 401) {
        throw redirect(307, "/");
    } else if (res.status !== 200) {
        const resError = await res.json().catch(() => ({ error: res.statusText }));
        throw error(res.status, resError);
    }

    const chapter: ChapterResponse = await res.json();

    return {
        comic_id: chapter.comic_id,
        username,
        chapter,
    }
}) satisfies PageLoad;
