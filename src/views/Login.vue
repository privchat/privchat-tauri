<script setup lang="ts">
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { message } from '@tauri-apps/plugin-dialog'
import router from '../router/router.ts'

const username = ref('')
const password = ref('')
const serverUrl = ref('tcp://127.0.0.1:9001')
const isSigningIn = ref(false)
const showCredentialForm = ref(false)
const authMode = ref<'signin' | 'signup'>('signin')

function showSignupForm() {
  authMode.value = 'signup'
  showCredentialForm.value = true
}

function showSignInForm() {
  authMode.value = 'signin'
  showCredentialForm.value = true
}

function backToWelcome() {
  showCredentialForm.value = false
}

async function handleSignin() {
  try {
    isSigningIn.value = true
    const command = authMode.value === 'signup' ? 'password_register' : 'password_login'
    await invoke(command, {
      username: username.value,
      password: password.value,
      serverUrl: serverUrl.value,
    })
    await router.replace({ name: 'home' })
  } catch (err: any) {
    const failedPrefix = authMode.value === 'signup' ? 'Signup failed' : 'Signin failed'
    await message(`${failedPrefix}: ${err?.message ?? err}`, {
      title: authMode.value === 'signup' ? 'Signup Error' : 'Signin Error',
      kind: 'error',
    })
  } finally {
    isSigningIn.value = false
  }
}
</script>

<template>
  <div class="login-container">
    <div class="login-card">
      <h1 class="title">Welcome to PrivChat!</h1>

      <template v-if="!showCredentialForm">
        <button class="btn btn-primary" @click="showSignupForm">Sign up</button>
        <button class="btn btn-secondary" @click="showSignInForm">Sign in</button>
      </template>

      <template v-else>
        <div class="form">
          <input v-model="serverUrl" class="input" placeholder="tcp://127.0.0.1:9001" />
          <input v-model="username" class="input" placeholder="username" />
          <input v-model="password" class="input" type="password" placeholder="password" />
        </div>

        <button class="btn btn-primary" @click="handleSignin" :disabled="isSigningIn">
          {{ isSigningIn ? (authMode === 'signup' ? 'Signing up...' : 'Signing in...') : (authMode === 'signup' ? 'Sign up' : 'Sign in') }}
        </button>
        <button class="btn btn-secondary" @click="backToWelcome" :disabled="isSigningIn">Back</button>
      </template>
    </div>
  </div>
</template>

<style scoped>
.login-container {
  min-height: 100vh;
  display: flex;
  align-items: center;
  justify-content: center;
  background-color: #f8fafc;
  padding: 20px;
}

.login-card {
  background: #ffffff;
  padding: 48px;
  width: 100%;
  max-width: 560px;
  text-align: center;
  border-radius: 24px;
  border: 1px solid #e2e8f0;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
}

.title {
  font-size: 56px;
  font-weight: 700;
  color: #1a1a1a;
  margin: 0 0 48px 0;
  line-height: 1.1;
}

.form {
  display: flex;
  flex-direction: column;
  gap: 10px;
  margin-bottom: 14px;
}

.input {
  width: 100%;
  border: 1px solid #d1d5db;
  border-radius: 12px;
  padding: 14px 16px;
  font-size: 15px;
  margin-bottom: 8px;
}

.btn {
  width: 100%;
  padding: 18px;
  border-radius: 14px;
  font-size: 16px;
  font-weight: 500;
  cursor: pointer;
  margin-bottom: 14px;
}

.btn-primary {
  background: #2563eb;
  color: white;
  border: none;
}

.btn-primary:hover:not(:disabled) {
  background: #1d4ed8;
}

.btn-secondary {
  background: #ffffff;
  color: #374151;
  border: 1px solid #cbd5e1;
}

.btn-secondary:hover:not(:disabled) {
  background: #f8fafc;
}

.btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}
</style>
