<script lang="ts">
    import Text from "$lib/components/Text.svelte";

    export let open: boolean;
</script>

<nav class:expanded={open}>
    <ul>
        <li>
            <a href="/"
                >🚀 {#if open}<Text fontSize="xl" --margin="0 0 0 1rem"
                        >lmao</Text
                    >{/if}</a
            >
        </li>
        <li>
            <a href="/"
                >🚀 {#if open}<Text fontSize="xl" --margin="0 0 0 1rem"
                        >lmao</Text
                    >{/if}</a
            >
        </li>
        <li>
            <a href="/"
                >🚀 {#if open}<Text fontSize="xl" --margin="0 0 0 1rem"
                        >lmao</Text
                    >{/if}</a
            >
        </li>
    </ul>
</nav>

<style>
    nav {
        height: 100%;
        background-color: gray;
        transition: ease-out 200ms;
        width: 60px;
        overflow: hidden;
    }

    .expanded {
        transition: ease-out 200ms;
        width: 200px;
    }

    ul {
        list-style: none;
        padding: 20px 15px 15px 20px;
        margin: 0;
    }

    li {
        width: 200px;
        height: 3.5rem;
    }

    a {
        display: flex;
        line-height: 1rem;
    }
</style>
