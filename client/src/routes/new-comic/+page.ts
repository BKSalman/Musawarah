import { error } from '@sveltejs/kit';
import type { PageLoad } from './$types';
import type { ComicGenre } from '../../../../bindings/ComicGenre';

export const load = (async ({ fetch }) => {
    const res = await fetch("http://localhost:6060/api/v1/comic-genres");

    if (res.status !== 200) {
        const errorMessage = await res.json();
        throw error(res.status, errorMessage.error);
    }

    const genres: ComicGenre[] = await res.json();

    return {
        genres: genres,
    }
}) satisfies PageLoad;

