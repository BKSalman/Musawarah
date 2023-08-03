import { error } from '@sveltejs/kit';
import type { PageServerLoad } from './$types';
import type { ChapterCommentResponse } from 'bindings/ChapterCommentResponse';
import type { ChapterResponse } from 'bindings/ChapterResponse';

export const load = (async ({ fetch, params }) => {
    const { comic_slug, username, chapter_number } = params;

    const chapterRes = await fetch(`http://localhost:6060/api/v1/comics/chapters/by_slug/${username}/${comic_slug}/${chapter_number}/`);

    if (chapterRes.status != 200) {
        const resError = await chapterRes.json().catch(() => ({ error: chapterRes.statusText }));
        throw error(chapterRes.status, resError);
    }

    const chapter: ChapterResponse = await chapterRes.json();

    const commentRes = await fetch(`http://localhost:6060/api/v1/comics/chapters/${chapter.id}/comments`);

    if (commentRes.status != 200) {
        const resError = await commentRes.json().catch(() => ({ error: commentRes.statusText }));
        throw error(commentRes.status, resError);
    }

    let comments: ChapterCommentResponse[] = await commentRes.json();

    let top_level_comments: ChapterCommentResponse[] = comments.filter((comment) => {
        return comment.parent_comment === null;
    });

    for (const comment of top_level_comments) {
        fill_children(comment, comments, 3);
    }

    return {
        chapter,
        comments: top_level_comments,
    };

}) satisfies PageServerLoad;

const fill_children = (comment: ChapterCommentResponse, comments: ChapterCommentResponse[], limit: number) => {
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
    }) as ChapterCommentResponse[];
}

const get_comment_by_id = (id: string, comments: ChapterCommentResponse[]): ChapterCommentResponse | undefined => {
    return comments.find((comment) => {
        return comment.id == id;
    });
}

