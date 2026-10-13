<template>
  <div id="forum-stats-filters">
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
      <Select v-model="metric" :options="metricOptions" optionLabel="label" optionValue="value" size="small" input-id="metricSelect" style="min-width: 10em" />
      <label for="metricSelect">{{ t('stats.metric') }}</label>
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
  <div v-else-if="overallForumStats">
    <div id="forum-stats-summary">
      <ContentContainer :containerTitle="t('stats.total_threads_created')">
        {{ formatNumber(overallForumStats.total_threads_created) }}
      </ContentContainer>
      <ContentContainer :containerTitle="t('stats.total_posts_created')">
        {{ formatNumber(overallForumStats.total_posts_created) }}
      </ContentContainer>
      <ContentContainer :containerTitle="t('stats.unique_thread_creators')">
        {{ formatNumber(overallForumStats.unique_thread_creators) }}
      </ContentContainer>
      <ContentContainer :containerTitle="t('stats.unique_posters')">
        {{ formatNumber(overallForumStats.unique_posters) }}
      </ContentContainer>
      <ContentContainer :containerTitle="t('stats.average_post_length')">
        {{ formatNumber(Math.round(overallForumStats.average_post_length)) }} {{ t('stats.characters') }}
      </ContentContainer>
      <ContentContainer :containerTitle="t('stats.average_posts_per_thread')">
        {{ overallForumStats.average_posts_per_thread.toFixed(1) }}
      </ContentContainer>
    </div>
    <h3>{{ t('stats.overall_forum_activity') }}</h3>
    <Chart class="chart" :options="overallChartOptions" />
    <div v-for="groupBy in selectedGroupBys" :key="groupBy" class="grouped-chart">
      <h3>{{ groupByLabel(groupBy) }}</h3>
      <ProgressSpinner v-if="!groupedStats[groupBy]" />
      <template v-else>
        <div class="grouped-legend">
          <span v-for="(attr, i) in groupedData[groupBy].attributes" :key="attr" class="legend-item">
            <span class="legend-color" :style="{ backgroundColor: CHART_COLORS[i % CHART_COLORS.length] }" />
            {{ attr }}
          </span>
        </div>
        <div class="grouped-charts-row">
          <Chart class="chart grouped-line-chart" :options="groupedData[groupBy].lineOptions" />
          <Chart class="chart grouped-pie-chart" :options="groupedData[groupBy].pieOptions" />
        </div>
      </template>
    </div>
  </div>
</template>

<script setup lang="ts">
import { defineAsyncComponent } from 'vue'
import ContentContainer from '@/components/ContentContainer.vue'
import MultiSelect from 'primevue/multiselect'
import ProgressSpinner from 'primevue/progressspinner'
import Select from 'primevue/select'
import FloatLabel from 'primevue/floatlabel'
import { useI18n } from 'vue-i18n'
import { computed, onMounted, reactive, ref, watch } from 'vue'
import { getForumStats, ForumStatsGroupBy, ForumStatsMetric, StatsInterval, type ForumStatsResponse } from '@/services/api-schema'
import { baseChartOptions, CHART_COLORS, groupedAttributePieChartOptions, textColor } from '@/services/charts'
import { formatDateToLocalString, formatDateTimeLabel, formatNumber } from '@/services/helpers'

const Chart = defineAsyncComponent({
  loader: () => import('highcharts-vue').then((m) => m.Chart),
  loadingComponent: ProgressSpinner,
})

const { t } = useI18n()

type TimeRange = 'this_week' | 'this_month' | 'this_year' | 'all_time'

const timeRange = ref<TimeRange>('this_year')
const interval = ref<StatsInterval>(StatsInterval.Month)
const metric = ref<ForumStatsMetric>(ForumStatsMetric.Posts)
const selectedGroupBys = ref<ForumStatsGroupBy[]>([ForumStatsGroupBy.Category, ForumStatsGroupBy.UserClass])

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

const metricOptions = [
  { label: t('stats.metric_threads'), value: ForumStatsMetric.Threads },
  { label: t('stats.metric_posts'), value: ForumStatsMetric.Posts },
]

const groupByOptions = [
  { label: t('stats.forum_group_by_category'), value: ForumStatsGroupBy.Category },
  { label: t('stats.forum_group_by_sub_category'), value: ForumStatsGroupBy.SubCategory },
  { label: t('stats.forum_group_by_thread'), value: ForumStatsGroupBy.Thread },
  { label: t('stats.forum_group_by_user'), value: ForumStatsGroupBy.User },
  { label: t('stats.forum_group_by_user_class'), value: ForumStatsGroupBy.UserClass },
]

const groupByLabelMap: Record<string, string> = Object.fromEntries(groupByOptions.map((o) => [o.value, o.label]))

const groupByLabel = (groupBy: ForumStatsGroupBy) => groupByLabelMap[groupBy] ?? groupBy

const loading = ref(false)
const overallForumStats = ref<ForumStatsResponse>()
const groupedStats = reactive<Record<string, ForumStatsResponse>>({})

const metricLabel = computed(() => (metric.value === ForumStatsMetric.Threads ? t('stats.metric_threads') : t('stats.metric_posts')))

const overallChartOptions = computed<Highcharts.Options>(() => {
  if (!overallForumStats.value) return {}
  const data = overallForumStats.value.data
  return {
    ...baseChartOptions,
    chart: { ...baseChartOptions.chart, type: 'line' },
    xAxis: {
      categories: data.map((d) => formatDateTimeLabel(d.period, interval.value)),
      labels: { style: { color: textColor() } },
    },
    yAxis: {
      title: { text: undefined },
      labels: { style: { color: textColor() } },
    },
    series: [
      {
        type: 'line',
        name: metricLabel.value,
        data: data.map((d, i) => ({ y: d.count, totalContentLength: d.total_content_length, index: i })),
        color: CHART_COLORS[0],
        marker: { enabled: false, states: { hover: { enabled: true, radius: 5 } } },
      },
    ],
    tooltip: {
      formatter() {
        const point = this as unknown as Highcharts.Point & { totalContentLength?: number }
        return `<b>${point.category}</b><br/>${point.series.name}: ${formatNumber(point.y ?? 0)}<br/>${t('stats.total_content_length')}: ${formatNumber(point.totalContentLength ?? 0)} ${t('stats.characters')}`
      },
    },
  }
})

const groupedData = computed(() => {
  const result: Record<string, { attributes: string[]; lineOptions: Highcharts.Options; pieOptions: Highcharts.Options }> = {}
  for (const groupBy of selectedGroupBys.value) {
    const stats = groupedStats[groupBy]
    if (!stats) continue

    const periods = [...new Set(stats.data.map((d) => d.period))].sort()
    const byAttr = new Map<string, Map<string, { count: number; totalContentLength: number }>>()
    for (const point of stats.data) {
      const attr = point.attribute_value!
      if (!byAttr.has(attr)) byAttr.set(attr, new Map())
      byAttr.get(attr)!.set(point.period, { count: point.count, totalContentLength: point.total_content_length })
    }

    const attributes = [...byAttr.keys()]

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
        data: periods.map((p) => byAttr.get(attr)?.get(p)?.count ?? 0),
        color: CHART_COLORS[i % CHART_COLORS.length],
        marker: { enabled: false, states: { hover: { enabled: true, radius: 5 } } },
      })),
    }

    const pieOptions = groupedAttributePieChartOptions(
      attributes.map((attr) => {
        let countSum = 0
        let lengthSum = 0
        for (const v of byAttr.get(attr)!.values()) {
          countSum += v.count
          lengthSum += v.totalContentLength
        }
        return { name: attr, y: countSum, extraValue: lengthSum }
      }),
      {
        count: metricLabel.value,
        extra: { label: t('stats.total_content_length'), format: (value) => `${formatNumber(value)} ${t('stats.characters')}` },
      },
    )

    result[groupBy] = { attributes, lineOptions, pieOptions }
  }
  return result
})

const fetchForumStats = () => {
  const { from, to } = dateRangeFromSelection.value

  loading.value = true
  getForumStats({
    from: formatDateToLocalString(from),
    to: formatDateToLocalString(to),
    interval: interval.value,
    group_by: ForumStatsGroupBy.None,
    metric: metric.value,
  })
    .then((data) => {
      overallForumStats.value = data
    })
    .finally(() => {
      loading.value = false
    })
}

const fetchGroupedStats = () => {
  const { from, to } = dateRangeFromSelection.value

  for (const groupBy of selectedGroupBys.value) {
    if (groupedStats[groupBy]) continue
    getForumStats({
      from: formatDateToLocalString(from),
      to: formatDateToLocalString(to),
      interval: interval.value,
      group_by: groupBy,
      metric: metric.value,
    }).then((data) => {
      groupedStats[groupBy] = data
    })
  }

  for (const key of Object.keys(groupedStats)) {
    if (!selectedGroupBys.value.includes(key as ForumStatsGroupBy)) {
      delete groupedStats[key]
    }
  }
}

onMounted(() => {
  fetchForumStats()
  fetchGroupedStats()
})

watch([timeRange, interval, metric], () => {
  for (const key of Object.keys(groupedStats)) delete groupedStats[key]
  fetchForumStats()
  fetchGroupedStats()
})

watch(selectedGroupBys, () => {
  fetchGroupedStats()
})
</script>

<style scoped>
#forum-stats-filters {
  display: flex;
  justify-content: center;
  gap: 15px;
  margin-bottom: 15px;
}

#forum-stats-summary {
  display: flex;
  justify-content: center;
  flex-wrap: wrap;
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
</style>
