<script lang="ts">
    import type { PageServerData } from './$types';

    export let data: PageServerData;

    const { comic } = data;
</script>


<div class="comic-container">
    <h1>{comic.title}</h1>
    {#if comic.description}
        <div>{comic.description}</div>
    {/if}
    {#each comic.chapters as chapter}
        {#if chapter.title}
            <div>{chapter.title}</div>
        {/if}
        <div>{chapter.number}</div>
        {#if chapter.description}
            <div>{chapter.description}</div>
        {/if}
        <a href={`${window.location}/${chapter.id}`}>lmao</a>
    {/each}
</div>

<style>

</style>
