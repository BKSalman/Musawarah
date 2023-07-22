<script lang="ts">
    export let data;
    const { comics, user } = data;
</script>

{#if user.role === "user"}
    <div class="error-box">
        <h2>Verify Your Email!</h2>
        <!-- TODO: add more stuff on why a user needs to verify account -->
        <p>
            Click <a href="/send-email" class="send-email">here</a> to send an email
            and verify your account.
        </p>
    </div>
{/if}
<div class="horizontal-container">
    <div class="sidebar">
        <h1>Profile</h1>
        <p><strong>Username: </strong>{user.username}</p>
        <p><strong>Display Name: </strong>{user.displayname}</p>
        <img src={user.profile_image.path} alt="profile" />
    </div>
    <div class="content">
        <h1>Comics</h1>
        {#each comics || [] as comic}
            <a href={`/${user.username}/${comic.id}`}>
                <div class="comic">
                    <h2 class="comic-title">{comic.title}</h2>
                    <p>{comic.description ?? ""}</p>
                </div>
            </a>
        {/each}
    </div>
</div>

<style>
    .send-email {
        text-decoration: underline;
        font-weight: bold;
    }

    .error-box {
        background-color: #fff3cd;
        border: 1px solid #ffeeba;
        color: #856404;
        border-radius: 4px;
        flex: 1;
        padding: 10px;
        margin-right: 20px;
        margin-bottom: 20px;
    }

    .horizontal-container {
        display: flex;
        justify-content: center;
    }

    .sidebar {
        background-color: #f1f1f1;
        padding: 20px;
    }

    .content {
        flex: 1;
        padding: 20px;
        margin-left: 20px;
        margin-right: 20px;
        background-color: #ffffff;
        box-shadow: 0 0 5px rgba(0, 0, 0, 0.2);
    }

    h1,
    h2,
    p {
        margin-bottom: 10px;
    }

    .comic {
        margin-bottom: 30px;
    }

    .comic-title {
        font-weight: bold;
    }

    a {
        all: unset;
        cursor: pointer;
    }
</style>
