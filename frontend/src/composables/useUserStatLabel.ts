import { useI18n } from 'vue-i18n'
import { DisplayableUserStats } from '@/services/api-schema'
import { usePublicArcadiaSettingsStore } from '@/stores/publicArcadiaSettings'

// translation of every user statistic, reused everywhere a statistic is named, so that they are
// never translated twice. the bonus points are named after the alias chosen by the site's staff
const translationKeys: Record<Exclude<DisplayableUserStats, typeof DisplayableUserStats.BonusPoints>, string> = {
  [DisplayableUserStats.JoinedAt]: 'user.joined_at',
  [DisplayableUserStats.LastSeen]: 'user.last_seen',
  [DisplayableUserStats.Uploaded]: 'general.uploaded',
  [DisplayableUserStats.RealUploaded]: 'general.uploaded_real',
  [DisplayableUserStats.Downloaded]: 'general.downloaded',
  [DisplayableUserStats.RealDownloaded]: 'general.downloaded_real',
  [DisplayableUserStats.Ratio]: 'general.ratio',
  [DisplayableUserStats.Seeding]: 'torrent.seeding',
  [DisplayableUserStats.Leeching]: 'torrent.leeching',
  [DisplayableUserStats.Snatched]: 'torrent.snatched',
  [DisplayableUserStats.SeedingSize]: 'user.seeding_size',
  [DisplayableUserStats.AverageSeedingTime]: 'user.average_seeding_time',
  [DisplayableUserStats.FreeleechTokens]: 'user.freeleech_tokens',
  [DisplayableUserStats.CurrentStreak]: 'user.current_streak',
  [DisplayableUserStats.HighestStreak]: 'user.highest_streak',
  [DisplayableUserStats.TitleGroups]: 'artist.title_groups',
  [DisplayableUserStats.EditionGroups]: 'edition_group.edition_group',
  [DisplayableUserStats.Torrents]: 'statistics.torrents',
  [DisplayableUserStats.ForumThreads]: 'community.forum_threads',
  [DisplayableUserStats.ForumPosts]: 'community.forum_posts',
  [DisplayableUserStats.CollagesStarted]: 'community.collages_started',
  [DisplayableUserStats.TitleGroupComments]: 'community.title_group_comments',
  [DisplayableUserStats.TitleGroupReviews]: 'community.title_group_reviews',
  [DisplayableUserStats.RequestComments]: 'community.request_comments',
  [DisplayableUserStats.ArtistComments]: 'community.artist_comments',
  [DisplayableUserStats.RequestsVoted]: 'community.request_voted',
  [DisplayableUserStats.RequestsFilled]: 'community.request_filled',
  [DisplayableUserStats.Invited]: 'community.invited',
  [DisplayableUserStats.Invitations]: 'user.invitations',
}

export const useUserStatLabel = () => {
  const { t } = useI18n()
  const publicArcadiaSettings = usePublicArcadiaSettingsStore()

  // the plural form is asked for the statistics translated with one, the other translations
  // are not pluralized and are returned as they are
  return (stat: DisplayableUserStats) => (stat === DisplayableUserStats.BonusPoints ? publicArcadiaSettings.bonus_points_alias : t(translationKeys[stat], 2))
}
