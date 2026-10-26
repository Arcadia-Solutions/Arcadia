<template>
  <Form
    ref="formRef"
    v-slot="$form"
    :initialValues="torrentRequestForm"
    :resolver
    @submit="onFormSubmit"
    validateOnSubmit
    :validateOnValueUpdate="false"
    validateOnBlur
  >
    <div id="create-torrent-request">
      <div>
        <div class="wrapper-center" style="color: red">{{ t('torrent_request.creation_explanation') }}</div>
        <div class="line">
          <div>
            <FloatLabel>
              <InputText v-model="torrentRequestForm.edition_name" size="small" name="edition_name" />
              <label for="edition_name">{{ t('torrent_request.edition_name') }}</label>
            </FloatLabel>
            <Message v-if="$form.edition_name?.invalid" severity="error" size="small" variant="simple">
              {{ $form.edition_name.error?.message }}
            </Message>
          </div>
          <div>
            <FloatLabel>
              <MultiSelect
                v-model="torrentRequestForm.source"
                inputId="source"
                :options="getSources(titleGroupStore.content_type)"
                class="select"
                size="small"
                display="chip"
                filter
                name="source"
              />
              <label for="source">{{ t('edition_group.source') }}</label>
            </FloatLabel>
            <Message v-if="$form.source?.invalid" severity="error" size="small" variant="simple">
              {{ $form.source.error?.message }}
            </Message>
          </div>
        </div>
        <div class="line">
          <div>
            <FloatLabel>
              <MultiSelect
                v-model="torrentRequestForm.container"
                inputId="container"
                :options="getSelectableContainers()"
                class="select"
                size="small"
                display="chip"
                filter
                name="container"
              />
              <label for="container">{{ t('torrent.container') }}</label>
            </FloatLabel>
            <Message v-if="$form.container?.invalid" severity="error" size="small" variant="simple">
              {{ $form.container.error?.message }}
            </Message>
          </div>
          <div>
            <FloatLabel v-if="isAttributeUsed('video_codec', titleGroupStore.content_type)">
              <MultiSelect
                v-model="torrentRequestForm.video_codec"
                inputId="video_codec"
                :options="getSelectableVideoCodecs()"
                class="select"
                size="small"
                display="chip"
                filter
                name="video_codec"
              />
              <label for="video_codec">{{ t('torrent.video_codec') }}</label>
            </FloatLabel>
            <Message v-if="$form.video_codec?.invalid" severity="error" size="small" variant="simple">
              {{ $form.video_codec.error?.message }}
            </Message>
          </div>
          <div>
            <FloatLabel v-if="isAttributeUsed('video_resolution', titleGroupStore.content_type)">
              <MultiSelect
                v-model="torrentRequestForm.video_resolution"
                inputId="video_resolution"
                :options="getSelectableVideoResolutions()"
                class="select"
                size="small"
                display="chip"
                filter
                name="video_resolution"
              />
              <label for="video_resolution">{{ t('torrent.video_resolution') }}</label>
            </FloatLabel>
            <Message v-if="$form.video_resolution?.invalid" severity="error" size="small" variant="simple">
              {{ $form.video_resolution.error?.message }}
            </Message>
          </div>
          <template v-if="torrentRequestForm.video_resolution.includes('Other')">
            <div>
              <FloatLabel>
                <InputNumber
                  v-model="torrentRequestForm.video_resolution_other_x"
                  inputId="video_resolution_other_x"
                  name="video_resolution_other_x"
                  size="small"
                  class="res-input"
                />
                <label for="video_resolution_other_x" class="res-label-text">Resolution X</label>
              </FloatLabel>
            </div>
            <div>
              <FloatLabel class="res-pick">
                <InputNumber
                  v-model="torrentRequestForm.video_resolution_other_y"
                  inputId="video_resolution_other_y"
                  name="video_resolution_other_y"
                  size="small"
                  class="res-input"
                />
                <label for="video_resolution_other_y" class="res-label-text">Resolution Y</label>
              </FloatLabel>
            </div>
          </template>
          <div>
            <FloatLabel v-if="isAttributeUsed('audio_codec', titleGroupStore.content_type) || editionGroupStore.additional_information.format === 'audiobook'">
              <MultiSelect
                v-model="torrentRequestForm.audio_codec"
                inputId="audio_codec"
                :options="getSelectableAudioCodecs()"
                class="select"
                size="small"
                display="chip"
                filter
                name="audio_codec"
              />
              <label for="audio_codec">{{ t('torrent.audio_codec') }}</label>
            </FloatLabel>
            <Message v-if="$form.audio_codec?.invalid" severity="error" size="small" variant="simple">
              {{ $form.audio_codec.error?.message }}
            </Message>
          </div>
          <div>
            <FloatLabel
              v-if="isAttributeUsed('audio_bitrate_sampling', titleGroupStore.content_type) || editionGroupStore.additional_information.format === 'audiobook'"
            >
              <MultiSelect
                v-model="torrentRequestForm.audio_bitrate_sampling"
                inputId="audio_bitrate_sampling"
                :options="getSelectableAudioBitrateSamplings()"
                class="select"
                size="small"
                display="chip"
                filter
                name="audio_bitrate_sampling"
              />
              <label for="audio_bitrate_sampling">{{ t('torrent.audio_bitrate_sampling') }}</label>
            </FloatLabel>
            <Message v-if="$form.audio_bitrate_sampling?.invalid" severity="error" size="small" variant="simple">
              {{ $form.audio_bitrate_sampling.error?.message }}
            </Message>
          </div>
          <div>
            <FloatLabel v-if="isAttributeUsed('audio_channels', titleGroupStore.content_type)">
              <MultiSelect
                v-model="torrentRequestForm.audio_channels"
                inputId="audio_channels"
                :options="getSelectableAudioChannels()"
                class="select"
                size="small"
                display="chip"
                filter
                name="audio_channels"
              />
              <label for="audio_channels">{{ t('torrent.audio_channels') }}</label>
            </FloatLabel>
            <Message v-if="$form.audio_channels?.invalid" severity="error" size="small" variant="simple">
              {{ $form.audio_channels.error?.message }}
            </Message>
          </div>
          <FloatLabel v-if="titleGroupStore.content_type !== 'music'">
            <MultiSelect
              v-model="torrentRequestForm.languages"
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
          <FloatLabel v-if="isAttributeUsed('subtitle_languages', titleGroupStore.content_type)">
            <MultiSelect
              v-model="torrentRequestForm.subtitle_languages"
              inputId="subtitle_languages"
              :options="getLanguages()"
              class="select"
              size="small"
              display="chip"
              filter
              name="subtitle_languages"
            />
            <label for="subtitle_languages">{{ t('torrent.subtitle_language', 2) }}</label>
          </FloatLabel>
          <Message v-if="$form.subtitle_languages?.invalid" severity="error" size="small" variant="simple">
            {{ $form.subtitle_languages.error?.message }}
          </Message>
        </div>
        <div class="line">
          <FloatLabel>
            <MultiSelect
              v-model="torrentRequestForm.features"
              display="chip"
              :options="getFeatures(titleGroupStore.content_type, '', editionGroupStore.source)"
              filter
              size="small"
              name="features"
            />
            <label for="features">{{ t('torrent.features') }}</label>
          </FloatLabel>
          <FloatLabel>
            <InputText v-model="torrentRequestForm.release_group" size="small" name="release_group" />
            <label for="release_group">{{ t('torrent.release_group') }}</label>
          </FloatLabel>
          <Message v-if="$form.release_group?.invalid" severity="error" size="small" variant="simple">
            {{ $form.release_group.error?.message }}
          </Message>
        </div>
        <div>
          <FloatLabel>
            <Textarea v-model="torrentRequestForm.description" name="description" class="textarea" autoResize rows="5" />
            <label for="description">{{ t('general.description') }}</label>
          </FloatLabel>
          <Message v-if="$form.description?.invalid" severity="error" size="small" variant="simple">
            {{ $form.description.error?.message }}
          </Message>
        </div>
        <div class="bounty-section">
          <span class="bold">{{ t('torrent_request.bounty') }}</span> <TorrentRequestVoteInputs ref="torrentRequestVoteInputsRef" />
        </div>
      </div>
      <div class="flex justify-content-center">
        <Button
          label="Create torrent request"
          type="submit"
          icon="pi pi-check"
          size="small"
          class="validate-button"
          :loading="creatingTorrentRequest"
          :disabled="validateButtonDisabled"
          v-tooltip.top="{ disabled: !validateButtonDisabled, value: t('torrent.complete_title_group_first') }"
        />
      </div>
    </div>
  </Form>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import FloatLabel from 'primevue/floatlabel'
import InputText from 'primevue/inputtext'
import Textarea from 'primevue/textarea'
import InputNumber from 'primevue/inputnumber'
import Button from 'primevue/button'
import MultiSelect from 'primevue/multiselect'
import Message from 'primevue/message'
import { type FormSubmitEvent } from '@primevue/forms'
import { Form } from '@primevue/forms'
import {
  getFeatures,
  getLanguages,
  getSources,
  getSelectableVideoCodecs,
  getSelectableVideoResolutions,
  getSelectableAudioCodecs,
  getSelectableAudioBitrateSamplings,
  getSelectableAudioChannels,
  getSelectableContainers,
  isAttributeUsed,
} from '@/services/helpers'
import { useEditionGroupStore } from '@/stores/editionGroup'
import { useTitleGroupStore } from '@/stores/titleGroup'
import { useI18n } from 'vue-i18n'
import { nextTick } from 'vue'
import type { VNodeRef } from 'vue'
import { pick } from 'lodash-es'
import { showToast } from '@/main'
import TorrentRequestVoteInputs from './TorrentRequestVoteInputs.vue'
import { useUserStore } from '@/stores/user'
import { createTorrentRequest, type TorrentRequest, type UserCreatedTorrentRequest } from '@/services/api-schema'

const formRef = ref<VNodeRef | null>(null)
const torrentRequestForm = ref<UserCreatedTorrentRequest>({
  title_group_id: 0,
  edition_name: '',
  release_group: '',
  description: '',
  languages: [],
  container: [],
  source: [],
  initial_vote: {
    torrent_request_id: 0,
    bounty_upload: 0,
    bounty_bonus_points: 0,
  },
  audio_codec: [],
  audio_channels: [],
  audio_bitrate_sampling: [],
  video_codec: [],
  features: [],
  subtitle_languages: [],
  video_resolution: [],
  video_resolution_other_x: null,
  video_resolution_other_y: null,
})

const creatingTorrentRequest = ref(false)
const titleGroupStore = ref(useTitleGroupStore())
const editionGroupStore = ref(useEditionGroupStore())

const { t } = useI18n()
const torrentRequestVoteInputsRef = ref<InstanceType<typeof TorrentRequestVoteInputs>>()
const userStore = useUserStore()

const props = defineProps<{
  initialTorrentRequest?: UserCreatedTorrentRequest
}>()
const emit = defineEmits<{
  done: [torrentRequest: TorrentRequest]
}>()

const resolver = (/*{ values }: FormResolverOptions*/) => {
  const errors: Partial<Record<keyof UserCreatedTorrentRequest, { message: string }[]>> = {}

  // if (values.container.length === 0) {
  //   errors.container = [{ message: t('error.select_container') }]
  // }
  // if (values.source.length === 0) {
  //   errors.source = [{ message: t('error.select_source') }]
  // }
  // if (values.languages.length === 0) {
  //   errors.languages = [{ message: t('error.select_at_least_x_language', [1]) }]
  // }
  // if (values.video_resolution.includes('Other')) {
  //   if (!values.video_resolution_other_x || isNaN(Number(values.video_resolution_other_x))) {
  //     errors.video_resolution_other_x = [{ message: 'Invalid resolution X' }]
  //   }
  //   if (!values.video_resolution_other_y || isNaN(Number(values.video_resolution_other_y))) {
  //     errors.video_resolution_other_y = [{ message: 'Invalid resolution Y' }]
  //   }
  // }

  return {
    errors,
  }
}

const onFormSubmit = ({ valid }: FormSubmitEvent) => {
  if (valid) {
    submitTorrentRequest()
  }
}

const submitTorrentRequest = () => {
  if (torrentRequestVoteInputsRef.value) {
    const newVote = torrentRequestVoteInputsRef.value.vote()
    if (!newVote) return
    torrentRequestForm.value.initial_vote.bounty_bonus_points = newVote.bounty_bonus_points
    torrentRequestForm.value.initial_vote.bounty_upload = newVote.bounty_upload
  } else {
    return
  }
  creatingTorrentRequest.value = true
  torrentRequestForm.value.title_group_id = titleGroupStore.value.id

  createTorrentRequest(torrentRequestForm.value)
    .then((data) => {
      showToast('', t('torrent_request.created_success'), 'success', 3000, true, 'tr')
      userStore.uploaded -= torrentRequestForm.value.initial_vote.bounty_upload
      userStore.bonus_points -= torrentRequestForm.value.initial_vote.bounty_bonus_points
      emit('done', data)
    })
    .finally(() => {
      creatingTorrentRequest.value = false
    })
}

const validateButtonDisabled = computed(() => {
  return titleGroupStore.value.id === 0 && !props.initialTorrentRequest
})

onMounted(async () => {
  if (props.initialTorrentRequest) {
    Object.assign(torrentRequestForm.value, pick(props.initialTorrentRequest, Object.keys(torrentRequestForm.value)))
    await nextTick()
    Object.keys(torrentRequestForm.value).forEach((key) => {
      try {
        formRef.value?.setFieldValue(key, torrentRequestForm.value[key as keyof typeof torrentRequestForm.value])
      } catch {
        // some fields fail because they are not in the primevueform, but they are in torrentRequestForm
      }
    })
  }
})
</script>

<style scoped>
.textarea {
  width: 100%;
  height: 10em;
}

.select {
  min-width: 200px;
}
.bounty-section {
  margin-top: 30px;
  display: flex;
  flex-direction: column;
  justify-content: center;
  align-items: center;
}
.res-label-text {
  font-size: small;
}
.res-input {
  width: 100px;
}
.validate-button {
  margin-top: 20px;
}
</style>
