<script lang="ts">
  import { z } from "zod";
  import {
    superForm,
    superValidateSync,
    setMessage,
    setError,
  } from "sveltekit-superforms/client";
  import { goto } from "$app/navigation";
  import type { ErrorResponse } from "$app/../../../bindings/ErrorResponse";
  import type { PageData } from "./$types";
  import type { ComicResponse } from "bindings/ComicResponse";

  export let data: PageData;

  const { genres } = data;

  const comicSchema = z.object({
    title: z.string().min(3),
    description: z.string().optional(),
    genres: z.array(z.number()).optional(),
  });

  const { form, errors, message, constraints, enhance } = superForm(
    superValidateSync(comicSchema),
    {
      SPA: true,
      validators: comicSchema,
      onUpdate: async ({ form }) => {
        console.log(form.valid);
        if (form.valid) {
          const genres_element = document.getElementById("genres");

          let genres: number[] = [];

          if (genres_element?.children) {
            for (const genre of genres_element.children) {
              const input = genre.firstChild as HTMLInputElement;
              if (input?.checked) {
                genres.push(parseInt(input?.value));
              }
            }
          }

          const res = await fetch("http://localhost:6060/api/v1/comics", {
            credentials: "include",
            method: "POST",
            headers: {
              "Content-Type": "application/json",
            },
            body: JSON.stringify({
              title: form.data.title,
              description: form.data.description,
              genres,
              is_visible: true,
            })
          });

          if (res.status !== 200) {
            const errorMessage: ErrorResponse = (await res.json());
            console.log(res.status, res.statusText, errorMessage.error);
            if (errorMessage.error.includes("title")) {
              setError(form, "title", errorMessage.error);
            } else {
              setError(form, "genres", errorMessage.error);
            }
            return;
          }

          console.log(await res.json());

          setMessage(form, "تم الانشاء بنجاح!");

          const response: ComicResponse = await res.json();
          await goto(`/${response.author.username}/${response.id}`);
        }
      },
      onError({ result, message }) {
        message.set(result.error.message);
      },
    }
  );

</script>

<div class="comic-container">
  <p>انشاء قصة مصورة</p>
  {#if $message}<h3 class="message">{$message}</h3>{/if}
  <form class="comic-form" method="POST" use:enhance>
    <div class="field">
      <input
        type="text"
        id="username-input"
        name="username"
        data-invalid={$errors.title}
        bind:value={$form.title}
        {...$constraints.title}
      />
      <div class="">
        <small class="star">*</small>
        <label for="title" id="title-label">العنوان</label>
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
      <label for="description" id="description-label">الوصف</label>
    </div>
    {#if $errors.description}<small class="invalid">{$errors.description}</small>{/if}
    <div class="field">
      <div id="genres">
      {#each genres || [] as genre}
        <div class="">
          <input name={genre.name} type="checkbox" value={genre.id}/>
          <label for={genre.name} id="genre">{genre.name}</label>
        </div>
      {/each}
      </div>
      <label for="description" id="description-label">التصنيفات</label>
    </div>
    {#if $errors.genres}<small class="invalid">{$errors.genres}</small>{/if}
    <button type="submit">انشاء</button>
  </form>
</div>

<style>
  .comic-container {
    display: flex;
    flex-direction: column;
    width: 100%;
    justify-content: start;
    align-items: center;
    margin-top: 5em;
  }
  .comic-form {
    display: flex;
    flex-direction: column;
    min-width: 70%;
    justify-content: center;
    align-items: center;
  }
  .comic-form > * {
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
    color: green;
  }
</style>

