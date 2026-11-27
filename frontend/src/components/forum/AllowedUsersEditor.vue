<template>
  <div class="allowed-users-editor">
    <UserSearchBar :placeholder="t('forum.add_allowed_user')" :clickableUserLink="false" :clearInputOnSelect="true" modelValue="" @userSelected="addUser" />
    <div v-for="user in allowedUsers" :key="user.id" class="user-row">
      <UsernameEnriched :user="user" />
      <Button
        type="button"
        icon="pi pi-times"
        size="small"
        text
        severity="secondary"
        :aria-label="t('forum.remove_allowed_user', { username: user.username })"
        @click="removeUser(user)"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import Button from 'primevue/button'
import UserSearchBar from '@/components/user/UserSearchBar.vue'
import UsernameEnriched from '@/components/user/UsernameEnriched.vue'
import type { UserLite } from '@/services/api-schema'

const { t } = useI18n()

const allowedUsers = defineModel<UserLite[]>({ required: true })

const addUser = (user: UserLite) => {
  if (allowedUsers.value.some((allowedUser) => allowedUser.id === user.id)) return
  allowedUsers.value = [...allowedUsers.value, user]
}

const removeUser = (user: UserLite) => {
  allowedUsers.value = allowedUsers.value.filter((allowedUser) => allowedUser.id !== user.id)
}
</script>

<style scoped>
.allowed-users-editor {
  display: flex;
  flex-direction: column;
  gap: 10px;
}
.user-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
}
</style>
