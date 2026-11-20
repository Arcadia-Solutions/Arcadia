import { useI18n } from 'vue-i18n'
import type { AdventCalendarDoorOpeningResult, AdventCalendarDoorRewardTotals } from '@/services/api-schema'
import { bytesToReadable, formatBp, formatNumber } from '@/services/helpers'
import { usePublicArcadiaSettingsStore } from '@/stores/publicArcadiaSettings'

// the rewards of a door opening (or the sums of several openings), one readable line per reward greater than 0
export const useAdventCalendarRewards = () => {
  const { t } = useI18n()
  const publicArcadiaSettings = usePublicArcadiaSettingsStore()

  return (rewards: AdventCalendarDoorOpeningResult | AdventCalendarDoorRewardTotals) => {
    const lines: string[] = []
    if (rewards.upload > 0) lines.push(`${bytesToReadable(rewards.upload)} ${t('advent_calendar.reward_upload').toLowerCase()}`)
    if (rewards.download_removed > 0) lines.push(`${bytesToReadable(rewards.download_removed)} ${t('advent_calendar.reward_download_removed').toLowerCase()}`)
    if (rewards.bonus_points > 0)
      lines.push(`${formatBp(rewards.bonus_points, publicArcadiaSettings.bonus_points_decimal_places)} ${publicArcadiaSettings.bonus_points_alias}`)
    if (rewards.freeleech_tokens > 0) lines.push(`${formatNumber(rewards.freeleech_tokens)} ${t('advent_calendar.reward_freeleech_tokens').toLowerCase()}`)
    if (rewards.invitations > 0) lines.push(`${formatNumber(rewards.invitations)} ${t('advent_calendar.reward_invitations').toLowerCase()}`)
    return lines
  }
}
