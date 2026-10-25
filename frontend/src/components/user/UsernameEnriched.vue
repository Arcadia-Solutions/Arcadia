<template>
  <span class="username-enriched" v-if="user">
    <RouterLink :to="`/user/${user.id}`" class="bold" v-if="!noLink">
      {{ user.username }}
    </RouterLink>
    <template v-else>
      {{ user.username }}
    </template>
    <i v-if="!noStatus && user.banned" v-tooltip.top="t('user.banned')" class="danger pi pi-ban" />
    <i v-if="!noStatus && !user.banned && user.warned" v-tooltip.top="t('user.warned')" class="warning pi pi-exclamation-triangle" />
    <template v-if="displayAllInfo">
      <span class="bold"> ({{ (user as UserLiteAvatar).class_name }}) </span>
      <span v-if="(user as UserLiteAvatar).custom_title"> ({{ (user as UserLiteAvatar).custom_title }}) </span>
    </template>
  </span>
  <span v-else>{{ t('general.anonymous') }}</span>
</template>

<script setup lang="ts">
import { RouterLink } from 'vue-router'
import { useI18n } from 'vue-i18n'
import type { UserLite, UserLiteAvatar } from '@/services/api-schema'

const { t } = useI18n()

defineProps<
  | {
      user: UserLite | null | undefined
      /* Cannot be true if user is UserLite since some properties are missing */
      displayAllInfo?: false
      noLink?: boolean
      noStatus?: boolean
    }
  | {
      user: UserLiteAvatar | null | undefined
      displayAllInfo?: boolean
      noLink?: boolean
      noStatus?: boolean
    }
>()
</script>
<style scoped>
.username-enriched {
  display: inline;
}
.pi {
  margin-left: 4px;
}
</style>
