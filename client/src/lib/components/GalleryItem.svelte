<script lang="ts">
  import type { ComicResponseBrief } from "../../../../bindings/ComicResponseBrief";

  export let comic: ComicResponseBrief;
</script>

<a class="gallery-item" href={`/${comic.id}`}>
</a>

<style>
  .gallery-item {
    position: relative;
    display: block;
    overflow: hidden;
    margin: 0;
  }

  .gallery-item img {
    /*
      display: block;
      position: absolute;
      top: 50%;
      left: 50%;
      min-height: 100%;
      min-width: 100%;
      transform: translate(-50%, -50%);
    */
    width: 100%;
    height: 100%;
    object-fit: cover;
  }
</style>
