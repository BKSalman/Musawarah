<script lang="ts">
    import { page } from '$app/stores';
    import type { PageServerData } from './$types';
    import ChapterThumbnail from '$lib/components/ChapterThumbnail.svelte';
    import Comment from '$lib/components/Comment.svelte';

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
        <span>chapters:</span>
        <div class="chapters">
            {#each comic.chapters as chapter}
                <ChapterThumbnail {chapter} />
            {/each}
        </div>
    <div class="comments">
        <span>comments:</span>
        {#each comments as comment}
            <Comment {comment}/>
        {/each}
    </div>
</div>

<style>
.comic {
    background: black;
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
