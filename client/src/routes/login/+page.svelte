<script lang="ts">
</script>

<div class="login-container">
    <p>تسجيل الدخول</p>
    <form class="login-form" method="POST">
        <div class="field">
            <input type="email" id="email-input" name="email" />
            <label for="email" id="email-label">البريد الالكتروني</label>
        </div>
        <div class="field">
            <input type="password" id="password-input" name="password" />
            <label for="password" id="password-label">كلمة المرور</label>
        </div>
        <button type="submit">سجل الدخول</button>
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
</style>
