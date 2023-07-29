import { error } from "@sveltejs/kit";
import type { PageLoad } from "./$types";
import type { ComicResponse } from "bindings/ComicResponse";

export const load = (async ({ params }) => {
    const { username, comic_slug } = params;

    const res = await fetch(`http://localhost:6060/api/v1/comics/by_slug/${comic_slug}/${username}`, {
        credentials: "include"
    });

    if (res.status !== 200) {
        const resError = await res.json().catch(() => ({ error: res.statusText }));
        throw error(res.status, resError);
    }

    const comic: ComicResponse = await res.json();

    return { username, comic };
}) satisfies PageLoad;
