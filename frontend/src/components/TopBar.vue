<template>
  <div id="top-bar">
    <div class="left">
      <div class="wrapper-center logo-wrapper" style="display: flex; flex-direction: column">
        <RouterLink to="/">
          <img src="@/assets/logo.svg" alt="Site Logo" class="logo" />
        </RouterLink>
        <span id="logo-subtitle" v-if="publicArcadiaSettings.logo_subtitle">{{ publicArcadiaSettings.logo_subtitle }}</span>
      </div>
    </div>
    <div class="user-stats">
      <span v-if="publicArcadiaSettings.displayed_top_bar_stats.includes('uploaded')" class="stat" v-tooltip.bottom="t('general.uploaded')">
        <i class="pi pi-upload" />
        {{ bytesToReadable(user.uploaded) }}
      </span>
      <span v-if="publicArcadiaSettings.displayed_top_bar_stats.includes('downloaded')" class="stat" v-tooltip.bottom="t('general.downloaded')">
        <i class="pi pi-download" />{{ bytesToReadable(user.downloaded) }}
      </span>
      <span v-if="publicArcadiaSettings.displayed_top_bar_stats.includes('torrents')" class="stat" v-tooltip.bottom="t('user.torrents', 2)">
        <i class="pi pi-file-arrow-up" />{{ formatNumber(user.torrents) }}
      </span>
      <span v-if="publicArcadiaSettings.displayed_top_bar_stats.includes('forum_posts')" class="stat" v-tooltip.bottom="t('community.forum_posts')">
        <i class="pi pi-comments" />{{ formatNumber(user.forum_posts) }}
      </span>
      <span v-if="publicArcadiaSettings.displayed_top_bar_stats.includes('seeding')" class="stat" v-tooltip.bottom="t('torrent.seeding')">
        <i class="pi pi-arrow-up" />{{ formatNumber(user.seeding) }}
      </span>
      <span v-if="publicArcadiaSettings.displayed_top_bar_stats.includes('leeching')" class="stat" v-tooltip.bottom="t('torrent.leeching')">
        <i class="pi pi-arrow-down" />{{ formatNumber(user.leeching) }}
      </span>
      <span v-if="publicArcadiaSettings.displayed_top_bar_stats.includes('seeding_size')" class="stat" v-tooltip.bottom="t('user.seeding_size')">
        <i class="pi pi-database" />{{ bytesToReadable(user.seeding_size) }}
      </span>
      <span
        v-if="publicArcadiaSettings.displayed_top_bar_stats.includes('average_seeding_time')"
        class="stat"
        v-tooltip.bottom="t('user.average_seeding_time')"
      >
        <i class="pi pi-clock" />{{ secondsToReadable(user.average_seeding_time) }}
      </span>
      <RouterLink
        v-if="publicArcadiaSettings.displayed_top_bar_stats.includes('bonus_points')"
        to="/shop?tab=activities"
        v-tooltip.bottom="publicArcadiaSettings.bonus_points_alias"
        class="stat clickable-stat"
      >
        <i class="pi pi-wallet" />{{ formatBp(user.bonus_points, publicArcadiaSettings.bonus_points_decimal_places) }}
      </RouterLink>
      <span v-if="publicArcadiaSettings.displayed_top_bar_stats.includes('freeleech_tokens')" class="stat" v-tooltip.bottom="t('user.freeleech_tokens')">
        <i class="pi pi-ticket" />{{ formatNumber(user.freeleech_tokens) }}
      </span>
      <span v-if="publicArcadiaSettings.displayed_top_bar_stats.includes('current_streak')" class="stat" v-tooltip.bottom="t('user.current_streak')">
        <i class="pi pi-bolt" />{{ formatNumber(user.current_streak) }}
      </span>
    </div>
    <div class="right">
      <RouterLink v-if="publicArcadiaSettings.running_advent_calendar_button_text" id="advent-calendar-button" to="/advent-calendar">
        <Button :label="publicArcadiaSettings.running_advent_calendar_button_text" icon="pi pi-gift" size="small" />
      </RouterLink>
      <NavMenu />
    </div>
  </div>
</template>

<script setup lang="ts">
import { useUserStore } from '@/stores/user'
import { bytesToReadable, formatBp, formatNumber, secondsToReadable } from '@/services/helpers'
import NavMenu from './nav_menu/NavMenu.vue'
import { usePublicArcadiaSettingsStore } from '@/stores/publicArcadiaSettings'
import { RouterLink } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { Button } from 'primevue'

const { t } = useI18n()
const user = useUserStore()
const publicArcadiaSettings = usePublicArcadiaSettingsStore()
</script>

<style scoped>
#top-bar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  background-color: var(--color-background-secondary);
  height: 45px;
  padding: 0 7px;
  width: 100%;
}
.right {
  display: flex;
  align-items: center;
  gap: 10px;
}
.left .logo {
  height: 40px;
  vertical-align: middle;
}
#logo-subtitle {
  font-size: 0.85em;
  margin-top: -4px;
  margin-bottom: -2px;
  font-weight: bold;
}
.user-stats {
  font-size: 0.85em;
  display: flex;
  align-items: center;
  .stat {
    margin: 0px 10px;
    display: flex;
    align-items: center;
  }
  i {
    margin-right: 7px;
  }
  .clickable-stat {
    cursor: pointer;
    text-decoration: none;
    color: inherit;
    transition: color 0.2s;

    &:hover {
      color: var(--color-primary);
    }
  }
}
</style>
