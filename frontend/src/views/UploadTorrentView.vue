<template>
  <div>
    <div class="title">{{ t('torrent.upload_torrent') }}</div>
    <Message v-if="publicArcadiaSettings.uploads_disabled" severity="warn" :pt="{ content: { style: 'justify-content: center; text-align: center' } }">
      <BBCodeRenderer v-if="publicArcadiaSettings.uploads_disabled_message" :content="publicArcadiaSettings.uploads_disabled_message" />
      <template v-else>{{ t('torrent.uploads_disabled') }}</template>
    </Message>
    <template v-else>
      <ContentContainer v-if="uploadInfo?.upload_page_top_text" style="margin-bottom: 20px">
        <BBCodeRenderer :content="uploadInfo.upload_page_top_text" />
      </ContentContainer>
      <div class="announce-url">
        <div class="explanation">{{ t('torrent.announce_url') }}:</div>
        <div class="url" v-tooltip.top="t('general.click_to_copy')" @click="copyAnnounceUrl">{{ uploadInfo?.announce_url }}</div>
      </div>
      <Accordion :value="titleGroupAccordionValue" class="upload-step-accordion">
        <AccordionPanel value="0" :disabled="titleGroupDisabled">
          <AccordionHeader>
            <div style="display: flex">
              {{ t('title_group.title') }}
              <div v-if="titleGroupStore.id !== 0" style="display: flex">
                :<TitleGroupSlimHeader style="margin-left: 5px" :titleGroup="titleGroupStore" :affiliatedArtists="[]" />
              </div>
            </div>
          </AccordionHeader>
          <AccordionContent>
            <!-- <CreateOrSelectTitleGroup @done="titleGroupDone" /> -->
            <CreateOrSelectTitleGroup v-if="uploadInfo" :uploadInfo @editionGroupDataFound="editionGroupDataFound" @done="titleGroupDone" />
          </AccordionContent>
        </AccordionPanel>
      </Accordion>
      <Accordion
        :value="editionGroupAccordionValue"
        class="upload-step-accordion"
        v-tooltip.top="{
          value: t('edition_group.complete_title_group_first'),
          disabled: titleGroupDisabled,
        }"
      >
        <AccordionPanel value="0" :disabled="editionGroupDisabled || !titleGroupDisabled">
          <AccordionHeader>
            <span>
              {{ t('torrent.edition') }}<span v-if="editionGroup" class="bold">: {{ getEditionGroupSlug(editionGroup) }}</span>
            </span>
          </AccordionHeader>
          <AccordionContent>
            <CreateOrSelectEditionGroup ref="editionRef" @done="editionGroupDone" />
          </AccordionContent>
        </AccordionPanel>
      </Accordion>
      <Accordion
        :value="torrentAccordionValue"
        class="upload-step-accordion"
        v-tooltip.top="{
          value: t('torrent.complete_edition_group_first'),
          disabled: editionGroupDisabled,
        }"
      >
        <AccordionPanel value="0" :disabled="!editionGroupDisabled">
          <AccordionHeader>{{ t('torrent.torrent') }}</AccordionHeader>
          <AccordionContent>
            <CreateOrEditTorrent :uploadInfo="uploadInfo" @done="torrentDone" />
          </AccordionContent>
        </AccordionPanel>
      </Accordion>
    </template>
  </div>
</template>
<script setup lang="ts">
import { onUnmounted, ref } from 'vue'
import Accordion from 'primevue/accordion'
import Message from 'primevue/message'
import AccordionPanel from 'primevue/accordionpanel'
import AccordionHeader from 'primevue/accordionheader'
import AccordionContent from 'primevue/accordioncontent'
import CreateOrSelectEditionGroup from '@/components/edition_group/CreateOrSelectEditionGroup.vue'
import CreateOrEditTorrent from '@/components/torrent/CreateOrEditTorrent.vue'
import { useEditionGroupStore } from '@/stores/editionGroup'
import { useTitleGroupStore } from '@/stores/titleGroup'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import TitleGroupSlimHeader from '@/components/title_group/TitleGroupSlimHeader.vue'
import { onMounted } from 'vue'
import { getEditionGroupSlug, formatBp } from '@/services/helpers'
import CreateOrSelectTitleGroup from '@/components/title_group/CreateOrSelectTitleGroup.vue'
import BBCodeRenderer from '@/components/community/BBCodeRenderer.vue'
import ContentContainer from '@/components/ContentContainer.vue'
import {
  getUploadInformation,
  type EditionGroupInfoLite,
  type TitleGroup,
  type TitleGroupLite,
  type Torrent,
  type UploadInformation,
  type UserCreatedEditionGroup,
} from '@/services/api-schema'
import { useUserStore } from '@/stores/user'
import { showToast } from '@/main'
import { usePublicArcadiaSettingsStore } from '@/stores/publicArcadiaSettings'

const router = useRouter()
const { t } = useI18n()
const userStore = useUserStore()
const publicArcadiaSettings = usePublicArcadiaSettingsStore()

const uploadInfo = ref<UploadInformation>()
const editionGroupStore = ref(useEditionGroupStore())
const titleGroupStore = ref(useTitleGroupStore())

const titleGroupAccordionValue = ref('0')
const titleGroupDisabled = ref(false)
const editionGroupAccordionValue = ref('')
const editionGroupDisabled = ref(false)
const editionRef = ref<InstanceType<typeof CreateOrSelectEditionGroup>>()
const torrentAccordionValue = ref('')
const editionGroup = ref<EditionGroupInfoLite | null>(null)

const copyAnnounceUrl = () => {
  if (uploadInfo.value?.announce_url) {
    navigator.clipboard.writeText(uploadInfo.value.announce_url).then(() => {
      showToast('', t('torrent.announce_url_copied'), 'success', 2000)
    })
  }
}

const titleGroupDone = (titleGroup?: TitleGroup | TitleGroupLite) => {
  titleGroupAccordionValue.value = ''
  titleGroupDisabled.value = true
  editionGroupAccordionValue.value = '0'
  if (titleGroup) {
    titleGroupStore.value.id = titleGroup.id
    titleGroupStore.value.content_type = titleGroup.content_type
    titleGroupStore.value.name = titleGroup.name
    titleGroupStore.value.original_release_date = titleGroup.original_release_date
    titleGroupStore.value.original_release_date_only_year_known = titleGroup.original_release_date_only_year_known
    if ('edition_groups' in titleGroup) {
      titleGroupStore.value.edition_groups = titleGroup.edition_groups
    } else {
      if (editionRef.value) editionRef.value.action = 'create'
    }
  }
}

const editionGroupDataFound = (eg: UserCreatedEditionGroup) => {
  if (editionRef.value) editionRef.value.updateEditionGroupForm(eg)
}

const editionGroupDone = (eg: EditionGroupInfoLite) => {
  editionGroup.value = eg
  // editionGroupStore.value = { ...eg, ...editionGroupStore.value }
  editionGroupStore.value.id = eg.id
  editionGroupStore.value.source = eg.source
  editionGroupStore.value.additional_information = eg.additional_information
  editionGroupAccordionValue.value = ''
  editionGroupDisabled.value = true
  torrentAccordionValue.value = '0'
}

const torrentDone = (torrent: Torrent) => {
  if (uploadInfo.value && uploadInfo.value.bonus_points_given_on_upload > 0) {
    const bonusPoints = uploadInfo.value.bonus_points_given_on_upload
    showToast(
      '',
      t('torrent.upload_reward_awarded', [formatBp(bonusPoints, publicArcadiaSettings.bonus_points_decimal_places), publicArcadiaSettings.bonus_points_alias]),
      'success',
      5000,
    )
    userStore.bonus_points += bonusPoints
    userStore.torrents += 1
  }
  router.push('/title-group/' + titleGroupStore.value.id + '?torrentId=' + torrent.id)
}

// the store is used to autofill some steps if the user comes from an existing title group
onMounted(() => {
  if (titleGroupStore.value.id !== 0) {
    titleGroupDone()
  }
  getUploadInformation().then((data) => {
    uploadInfo.value = data
  })
})
onUnmounted(() => {
  titleGroupStore.value.$reset()
  editionGroupStore.value.$reset()
})
</script>

<style scoped>
.announce-url {
  display: flex;
  justify-content: center;
  margin-bottom: 15px;
  .explanation {
    margin-right: 10px;
    font-weight: bold;
  }
  .url {
    cursor: pointer;
    &:hover {
      color: var(--color-primary);
    }
  }
}
.form {
  padding-top: 5px;
}
.title {
  margin-bottom: 20px;
  color: var(--color-primary);
}
.upload-step-accordion {
  margin-bottom: 20px;
}
</style>
