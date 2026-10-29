<template>
  <div id="torrent-stats-filters">
    <FloatLabel>
      <Select
        v-model="timeRange"
        :options="timeRangeOptions"
        optionLabel="label"
        optionValue="value"
        size="small"
        input-id="timeRangeSelect"
        style="min-width: 10em"
      />
      <label for="timeRangeSelect">{{ t('stats.time_range') }}</label>
    </FloatLabel>
    <FloatLabel>
      <Select
        v-model="interval"
        :options="intervalOptions"
        optionLabel="label"
        optionValue="value"
        size="small"
        input-id="intervalSelect"
        style="min-width: 10em"
      />
      <label for="intervalSelect">{{ t('stats.interval') }}</label>
    </FloatLabel>
    <FloatLabel>
      <MultiSelect
        v-model="selectedGroupBys"
        :options="groupByOptions"
        optionLabel="label"
        optionValue="value"
        size="small"
        input-id="groupBySelect"
        style="min-width: 15em"
        :maxSelectedLabels="2"
      />
      <label for="groupBySelect">{{ t('stats.group_by') }}</label>
    </FloatLabel>
  </div>
  <ProgressSpinner v-if="loading" />
  <div v-else-if="overallTorrentStats">
    <div id="torrent-stats-summary">
      <ContentContainer :containerTitle="t('stats.unique_uploaders')">
        {{ formatNumber(overallTorrentStats.unique_uploaders) }}
      </ContentContainer>
      <ContentContainer :containerTitle="t('stats.total_size')">
        {{ bytesToReadable(totalSize) }}
      </ContentContainer>
      <ContentContainer :containerTitle="t('stats.total_torrents')">
        {{ formatNumber(totalCount) }}
      </ContentContainer>
      <ContentContainer :containerTitle="t('stats.total_snatches')">
        {{ formatNumber(totalSnatches) }}
      </ContentContainer>
      <ContentContainer :containerTitle="t('stats.total_snatched_size')">
        {{ bytesToReadable(totalSnatchedSize) }}
      </ContentContainer>
      <ContentContainer :containerTitle="t('stats.total_seeding_size')">
        {{ bytesToReadable(overallTorrentStats.total_seeding_size) }}
      </ContentContainer>
    </div>
    <h3>{{ t('stats.torrents_and_snatches_over_time') }}</h3>
    <Chart class="chart" :options="overallChartOptions" />
    <template v-if="releaseYearChartOptions">
      <h3>{{ t('stats.title_groups_per_release_year') }}</h3>
      <p v-if="noReleaseDateCount > 0" class="release-year-no-date">
        {{ t('stats.title_groups_without_release_date') }}: <b>{{ formatNumber(noReleaseDateCount) }}</b>
      </p>
      <Chart class="chart" :options="releaseYearChartOptions" />
    </template>
    <div v-for="groupBy in selectedGroupBys" :key="groupBy" class="grouped-chart">
      <ProgressSpinner v-if="!groupedStats[groupBy]" />
      <template v-else>
        <template v-for="metric in groupedData[groupBy].metrics" :key="metric.label">
          <h3>{{ metric.label }}</h3>
          <div class="grouped-legend">
            <span v-for="(attr, i) in groupedData[groupBy].attributes" :key="attr" class="legend-item">
              <span class="legend-color" :style="{ backgroundColor: CHART_COLORS[i % CHART_COLORS.length] }" />
              {{ attr }}
            </span>
          </div>
          <div class="grouped-charts-row">
            <Chart class="chart grouped-line-chart" :options="metric.lineOptions" />
            <Chart class="chart grouped-pie-chart" :options="metric.pieOptions" />
          </div>
        </template>
      </template>
    </div>
  </div>
</template>

<script setup lang="ts">
import ContentContainer from '@/components/ContentContainer.vue'
import { Chart } from 'highcharts-vue'
import Highcharts from 'highcharts'
import MultiSelect from 'primevue/multiselect'
import ProgressSpinner from 'primevue/progressspinner'
import Select from 'primevue/select'
import FloatLabel from 'primevue/floatlabel'
import { useI18n } from 'vue-i18n'
import { computed, onMounted, reactive, ref, watch } from 'vue'
import { getTorrentStats, StatsInterval, TorrentStatsGroupBy, type TorrentStatsDataPoint, type TorrentStatsResponse } from '@/services/api-schema'
import { baseChartOptions, CHART_COLORS, groupedAttributePieChartOptions, textColor, titleGroupsPerReleaseYearChartOptions } from '@/services/charts'
import { bytesToReadable, formatDateToLocalString, formatDateTimeLabel, formatNumber } from '@/services/helpers'

const { t } = useI18n()

type TimeRange = 'this_week' | 'this_month' | 'this_year' | 'all_time'

const timeRange = ref<TimeRange>('this_year')
const interval = ref<StatsInterval>(StatsInterval.Month)
const selectedGroupBys = ref<TorrentStatsGroupBy[]>([TorrentStatsGroupBy.ContentType, TorrentStatsGroupBy.Source])

const timeRangeOptions = [
  { label: t('stats.this_week'), value: 'this_week' },
  { label: t('stats.this_month'), value: 'this_month' },
  { label: t('stats.this_year'), value: 'this_year' },
  { label: t('stats.all_time'), value: 'all_time' },
]

const dateRangeFromSelection = computed(() => {
  const now = new Date()
  const to = now
  let from: Date
  switch (timeRange.value) {
    case 'this_week':
      from = new Date(now.getFullYear(), now.getMonth(), now.getDate() - 7)
      break
    case 'this_month':
      from = new Date(now.getFullYear(), now.getMonth(), now.getDate() - 30)
      break
    case 'this_year':
      from = new Date(now.getFullYear() - 1, now.getMonth(), now.getDate())
      break
    case 'all_time':
      // arcadia wasn't possibly used before this date :)
      from = new Date(2025, 0, 1)
      break
  }
  return { from, to }
})

const intervalOptions = [
  { label: t('stats.hour'), value: StatsInterval.Hour },
  { label: t('stats.day'), value: StatsInterval.Day },
  { label: t('stats.week'), value: StatsInterval.Week },
  { label: t('stats.month'), value: StatsInterval.Month },
  { label: t('stats.year'), value: StatsInterval.Year },
]

const groupByOptions = [
  { label: t('stats.group_by_content_type'), value: TorrentStatsGroupBy.ContentType },
  { label: t('stats.group_by_source'), value: TorrentStatsGroupBy.Source },
  { label: t('stats.group_by_video_resolution'), value: TorrentStatsGroupBy.VideoResolution },
  { label: t('stats.group_by_video_codec'), value: TorrentStatsGroupBy.VideoCodec },
  { label: t('stats.group_by_audio_codec'), value: TorrentStatsGroupBy.AudioCodec },
  { label: t('stats.group_by_audio_channels'), value: TorrentStatsGroupBy.AudioChannels },
  { label: t('stats.group_by_audio_bitrate_sampling'), value: TorrentStatsGroupBy.AudioBitrateSampling },
  { label: t('stats.group_by_container'), value: TorrentStatsGroupBy.Container },
  { label: t('stats.group_by_upload_method'), value: TorrentStatsGroupBy.UploadMethod },
  { label: t('stats.group_by_category'), value: TorrentStatsGroupBy.Category },
  { label: t('stats.group_by_platform'), value: TorrentStatsGroupBy.Platform },
  { label: t('stats.group_by_original_language'), value: TorrentStatsGroupBy.OriginalLanguage },
  { label: t('stats.group_by_country_from'), value: TorrentStatsGroupBy.CountryFrom },
]

const groupByLabelMap: Record<string, string> = Object.fromEntries(groupByOptions.map((o) => [o.value, o.label]))

const groupByLabel = (groupBy: TorrentStatsGroupBy) => groupByLabelMap[groupBy] ?? groupBy

const loading = ref(false)
const overallTorrentStats = ref<TorrentStatsResponse>()
const groupedStats = reactive<Record<string, TorrentStatsResponse>>({})

const totalSize = computed(() => overallTorrentStats.value?.data.reduce((sum, d) => sum + d.total_size, 0) ?? 0)
const totalCount = computed(() => overallTorrentStats.value?.data.reduce((sum, d) => sum + d.count, 0) ?? 0)
const totalSnatches = computed(() => overallTorrentStats.value?.data.reduce((sum, d) => sum + d.snatches, 0) ?? 0)
const totalSnatchedSize = computed(() => overallTorrentStats.value?.data.reduce((sum, d) => sum + d.snatched_size, 0) ?? 0)

const overallChartOptions = computed<Highcharts.Options>(() => {
  if (!overallTorrentStats.value) return {}
  const data = overallTorrentStats.value.data
  const deletions = overallTorrentStats.value.deletions
  const deletionsByPeriod = new Map(deletions.map((d) => [d.period, d]))
  return {
    ...baseChartOptions,
    chart: { ...baseChartOptions.chart, type: 'line' },
    legend: { enabled: true, itemStyle: { color: textColor() } },
    xAxis: {
      categories: data.map((d) => formatDateTimeLabel(d.period, interval.value)),
      labels: { style: { color: textColor() } },
    },
    yAxis: [
      {
        title: { text: undefined },
        labels: { style: { color: textColor() } },
      },
      {
        title: { text: t('stats.snatches'), style: { color: CHART_COLORS[2] } },
        labels: { style: { color: CHART_COLORS[2] } },
        opposite: true,
      },
    ],
    series: [
      {
        type: 'line',
        name: t('stats.uploads'),
        data: data.map((d) => ({ y: d.count, totalSize: d.total_size })),
        color: CHART_COLORS[0],
        marker: { enabled: false, states: { hover: { enabled: true, radius: 5 } } },
      },
      {
        type: 'line',
        name: t('stats.deletions'),
        data: data.map((d) => {
          const del = deletionsByPeriod.get(d.period)
          return {
            y: del?.count ?? 0,
            trumped: del?.trumped ?? 0,
            duplicate: del?.duplicate ?? 0,
            other: del?.other ?? 0,
          }
        }),
        color: CHART_COLORS[1],
        marker: { enabled: false, states: { hover: { enabled: true, radius: 5 } } },
      },
      {
        type: 'line',
        name: t('stats.snatches'),
        yAxis: 1,
        data: data.map((d) => ({ y: d.snatches, totalSize: d.snatched_size })),
        color: CHART_COLORS[2],
        marker: { enabled: false, states: { hover: { enabled: true, radius: 5 } } },
      },
    ],
    tooltip: {
      shared: true,
      useHTML: true,
      formatter() {
        const ctx = this as unknown as {
          points: (Highcharts.Point & {
            category: string
            totalSize?: number
            trumped?: number
            duplicate?: number
            other?: number
          })[]
        }
        const lines = ctx.points
          .map((point) => {
            if (point.series.name === t('stats.deletions')) {
              const reasons = `<ul style="margin: 2px 0 0 16px; padding: 0;">
                <li>${t('notification.deletion_reason_trumped')}: ${formatNumber(point.trumped ?? 0)}</li>
                <li>${t('notification.deletion_reason_duplicate')}: ${formatNumber(point.duplicate ?? 0)}</li>
                <li>${t('notification.deletion_reason_other')}: ${formatNumber(point.other ?? 0)}</li>
              </ul>`
              return `<span style="color:${point.series.color}">●</span> ${point.series.name}: <b>${formatNumber(point.y ?? 0)}</b>${reasons}`
            }
            return `<span style="color:${point.series.color}">●</span> ${point.series.name}: <b>${formatNumber(point.y ?? 0)}</b><br/>${t('stats.total_size')}: ${bytesToReadable(point.totalSize ?? 0)}`
          })
          .join('<br/>')
        return `<b>${ctx.points[0].category}</b><br/>${lines}`
      },
    },
  }
})

const noReleaseDateCount = computed(() => overallTorrentStats.value?.title_groups_per_release_year.find((entry) => entry.year == null)?.count ?? 0)

const releaseYearChartOptions = computed<Highcharts.Options | null>(() =>
  overallTorrentStats.value
    ? titleGroupsPerReleaseYearChartOptions(overallTorrentStats.value.title_groups_per_release_year, {
        titleGroups: t('stats.title_groups'),
        count: t('stats.count'),
      })
    : null,
)

const groupedData = computed(() => {
  const result: Record<string, { attributes: string[]; metrics: { label: string; lineOptions: Highcharts.Options; pieOptions: Highcharts.Options }[] }> = {}
  for (const groupBy of selectedGroupBys.value) {
    const stats = groupedStats[groupBy]
    if (!stats) continue

    const periods = [...new Set(stats.data.map((d) => d.period))].sort()
    const byAttr = new Map<string, Map<string, TorrentStatsDataPoint>>()
    for (const point of stats.data) {
      const attr = point.attribute_value!
      if (!byAttr.has(attr)) byAttr.set(attr, new Map())
      byAttr.get(attr)!.set(point.period, point)
    }

    const attributes = [...byAttr.keys()]

    const buildMetricCharts = (
      label: string,
      countOf: (point: TorrentStatsDataPoint) => number,
      sizeOf: (point: TorrentStatsDataPoint) => number,
      sizeLabel: string,
    ) => {
      const lineOptions: Highcharts.Options = {
        ...baseChartOptions,
        chart: { ...baseChartOptions.chart, type: 'line' },
        xAxis: {
          categories: periods.map((p) => formatDateTimeLabel(p, interval.value)),
          labels: { style: { color: textColor() } },
        },
        yAxis: {
          title: { text: undefined },
          labels: { style: { color: textColor() } },
        },
        tooltip: {
          shared: true,
        },
        series: attributes.map((attr, i) => ({
          type: 'line' as const,
          name: attr,
          data: periods.map((p) => {
            const point = byAttr.get(attr)?.get(p)
            return point ? countOf(point) : 0
          }),
          color: CHART_COLORS[i % CHART_COLORS.length],
          marker: { enabled: false, states: { hover: { enabled: true, radius: 5 } } },
        })),
      }

      const pieOptions = groupedAttributePieChartOptions(
        attributes.map((attr) => {
          let countSum = 0
          let sizeSum = 0
          for (const point of byAttr.get(attr)!.values()) {
            countSum += countOf(point)
            sizeSum += sizeOf(point)
          }
          return { name: attr, y: countSum, extraValue: sizeSum }
        }),
        { count: t('stats.count'), extra: { label: sizeLabel, format: bytesToReadable } },
      )

      return { label, lineOptions, pieOptions }
    }

    result[groupBy] = {
      attributes,
      metrics: [
        buildMetricCharts(
          t('stats.torrents_by', { attribute: groupByLabel(groupBy).toLowerCase() }),
          (p) => p.count,
          (p) => p.total_size,
          t('stats.total_size'),
        ),
        buildMetricCharts(
          t('stats.snatches_by', { attribute: groupByLabel(groupBy).toLowerCase() }),
          (p) => p.snatches,
          (p) => p.snatched_size,
          t('stats.total_snatched_size'),
        ),
      ],
    }
  }
  return result
})

const fetchTorrentStats = () => {
  const { from, to } = dateRangeFromSelection.value

  loading.value = true
  getTorrentStats({
    from: formatDateToLocalString(from),
    to: formatDateToLocalString(to),
    interval: interval.value,
    group_by: TorrentStatsGroupBy.None,
  })
    .then((data) => {
      overallTorrentStats.value = data
    })
    .finally(() => {
      loading.value = false
    })
}

const fetchGroupedStats = () => {
  const { from, to } = dateRangeFromSelection.value

  for (const groupBy of selectedGroupBys.value) {
    if (groupedStats[groupBy]) continue
    getTorrentStats({
      from: formatDateToLocalString(from),
      to: formatDateToLocalString(to),
      interval: interval.value,
      group_by: groupBy,
    }).then((data) => {
      groupedStats[groupBy] = data
    })
  }

  for (const key of Object.keys(groupedStats)) {
    if (!selectedGroupBys.value.includes(key as TorrentStatsGroupBy)) {
      delete groupedStats[key]
    }
  }
}

onMounted(() => {
  fetchTorrentStats()
  fetchGroupedStats()
})

watch([timeRange, interval], () => {
  for (const key of Object.keys(groupedStats)) delete groupedStats[key]
  fetchTorrentStats()
  fetchGroupedStats()
})

watch(selectedGroupBys, () => {
  fetchGroupedStats()
})
</script>

<style scoped>
#torrent-stats-filters {
  display: flex;
  justify-content: center;
  gap: 15px;
  margin-bottom: 15px;
}

#torrent-stats-summary {
  display: flex;
  justify-content: center;
  gap: 15px;
  margin-bottom: 25px;
  :deep(.content-body) {
    font-size: 1.2em;
    font-weight: bold;
    text-align: center;
  }
}

.chart {
  height: 30vh;
}

.grouped-chart {
  margin-top: 30px;
}

.grouped-legend {
  display: flex;
  justify-content: center;
  flex-wrap: wrap;
  gap: 8px 16px;
  margin-bottom: 10px;
}

.legend-item {
  display: flex;
  align-items: center;
  gap: 5px;
  font-size: 0.85em;
}

.legend-color {
  display: inline-block;
  width: 12px;
  height: 12px;
  border-radius: 2px;
}

.grouped-charts-row {
  display: flex;
}

.grouped-line-chart {
  flex: 2;
}

.grouped-pie-chart {
  flex: 1;
}
h3 {
  text-align: center;
  margin-bottom: 10px;
  font-weight: bold;
}

.release-year-no-date {
  text-align: center;
  margin-bottom: 10px;
  font-size: 0.9em;
}
</style>
