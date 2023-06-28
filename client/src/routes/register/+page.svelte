<script lang="ts">
  import { z } from "zod";
  import {
    superForm,
    superValidateSync,
    setError,
  } from "sveltekit-superforms/client";
  import { goto } from "$app/navigation";
  import type { ErrorResponse } from "bindings/ErrorResponse";

  const registerSchema = z.object({
    username: z.string().min(5),
    email: z.string().email(),
    password: z.string().min(8),
  });

  const { form, errors, constraints, enhance } = superForm(
    superValidateSync(registerSchema),
    {
      SPA: true,
      validators: registerSchema,
      onUpdate: async ({ form }) => {
        if (form.valid) {
          const res = await fetch("http://localhost:6060/api/v1/users", {
            method: "POST",
            headers: {
              "Content-Type": "application/json",
            },
            body: JSON.stringify(form.data),
          });

          await goto("/login");
        }
      },
    }
  );
</script>

<div class="register-container">
  <p>التسجيل</p>
  <form class="register-form" method="POST" use:enhance>
    <div class="field">
      <input
        type="text"
        id="username-input"
        name="username"
        data-invalid={$errors.username}
        bind:value={$form.username}
        {...$constraints.username}
      />
      <label for="username" id="username-label">اسم المستخدم</label>
    </div>
    {#if $errors.username}<small class="invalid">{$errors.username}</small>{/if}
    <div class="field">
      <input
        type="email"
        id="email-input"
        name="email"
        data-invalid={$errors.email}
        bind:value={$form.email}
        {...$constraints.email}
      />
      <label for="email" id="email-label">البريد الالكتروني</label>
    </div>
    {#if $errors.email}<small class="invalid">{$errors.email}</small>{/if}
    <div class="field">
      <input
        type="password"
        id="password-input"
        name="password"
        data-invalid={$errors.password}
        bind:value={$form.password}
        {...$constraints.password}
      />
      <label for="password" id="password-label">كلمة المرور</label>
    </div>
    {#if $errors.password}<small class="invalid">{$errors.password}</small>{/if}
    <button type="submit">أنشئ الحساب</button>
  </form>
  <p>لديك حساب؟ <a href="/login">سجل دخولك</a></p>
</div>

<style>
  .register-container {
    display: flex;
    flex-direction: column;
    width: 100%;
    justify-content: start;
    align-items: center;
    margin-top: 5em;
  }
  .register-form {
    display: flex;
    flex-direction: column;
    min-width: 70%;
    justify-content: center;
    align-items: center;
  }
  .register-form > * {
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
</style>
