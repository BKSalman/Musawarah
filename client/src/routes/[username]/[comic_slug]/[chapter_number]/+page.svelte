<script lang="ts">
    import { page } from '$app/stores';
    import type { PageServerData } from './$types';
    import Comment from "$lib/components/Comment.svelte";
    import { currentUser } from '../../../stores';
    import { goto } from '$app/navigation';
    import type { ChapterCommentResponse } from 'bindings/ChapterCommentResponse';

    export let data: PageServerData;

    let { chapter, comments } = data;

    async function sendComment(
        e: SubmitEvent,
        parent_comment_id: string | null,
        chapter_id: string
    ) {
        const form = new FormData(e.target as HTMLFormElement);

        const comment = form.get("comment");

        if (comment?.toString() == null || comment?.toString().length < 1) {
            return;
        }

        const res = await fetch(
            `http://localhost:6060/api/v1/comics/chapters/${chapter_id}/comments`,
            {
                credentials: "include",
                method: "POST",
                headers: {
                    "Content-Type": "application/json",
                },
                body: JSON.stringify({
                    content: comment,
                    parent_comment_id: parent_comment_id,
                }),
            }
        );

        if (res.status >= 400) {
            // TODO: add message to the user that they need to log in
            await goto("/login");
        } else {
            comments.unshift((await res.json()) as ChapterCommentResponse);
            // force it to refresh
            comments = comments;
        }
    }
</script>

<div class="main-container">
    <div class="title-bar">
        <div class="title"><strong>Title: </strong>{chapter.title}</div>

        {#if chapter.description}
            <div class="description"><strong>Description: </strong>{chapter.description}</div>
        {/if}
        <a href={`${$page.url.href}/chapter-settings`}>Settings</a>
    </div>
    <br/>
    <div class="pages">
        {#each chapter.pages as page}
          {#if page.description}
              <div class="">{page.description}</div>
          {/if}
          <img class="page-image" src={`https://pub-26fa98a6ad0f4dd388ce1e8e1450be41.r2.dev/${page.image.path}`} alt="page"/>
        {/each}
    </div>
    <div class="comments">
        <span>comments:</span>
        {#if $currentUser}
            <form
                class="new-comment"
                on:submit|preventDefault={(e) => sendComment(e, null, chapter.id)}
            >
                <input type="text" name="comment" placeholder="Add a comment" />
                <button type="submit">send</button>
            </form>
        {:else}
            <h3><a href="/login">need to be logged in</a></h3>
        {/if}
        {#each comments as comment}
            <Comment {comment} />
        {/each}
    </div>
</div>

<style>
.main-container {
    display: flex;
    flex-direction: column;
    width: 80%;
}
.main-container > * {
    margin-bottom: 10px;
}
.pages {
    flex: 1;
    padding: 20px;
    margin-left: 20px;
    margin-right: 20px;
    background-color: #ffffff;
    box-shadow: 0 0 5px rgba(0, 0, 0, 0.2);
}
.page-image {
    width: 100%;
}
</style>
