<script lang="ts">
  import type { PostResponse } from "../../../../bindings/PostResponse";

  export let post: PostResponse;
</script>

<a class="gallery-item" href={`/${post.user.username}/${post.id}`}>
  <img
    src={`https://pub-57f47fc2081648efaa3f351802d15c5d.r2.dev/${post.image.path}`}
    alt=""
  />
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
