<template>
  <LazyAutoComplete
    v-model="username"
    :suggestions="foundUsers"
    @complete="search"
    size="small"
    :placeholder
    optionLabel="username"
    @option-select="userSelected"
    @input="onInput"
  >
    <template #option="slotProps">
      <RouterLink v-if="clickableUserLink" :to="`/user/${slotProps.option.id}`" style="width: 100%">
        {{ slotProps.option.username }}
      </RouterLink>
      <span v-else>{{ slotProps.option.username }}</span>
    </template>
  </LazyAutoComplete>
</template>

<script lang="ts" setup>
import { ref } from 'vue'
import type { AutoCompleteOptionSelectEvent } from 'primevue'
import LazyAutoComplete from '@/components/LazyAutoComplete.vue'
import { searchUsersLite, type UserLite } from '@/services/api-schema'

const props = defineProps<{
  placeholder: string
  clearInputOnSelect: boolean
  modelValue: string
  clickableUserLink: boolean
  initialValue?: string
}>()

const emit = defineEmits<{
  'update:modelValue': [string]
  userSelected: [UserLite]
}>()

const username = ref<string>(props.initialValue || '')

const foundUsers = ref<UserLite[]>()

const userSelected = (event: AutoCompleteOptionSelectEvent) => {
  const selectedUsername = event.value.username
  emit('userSelected', event.value)
  emit('update:modelValue', selectedUsername)
  if (props.clearInputOnSelect) {
    username.value = ''
  } else {
    username.value = selectedUsername
  }
}

const onInput = () => {
  emit('update:modelValue', username.value)
}

const search = () => {
  if (username.value !== '') {
    searchUsersLite(username.value).then((users) => {
      foundUsers.value = users
    })
  } else {
    foundUsers.value = []
  }
}
</script>
