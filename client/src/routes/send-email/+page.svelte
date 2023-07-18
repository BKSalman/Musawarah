<script lang="ts">
  import { onMount } from "svelte";
  let message: string | null = null;
  let send_email = async () => {
    message = null;
    const res = await fetch(
      "http://localhost:6060/api/v1/users/email_verification",
      {
        credentials: "include",
        method: "POST",
      }
    );
    if (res.status === 200) {
      message = "Email Has been sent! Check your inbox or junk folder.";
    } else {
      message = "Failed to send email. Something went wrong.";
    }
  };
  onMount(async () => {
    await send_email();
  });
</script>

{#if message}
  <h2>{message}</h2>
{:else}
  <div class="loader" />
  <div class="loader-text">Sending Email...</div>
{/if}
<!-- TODO: find a way to limit the amount a user can request an email resend  -->
<div class="button-wrapper">
  {#if message}
    <button class="email-resend" on:click={() => send_email()}
      >Resend the email</button
    >
  {/if}
</div>

<style>
  .loader {
    border: 5px solid #ccc;
    border-top-color: #333;
    border-radius: 50%;
    width: 50px;
    height: 50px;
    animation: spin 1s ease-in-out infinite;
    margin: 25px auto;
  }

  .loader-text {
    text-align: center;
    font-size: 24px;
    margin-top: 20px;
  }

  .email-resend {
    margin-top: 25px;
    text-align: center;
    font-size: 18px;
    background-color: yellow;
    border: none;
    padding: 8px 16px;
    text-decoration: none;
    display: inline-block;
    cursor: pointer;
    border-radius: 5px;
    margin-right: 5px;
  }

  .button-wrapper {
    display: flex;
    align-items: center;
    justify-content: center;
  }

  @keyframes spin {
    0% {
      transform: rotate(0);
    }

    100% {
      transform: rotate(360deg);
    }
  }
</style>
