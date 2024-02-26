<script lang="ts">
    import { page } from "$app/stores";
    import Comment from "$lib/components/Comment.svelte";
    import { currentUser } from "../../../stores";
    import { goto } from "$app/navigation";
    import type { ChapterCommentResponse } from "bindings/ChapterCommentResponse";
    import Fa from "svelte-fa";
    import { faArrowLeft } from "@fortawesome/free-solid-svg-icons";

    export let data;

    let { chapter, comments } = data;

    let inputComment = "";

    async function sendComment(chapter_id: string) {
        if (inputComment.length < 1) {
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
                    content: inputComment,
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
            inputComment = "";
        }
    }
</script>

<div class="main-container">
    <div>
        <a
            class="back-button"
            href={`/${$page.params.username}/${$page.params.comic_slug}`}
            ><Fa size="1.5x" icon={faArrowLeft} /></a
        >
    </div>
    <div class="title-bar">
        <div class="title"><strong>Title: </strong>{chapter.title}</div>

        {#if chapter.description}
            <div class="description">
                <strong>Description: </strong>{chapter.description}
            </div>
        {/if}
        <a href={`${$page.url.href}/chapter-settings`}>Settings</a>
    </div>
    <br />
    <div class="pages">
        {#each chapter.pages as page}
            {#if page.description}
                <div class="">{page.description}</div>
            {/if}
            <img
                class="page-image"
                src={`http://localhost:6060/api/v1/images/${page.image.path}`}
                alt="page"
            />
        {/each}
    </div>
    <div class="comments">
        <span>comments:</span>
        {#if $currentUser}
            <div class="new-comment">
                <input
                    type="text"
                    name="comment"
                    placeholder="Add a comment"
                    bind:value={inputComment}
                    on:keypress={(e) => {
                        if (e.key === "Enter") sendComment(chapter.id);
                    }}
                />
                <button type="submit" on:click={() => sendComment(chapter.id)}
                    >send</button
                >
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
        pointer-events: none;
    }
    .back-button {
        background: none;
        border: none;
        cursor: pointer;
        color: black;
    }
</style>
