import { error } from '@sveltejs/kit';
import type { PageServerLoad } from './$types';
import type { ComicResponse } from 'bindings/ComicResponse';
import type { ComicCommentResponse } from 'bindings/ComicCommentResponse';

export const load = (async ({ fetch, params }) => {
    const { comic_slug, username } = params;

    const comicRes = await fetch(`http://localhost:6060/api/v1/comics/by_slug/${comic_slug}/${username}`);

    if (comicRes.status != 200) {
        const resError = await comicRes.json().catch(() => ({ error: comicRes.statusText }));
        throw error(comicRes.status, resError);
    }

    const comic: ComicResponse = await comicRes.json();

    const commentRes = await fetch(`http://localhost:6060/api/v1/comics/${comic.id}/comments`);

    if (commentRes.status != 200) {
        const resError = await commentRes.json().catch(() => ({ error: commentRes.statusText }));
        throw error(commentRes.status, resError);
    }

    let comments: ComicCommentResponse[] = await commentRes.json();

    let top_level_comments: ComicCommentResponse[] = comments.filter((comment) => {
        return comment.parent_comment === null;
    });

    for (const comment of top_level_comments) {
        fill_children(comment, comments, 3);
    }

    return {
        comic: comic,
        comments: top_level_comments,
    };

}) satisfies PageServerLoad;

const fill_children = (comment: ComicCommentResponse, comments: ComicCommentResponse[], limit: number) => {
    if (limit <= 0) {
        // limit is removed for now

        // comment.child_comments = [];
        // return;
    }

    comment.child_comments = comment.child_comments_ids?.map((child_id) => {
        const child_comment = get_comment_by_id(child_id, comments);

        if (child_comment) {
            fill_children(child_comment, comments, limit - 1);

            return child_comment;
        }
    }) as ComicCommentResponse[];
}

const get_comment_by_id = (id: string, comments: ComicCommentResponse[]): ComicCommentResponse | undefined => {
    return comments.find((comment) => {
        return comment.id == id;
    });
}
