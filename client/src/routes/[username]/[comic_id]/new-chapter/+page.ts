import type { PageLoad } from "./$types";

export const load = (({ params }) => {
    const { username, comic_id } = params;
    return { username, comic_id };
}) satisfies PageLoad;
