<script lang="ts">
    import "../app.css";
    import Navbar from "$lib/components/Navbar.svelte";
    import Sidebar from "$lib/components/Sidebar.svelte";

    let open = false;
</script>

<Navbar bind:open />
<div class="layout-container">
    <Sidebar {open} />
    <slot />
</div>

<br />

<footer>footer</footer>

<style>
    .layout-container {
        display: grid;
        height: 100%;
        grid-template-columns: 1fr 7fr;
    }
</style>

