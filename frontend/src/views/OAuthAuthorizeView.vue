<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { oAuthAuthorize } from '@/services/api-schema'

const { t } = useI18n()
const failed = ref(false)

// the discovery document's authorization endpoint: the client sent the browser here, and the
// backend validates the request it forwards with the user's credentials
onMounted(() => {
  oAuthAuthorize({ authorization_query: window.location.search.slice(1) })
    .then((authorization) => {
      window.location.href = authorization.redirect_url
    })
    .catch(() => {
      failed.value = true
    })
})
</script>

<template>
  <p class="oauth-redirect">{{ t(failed ? 'oauth.failed' : 'oauth.signing_in') }}</p>
</template>

<style scoped>
.oauth-redirect {
  margin: 4rem auto;
  text-align: center;
}
</style>
