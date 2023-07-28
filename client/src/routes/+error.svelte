<script lang="ts">
    import { page } from '$app/stores';

    const message = $page.error?.error;
    const details = $page.error?.details;
    const status = $page.status
</script>
  
<h1 class="container">
    <h1>{status}</h1>
    <h5>{message}</h5>
    {#each details || [] as detail}
        <div>{detail}</div>
    {/each}
</h1>

<style>
.container {
    margin: auto;
    display: flex;
    align-items: center;
    justify-content: center;
    flex-direction: column;
}
</style>
