import { error } from '@sveltejs/kit';
import type { PageServerLoad } from './$types';
import type { ComicResponse } from 'bindings/ComicResponse';
import type { ComicCommentResponse } from 'bindings/ComicCommentResponse';
import _ from "lodash";

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
    
    for (const comment of comments) {
        fill_children(comment, 4);
    }

    console.log(comments);

    return {
        comic: comic,
        comments: comments,
    };

}) satisfies PageServerLoad;

const fill_children = (parent_comment: ComicCommentResponse, comments: ComicCommentResponse[], limit: number) => {
    let result: any = [];

    if (limit <= 0) {
        return result;
    }

    for (const child_id of parent_comment.child_comments) {
        child_comment = get_comment_by_id(child_id, comments);
        fill_children(comment.child_comments, limit - 1);

        result.push(comment);

        // for (const child_comment of comments) {
        //     if (parent_comment.child_comments?.find((id) => id == child_comment.id)) {
        //         children.push(child_comment);
        //         result.splice(result.findIndex((c) => c.id == child_comment.id), 1);
        //     }
        // }

        // if (children.length > 0) {
        //     parent_comment.child_comments = children;
        // }
    }
}

const get_comment_by_id = (id: string, comments: ComicCommentResponse) => {
    
}