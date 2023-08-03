<script lang="ts">
    import type { PageServerData } from "./$types";
    import ChapterThumbnail from "$lib/components/ChapterThumbnail.svelte";
    import Comment from "$lib/components/Comment.svelte";
    import { currentUser } from "../../stores";
    import { goto } from "$app/navigation";
    import type { ComicCommentResponse } from "bindings/ComicCommentResponse";

    export let data: PageServerData;

    let { comic, comments } = data;

    let inputComment = "";

    async function sendComment(comic_id: string) {
        if (inputComment.length < 1) {
            return;
        }

        const res = await fetch(
            `http://localhost:6060/api/v1/comics/${comic_id}/comments`,
            {
                credentials: "include",
                method: "POST",
                headers: {
                    "Content-Type": "application/json",
                },
                body: JSON.stringify({
                    content: inputComment,
                }),
            }
        );

        if (res.status >= 400) {
            // TODO: add message to the user that they need to log in
            await goto("/login");
        } else {
            comments.unshift((await res.json()) as ComicCommentResponse);
            // force it to refresh
            comments = comments;
            inputComment = "";
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
    {#if $currentUser && $currentUser.id == comic.author.id}
        <a href={`/${comic.author.username}/${comic.slug}/new-chapter`}
            >New chapter</a
        >
    {/if}
    <div class="chapters">
        {#each comic.chapters as chapter}
            <ChapterThumbnail {chapter} />
        {/each}
    </div>
    <div class="comments">
        <span>comments:</span>
        {#if $currentUser}
            <div class="new-comment">
                <input type="text" name="comment" placeholder="Add a comment"
                    bind:value={inputComment}
                    on:keypress={(e) => { if (e.key === "Enter") sendComment(comic.id) }}/>

                <button type="submit" on:click={() => sendComment(comic.id)}>send</button>
            </div>
        {:else}
            <h3><a href="/login">need to be logged in</a></h3>
        {/if}
        {#each comments as comment}
            <Comment {comment} />
        {/each}
    </div>
</div>

<style>
    .comic {
        background: #aaaaaa;
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
