<script lang="ts">
  import { z } from "zod";
  import {
    superForm,
    superValidateSync,
    setMessage,
  } from "sveltekit-superforms/client";
  import { goto } from "$app/navigation";
  import type { ErrorResponse } from "bindings/ErrorResponse";

  const loginSchema = z.object({
    email: z.string().email(),
    password: z.string().min(8),
  });

  const { form, message, errors, constraints, enhance } = superForm(
    superValidateSync(loginSchema),
    {
      SPA: true,
      validators: loginSchema,
      onUpdate: async ({ form }) => {
        if (form.valid) {
          const res = await fetch("http://localhost:6060/api/v1/users/login", {
            credentials: "include",
            method: "POST",
            headers: {
              "Content-Type": "application/json",
            },
            body: JSON.stringify(form.data),
          });
          if (res.status >= 400) {
            const err: ErrorResponse = await res.json();
            setMessage(form, err.error);
          } else {
            await goto("/");
          }
        }
      },
    }
  );
</script>

<div class="login-container">
  <p>تسجيل الدخول</p>
  {#if $message}<h3 class="invalid">{$message}</h3>{/if}
  <form class="login-form" method="POST" use:enhance>
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
    <button type="submit">سجل الدخول</button>
    <p>ليس لديك حساب؟ <a href="/register">سجل دخولك</a></p>
  </form>
</div>

<style>
  .login-container {
    display: flex;
    flex-direction: column;
    width: 100%;
    justify-content: start;
    align-items: center;
    margin-top: 5em;
  }
  .login-form {
    display: flex;
    flex-direction: column;
    min-width: 70%;
    justify-content: center;
    align-items: center;
  }
  .login-form > * {
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
  .invalid {
    color: red;
  }
</style>
