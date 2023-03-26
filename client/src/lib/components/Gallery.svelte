<script lang="ts">
  import type { PostResponse } from "../../../../bindings/PostResponse";
  import GalleryItem from "./GalleryItem.svelte";

  export let posts: PostResponse[];
</script>

<div class="gallery">
  {#each posts as post}
    <GalleryItem {post} />
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
