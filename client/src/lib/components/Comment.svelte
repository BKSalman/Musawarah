<script lang="ts">
    import { goto } from "$app/navigation";
    import type { ComicCommentResponse } from "bindings/ComicCommentResponse";

    export let comment: ComicCommentResponse;
    export let indent = 0;

    async function sendComment(e: SubmitEvent, parent_comment_id: string | null) {
        const form = new FormData(e.target as HTMLFormElement);

        const formComment = form.get("comment");

        if (formComment?.toString() == null || formComment?.toString().length < 1) {
            return;
        }

        const res = await fetch(`http://localhost:6060/api/v1/comics/${comment.comic_id}/comments`, {
            credentials: "include",
            method: "POST",
            headers: {
              "Content-Type": "application/json",
            },
            body: JSON.stringify({
                content: formComment,
                parent_comment_id: parent_comment_id
            }),
        });

        if (res.status >= 400) {
            // TODO: add message to the user that they need to log in
            await goto("/login");
        } else {
            comment.child_comments.push(await res.json() as ComicCommentResponse)
            // just to force it to refresh
            comment = comment;
        }
    }
</script>

<div style="padding-left: {indent}rem" class="comment">
    <div class="">{comment.user.username}</div>
    <div class="">{comment.content}</div>
    <form class="new-reply" on:submit|preventDefault={(e) => sendComment(e, comment.id)}>
        <input type="text" name="comment" placeholder="Add a reply">
        <button type="submit">send</button>
    </form>
    <div class="children">
        {#each comment.child_comments || [] as child}
            <svelte:self comment={child} indent={indent + 0.1}/>
        {/each}
    </div>
</div>

<style>
.children {
    margin-top: 10px;
    margin-left: 15px;
}
.comment {
    background: #AAAAAA;
    margin-bottom: 10px;
    border-radius: 5px;
    width: 90%;
}
</style>
