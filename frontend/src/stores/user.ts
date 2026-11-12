import type { User } from '@/services/api-schema'
import { defineStore } from 'pinia'

const initialState: User = {
  artist_comments: 0,
  avatar: null,
  average_seeding_time: 0,
  bonus_points: 0,
  class_name: 'newbie',
  class_locked: false,
  permissions: [],
  title_groups: 0,
  edition_groups: 0,
  torrents: 0,
  collages_started: 0,
  created_at: '',
  description: '',
  downloaded: 0,
  email: '',
  forum_posts: 0,
  forum_threads: 0,
  freeleech_tokens: 0,
  id: 0,
  invitations: 0,
  invited: 0,
  last_seen: '',
  leeching: 0,
  password_hash: '',
  real_downloaded: 0,
  real_uploaded: 0,
  registered_from_ip: '',
  request_comments: 0,
  requests_filled: 0,
  requests_voted: 0,
  seeding: 0,
  seeding_size: 0,
  snatched: 0,
  title_group_comments: 0,
  title_group_ratings: 0,
  uploaded: 0,
  username: '',
  warned: false,
  banned: false,
  passkey: '',
  css_sheet_name: '',
  current_streak: 0,
  highest_streak: 0,
  irc_site_embed_enabled: true,
  show_upload_method: false,
  irc_password: null,
}

export const useUserStore = defineStore('user', {
  state: (): User => initialState,

  actions: {
    setUser(user: User) {
      Object.assign(this.$state, user)
    },
    removeUser() {
      Object.assign(this.$state, initialState)
    },
  },
})
