import type { ComicResponse } from '../../../bindings/ComicResponse';
import type { PageServerLoad } from './$types';

export const load = (async ({ fetch }) => {
    const res = await fetch("http://127.0.0.1:6060/api/v1/comics");

    if (res.status != 200) {
        return {
            status: res.status,
            error: await res.json()
        };
    }

    const data: ComicResponse[] = await res.json();

    return {
        comics: data
    };
}) satisfies PageServerLoad;