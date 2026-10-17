<template>
  <div id="user-stats-filters">
    <FloatLabel>
      <Select
        v-model="timeRange"
        :options="timeRangeOptions"
        optionLabel="label"
        optionValue="value"
        size="small"
        input-id="userTimeRangeSelect"
        style="min-width: 10em"
      />
      <label for="userTimeRangeSelect">{{ t('stats.time_range') }}</label>
    </FloatLabel>
    <FloatLabel>
      <Select
        v-model="interval"
        :options="intervalOptions"
        optionLabel="label"
        optionValue="value"
        size="small"
        input-id="userIntervalSelect"
        style="min-width: 10em"
      />
      <label for="userIntervalSelect">{{ t('stats.interval') }}</label>
    </FloatLabel>
  </div>
  <ProgressSpinner v-if="loading" />
  <div v-else-if="userStats">
    <div id="user-stats-summary">
      <ContentContainer :containerTitle="t('stats.new_users')">
        {{ formatNumber(userStats.new_users) }}
      </ContentContainer>
    </div>
    <h3>{{ t('stats.user_flux') }}</h3>
    <Chart class="chart" :options="userFluxChartOptions" />
    <h3>{{ t('stats.torrent_clients_repartition') }}</h3>
    <Chart class="chart" :options="torrentClientChartOptions" />
  </div>
</template>

<script setup lang="ts">
import { defineAsyncComponent, computed, onMounted, ref, watch } from 'vue'
import ContentContainer from '@/components/ContentContainer.vue'
import ProgressSpinner from 'primevue/progressspinner'
import Select from 'primevue/select'
import FloatLabel from 'primevue/floatlabel'
import { useI18n } from 'vue-i18n'
import { getUserStats, StatsInterval, type UserStatsResponse } from '@/services/api-schema'
import { baseChartOptions, CHART_COLORS, sideLegend, textColor } from '@/services/charts'
import { formatDateToLocalString, formatDateTimeLabel, formatNumber } from '@/services/helpers'

const Chart = defineAsyncComponent({
  loader: () => import('highcharts-vue').then((m) => m.Chart),
  loadingComponent: ProgressSpinner,
})

const { t } = useI18n()

type TimeRange = 'this_week' | 'this_month' | 'this_year' | 'all_time'

const timeRange = ref<TimeRange>('this_year')
const interval = ref<StatsInterval>(StatsInterval.Month)

const timeRangeOptions = [
  { label: t('stats.this_week'), value: 'this_week' },
  { label: t('stats.this_month'), value: 'this_month' },
  { label: t('stats.this_year'), value: 'this_year' },
  { label: t('stats.all_time'), value: 'all_time' },
]

const intervalOptions = [
  { label: t('stats.hour'), value: StatsInterval.Hour },
  { label: t('stats.day'), value: StatsInterval.Day },
  { label: t('stats.week'), value: StatsInterval.Week },
  { label: t('stats.month'), value: StatsInterval.Month },
  { label: t('stats.year'), value: StatsInterval.Year },
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

const loading = ref(false)
const userStats = ref<UserStatsResponse>()

// Clients below that share of the pie are summed into a single "other clients" slice.
const OTHER_CLIENTS_MIN_PERCENTAGE = 0.5

const userFluxChartOptions = computed<Highcharts.Options>(() => {
  if (!userStats.value) return {}
  const data = userStats.value.data
  return {
    chart: { backgroundColor: 'transparent', type: 'line' },
    title: { text: undefined },
    credits: { enabled: false },
    legend: { enabled: false },
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
        name: t('stats.new_users'),
        data: data.map((d) => d.count),
        color: CHART_COLORS[0],
        marker: { enabled: false, states: { hover: { enabled: true, radius: 5 } } },
      },
    ],
    tooltip: {
      formatter() {
        const point = this as unknown as Highcharts.Point
        return `<b>${point.category}</b><br/>${point.series.name}: ${formatNumber(point.y ?? 0)}`
      },
    },
  }
})

const torrentClientChartOptions = computed<Highcharts.Options>(() => {
  if (!userStats.value || !userStats.value.torrent_clients) return {}
  const clients = userStats.value.torrent_clients
  const total = clients.reduce((sum, client) => sum + client.count, 0)
  const data: { name: string; y: number }[] = []
  let otherClientsCount = 0
  for (const client of clients) {
    if (total > 0 && (client.count / total) * 100 < OTHER_CLIENTS_MIN_PERCENTAGE) {
      otherClientsCount += client.count
    } else {
      data.push({ name: client.client, y: client.count })
    }
  }
  if (otherClientsCount > 0) {
    data.push({ name: t('stats.other_clients'), y: otherClientsCount })
  }
  return {
    ...baseChartOptions,
    chart: { ...baseChartOptions.chart, type: 'pie' },
    tooltip: {
      pointFormat: '{series.name}: <b>{point.y} ({point.percentage:.1f}%)</b>',
    },
    plotOptions: {
      pie: {
        allowPointSelect: true,
        cursor: 'pointer',
        dataLabels: {
          enabled: true,
          format: '<b>{point.name}</b>: {point.percentage:.1f}%',
          style: {
            color: textColor(),
          },
        },
        showInLegend: true,
      },
    },
    series: [
      {
        type: 'pie',
        name: t('stats.torrent_clients'),
        data,
      },
    ],
    legend: sideLegend(),
  }
})

const fetchUserStats = () => {
  const { from, to } = dateRangeFromSelection.value

  loading.value = true
  getUserStats({
    from: formatDateToLocalString(from),
    to: formatDateToLocalString(to),
    interval: interval.value,
  })
    .then((data) => {
      userStats.value = data
    })
    .finally(() => {
      loading.value = false
    })
}

onMounted(() => {
  fetchUserStats()
})

watch([timeRange, interval], () => {
  fetchUserStats()
})
</script>

<style scoped>
#user-stats-filters {
  display: flex;
  justify-content: center;
  gap: 15px;
  margin-bottom: 15px;
}

#user-stats-summary {
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

h3 {
  text-align: center;
  margin-bottom: 10px;
  font-weight: bold;
}
</style>
