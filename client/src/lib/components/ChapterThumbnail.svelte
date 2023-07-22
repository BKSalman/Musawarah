<script lang="ts">
    import { page } from '$app/stores';
    import type { ChapterResponseBrief } from 'bindings/ChapterResponseBrief';

    export let chapter: ChapterResponseBrief;
</script>

<div class="chapter">
    {#if chapter.title}
      <span>{chapter.title}</span>
    {/if}
    <a href={`${$page.url.pathname}/${chapter.id}`}><div>{chapter.number}</div></a>
</div>

<style>
.chapter {
    min-width: 1rem;
    min-height: 1.5rem;
    max-width: fit-content;
    max-height: fit-content;
    background: #AAAAAA;
}
</style>

