<script lang="ts">
    import Text from "$lib/components/Text.svelte";
    import Fa from "svelte-fa";
    import { faHome, faPenToSquare, faPersonWalking } from "@fortawesome/free-solid-svg-icons";
    import { currentUser } from "../../routes/stores";

    export let open: boolean;

    async function logout() {
        await fetch("http://localhost:6060/api/v1/users/logout", {
            credentials: "include",
        });
        await currentUser.refresh();
    }
</script>

<nav class:expanded={open}>
    <ul>
        <li>
            <a class="link" href="/">
                <Fa size="1.5x" icon={faHome} />
                {#if open}
                    <Text fontSize="xl" --margin="0 0 0 1rem">Home</Text>
                {/if}
            </a>
        </li>
        {#if $currentUser}
            <li>
                <a class="link" href="/new-comic">
                    <Fa size="1.5x" icon={faPenToSquare} />
                    {#if open}
                        <Text fontSize="xl" --margin="0 0 0 1rem"
                            >New comic</Text>
                    {/if}
                </a>
            </li>
            <li>
                <button class="link" on:click={logout}>
                    <Fa size="2.5x" icon={faPersonWalking} />
                    {#if open}
                        <Text fontSize="xl" --margin="0 0 0 1rem"
                            >New comic</Text>
                    {/if}
                </button>
            </li>
        {/if}
    </ul>
</nav>

<style>
    nav {
        height: 100%;
        background-color: gray;
        transition: ease-out 200ms;
        width: 60px;
        overflow: hidden;
        margin-top: 3em;
        position: fixed;
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
</style>
