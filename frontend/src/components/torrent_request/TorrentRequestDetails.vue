<template>
  <ContentContainer id="torrent-request-details">
    <table>
      <tbody>
        <tr v-for="requirement in filteredRequirements" :key="requirement">
          <td class="name">{{ t(`torrent_request.acceptable_${requirement}`) }}</td>
          <td class="value">
            {{
              torrentRequest[requirement] && typeof torrentRequest[requirement] === 'object' && (torrentRequest[requirement] as string[] | string).length > 0
                ? (torrentRequest[requirement] as string[]).join(', ')
                : t('general.any')
            }}
          </td>
        </tr>
      </tbody>
      <tbody class="extra-info">
        <tr>
          <td class="name">{{ t('torrent_request.bounty') }}</td>
          <td class="value">
            <template v-if="publicArcadiaSettings.torrent_request_vote_currencies.includes(TorrentRequestVoteCurrency.Upload)">
              {{ bytesToReadable(votes.reduce((sum, vote) => sum + vote.bounty_upload, 0)) }}
            </template>
            <template v-if="publicArcadiaSettings.torrent_request_vote_currencies.length === 2"> + </template>
            <template v-if="publicArcadiaSettings.torrent_request_vote_currencies.includes(TorrentRequestVoteCurrency.BonusPoints)">
              {{
                formatBp(
                  votes.reduce((sum, vote) => sum + vote.bounty_bonus_points, 0),
                  publicArcadiaSettings.bonus_points_decimal_places,
                )
              }}
              {{ publicArcadiaSettings.bonus_points_alias }}
            </template>
            <span class="votes-amount">({{ votes.length }} {{ t('torrent_request.vote', votes.length) }})</span>
          </td>
        </tr>
      </tbody>
    </table>
    <div v-if="torrentRequest.filled_by_torrent_id" class="filled-info">
      <RouterLink :to="`/torrent/${torrentRequest.filled_by_torrent_id}`">{{ t('torrent_request.filled') }}</RouterLink>
      <span><TimeAgo :date="torrentRequest.filled_at" /> {{ t('general.by') }} <UsernameEnriched :user="filledByUser" /></span>
    </div>
    <template v-else>
      <div class="new-vote">
        <span class="bold">{{ t('torrent_request.new_vote') }}</span>
        <TorrentRequestVoteInputs showVoteBtn @vote="vote" :loading="newVoteLoading" />
      </div>
      <div class="fill-request">
        <span class="bold">{{ t('torrent_request.fill_request') }}</span>
        <InputText v-model="fillLink" :placeholder="t('torrent.permalink')" size="small" style="width: 30em" />
        <Button :label="t('torrent_request.fill')" :loading="fillLoading" @click="fill" size="small" />
      </div>
    </template>
  </ContentContainer>
</template>

<script lang="ts" setup>
import ContentContainer from '../ContentContainer.vue'
import { useI18n } from 'vue-i18n'
import { ref, computed } from 'vue'
import { bytesToReadable, isAttributeUsed, formatBp } from '@/services/helpers'
import { useUserStore } from '@/stores/user'
import { usePublicArcadiaSettingsStore } from '@/stores/publicArcadiaSettings'
import { showToast } from '@/main'
import TorrentRequestVoteInputs from './TorrentRequestVoteInputs.vue'
import InputText from 'primevue/inputtext'
import Button from 'primevue/button'
import {
  createTorrentRequestVote,
  fillTorrentRequest,
  TorrentRequestVoteCurrency,
  type ContentType,
  type TorrentRequest,
  type TorrentRequestVote,
  type TorrentRequestVoteHierarchy,
  type UserCreatedTorrentRequestVote,
  type UserLite,
} from '@/services/api-schema'
import UsernameEnriched from '../user/UsernameEnriched.vue'

const { t } = useI18n()

const requirements = ref<(keyof TorrentRequest)[]>([
  'audio_bitrate_sampling',
  'audio_channels',
  'audio_codec',
  'container',
  'edition_name',
  'features',
  'source',
  'video_codec',
  'video_resolution',
])
const filteredRequirements = computed(() => {
  return requirements.value.filter((requirement) => isAttributeUsed(requirement, props.contentType))
})

const props = defineProps<{
  torrentRequest: TorrentRequest
  filledByUser?: UserLite | null
  votes: TorrentRequestVote[]
  contentType: ContentType
}>()

const emit = defineEmits<{
  voted: [TorrentRequestVoteHierarchy]
  filled: [number, boolean]
}>()

const userStore = useUserStore()
const publicArcadiaSettings = usePublicArcadiaSettingsStore()
const newVoteLoading = ref(false)
const fillLink = ref('')
const fillLoading = ref(false)

const vote = async (newVote: UserCreatedTorrentRequestVote) => {
  newVoteLoading.value = true

  createTorrentRequestVote({ ...newVote, torrent_request_id: props.torrentRequest.id })
    .then((castedVote) => {
      emit('voted', { ...castedVote, created_by: userStore })
      userStore.uploaded -= castedVote.bounty_upload
      userStore.bonus_points -= castedVote.bounty_bonus_points
      showToast('', t('torrent_request.vote_successful'), 'success', 3000, true, 'tr')
    })
    .finally(() => (newVoteLoading.value = false))
}

const fill = () => {
  const url = new URL(fillLink.value, window.location.origin)
  const torrentIdParam = url.searchParams.get('torrentId')
  if (!torrentIdParam) {
    showToast('', t('torrent_request.invalid_fill_link'), 'error', 3000, true, 'tr')
    return
  }
  const torrentId = parseInt(torrentIdParam)
  if (isNaN(torrentId)) {
    showToast('', t('torrent_request.invalid_fill_link'), 'error', 3000, true, 'tr')
    return
  }

  fillLoading.value = true
  fillTorrentRequest({ torrent_id: torrentId, torrent_request_id: props.torrentRequest.id })
    .then((data) => {
      showToast('', t('torrent_request.fill_successful'), 'success', 3000, true, 'tr')
      emit('filled', torrentId, data.filler_is_uploader)
    })
    .finally(() => (fillLoading.value = false))
}
</script>
<style scoped>
table {
  .name {
    vertical-align: middle;
    text-align: right;
    padding-right: 5px;
    font-weight: bold;
  }
  .value {
    text-align: left;
  }
}
.votes-amount {
  margin-left: 4px;
  font-size: 0.9em;
}
.new-vote {
  margin-top: 40px;
  display: flex;
  align-items: center;
}
.fill-request {
  margin-top: 20px;
  display: flex;
  align-items: center;
  gap: 10px;
}
.filled-info {
  margin-top: 40px;
  display: flex;
  align-items: center;
  gap: 10px;
}
</style>
