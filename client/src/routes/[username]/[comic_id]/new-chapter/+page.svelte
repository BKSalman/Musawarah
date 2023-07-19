<script lang="ts">
    import { goto } from "$app/navigation";
    import type { ErrorResponse } from "bindings/ErrorResponse";
    import { setError, setMessage, superForm, superValidateSync } from "sveltekit-superforms/client";
    import { z } from "zod";
    import type { PageData } from "./$types";
    import type { ChapterResponseBrief } from "bindings/ChapterResponseBrief";

    export let data: PageData;

    const { username, comic_id } = data;

    const chapterSchema = z.object({
      title: z.string().optional(),
      description: z.string().optional(),
      // TODO: get chapter numbers from backend to check what are the taken numbers
      number: z.number().default(1),
    });

    const { form, errors, message, constraints, enhance } = superForm(
      superValidateSync(chapterSchema),
      {
        SPA: true,
        validators: chapterSchema,
        onUpdate: async ({ form }) => {
          console.log(form.valid);
          if (form.valid) {
            const res = await fetch(`http://localhost:6060/api/v1/comics/${comic_id}/chapters`, {
              credentials: "include",
              method: "POST",
              headers: {
                "Content-Type": "application/json",
              },
              body: JSON.stringify({
                title: form.data.title,
                description: form.data.description,
                number: form.data.number,
              }),
            });

            if (res.status !== 200) {
              const errorMessage: ErrorResponse = await res.json();
              console.log(res.status, res.statusText, errorMessage.error);
              if (errorMessage.error.includes("title")) {
                  setError(form, "title", errorMessage.error);
              } else if (errorMessage.error.includes("description")) {
                  setError(form, "description", errorMessage.error);
              } else if (errorMessage.error.includes("number")) {
                  setError(form, "number", errorMessage.error);
              } else {
                  setMessage(form, errorMessage.error);
              }

              return;
            }
            setMessage(form, "تم الانشاء بنجاح!");

            const response: ChapterResponseBrief = await res.json();
            // TODO: add user field to chapter response instead of using the slug
            await goto(`/${username}/${comic_id}/${response.id}`);
          }
        },
        onError({ result, message }) {
          message.set(result.error.message);
        },
      }
    );
</script>

<div class="main-container">
  <p>انشاء فصل</p>
  {#if $message}<h3 class="message">{$message}</h3>{/if}
  <form class="chapter-form" method="POST" use:enhance>
    <div class="field">
      <input
        type="text"
        id="title-input"
        name="title"
        data-invalid={$errors.title}
        bind:value={$form.title}
        {...$constraints.title}
      />
      <div>
        <small class="star">*</small>
        <label for="title-input" id="title-label">العنوان</label>
      </div>
    </div>
    {#if $errors.title}<small class="invalid">{$errors.title}</small>{/if}
    <div class="field">
      <input
        type="text"
        id="description-input"
        name="description"
        data-invalid={$errors.description}
        bind:value={$form.description}
        {...$constraints.description}
      />
      <label for="description-input" id="description-label">الوصف</label>
    </div>
    {#if $errors.description}<small class="invalid">{$errors.description}</small >{/if}
    <div class="field">
      <input
        type="number"
        id="chapter-number-input"
        name="chapter-number"
        data-invalid={$errors.number}
        bind:value={$form.number}
        {...$constraints.number}
      />
      <div>
          <small class="star">*</small>
          <label for="chapter-number-input" id="number-label">رقم الفصل</label>
      </div>
    </div>
    {#if $errors.number}<small class="invalid">{$errors.number}</small >{/if}
    <button type="submit">انشاء</button>
  </form>
</div>

<style>
  .main-container {
    display: flex;
    flex-direction: column;
    width: 100%;
    justify-content: start;
    align-items: center;
    margin-top: 5em;
  }
  .chapter-form {
    display: flex;
    flex-direction: column;
    min-width: 70%;
    justify-content: center;
    align-items: center;
  }
  .chapter-form > * {
    margin-top: 10px;
  }
  button {
    padding: 5px;
    cursor: pointer;
  }
  input {
    padding: 5px;
    border-radius: 5px;
    border: 1px solid rgb(152, 152, 167);
    box-shadow: 0px 5px 5px rgba(0, 0, 0, 0.161);
  }
  .field {
    display: flex;
    justify-content: space-between;
    width: 20em;
  }
  .star {
    color: red;
  }
  #genres {
    display: flex;
    flex-direction: column;
  }
  .invalid {
    color: red;
  }
  .message {
    color: red;
  }
</style>