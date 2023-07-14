import { error } from '@sveltejs/kit';
import type { PageServerLoad } from './$types';
import type { ComicResponse } from 'bindings/ComicResponse';
import type { ComicCommentResponse } from 'bindings/ComicCommentResponse';

export const load = (async ({ fetch, params, cookies}) => {
    const { comic_id, username } = params;

    const comic_res = await fetch(`http://localhost:6060/api/v1/comics/${comic_id}`);

    if (comic_res.status != 200) {
        const errorMessage = await comic_res.json();
        throw error(comic_res.status, errorMessage);
    }

    const comic: ComicResponse = await comic_res.json();

    const comment_res = await fetch(`http://localhost:6060/api/v1/comics/${comic_id}/comments`);

    if (comment_res.status != 200) {
        const errorMessage = await comment_res.json();
        throw error(comment_res.status, errorMessage);
    }

    let comments: ComicCommentResponse[] = await comment_res.json();

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
        comment.child_comments = [];
        return;
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