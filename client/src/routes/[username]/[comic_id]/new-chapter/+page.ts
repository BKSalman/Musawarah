import { error } from "@sveltejs/kit";
import type { PageLoad } from "./$types";
import type { ComicResponse } from "bindings/ComicResponse";

export const load = (async ({ params }) => {
    const { username, comic_id } = params;

    const res = await fetch(`http://localhost:6060/api/v1/comics/${comic_id}`, {
        credentials: "include"
    });

    if (res.status !== 200) {
        throw error(res.status, await res.json());
    }

    const comic: ComicResponse = await res.json();
    
    return { username, comic };
}) satisfies PageLoad;
