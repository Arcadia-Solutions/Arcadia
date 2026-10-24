<template>
  <div class="user-badges-table">
    <Tabs :value="innerTab" @update:value="(t) => (innerTab = t as string)">
      <TabList>
        <Tab value="badges">{{ t('user_badge.user_badge', 2) }}</Tab>
        <Tab value="categories">{{ t('user_badge.user_badge_category', 2) }}</Tab>
      </TabList>
      <TabPanels>
        <TabPanel value="badges">
          <div class="actions" v-if="canCreateBadge">
            <i class="pi pi-plus cursor-pointer" v-tooltip.top="t('user_badge.create_user_badge')" @click="openBadgeDialog()" />
          </div>
          <DataTable :value="visibleBadges" :loading="loadingBadges" size="small">
            <Column :header="t('user_badge.image_url')" style="width: 80px">
              <template #body="{ data }">
                <img v-if="data.image_url" :src="data.image_url" class="badge-thumb" :alt="data.name" />
              </template>
            </Column>
            <Column field="name" :header="t('general.name')" />
            <Column field="description" :header="t('general.description')" />
            <Column :header="t('user_badge.category')">
              <template #body="{ data }">
                {{ categories.find((c) => c.id === data.category_id)?.name ?? '' }}
              </template>
            </Column>
            <Column :header="t('user_badge.badge_type')">
              <template #body="{ data }">
                {{ t(`user_badge.type_${data.badge_type}`) }}
              </template>
            </Column>
            <Column :header="t('user_badge.is_secret')" style="width: 80px">
              <template #body="{ data }">
                <i v-if="data.is_secret" class="pi pi-check" style="color: green" />
                <i v-else class="pi pi-times" style="color: red" />
              </template>
            </Column>
            <Column style="width: 80px">
              <template #body="{ data }">
                <i v-if="canEditBadge" v-tooltip.top="t('general.edit')" class="action pi pi-pen-to-square cursor-pointer" @click="openBadgeDialog(data)" />
                <i
                  v-if="canDeleteBadge"
                  v-tooltip.top="t('general.delete')"
                  class="action pi pi-trash cursor-pointer"
                  style="color: var(--p-red-500); margin-left: 8px"
                  @click="confirmDeleteBadge(data)"
                />
              </template>
            </Column>
          </DataTable>
        </TabPanel>

        <TabPanel value="categories">
          <div class="actions" v-if="canCreateCategory">
            <i class="pi pi-plus cursor-pointer" v-tooltip.top="t('user_badge.create_user_badge_category')" @click="openCategoryDialog()" />
          </div>
          <DataTable :value="categories" :loading="loadingCategories" size="small" stripedRows>
            <Column field="name" :header="t('general.name')" />
            <Column field="created_at" :header="t('general.created_at')">
              <template #body="{ data }"><TimeAgo :date="data.created_at" /></template>
            </Column>
            <Column style="width: 80px">
              <template #body="{ data }">
                <i
                  v-if="canEditCategory"
                  v-tooltip.top="t('general.edit')"
                  class="action pi pi-pen-to-square cursor-pointer"
                  @click="openCategoryDialog(data)"
                />
                <i
                  v-if="canDeleteCategory"
                  v-tooltip.top="t('general.delete')"
                  class="action pi pi-trash cursor-pointer"
                  style="color: var(--p-red-500); margin-left: 8px"
                  @click="confirmDeleteCategory(data)"
                />
              </template>
            </Column>
          </DataTable>
        </TabPanel>
      </TabPanels>
    </Tabs>

    <Dialog
      closeOnEscape
      modal
      :header="badgeBeingEdited ? t('user_badge.edit_user_badge') : t('user_badge.create_user_badge')"
      v-model:visible="badgeDialogVisible"
    >
      <CreateOrEditUserBadgeDialog v-if="badgeDialogVisible" :categories="categories" :initialBadge="badgeBeingEdited" @done="onBadgeSaved" />
    </Dialog>

    <Dialog
      closeOnEscape
      modal
      :header="categoryBeingEdited ? t('user_badge.edit_user_badge_category') : t('user_badge.create_user_badge_category')"
      v-model:visible="categoryDialogVisible"
    >
      <CreateOrEditUserBadgeCategoryDialog v-if="categoryDialogVisible" :initialCategory="categoryBeingEdited" @done="onCategorySaved" />
    </Dialog>

    <Dialog closeOnEscape modal :header="t('user_badge.delete_user_badge')" v-model:visible="deleteBadgeDialogVisible">
      <div class="delete-dialog">
        <p>{{ t('user_badge.confirm_delete_user_badge') }}</p>
        <Button :label="t('general.delete')" severity="danger" size="small" :loading="deletingBadge" @click="deleteBadge" />
      </div>
    </Dialog>

    <Dialog closeOnEscape modal :header="t('user_badge.delete_user_badge_category')" v-model:visible="deleteCategoryDialogVisible">
      <div class="delete-dialog">
        <p>{{ t('user_badge.confirm_delete_user_badge_category') }}</p>
        <Button :label="t('general.delete')" severity="danger" size="small" :loading="deletingCategory" @click="deleteCategory" />
      </div>
    </Dialog>
  </div>
</template>

<script setup lang="ts">
import TimeAgo from '@/components/TimeAgo.vue'
import { ref, computed, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { Column, DataTable, Dialog, Button, Tabs, TabList, Tab, TabPanels, TabPanel } from 'primevue'
import {
  listUserBadges,
  listUserBadgeCategories,
  deleteUserBadge as deleteUserBadgeApi,
  deleteUserBadgeCategory as deleteUserBadgeCategoryApi,
  type UserBadge,
  type UserBadgeCategory,
  type UserBadgeListItem,
} from '@/services/api-schema'
import { useUserStore } from '@/stores/user'
import { showToast } from '@/main'
import CreateOrEditUserBadgeDialog from '@/components/user/CreateOrEditUserBadgeDialog.vue'
import CreateOrEditUserBadgeCategoryDialog from '@/components/user/CreateOrEditUserBadgeCategoryDialog.vue'

const { t } = useI18n()
const userStore = useUserStore()

const innerTab = ref('badges')

const badges = ref<UserBadgeListItem[]>([])
const categories = ref<UserBadgeCategory[]>([])
const loadingBadges = ref(true)
const loadingCategories = ref(true)

const badgeDialogVisible = ref(false)
const categoryDialogVisible = ref(false)
const badgeBeingEdited = ref<UserBadge | undefined>()
const categoryBeingEdited = ref<UserBadgeCategory | undefined>()

const deleteBadgeDialogVisible = ref(false)
const deleteCategoryDialogVisible = ref(false)
const badgeBeingDeleted = ref<UserBadge | null>(null)
const categoryBeingDeleted = ref<UserBadgeCategory | null>(null)
const deletingBadge = ref(false)
const deletingCategory = ref(false)

const isFullBadge = (b: UserBadgeListItem): b is UserBadge => 'name' in b
const visibleBadges = computed(() => badges.value.filter(isFullBadge))

const canCreateBadge = computed(() => userStore.permissions.includes('create_user_badge'))
const canEditBadge = computed(() => userStore.permissions.includes('edit_user_badge'))
const canDeleteBadge = computed(() => userStore.permissions.includes('delete_user_badge'))
const canCreateCategory = computed(() => userStore.permissions.includes('create_user_badge_category'))
const canEditCategory = computed(() => userStore.permissions.includes('edit_user_badge_category'))
const canDeleteCategory = computed(() => userStore.permissions.includes('delete_user_badge_category'))

const openBadgeDialog = (badge?: UserBadge) => {
  badgeBeingEdited.value = badge
  badgeDialogVisible.value = true
}

const openCategoryDialog = (category?: UserBadgeCategory) => {
  categoryBeingEdited.value = category
  categoryDialogVisible.value = true
}

const onBadgeSaved = (saved: UserBadge) => {
  if (badgeBeingEdited.value) {
    const idx = badges.value.findIndex((b) => b.id === saved.id)
    if (idx !== -1) badges.value[idx] = saved
  } else {
    badges.value.push(saved)
  }
  badgeDialogVisible.value = false
}

const onCategorySaved = (saved: UserBadgeCategory) => {
  if (categoryBeingEdited.value) {
    const idx = categories.value.findIndex((c) => c.id === saved.id)
    if (idx !== -1) categories.value[idx] = saved
  } else {
    categories.value.push(saved)
  }
  categoryDialogVisible.value = false
}

const confirmDeleteBadge = (badge: UserBadge) => {
  badgeBeingDeleted.value = badge
  deleteBadgeDialogVisible.value = true
}
const confirmDeleteCategory = (category: UserBadgeCategory) => {
  categoryBeingDeleted.value = category
  deleteCategoryDialogVisible.value = true
}

const deleteBadge = () => {
  if (!badgeBeingDeleted.value) return
  deletingBadge.value = true
  const id = badgeBeingDeleted.value.id
  deleteUserBadgeApi(id)
    .then(() => {
      badges.value = badges.value.filter((b) => b.id !== id)
      showToast('', t('user_badge.user_badge_deleted_success'), 'success', 2000)
      deleteBadgeDialogVisible.value = false
    })
    .finally(() => {
      deletingBadge.value = false
    })
}

const deleteCategory = () => {
  if (!categoryBeingDeleted.value) return
  deletingCategory.value = true
  const id = categoryBeingDeleted.value.id
  deleteUserBadgeCategoryApi(id)
    .then(() => {
      categories.value = categories.value.filter((c) => c.id !== id)
      showToast('', t('user_badge.user_badge_category_deleted_success'), 'success', 2000)
      deleteCategoryDialogVisible.value = false
    })
    .finally(() => {
      deletingCategory.value = false
    })
}

onMounted(() => {
  listUserBadges()
    .then((data) => {
      badges.value = data
    })
    .finally(() => {
      loadingBadges.value = false
    })
  listUserBadgeCategories()
    .then((data) => {
      categories.value = data
    })
    .finally(() => {
      loadingCategories.value = false
    })
})
</script>

<style scoped>
.user-badges-table {
  margin-top: 20px;
}
.actions {
  color: white;
  margin-bottom: 15px;
  display: flex;
  justify-content: center;
}
.badge-thumb {
  width: 48px;
  height: 48px;
  object-fit: contain;
  border-radius: 6px;
}
.delete-dialog {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 20px;
}
</style>
