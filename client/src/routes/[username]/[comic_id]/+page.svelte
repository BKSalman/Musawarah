<script lang="ts">
    import type { PageServerData } from './$types';

    export let data: PageServerData;

    const { comic } = data;
</script>


<div class="comic-container">
    <h1>{comic.title}</h1>
    <div>{comic.description}</div>
    {#each comic.chapters as chapter}
        <div>{chapter.number}</div>
        <div>{chapter.description}</div>
        <a href={`${window.location}/${chapter.id}`}>lmao</a>
    {/each}
</div>

<style>

</style>
