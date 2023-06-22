<script lang="ts">
    import type { ComicCommentResponse } from "bindings/ComicCommentResponse";

    export let comment: ComicCommentResponse;
    export let indent = 0;
</script>

<div style="padding-left: {indent}rem" class="comment">
    <div class="">{comment.user.username}</div>
    <div class="">{comment.content}</div>
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
    background: black;
    margin-bottom: 10px;
    border-radius: 5px;
    width: 90%;
}
</style>
