<template>
  <Form :initialValues="form" @submit="handleLogin" class="form">
    <InputText class="form-item" name="username" type="text" :placeholder="t('user.username')" v-model="form.username" />
    <Password class="form-item" name="password" v-model="form.password" :placeholder="t('user.password')" :feedback="false" toggleMask />
    <div class="remember-wrapper">
      <Checkbox inputId="remember-me" v-model="form.remember_me" binary />
      <label for="remember-me"> {{ t('auth.remember_me') }} </label>
    </div>
    <Button class="form-item w-full" type="submit" severity="secondary" :label="t('user.login')" :loading />
  </Form>
</template>
<script setup lang="ts">
import InputText from 'primevue/inputtext'
import Password from 'primevue/password'
import { Form } from '@primevue/forms'
import Button from 'primevue/button'
import Checkbox from 'primevue/checkbox'
import { useRouter, useRoute } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { ref } from 'vue'
import { login, type Login } from '@/services/api-schema'

const form = ref<Login>({
  username: '',
  password: '',
  remember_me: false,
})

const router = useRouter()
const route = useRoute()
const loading = ref(false)
const { t } = useI18n()

const handleLogin = async () => {
  loading.value = true
  login(form.value)
    .then(async (data) => {
      localStorage.setItem('token', data.token)
      localStorage.setItem('refreshToken', data.refresh_token)
      // honor a ?redirect target (e.g. the SSO authorize page sends the user here to log in)
      const redirect = route.query.redirect
      router.push(typeof redirect === 'string' ? redirect : '/')
    })
    .finally(() => {
      loading.value = false
    })
}
</script>
<style scoped>
.remember-wrapper {
  display: flex;
  gap: 10px;
  margin-top: 10px;
  align-items: center;
}
.form {
  display: flex;
  flex-direction: column;
}
</style>
