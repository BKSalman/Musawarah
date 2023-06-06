<script lang="ts">
    import { page } from '$app/stores';
    import type { PageServerData } from './$types';

    export let data: PageServerData;

    const { comic, comments } = data;
</script>


<div class="comic-container">
    <div class="comic">
        <h1>{comic.title}</h1>
        {#if comic.description}
            <div>{comic.description}</div>
        {/if}
    </div>
        <div class="chapters">
        <span>chapters:</span>
            {#each comic.chapters as chapter}
                <div class="chapter">
                    <a href={`${$page.url.pathname}/${chapter.id}`}><div>{chapter.number}</div></a>
                    {#if chapter.title}
                        <div>{chapter.title}</div>
                    {/if}
                </div>
            {/each}
        </div>
    <div class="comments">
        <span>comments:</span>
        {#each comments as comment}
            <div class="comment">
                <div class="">{comment.user.username}</div>
                <div class="">{comment.content}</div>
                <div class="children">
                    {#if comment.child_comments}
                        {#each comment.child_comments as comment}
                            <div class="">{comment.user.username}</div>
                            <div class="">{comment.content}</div>
                        {/each}
                    {/if}
                </div>
            </div>
        {/each}
    </div>
</div>

<style>
.children {
    margin-top: 10px;
    margin-left: 15px;
}
.comic {
    background: black;
    width: 90%;
    margin-bottom: 10px;
}
.chapter {
    background: black;
    width: 90%;
    margin-bottom: 10px;
}
.comments {
    margin-bottom: 10px;
}
.comment {
    background: black;
    margin-bottom: 10px;
    border-radius: 5px;
    width: 80%;
}
</style>
