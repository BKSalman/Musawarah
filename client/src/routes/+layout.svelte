<script>

    import Navbar from "$lib/components/Navbar.svelte";
    import Sidebar from "$lib/components/Sidebar.svelte";

</script>

<Navbar/>
<div class="main-container">
    <Sidebar/>
    <slot />
</div>

<br/>

<footer>
    footer
</footer>

<style>
    .main-container {
        display: grid;
        grid-template-columns: 1fr 7fr;
    }
</style>