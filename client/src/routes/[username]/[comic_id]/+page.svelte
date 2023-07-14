<script lang="ts">
    import type { PageServerData } from './$types';
    import ChapterThumbnail from '$lib/components/ChapterThumbnail.svelte';
    import Comment from '$lib/components/Comment.svelte';
    import { currentUser } from '../../stores';
    import { goto } from '$app/navigation';
    import type { ComicCommentResponse } from 'bindings/ComicCommentResponse';

    export let data: PageServerData;

    let { comic, comments } = data;
    async function sendComment(e: SubmitEvent, parent_comment_id: string | null, comic_id: string) {
        const form = new FormData(e.target as HTMLFormElement);

        const comment = form.get("comment");

        if (comment?.toString() == null || comment?.toString().length < 1) {
            return;
        }

        const res = await fetch(`http://localhost:6060/api/v1/comics/${comic_id}/comments`, {
            credentials: "include",
            method: "POST",
            headers: {
              "Content-Type": "application/json",
            },
            body: JSON.stringify({
                content: comment,
                parent_comment_id: parent_comment_id
            }),
        });

        if (res.status >= 400) {
            // TODO: add message to the user that they need to log in
            await goto("/login");
        } else {
            comments.unshift(await res.json() as ComicCommentResponse)
            // just to force it to refresh
            comments = comments;
        }
    }
</script>


<div class="comic-container">
    <div class="comic">
        <h1>{comic.title}</h1>
        {#if comic.description}
            <div>{comic.description}</div>
        {/if}
    </div>
        <span>chapters:</span>
        <div class="chapters">
            {#each comic.chapters as chapter}
                <ChapterThumbnail {chapter} />
            {/each}
        </div>
    <div class="comments">
        <span>comments:</span>
        {#if $currentUser}
            <form class="new-comment" on:submit|preventDefault={(e) => sendComment(e, null, comic.id)}>
                <input type="text" name="comment" placeholder="Add a comment">
                <button type="submit">send</button>
            </form>
        {/if}
        {#each comments as comment}
            <Comment {comment}/>
        {/each}
    </div>
</div>

<style>
.comic {
    background: #AAAAAA;
    width: 90%;
    margin-bottom: 10px;
}
.chapters {
    width: 90%;
    display: grid;
    grid-template-columns: auto auto auto;
    gap: 5px 5px;
    place-items: center;
}
.comments {
    margin-bottom: 10px;
}
</style>
