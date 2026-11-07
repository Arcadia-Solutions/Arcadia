<template>
  <div class="change-password">
    <Form :initialValues="form" :resolver @submit="submit" validateOnSubmit :validateOnValueUpdate="false" validateOnBlur v-slot="$form">
      <FloatLabel class="input">
        <Password name="current_password" v-model="form.current_password" :feedback="false" toggleMask fluid />
        <label>{{ t('user.current_password') }}</label>
        <Message v-if="$form.current_password?.invalid" severity="error" size="small" variant="simple">
          {{ $form.current_password.error?.message }}
        </Message>
      </FloatLabel>
      <FloatLabel class="input">
        <Password name="new_password" v-model="form.new_password" toggleMask fluid />
        <label>{{ t('user.new_password') }}</label>
        <Message v-if="$form.new_password?.invalid" severity="error" size="small" variant="simple">
          {{ $form.new_password.error?.message }}
        </Message>
      </FloatLabel>
      <FloatLabel class="input">
        <Password name="new_password_verify" v-model="form.new_password_verify" :feedback="false" toggleMask fluid />
        <label>{{ t('user.new_password_verify') }}</label>
        <Message v-if="$form.new_password_verify?.invalid" severity="error" size="small" variant="simple">
          {{ $form.new_password_verify.error?.message }}
        </Message>
      </FloatLabel>
      <div class="wrapper-center">
        <Button :label="t('general.confirm')" size="small" type="submit" :loading />
      </div>
    </Form>
  </div>
</template>

<script setup lang="ts">
import { showToast } from '@/main'
import { changeUserPassword, type UserChangedPassword } from '@/services/api-schema'
import { validatePasswordStrength } from '@/services/helpers'
import { FloatLabel, Message, Password } from 'primevue'
import { Form, type FormResolverOptions, type FormSubmitEvent } from '@primevue/forms'
import Button from 'primevue/button'
import { ref } from 'vue'
import { useI18n } from 'vue-i18n'

const { t } = useI18n()

const emit = defineEmits<{
  saved: []
}>()

const form = ref<UserChangedPassword>({
  current_password: '',
  new_password: '',
  new_password_verify: '',
})
const loading = ref(false)

const resolver = ({ values }: FormResolverOptions) => {
  const errors: Partial<Record<keyof UserChangedPassword, { message: string }[]>> = {}

  if (!values.current_password) {
    errors.current_password = [{ message: t('auth_validation.password_too_short') }]
  }

  const passwordValidation = validatePasswordStrength(values.new_password, t)
  if (!passwordValidation.isValid) {
    errors.new_password = [{ message: passwordValidation.message }]
  }

  if (values.new_password !== values.new_password_verify) {
    errors.new_password_verify = [{ message: t('auth_validation.password_mismatch') }]
  }

  return { errors }
}

const submit = ({ valid }: FormSubmitEvent) => {
  if (!valid) {
    return
  }
  loading.value = true
  changeUserPassword({
    current_password: form.value.current_password,
    new_password: form.value.new_password,
    new_password_verify: form.value.new_password_verify,
  })
    .then((tokens) => {
      // changing the password closes every session, the returned tokens keep this one open
      localStorage.setItem('arcadia_token', tokens.token)
      localStorage.setItem('arcadia_refreshToken', tokens.refresh_token)
      showToast('Success', t('user.password_changed_success'), 'success', 4000)
      emit('saved')
    })
    .finally(() => {
      loading.value = false
    })
}
</script>

<style scoped>
.change-password {
  width: 60vw;
}
.input {
  margin-bottom: 25px;
  width: 60%;
}
</style>
