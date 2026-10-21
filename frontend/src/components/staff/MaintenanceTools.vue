<template>
  <div class="maintenance-tools">
    <div class="tool">
      <div class="tool-info">
        <h3>{{ t('maintenance_tools.rehash_torrents') }}</h3>
        <p>{{ t('maintenance_tools.rehash_torrents_description') }}</p>
      </div>
      <Button size="small" :label="t('maintenance_tools.run')" :loading="rehashing" @click="rehashTorrents" />
    </div>
    <div class="tool">
      <div class="tool-info">
        <h3>{{ t('maintenance_tools.recompute_cached_amounts') }}</h3>
        <p>{{ t('maintenance_tools.recompute_cached_amounts_description') }}</p>
      </div>
      <Button size="small" :label="t('maintenance_tools.run')" :loading="recomputingCachedAmounts" @click="recomputeCachedAmountsTool" />
    </div>
    <div class="tool">
      <div class="tool-info">
        <h3>{{ t('maintenance_tools.check_missing_registration_ips') }}</h3>
        <p>{{ t('maintenance_tools.check_missing_registration_ips_description') }}</p>
      </div>
      <Button size="small" :label="t('maintenance_tools.run')" :loading="checkingRegistrationIps" @click="checkMissingRegistrationIpsTool" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { useI18n } from 'vue-i18n'
import Button from 'primevue/button'
import { rehashTorrentsWithSourceTag, recomputeCachedAmounts, checkMissingRegistrationIps } from '@/services/api-schema'
import { showToast } from '@/main'

const { t } = useI18n()

const rehashing = ref(false)
const recomputingCachedAmounts = ref(false)
const checkingRegistrationIps = ref(false)

const rehashTorrents = () => {
  rehashing.value = true
  // No timeout: rehashing every torrent can take a long time.
  rehashTorrentsWithSourceTag({ timeout: 0 })
    .then((data) => {
      showToast('Success', t('maintenance_tools.rehash_torrents_success', { amount: data.updated_torrents_amount }), 'success', 5000)
    })
    .finally(() => {
      rehashing.value = false
    })
}

const recomputeCachedAmountsTool = () => {
  recomputingCachedAmounts.value = true
  // No timeout: recomputing every cached count can take a long time.
  recomputeCachedAmounts({ timeout: 0 })
    .then(() => {
      showToast('Success', t('maintenance_tools.recompute_cached_amounts_success'), 'success', 5000)
    })
    .finally(() => {
      recomputingCachedAmounts.value = false
    })
}

const checkMissingRegistrationIpsTool = () => {
  checkingRegistrationIps.value = true
  // No timeout: every check is a request to a provider.
  checkMissingRegistrationIps({ timeout: 0 })
    .then((data) => {
      showToast('Success', t('maintenance_tools.check_missing_registration_ips_success', { amount: data.checked_users_amount }), 'success', 5000)
    })
    .finally(() => {
      checkingRegistrationIps.value = false
    })
}
</script>

<style scoped>
.maintenance-tools {
  display: flex;
  flex-direction: column;
  gap: 20px;
}
.tool {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 20px;
  padding: 15px;
  border: 1px solid var(--color-border);
  border-radius: 5px;
}
.tool-info h3 {
  margin: 0 0 5px;
}
.tool-info p {
  margin: 0;
  opacity: 0.8;
}
</style>
