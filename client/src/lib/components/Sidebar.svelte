<script lang="ts">
    import Text from "$lib/components/Text.svelte";
    import Fa from "svelte-fa";
    import { faHome, faUser } from "@fortawesome/free-solid-svg-icons";

    export let open: boolean;
</script>

<nav class:expanded={open}>
    <ul>
        <li>
            <a class="link" href="/"
                ><Fa size="1.5x" icon={faHome} />
                {#if open}<Text fontSize="xl" --margin="0 0 0 1rem">Home</Text
                    >{/if}</a
            >
        </li>
        <li>
            <a class="link" href="/login"
                ><Fa size="1.5x" icon={faUser} />
                {#if open}<Text fontSize="xl" --margin="0 0 0 1rem">Login</Text
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
        padding: 20px 15px 15px 15px;
        margin: 0;
    }

    li {
        width: 200px;
        height: 3.5rem;
    }

    a {
        display: flex;
        line-height: 1.5rem;
    }
</style>
