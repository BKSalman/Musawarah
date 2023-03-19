<script>

</script>

<div class="gallery">
    <slot></slot>
</div>

<style>
.gallery {
  display: grid;
  margin: auto;
  gap: 0.25rem;
  grid-template-columns: repeat(auto-fit, minmax(240px, 1fr));
  grid-auto-rows: 240px;
}
</style>