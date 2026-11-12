<template>
  <div>
    <p>{{ t('user_settings.paranoia_explanation') }}</p>
    <DataTable :value="rows" size="small">
      <Column :header="t('user_settings.paranoia_information')">
        <template #body="{ data }">
          <label :for="`paranoia_count_${data.stat}`">{{ userStatLabel(data.stat) }}</label>
        </template>
      </Column>
      <Column :header="t('user_settings.paranoia_show_count')">
        <template #body="{ data }">
          <Checkbox
            binary
            size="small"
            :inputId="`paranoia_count_${data.stat}`"
            :modelValue="!settings.paranoia_hidden_stats.includes(data.stat)"
            @update:modelValue="setStatShown(data.stat, $event)"
          />
        </template>
      </Column>
      <Column :header="t('user_settings.paranoia_show_list')">
        <template #body="{ data }">
          <Checkbox
            v-if="data.list"
            binary
            size="small"
            :inputId="`paranoia_list_${data.stat}`"
            :disabled="settings.paranoia_hidden_stats.includes(data.stat)"
            :modelValue="!settings.paranoia_hidden_lists.includes(data.list)"
            @update:modelValue="setListShown(data.list, $event)"
          />
        </template>
      </Column>
    </DataTable>
  </div>
</template>
<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { Checkbox, Column, DataTable } from 'primevue'
import { DisplayableUserStats, HideableUserList, type UserSettings } from '@/services/api-schema'
import { useUserStatLabel } from '@/composables/useUserStatLabel'
import { usePublicArcadiaSettingsStore } from '@/stores/publicArcadiaSettings'

const { t } = useI18n()
const publicArcadiaSettings = usePublicArcadiaSettingsStore()
const userStatLabel = useUserStatLabel()

const settings = defineModel<UserSettings>({ required: true })

// statistics that are also displayed as a list, which can be hidden separately
const listsByStat: Partial<Record<DisplayableUserStats, HideableUserList>> = {
  [DisplayableUserStats.Torrents]: HideableUserList.Torrents,
  [DisplayableUserStats.Snatched]: HideableUserList.Snatched,
  [DisplayableUserStats.ForumPosts]: HideableUserList.ForumPosts,
  [DisplayableUserStats.TitleGroupComments]: HideableUserList.TitleGroupComments,
  [DisplayableUserStats.TitleGroupRatings]: HideableUserList.TitleGroupRatings,
  [DisplayableUserStats.RequestComments]: HideableUserList.RequestComments,
}

const rows = computed(() => publicArcadiaSettings.displayable_user_stats.map((stat) => ({ stat, list: listsByStat[stat] })))

const setStatShown = (stat: DisplayableUserStats, shown: boolean) => {
  if (shown) {
    settings.value.paranoia_hidden_stats = settings.value.paranoia_hidden_stats.filter((hiddenStat) => hiddenStat !== stat)
  } else if (!settings.value.paranoia_hidden_stats.includes(stat)) {
    settings.value.paranoia_hidden_stats.push(stat)
    // a list cannot be shown when the count it belongs to is hidden
    const list = listsByStat[stat]
    if (list) {
      setListShown(list, false)
    }
    // the ratio is computed from the uploaded and downloaded amounts, they are hidden with it
    if (stat === DisplayableUserStats.Ratio) {
      setStatShown(DisplayableUserStats.Uploaded, false)
      setStatShown(DisplayableUserStats.Downloaded, false)
    }
  }
}

const setListShown = (list: HideableUserList, shown: boolean) => {
  if (shown) {
    settings.value.paranoia_hidden_lists = settings.value.paranoia_hidden_lists.filter((hiddenList) => hiddenList !== list)
  } else if (!settings.value.paranoia_hidden_lists.includes(list)) {
    settings.value.paranoia_hidden_lists.push(list)
  }
}
</script>
