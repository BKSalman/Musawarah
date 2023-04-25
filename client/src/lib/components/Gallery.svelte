<script lang="ts">
  import type { ComicResponseBrief } from "../../../../bindings/ComicResponseBrief";
  import GalleryItem from "./GalleryItem.svelte";

  export let comics: ComicResponseBrief[] | null;
</script>

<div class="gallery">
  {#each comics || [] as comic}
    <GalleryItem {comic} />
  {/each}
</div>

<style>
  .gallery {
    display: grid;
    margin: auto;
    gap: 0.25rem;
    grid-template-columns: repeat(auto-fit, minmax(240px, 1fr));
    grid-auto-rows: 240px;
    height: 100vh;
    width: 100%;
  }
</style>
