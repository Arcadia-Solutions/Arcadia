<template>
  <div class="donations-table">
    <div class="actions wrapper-center" v-if="userStore.permissions.includes('create_donation')" style="color: white; margin-bottom: 20px">
      <i class="pi pi-plus cursor-pointer" v-tooltip.top="t('donation.create_donation')" @click="openDialog()" />
    </div>
    <DonationSearchForm v-if="initialForm" ref="searchFormRef" :loading :initialForm />
    <PaginatedResults
      v-if="initialForm && donations"
      :totalPages
      :initialPage="initialForm.page"
      :totalItems="totalItems"
      :pageSize
      @changePage="searchFormRef.changePage($event.page)"
    >
      <DataTable :value="donations" scrollable scrollHeight="70vh" size="small">
        <Column field="amount" :header="t('donation.amount')">
          <template #body="slotProps">
            {{ slotProps.data.amount.toFixed(2) }}
          </template>
        </Column>
        <Column field="donated_by.username" :header="t('donation.donated_by')">
          <template #body="slotProps">
            <UsernameEnriched :user="slotProps.data.donated_by" />
          </template>
        </Column>
        <Column field="donated_at" :header="t('donation.donated_at')">
          <template #body="slotProps">
            <TimeAgo :date="slotProps.data.donated_at" />
          </template>
        </Column>
        <Column field="created_by.username" :header="t('general.created_by')">
          <template #body="slotProps">
            <UsernameEnriched :user="slotProps.data.created_by" />
          </template>
        </Column>
        <Column field="created_at" :header="t('general.created_at')">
          <template #body="slotProps">
            <TimeAgo :date="slotProps.data.created_at" />
          </template>
        </Column>
        <Column field="note" :header="t('donation.note')">
          <template #body="slotProps">
            {{ slotProps.data.note || '-' }}
          </template>
        </Column>
        <Column :header="t('general.action', 2)">
          <template #body="slotProps">
            <i
              v-if="canEdit(slotProps.data)"
              class="pi pi-pen-to-square cursor-pointer"
              v-tooltip.top="t('general.edit')"
              @click="openDialog(slotProps.data)"
            />
          </template>
        </Column>
      </DataTable>
    </PaginatedResults>
    <Dialog closeOnEscape modal :header="isEditMode() ? t('donation.edit_donation') : t('donation.create_donation')" v-model:visible="dialogVisible">
      <CreateOrEditDonation :initialDonation="donationBeingEdited" @created="onDonationCreated" @edited="onDonationEdited" />
    </Dialog>
  </div>
</template>

<script setup lang="ts">
import TimeAgo from '@/components/TimeAgo.vue'
import { ref, onMounted, computed, watch, nextTick } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRoute } from 'vue-router'
import { Column, DataTable, Dialog } from 'primevue'
import { searchDonations, type DonationSearchResult, type SearchDonationsRequest, type Donation } from '@/services/api-schema'
import CreateOrEditDonation from '@/components/staff/CreateOrEditDonation.vue'
import DonationSearchForm from '@/components/staff/DonationSearchForm.vue'
import PaginatedResults from '@/components/PaginatedResults.vue'
import { useUserStore } from '@/stores/user'
import type { VNodeRef } from 'vue'
import UsernameEnriched from '../user/UsernameEnriched.vue'

const { t } = useI18n()
const userStore = useUserStore()
const route = useRoute()

const searchFormRef = ref<VNodeRef | null>(null)
const donations = ref<DonationSearchResult[]>()
const loading = ref(false)
const dialogVisible = ref(false)
const donationBeingEdited = ref<DonationSearchResult>()
const initialForm = ref<SearchDonationsRequest | null>(null)
const totalItems = ref(0)
const pageSize = ref(20)
const totalPages = computed(() => Math.ceil(totalItems.value / pageSize.value))

const isEditMode = () => !!donationBeingEdited.value

const openDialog = (donation?: DonationSearchResult) => {
  donationBeingEdited.value = donation
  dialogVisible.value = true
}

const canEdit = (donation: DonationSearchResult): boolean => {
  return userStore.permissions.includes('edit_donation') || donation.created_by_id === userStore.id
}

const onDonationCreated = () => {
  dialogVisible.value = false
  loadFormFromUrl()
}

const onDonationEdited = (editedDonation: Donation) => {
  const index = donations.value?.findIndex((d) => d.id === editedDonation.id)
  if (index !== undefined && index !== -1 && donations.value) {
    donations.value[index] = { ...donations.value[index], ...editedDonation }
  }
  dialogVisible.value = false
}

const search = async (form: SearchDonationsRequest) => {
  loading.value = true
  const response = await searchDonations(form).finally(() => {
    loading.value = false
  })
  pageSize.value = form.page_size
  totalItems.value = response.total_items
  donations.value = response.results
}

const loadFormFromUrl = async () => {
  loading.value = true
  initialForm.value = null
  await nextTick()

  const form: SearchDonationsRequest = {
    page: route.query.page ? parseInt(route.query.page as string) : 1,
    page_size: route.query.page_size ? parseInt(route.query.page_size as string) : 20,
    donated_by_id: route.query.donated_by_id ? parseInt(route.query.donated_by_id as string) : null,
    created_by_id: route.query.created_by_id ? parseInt(route.query.created_by_id as string) : null,
    min_amount: route.query.min_amount ? parseFloat(route.query.min_amount as string) : null,
    max_amount: route.query.max_amount ? parseFloat(route.query.max_amount as string) : null,
    donated_at_start: route.query.donated_at_start?.toString() ?? null,
    donated_at_end: route.query.donated_at_end?.toString() ?? null,
    order_by_column: (route.query.order_by_column as 'donated_at' | 'created_at' | 'amount') ?? 'donated_at',
    order_by_direction: (route.query.order_by_direction as 'asc' | 'desc') ?? 'desc',
  }

  initialForm.value = form
  pageSize.value = form.page_size
  search(form)
}

onMounted(() => {
  loadFormFromUrl()
})

watch(
  () => route.query,
  () => {
    loadFormFromUrl()
  },
  { deep: true },
)
</script>

<style scoped>
.donations-table {
  margin-top: 20px;
}
</style>
