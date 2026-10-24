<template>
  <div class="api-keys">
    <Message v-if="createdApiKey" severity="warn" class="created-api-key">
      <div>{{ t('user_settings.api_key_created_warning') }}</div>
      <div class="created-api-key-value">
        <code>{{ createdApiKey.value }}</code>
        <Button :label="t('general.copy')" size="small" severity="secondary" @click="copyCreatedApiKey" />
      </div>
    </Message>
    <DataTable :value="apiKeys" size="small" :loading>
      <template #empty>{{ t('user_settings.no_api_keys') }}</template>
      <Column :header="t('user_settings.api_key_name')" field="name" />
      <Column :header="t('user_settings.api_key_value')">
        <template #body="{ data }">
          <code>••••{{ data.last_four }}</code>
        </template>
      </Column>
      <Column :header="t('user_settings.api_key_scopes')">
        <template #body="{ data }">
          <Tag v-for="scope in data.scopes" :key="scope" :value="t(`user_settings.api_key_scope.${scope}`)" severity="secondary" class="scope" />
        </template>
      </Column>
      <Column :header="t('user_settings.api_key_created_at')">
        <template #body="{ data }"><TimeAgo :date="data.created_at" /></template>
      </Column>
      <Column :header="t('user_settings.api_key_last_used_at')">
        <template #body="{ data }"
          ><TimeAgo v-if="data.last_used_at" :date="data.last_used_at" /><template v-else>{{ t('user_settings.api_key_never_used') }}</template></template
        >
      </Column>
      <Column>
        <template #body="{ data }">
          <Button icon="pi pi-trash" size="small" severity="danger" text @click="apiKeyToDelete = data" />
        </template>
      </Column>
    </DataTable>
    <div class="creation">
      <InputText v-model="newApiKeyName" size="small" maxlength="30" :placeholder="t('user_settings.api_key_name')" />
      <MultiSelect
        v-model="newApiKeyScopes"
        size="small"
        :options="Object.values(APIKeyScope).map((scope) => ({ scope, label: t(`user_settings.api_key_scope.${scope}`) }))"
        optionLabel="label"
        optionValue="scope"
        :placeholder="t('user_settings.api_key_scopes')"
      />
      <Button
        :label="t('user_settings.create_api_key')"
        size="small"
        :disabled="newApiKeyName.length === 0 || newApiKeyScopes.length === 0"
        :loading="creating"
        @click="create"
      />
    </div>
  </div>
  <Dialog closeOnEscape modal :header="t('user_settings.delete_api_key')" :visible="apiKeyToDelete !== null" @update:visible="apiKeyToDelete = null">
    <p>{{ t('user_settings.delete_api_key_confirm', { name: apiKeyToDelete?.name }) }}</p>
    <template #footer>
      <Button :label="t('general.cancel')" severity="secondary" size="small" @click="apiKeyToDelete = null" />
      <Button :label="t('general.confirm')" severity="danger" size="small" :loading="deleting" @click="deleteKey" />
    </template>
  </Dialog>
</template>
<script setup lang="ts">
import TimeAgo from '@/components/TimeAgo.vue'
import { onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { Button, Column, DataTable, Dialog, InputText, Message, MultiSelect, Tag } from 'primevue'
import { APIKeyScope, createAPIKey, deleteAPIKey, getAPIKeys, type APIKey, type CreatedAPIKey } from '@/services/api-schema'
import { showToast } from '@/main'

const { t } = useI18n()

const apiKeys = ref<APIKey[]>([])
const createdApiKey = ref<CreatedAPIKey | null>(null)
const newApiKeyName = ref('')
const newApiKeyScopes = ref<APIKeyScope[]>([])
const loading = ref(true)
const creating = ref(false)
// the API key awaiting the user's confirmation, `null` when the confirmation dialog is closed
const apiKeyToDelete = ref<APIKey | null>(null)
const deleting = ref(false)

onMounted(() => {
  getAPIKeys()
    .then((keys) => {
      apiKeys.value = keys
    })
    .finally(() => {
      loading.value = false
    })
})

const create = () => {
  creating.value = true
  createAPIKey({ name: newApiKeyName.value, scopes: newApiKeyScopes.value })
    .then((created) => {
      createdApiKey.value = created
      apiKeys.value.unshift(created.api_key)
      newApiKeyName.value = ''
      newApiKeyScopes.value = []
    })
    .finally(() => {
      creating.value = false
    })
}

const deleteKey = () => {
  const apiKeyId = apiKeyToDelete.value?.id
  if (apiKeyId === undefined) return
  deleting.value = true
  deleteAPIKey(apiKeyId)
    .then(() => {
      apiKeys.value = apiKeys.value.filter((apiKey) => apiKey.id !== apiKeyId)
      if (createdApiKey.value?.api_key.id === apiKeyId) {
        createdApiKey.value = null
      }
      apiKeyToDelete.value = null
      showToast('', t('user_settings.api_key_deleted'), 'success', 3000)
    })
    .finally(() => {
      deleting.value = false
    })
}

const copyCreatedApiKey = () => {
  if (!createdApiKey.value) return
  navigator.clipboard.writeText(createdApiKey.value.value).then(() => {
    showToast('', t('user_settings.api_key_copied'), 'success', 3000)
  })
}
</script>
<style scoped>
.api-keys {
  width: 60vw;
}
.created-api-key {
  margin-bottom: 15px;
}
.created-api-key-value {
  align-items: center;
  display: flex;
  gap: 10px;
  margin-top: 5px;
}
.scope {
  margin-right: 5px;
}
.creation {
  align-items: center;
  display: flex;
  gap: 15px;
  margin-top: 20px;
}
</style>
