<template>
  <div class="catalog-stats">
    <ContentContainer :containerTitle="t('stats.title_groups_per_release_year')">
      <Chart v-if="releaseYearChartOptions" class="release-year-chart" :options="releaseYearChartOptions" />
      <div v-if="titlesWithoutReleaseDate > 0" class="caption">
        {{ t('stats.title_groups_without_release_date') }}: <b>{{ formatNumber(titlesWithoutReleaseDate) }}</b>
      </div>
      <div v-else-if="!releaseYearChartOptions" class="caption">{{ t('stats.no_data') }}</div>
    </ContentContainer>
    <!-- in general series have a single content type, so its pie would be flat -->
    <ContentContainer :containerTitle="t('stats.group_by_content_type')">
      <Chart v-if="contentTypeChartOptions" class="pie-chart" :options="contentTypeChartOptions" />
      <div v-else class="caption">{{ t('stats.no_data') }}</div>
    </ContentContainer>
  </div>
</template>

<script setup lang="ts">
import ContentContainer from '@/components/ContentContainer.vue'
import { useI18n } from 'vue-i18n'
import { computed } from 'vue'
import { type TitleGroupStatsResponse } from '@/services/api-schema'
import { attributePieChartOptions, titleGroupsPerReleaseYearBarChartOptions } from '@/services/charts'
import { formatNumber } from '@/services/helpers'

const { t } = useI18n()

const props = defineProps<{
  titleGroupStats: TitleGroupStatsResponse
}>()

const releaseYearChartOptions = computed(() =>
  titleGroupsPerReleaseYearBarChartOptions(props.titleGroupStats.title_groups_per_release_year, {
    titleGroups: t('stats.title_groups'),
    count: t('stats.count'),
  }),
)

const titlesWithoutReleaseDate = computed(() => props.titleGroupStats.title_groups_per_release_year.find((entry) => entry.year == null)?.count ?? 0)

const contentTypeChartOptions = computed(() =>
  attributePieChartOptions(
    props.titleGroupStats.content_types.map((dataPoint) => ({ ...dataPoint, attribute_value: t(`title_group.content_type.${dataPoint.attribute_value}`) })),
    { count: t('stats.count') },
  ),
)
</script>

<style scoped>
.catalog-stats {
  width: 100%;
  > * + * {
    margin-top: 10px;
  }
}

.release-year-chart {
  height: 140px;
}

.pie-chart {
  height: 200px;
}

.caption {
  text-align: center;
  font-size: 0.9em;
}
</style>
