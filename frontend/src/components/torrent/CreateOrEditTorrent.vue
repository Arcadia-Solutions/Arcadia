<template>
  <Form
    ref="formRef"
    v-slot="$form"
    :initialValues="torrentForm"
    :resolver
    @submit="onFormSubmit"
    @keydown.enter="onFormKeydown"
    validateOnSubmit
    validateOnValueUpdate
    validateOnBlur
    validateOnMount
  >
    <div id="create-torrent">
      <div class="mediainfo">
        <FloatLabel>
          <Textarea v-model="torrentForm.mediainfo" name="mediainfo" class="textarea pre-style" rows="5" @update:model-value="mediainfoUpdated" />
          <label for="mediainfo">{{ t('torrent.mediainfo_autofill') }}</label>
        </FloatLabel>
        <Message v-if="$form.mediainfo?.invalid" severity="error" size="small" variant="simple">
          {{ $form.mediainfo.error?.message }}
        </Message>
      </div>
      <div>
        <div class="line togglable-input">
          <div class="checkbox" v-tooltip.top="t('torrent.extras_hint')">
            <Checkbox v-model="isExtras" binary inputId="is_extras" name="is_extras" />
            <label for="is_extras"> {{ t('torrent.extras.extras') }} </label>
          </div>
          <div v-if="isExtras">
            <FloatLabel>
              <MultiSelect
                v-model="torrentForm.extras"
                inputId="extras"
                :options="getSelectableExtras(titleGroupStore.content_type)"
                class="select"
                size="small"
                display="chip"
                filter
                name="extras"
              >
                <template #option="slotProps">
                  {{ t(`torrent.extras.${slotProps.option}`) }}
                </template>
                <template #chip="{ value }">
                  <Chip removable @remove="removeExtrasValue(value)">
                    {{ t(`torrent.extras.${value}`) }}
                  </Chip>
                </template>
                <label for="extras">{{ t('torrent.extras.extras') }}</label>
              </MultiSelect>
            </FloatLabel>
            <Message v-if="$form.extras?.invalid" severity="error" size="small" variant="simple">
              {{ $form.extras.error?.message }}
            </Message>
          </div>
        </div>
        <div class="line">
          <div class="release-name">
            <FloatLabel>
              <InputText v-model="torrentForm.release_name" size="small" name="release_name" />
              <label for="release_name">{{ t('torrent.release_name') }}</label>
            </FloatLabel>
            <Message v-if="$form.release_name?.invalid" severity="error" size="small" variant="simple">
              {{ $form.release_name.error?.message }}
            </Message>
          </div>
          <div>
            <FloatLabel>
              <InputText v-model="torrentForm.release_group" size="small" name="release_group" />
              <label for="release_group">{{ t('torrent.release_group') }}</label>
            </FloatLabel>
            <Message v-if="$form.release_group?.invalid" severity="error" size="small" variant="simple">
              {{ $form.release_group.error?.message }}
            </Message>
          </div>
        </div>
        <div>
          <FloatLabel v-tooltip.top="t('torrent.description_hint')">
            <Textarea v-model="torrentForm.description" name="description" class="textarea" autoResize rows="5" />
            <label for="description">{{ t('general.description') }}</label>
          </FloatLabel>
          <Message v-if="$form.description?.invalid" severity="error" size="small" variant="simple">
            {{ $form.release_name.error?.message }}
          </Message>
        </div>
        <div class="line">
          <div>
            <FloatLabel v-tooltip.top="t('torrent.container_hint')">
              <InputText v-model="torrentForm.container" size="small" name="container" />
              <label for="container">{{ t('torrent.container') }}</label>
            </FloatLabel>
            <Message v-if="$form.container?.invalid" severity="error" size="small" variant="simple">
              {{ $form.container.error?.message }}
            </Message>
          </div>
          <div v-if="isAttributeUsed('video_codec', titleGroupStore.content_type)">
            <FloatLabel>
              <Select
                v-model="torrentForm.video_codec"
                inputId="video_codec"
                :options="getSelectableVideoCodecs()"
                class="select"
                size="small"
                name="video_codec"
              />
              <label for="video_codec">{{ t('torrent.video_codec') }}</label>
            </FloatLabel>
            <Message v-if="$form.video_codec?.invalid" severity="error" size="small" variant="simple">
              {{ $form.video_codec.error?.message }}
            </Message>
          </div>
          <div v-if="isAttributeUsed('video_resolution', titleGroupStore.content_type)">
            <FloatLabel>
              <Select
                v-model="torrentForm.video_resolution"
                inputId="video_resolution"
                :options="getSelectableVideoResolutions()"
                class="select"
                size="small"
                name="video_resolution"
              />
              <label for="video_resolution">{{ t('torrent.video_resolution') }}</label>
            </FloatLabel>
            <Message v-if="$form.video_resolution?.invalid" severity="error" size="small" variant="simple">
              {{ $form.video_resolution.error?.message }}
            </Message>
          </div>
          <template v-if="torrentForm.video_resolution == 'Other'">
            <div>
              <FloatLabel>
                <InputNumber
                  v-model="torrentForm.video_resolution_other_x"
                  inputId="video_resolution_other_x"
                  name="video_resolution_other_x"
                  type="number"
                  size="small"
                  class="res-input"
                />
                <label for="video_resolution_other_x" class="res-label-text">Resolution X</label>
              </FloatLabel>
            </div>
            <div>
              <FloatLabel class="res-pick">
                <InputNumber
                  v-model="torrentForm.video_resolution_other_y"
                  inputId="video_resolution_other_y"
                  name="video_resolution_other_y"
                  size="small"
                  type="number"
                  class="res-input"
                />
                <label for="video_resolution_other_y" class="res-label-text">Resolution Y</label>
              </FloatLabel>
            </div>
          </template>
          <div v-if="isAttributeUsed('audio_codec', titleGroupStore.content_type) || editionGroupStore.additional_information.format === 'audiobook'">
            <FloatLabel>
              <Select
                v-model="torrentForm.audio_codec"
                inputId="audio_codec"
                :options="getSelectableAudioCodecs()"
                class="select"
                size="small"
                name="audio_codec"
              />
              <label for="audio_codec">{{ t('torrent.audio_codec') }}</label>
            </FloatLabel>
            <Message v-if="$form.audio_codec?.invalid" severity="error" size="small" variant="simple">
              {{ $form.audio_codec.error?.message }}
            </Message>
          </div>
          <div
            v-if="isAttributeUsed('audio_bitrate_sampling', titleGroupStore.content_type) || editionGroupStore.additional_information.format === 'audiobook'"
          >
            <FloatLabel>
              <Select
                v-model="torrentForm.audio_bitrate_sampling"
                inputId="audio_coded"
                :options="getSelectableAudioBitrateSamplings()"
                class="select"
                size="small"
                name="audio_bitrate_sampling"
              />
              <label for="audio_bitrate_sampling">{{ t('torrent.audio_bitrate_sampling') }}</label>
            </FloatLabel>
            <Message v-if="$form.audio_bitrate_sampling?.invalid" severity="error" size="small" variant="simple">
              {{ $form.audio_bitrate_sampling.error?.message }}
            </Message>
          </div>
          <div v-if="isAttributeUsed('audio_channels', titleGroupStore.content_type)">
            <FloatLabel>
              <Select
                v-model="torrentForm.audio_channels"
                inputId="audio_channels"
                :options="getSelectableAudioChannels()"
                class="select"
                size="small"
                name="audio_channels"
              />
              <label for="audio_channels">{{ t('torrent.audio_channels') }}</label>
            </FloatLabel>
            <Message v-if="$form.audio_channels?.invalid" severity="error" size="small" variant="simple">
              {{ $form.audio_channels.error?.message }}
            </Message>
          </div>
        </div>
        <div>
          <FloatLabel v-if="titleGroupStore.content_type !== 'music'">
            <MultiSelect
              v-model="torrentForm.languages"
              inputId="languages"
              :options="getLanguages()"
              class="select"
              size="small"
              display="chip"
              filter
              name="languages"
            />
            <label for="languages">{{ t('general.language', 2) }}</label>
          </FloatLabel>
          <Message v-if="$form.languages?.invalid" severity="error" size="small" variant="simple">
            {{ $form.languages.error?.message }}
          </Message>
        </div>
        <FloatLabel>
          <MultiSelect
            v-model="torrentForm.features"
            display="chip"
            :options="getFeatures(titleGroupStore.content_type, editionGroupStore.additional_information.format, editionGroupStore.source)"
            filter
            size="small"
            name="features"
          />
          <label for="features">{{ t('torrent.features') }}</label>
        </FloatLabel>
        <FloatLabel>
          <InputText v-tooltip.top="t('torrent.extra_text_hint')" v-model="torrentForm.extra_text" size="small" name="extra_text" maxlength="50" />
          <label for="extra_text">{{ t('torrent.extra_text') }}</label>
        </FloatLabel>
        <!-- <FloatLabel >
          <InputText v-model="torrentForm.duration" size="small" name="duration" />
          <label for="duration">Duration (total, in seconds)</label>
        </FloatLabel> -->
        <!-- <FloatLabel>
        <InputText v-model="torrentForm.audio_bitrate" size="small" name="audio_bitrate" />
        <label for="audio_codec">Audio bitrate (in kb/s)</label>
      </FloatLabel> -->
        <FormField v-if="!initialTorrent" v-slot="$field" name="torrent_file" :initialValue="torrentForm.torrent_file">
          <FileUpload
            ref="torrentFile"
            accept=".torrent"
            :chooseLabel="t('torrent.torrent_file')"
            :cancel-label="t('general.remove')"
            :showUploadButton="false"
            :fileLimit="1"
            @select="onFileSelect"
            v-bind="$field.props"
          >
            <template #content="{ files }">
              {{ files.length != 0 ? files[0].name : 'Select a file' }}
            </template>
          </FileUpload>
          <Message v-if="$form.torrent_file?.invalid" severity="error" size="small" variant="simple">
            {{ $form.torrent_file.error?.message }}
          </Message>
        </FormField>
        <FloatLabel v-if="effectiveUploadInfo?.allow_uploader_set_torrent_bonus_points_cost" style="margin-top: 30px">
          <InputNumber v-model="displayBonusPointsSnatchCost" name="bonus_points_snatch_cost" :min="0" :step="1" size="small" />
          <label for="bonus_points_snatch_cost">{{ publicArcadiaSettings.bonus_points_alias }} {{ t('torrent.snatch_cost') }}</label>
        </FloatLabel>
        <div v-if="!initialTorrent || userStore.permissions.includes('edit_torrent_trumpable')" class="line togglable-input" style="margin-top: 20px">
          <div class="checkbox" v-tooltip.top="t('torrent.trumpable_hint')">
            <Checkbox v-model="isTrumpable" binary inputId="is_trumpable" name="is_trumpable" />
            <label for="is_trumpable"> {{ t('torrent.trumpable') }} </label>
          </div>
          <div v-if="isTrumpable">
            <FloatLabel>
              <InputText v-model="torrentForm.trumpable" :placeholder="t('torrent.trump_reason')" size="small" style="width: 30em" />
            </FloatLabel>
            <Message v-if="$form.trumpable?.invalid" severity="error" size="small" variant="simple">
              {{ $form.trumpable.error?.message }}
            </Message>
          </div>
        </div>
        <div class="flex align-items-center upload-as-anonymous">
          <Checkbox v-model="torrentForm.uploaded_as_anonymous" inputId="anonymous" name="anonymous" binary />
          <label for="anonymous" style="margin-left: 5px"> {{ t('torrent.upload_as_anonymous') }}</label>
        </div>
      </div>
      <div class="flex justify-content-center">
        <Button
          :label="initialTorrent ? t('general.confirm') : t('torrent.upload_torrent')"
          type="submit"
          icon="pi pi-check"
          size="small"
          class="validate-button"
          :loading="uploadingTorrent"
        />
      </div>
    </div>
  </Form>
</template>

<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import FloatLabel from 'primevue/floatlabel'
import InputText from 'primevue/inputtext'
import Textarea from 'primevue/textarea'
import Select from 'primevue/select'
import Button from 'primevue/button'
import InputNumber from 'primevue/inputnumber'
import Checkbox from 'primevue/checkbox'
import FileUpload, { type FileUploadSelectEvent } from 'primevue/fileupload'
import MultiSelect from 'primevue/multiselect'
import Message from 'primevue/message'
import { FormField, type FormResolverOptions, type FormSubmitEvent } from '@primevue/forms'
import { Form } from '@primevue/forms'
import { getFileInfo } from '@/services/fileinfo/fileinfo.js'
import {
  getSelectableExtras,
  getFeatures,
  getLanguages,
  getSelectableVideoCodecs,
  getSelectableVideoResolutions,
  getSelectableAudioCodecs,
  getSelectableAudioBitrateSamplings,
  getSelectableAudioChannels,
  isAttributeUsed,
  rawToDisplayBp,
  displayToRawBp,
  onFormKeydown,
} from '@/services/helpers'
import { useEditionGroupStore } from '@/stores/editionGroup'
import { uploadTorrent } from '@/services/api/torrentService'
import { useTitleGroupStore } from '@/stores/titleGroup'
import { useI18n } from 'vue-i18n'
import { nextTick } from 'vue'
import type { VNodeRef } from 'vue'
import { pick } from 'lodash-es'
import { showToast } from '@/main'
import { Chip } from 'primevue'
import {
  ContentType,
  editTorrent,
  getUploadInformation,
  type EditedTorrent,
  type Extras,
  type Torrent,
  type TorrentHierarchyLite,
  type UploadedTorrent,
  type UploadInformation,
} from '@/services/api-schema'
import { usePublicArcadiaSettingsStore } from '@/stores/publicArcadiaSettings'
import { useUserStore } from '@/stores/user'

const userStore = useUserStore()

const formRef = ref<VNodeRef | null>(null)
const torrentFile = ref({ files: [] as unknown[] })
const torrentForm = ref({
  id: 0,
  extras: [] as Extras[],
  edition_group_id: 0,
  release_name: '',
  release_group: '',
  extra_text: null as string | null,
  mediainfo: '',
  description: '',
  languages: ['English'] as string[],
  container: '',
  video_codec: null as string | null,
  video_resolution: null as string | null,
  video_resolution_other_x: null as number | null,
  video_resolution_other_y: null as number | null,
  duration: null,
  audio_codec: null as string | null,
  audio_bitrate: null as string | null,
  subtitle_languages: [] as string[],
  features: [] as string[],
  audio_channels: null as string | null,
  audio_bitrate_sampling: null as string | null,
  torrent_file: '',
  uploaded_as_anonymous: false,
  trumpable: '' as string | null,
  bonus_points_snatch_cost: 0,
})
const isExtras = ref(false)
const isTrumpable = ref(false)
const uploadingTorrent = ref(false)
const titleGroupStore = ref(useTitleGroupStore())
const editionGroupStore = ref(useEditionGroupStore())
const fetchedUploadInfo = ref<UploadInformation | null>(null)

const { t } = useI18n()
const publicArcadiaSettings = usePublicArcadiaSettingsStore()

const displayBonusPointsSnatchCost = computed({
  get: () => rawToDisplayBp(torrentForm.value.bonus_points_snatch_cost, publicArcadiaSettings.bonus_points_decimal_places),
  set: (value: number) => {
    torrentForm.value.bonus_points_snatch_cost = displayToRawBp(value, publicArcadiaSettings.bonus_points_decimal_places)
  },
})

const props = defineProps<{
  initialTorrent?: EditedTorrent | TorrentHierarchyLite
  uploadInfo?: UploadInformation
}>()
const emit = defineEmits<{
  done: [torrent: Torrent]
}>()

const effectiveUploadInfo = computed(() => props.uploadInfo ?? fetchedUploadInfo.value)

watch(
  () => props.uploadInfo,
  (uploadInfo) => {
    if (uploadInfo && !props.initialTorrent) {
      torrentForm.value.bonus_points_snatch_cost = uploadInfo.default_torrent_bonus_points_cost
    }
  },
  { immediate: true },
)

watch(isExtras, () => {
  nextTick().then(() => formRef.value?.validate())
})

watch(
  () => titleGroupStore.value.content_type,
  (contentType) => {
    if (contentType === ContentType.Book && !torrentForm.value.container) {
      // set this to the most common container for books
      torrentForm.value.container = 'pdf'
      formRef.value?.setFieldValue('container', 'pdf')
    }
  },
  { immediate: true },
)

const resolver = ({ values }: FormResolverOptions) => {
  const errors: Partial<Record<keyof UploadedTorrent, { message: string }[]>> = {}
  if (!isExtras.value && isAttributeUsed('mediainfo', titleGroupStore.value.content_type) && !values.mediainfo?.trim()) {
    errors.mediainfo = [{ message: t('error.enter_mediainfo') }]
  }
  if (isExtras.value && values.extras.length === 0) {
    errors.extras = [{ message: t('error.select_extras') }]
  }
  if (isTrumpable.value && values.trumpable === '') {
    errors.trumpable = [{ message: t('error.enter_trumpable_information') }]
  }
  // if (values.release_name.length < 5) {
  //   errors.release_name = [{ message: t('error.write_more_than_x_chars', [5]) }]
  // }
  // if (values.release_group.length < 2) {
  //   errors.release_group = [{ message: 'Write more than 2 characters' }]
  // }
  // if (values.description == '') {
  //   errors.description = [{ message: 'Write a description' }]
  // }
  if (values.container == '') {
    errors.container = [{ message: t('error.select_container') }]
  }
  if (!isExtras.value && isAttributeUsed('video_codec', titleGroupStore.value.content_type) && !values.video_codec) {
    errors.video_codec = [{ message: t('error.select_codec') }]
  }
  if (!isExtras.value && isAttributeUsed('video_resolution', titleGroupStore.value.content_type) && !values.video_resolution) {
    errors.video_resolution = [{ message: t('error.select_resolution') }]
  }
  if (!isExtras.value && isAttributeUsed('audio_codec', titleGroupStore.value.content_type) && !values.audio_codec) {
    errors.audio_codec = [{ message: t('error.select_codec') }]
  }
  // if (!isExtras.value && isAttributeUsed('audio_bitrate_sampling', titleGroupStore.value.content_type) && !values.audio_bitrate_sampling) {
  //   errors.audio_bitrate_sampling = [{ message: t('error.select_bitrate') }]
  // }
  if (titleGroupStore.value.content_type !== 'music' && values.languages && values.languages.length === 0) {
    errors.languages = [{ message: t('error.select_at_least_x_language', [1]) }]
  }
  if (!torrentForm.value.torrent_file) {
    errors.torrent_file = [{ message: t('error.select_torrent_file') }]
  }
  if (!isExtras.value && values.video_resolution === 'Other') {
    if (!values.video_resolution_other_x || isNaN(Number(values.video_resolution_other_x))) {
      errors.video_resolution_other_x = [{ message: 'Invalid resolution X' }]
    }
    if (!values.video_resolution_other_y || isNaN(Number(values.video_resolution_other_y))) {
      errors.video_resolution_other_y = [{ message: 'Invalid resolution Y' }]
    }
  }

  return {
    errors,
  }
}
const onFormSubmit = ({ valid }: FormSubmitEvent) => {
  if (valid) {
    sendTorrent()
  }
}
const onFileSelect = (event: FileUploadSelectEvent) => {
  if (event.files && event.files.length > 0) {
    // keep a single file
    const file = event.files[event.files.length - 1] as string
    // torrentFile.value.files.clear()
    torrentFile.value.files = [file]
    torrentForm.value.torrent_file = file
  }
}
const mediainfoUpdated = async () => {
  if (!formRef.value) return
  // only fill release name/group for movies, tv-shows and music
  const mediainfoExtractedInfo = getFileInfo(
    torrentForm.value.mediainfo,
    ([ContentType.Movie, ContentType.TvShow, ContentType.Music, ContentType.LivePerformance] as ContentType[]).includes(titleGroupStore.value.content_type),
  )
  if (mediainfoExtractedInfo) {
    torrentForm.value.mediainfo = mediainfoExtractedInfo.sanitizedMediainfo
    torrentForm.value.release_name = mediainfoExtractedInfo.release_name
    torrentForm.value.release_group = mediainfoExtractedInfo.release_group
    if (mediainfoExtractedInfo.audio_languages.length > 0) {
      torrentForm.value.languages = mediainfoExtractedInfo.audio_languages
    }
    torrentForm.value.container = mediainfoExtractedInfo.container
    torrentForm.value.video_codec = mediainfoExtractedInfo.video_codec
    if (typeof mediainfoExtractedInfo.video_resolution === 'string') {
      torrentForm.value.video_resolution = mediainfoExtractedInfo.video_resolution
    } else if (mediainfoExtractedInfo.video_resolution !== null) {
      torrentForm.value.video_resolution = 'Other'
      torrentForm.value.video_resolution_other_x = Number(mediainfoExtractedInfo.video_resolution![0].replace(/[^\d]/g, ''))
      torrentForm.value.video_resolution_other_y = Number(mediainfoExtractedInfo.video_resolution![1].replace(/[^\d]/g, ''))
    }
    torrentForm.value.audio_codec = mediainfoExtractedInfo.audio_codec
    torrentForm.value.subtitle_languages = mediainfoExtractedInfo.subtitle_languages
    torrentForm.value.features = mediainfoExtractedInfo.features
    torrentForm.value.audio_channels = mediainfoExtractedInfo.audio_channels
    await nextTick()
    Object.keys(torrentForm.value).forEach((key) => {
      try {
        formRef.value?.setFieldValue(key, torrentForm.value[key as keyof typeof torrentForm.value])
      } catch {
        // some fields fail because they are not in the primevueform, but they are in torrentForm
      }
    })
    showToast('', t('torrent.autofilled_mediainfo'), 'success', 4000, true, 'tr')
  }
}
const sendTorrent = () => {
  uploadingTorrent.value = true
  if (!isExtras.value) {
    torrentForm.value.extras = []
  }
  if (!isTrumpable.value) {
    torrentForm.value.trumpable = null
  }
  if (!torrentForm.value.extra_text?.trim()) {
    torrentForm.value.extra_text = null
  }
  if (props.initialTorrent) {
    torrentForm.value.id = props.initialTorrent.id
    editTorrent(torrentForm.value as EditedTorrent)
      .then((data) => {
        showToast('', t('torrent.torrent_edited_success'), 'success', 3000, true, 'tr')
        emit('done', data)
      })
      .finally(() => {
        uploadingTorrent.value = false
      })
  } else {
    torrentForm.value.edition_group_id = editionGroupStore.value.id
    uploadTorrent(torrentForm.value)
      .then((data) => {
        emit('done', data)
      })
      .finally(() => {
        uploadingTorrent.value = false
      })
  }
}
const removeExtrasValue = (item: Extras) => {
  const index = torrentForm.value.extras.indexOf(item)

  if (index !== -1) {
    torrentForm.value.extras.splice(index, 1)
  }
}
onMounted(async () => {
  if (props.initialTorrent) {
    Object.assign(torrentForm.value, pick(props.initialTorrent, Object.keys(torrentForm.value)))
    if (props.initialTorrent.extras.length > 0) {
      isExtras.value = true
    }
    if (props.initialTorrent.trumpable) {
      isTrumpable.value = true
    }
    if (!props.uploadInfo) {
      getUploadInformation().then((data) => {
        fetchedUploadInfo.value = data
      })
    }
    await nextTick()
    // some field is apparently undefined, the whole form seems to still get populated though
    // formRef.value?.setValues(torrentForm.value)
    Object.keys(torrentForm.value).forEach((key) => {
      try {
        formRef.value?.setFieldValue(key, torrentForm.value[key as keyof typeof torrentForm.value])
      } catch {
        // some fields fail because they are not in the primevueform, but they are in torrentForm
      }
    })
  }
})
</script>
<style scoped>
#create-torrent {
  padding-top: 20px;
}
.textarea {
  width: 100%;
  height: 10em;
}
.mediainfo .p-floatlabel {
  margin-top: 0;
}
.release-name {
  width: 100%;
  input {
    width: 100%;
  }
}
.line {
  .checkbox {
    display: flex;
    align-items: center;
    .p-checkbox {
      margin-right: 4px;
    }
  }
}
.select {
  min-width: 200px;
}
.togglable-input {
  margin-top: 30px;
  display: flex;
  align-items: center;
  .p-floatlabel {
    margin-top: 0;
    margin-left: 10px;
  }
}

.res-label-text {
  font-size: small;
}

.res-input {
  width: 100px;
}

.upload-as-anonymous {
  margin-top: 20px;
}
.validate-button {
  margin-top: 20px;
}
</style>
<style>
#create-torrent {
  .p-fileupload {
    max-width: 300px;
    margin-top: 15px;
  }
}
</style>
