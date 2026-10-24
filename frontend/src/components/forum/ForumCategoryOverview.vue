<template>
  <div>
    <div class="top">
      <div class="title">{{ forumCategory.name }}</div>
      <div class="actions">
        <RouterLink
          :to="`/forum/category/${forumCategory.id}/edit`"
          v-if="userStore.permissions.includes('edit_forum_category')"
          v-tooltip.top="t('forum.edit_category')"
        >
          <i class="pi pi-pen-to-square" />
        </RouterLink>
        <RouterLink
          :to="{ path: '/forum/sub-category/new', query: { categoryId: forumCategory.id, categoryName: forumCategory.name } }"
          v-if="userStore.permissions.includes('create_forum_sub_category')"
          v-tooltip.top="t('forum.create_sub_category')"
        >
          <i class="pi pi-plus" />
        </RouterLink>
        <i
          v-if="userStore.permissions.includes('edit_forum_sub_category')"
          class="pi pi-arrow-right-arrow-left"
          v-tooltip.top="t('forum.reorder_subcategories')"
          @click="reorderSubCategoriesDialogVisible = true"
        />
        <i
          v-if="userStore.permissions.includes('delete_forum_category')"
          class="pi pi-trash"
          v-tooltip.top="t('forum.delete_category')"
          @click="deleteCategoryDialogVisible = true"
        />
      </div>
    </div>
    <DataTable :value="forumCategory.sub_categories">
      <Column style="width: 30%" field="name" :header="t('general.name')">
        <template #body="slotProps">
          <RouterLink :to="'/forum/sub-category/' + slotProps.data.id">
            {{ slotProps.data.name }}
          </RouterLink>
        </template>
      </Column>
      <Column style="width: 35%" field="latest_post_in_thread.name" :header="t('forum.latest_post')">
        <template #body="slotProps">
          <RouterLink :to="'/forum/thread/' + slotProps.data.latest_post_in_thread.thread_id" v-if="slotProps.data.latest_post_in_thread">
            {{ slotProps.data.latest_post_in_thread.name }}
          </RouterLink>
        </template>
      </Column>
      <Column style="width: 25%" field="latest_post_in_thread.created_at">
        <template #body="slotProps">
          <template v-if="slotProps.data.latest_post_in_thread">
            <TimeAgo :date="slotProps.data.latest_post_in_thread.created_at" /> {{ t('general.by') }}
            <RouterLink :to="'/user/' + slotProps.data.latest_post_in_thread.created_by.id">
              {{ slotProps.data.latest_post_in_thread.created_by.username }}
            </RouterLink>
          </template>
        </template>
      </Column>
      <Column style="width: 5%" field="threads_amount" :header="t('forum.thread', 2)">
        <template #body="slotProps">{{ formatNumber(slotProps.data.threads_amount) }}</template>
      </Column>
      <Column style="width: 5%" field="posts_amount" :header="t('forum.posts')">
        <template #body="slotProps">{{ formatNumber(slotProps.data.posts_amount) }}</template>
      </Column>
    </DataTable>
    <Dialog closeOnEscape modal :header="t('forum.delete_category')" v-model:visible="deleteCategoryDialogVisible">
      <DeleteDialog
        :message="t('forum.confirm_delete_category')"
        :action="() => deleteForumCategory(forumCategory.id)"
        :successMessage="t('forum.category_deleted_success')"
        @deleted="onCategoryDeleted"
      />
    </Dialog>
    <Dialog closeOnEscape modal :header="t('forum.delete_subcategory')" v-model:visible="deleteSubCategoryDialogVisible">
      <DeleteDialog
        v-if="subCategoryToDelete"
        :message="t('forum.confirm_delete_subcategory')"
        :action="() => deleteForumSubCategory(subCategoryToDelete!)"
        :successMessage="t('forum.subcategory_deleted_success')"
        @deleted="onSubCategoryDeleted"
      />
    </Dialog>
    <Dialog closeOnEscape modal :header="t('forum.reorder_subcategories')" v-model:visible="reorderSubCategoriesDialogVisible">
      <ReorderForumCategoryOrSubCategoryDialog
        v-if="reorderSubCategoriesDialogVisible"
        mode="sub-categories"
        :category-id="forumCategory.id"
        :initial-items="forumCategory.sub_categories.map((s) => ({ id: s.id, name: s.name }))"
        @reordered="onSubCategoriesReordered"
      />
    </Dialog>
  </div>
</template>

<script setup lang="ts">
import TimeAgo from '@/components/TimeAgo.vue'
import { useI18n } from 'vue-i18n'
import DataTable from 'primevue/datatable'
import Column from 'primevue/column'
import { formatNumber } from '@/services/helpers'
import { RouterLink, useRouter } from 'vue-router'
import { deleteForumCategory, deleteForumSubCategory, type ForumCategoryHierarchy } from '@/services/api-schema'
import { useUserStore } from '@/stores/user'
import { Dialog } from 'primevue'
import { ref } from 'vue'
import DeleteDialog from '@/components/DeleteDialog.vue'
import ReorderForumCategoryOrSubCategoryDialog from './ReorderForumCategoryOrSubCategoryDialog.vue'

defineProps<{
  forumCategory: ForumCategoryHierarchy
}>()

const userStore = useUserStore()
const router = useRouter()
const { t } = useI18n()

const deleteCategoryDialogVisible = ref(false)
const deleteSubCategoryDialogVisible = ref(false)
const reorderSubCategoriesDialogVisible = ref(false)
const subCategoryToDelete = ref<number | null>(null)

const onCategoryDeleted = () => {
  deleteCategoryDialogVisible.value = false
  router.go(0)
}

const onSubCategoryDeleted = () => {
  deleteSubCategoryDialogVisible.value = false
  router.go(0)
}

const onSubCategoriesReordered = () => {
  reorderSubCategoriesDialogVisible.value = false
  router.go(0)
}
</script>

<style scoped>
.top {
  display: flex;
  justify-content: space-between;
  align-items: flex-end;
  margin-bottom: 5px;
  i {
    color: white;
    margin-left: 7px;
    cursor: pointer;
  }
}
</style>
