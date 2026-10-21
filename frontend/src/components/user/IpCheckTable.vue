<template>
  <DataTable :value="[check]" size="small">
    <Column field="provider" :header="t('user_registration_ip.provider')" />
    <Column :header="t('user_registration_ip.location')">
      <template #body="{ data }">{{ location(data) }}</template>
    </Column>
    <Column :header="t('user_registration_ip.network')">
      <template #body="{ data }">{{ [data.asn, data.organization].filter(Boolean).join(' ') }}</template>
    </Column>
    <Column v-for="flag in flags" :key="flag" :header="t(`user_registration_ip.${flag}`)">
      <template #body="{ data }">
        <span :class="{ flagged: data[flag] }">{{ flagLabel(data[flag]) }}</span>
      </template>
    </Column>
  </DataTable>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import DataTable from 'primevue/datatable'
import Column from 'primevue/column'
import type { IpCheckProvider, IpCheckResult } from '@/services/api-schema'

const { t } = useI18n()

defineProps<{
  check: IpCheckResult & { provider?: IpCheckProvider | null }
}>()

const flags = ['is_vpn', 'is_proxy', 'is_tor', 'is_hosting'] as const

const location = (result: IpCheckResult) => {
  const place = [result.city, result.region, result.country].filter(Boolean).join(', ')
  return result.country_code ? `${place} (${result.country_code})` : place
}

// null when the provider does not report it
const flagLabel = (value?: boolean | null) => (value == null ? '?' : t(value ? 'general.yes' : 'general.no'))
</script>

<style scoped>
.flagged {
  color: var(--p-red-400);
  font-weight: bold;
}
</style>
