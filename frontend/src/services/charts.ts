import type Highcharts from 'highcharts'
import type { TitleGroupAttributeCountDataPoint, TitleGroupsPerReleaseYearDataPoint } from '@/services/api-schema'
import { formatNumber } from '@/services/helpers'
import { defineAsyncComponent } from 'vue'
import ProgressSpinner from 'primevue/progressspinner'

const loadChart = () => import('highcharts-vue').then((m) => m.Chart)

// Highcharts is only downloaded once a component actually renders a chart.
export const Chart = defineAsyncComponent({ loader: loadChart, loadingComponent: ProgressSpinner })

// The map module registers the mapChart constructor, it must run after highcharts core (pulled in by highcharts-vue) is evaluated.
export const MapChart = defineAsyncComponent({
  loader: () => loadChart().then((chart) => import('highcharts/modules/map').then(() => chart)),
  loadingComponent: ProgressSpinner,
})

export const CHART_COLORS = [
  '#3B82F6',
  '#EF4444',
  '#10B981',
  '#F59E0B',
  '#8B5CF6',
  '#EC4899',
  '#06B6D4',
  '#F97316',
  '#84CC16',
  '#6366F1',
  '#14B8A6',
  '#E11D48',
  '#A855F7',
  '#0EA5E9',
  '#D946EF',
  '#65A30D',
]

export const textColor = () => getComputedStyle(document.documentElement).getPropertyValue('color') || '#ccc'

/** Main color of the site, read from the CSS variable so a theme change follows. */
export const primaryColor = () => getComputedStyle(document.documentElement).getPropertyValue('--color-primary').trim() || '#5fe1ab'

export const baseChartOptions: Highcharts.Options = {
  chart: {
    backgroundColor: 'transparent',
  },
  title: { text: undefined },
  credits: { enabled: false },
  legend: { enabled: false },
}

/** Legend drawn by Highcharts itself, for the narrow panels with no room for an HTML one. */
export const sideLegend = (): Highcharts.LegendOptions => ({
  enabled: true,
  align: 'right',
  layout: 'vertical',
  verticalAlign: 'middle',
  itemStyle: { color: textColor() },
})

/** Beyond this many years in one catalog, the gaps are left out instead: a single mistyped
 * release year would otherwise turn the axis into thousands of empty categories. */
const MAX_FILLED_YEARS = 100

const rangeInclusive = (from: number, to: number): number[] => Array.from({ length: to - from + 1 }, (_, offset) => from + offset)

/** Category/data pair of the release-year charts, the years without any filled in with a zero.
 * A `maxFilledYears` cap leaves the gaps out beyond that span, e.g. thousands of years caused by
 * one mistyped release year. */
const releaseYearCategories = (dataPoints: TitleGroupsPerReleaseYearDataPoint[], maxFilledYears?: number): { categories: string[]; data: number[] } | null => {
  const countsByYear = new Map<number, number>()
  for (const dataPoint of dataPoints) {
    if (dataPoint.year != null) countsByYear.set(dataPoint.year, dataPoint.count)
  }
  // without a single dated title group, there is no axis to draw
  if (countsByYear.size === 0) return null

  const years = [...countsByYear.keys()].sort((first, second) => first - second)
  const yearsToPlot =
    maxFilledYears === undefined || years[years.length - 1] - years[0] <= maxFilledYears ? rangeInclusive(years[0], years[years.length - 1]) : years
  return { categories: yearsToPlot.map(String), data: yearsToPlot.map((year) => countsByYear.get(year) ?? 0) }
}

const releaseYearTooltip = (labels: { titleGroups: string; count: string }) =>
  function (this: Highcharts.Point) {
    return `<b>${this.category}</b><br/>${labels.count}: ${formatNumber(this.y ?? 0)}`
  }

/** Area chart of how many title groups were released each year, filling the years without any. */
export const titleGroupsPerReleaseYearChartOptions = (
  dataPoints: TitleGroupsPerReleaseYearDataPoint[],
  labels: { titleGroups: string; count: string },
): Highcharts.Options | null => {
  const { categories, data } = releaseYearCategories(dataPoints, MAX_FILLED_YEARS) ?? {}
  if (!categories || !data) return null

  return {
    ...baseChartOptions,
    chart: { ...baseChartOptions.chart, type: 'area' },
    xAxis: {
      categories,
      labels: { style: { color: textColor() } },
    },
    yAxis: {
      title: { text: undefined },
      labels: { style: { color: textColor() } },
    },
    series: [
      {
        type: 'area',
        name: labels.titleGroups,
        data,
        color: CHART_COLORS[0],
        marker: { enabled: false, states: { hover: { enabled: true, radius: 5 } } },
      },
    ],
    tooltip: { formatter: releaseYearTooltip(labels) },
  }
}

/** Column chart of how many title groups were released each year for the narrow catalog sidebars,
 * every year of the range shown even when a year has no release. */
export const titleGroupsPerReleaseYearBarChartOptions = (
  dataPoints: TitleGroupsPerReleaseYearDataPoint[],
  labels: { titleGroups: string; count: string },
): Highcharts.Options | null => {
  const { categories, data } = releaseYearCategories(dataPoints) ?? {}
  if (!categories || !data) return null

  return {
    ...baseChartOptions,
    chart: { ...baseChartOptions.chart, type: 'column' },
    xAxis: {
      categories,
      labels: { style: { color: textColor() } },
    },
    yAxis: {
      min: 0,
      allowDecimals: false,
      title: { text: undefined },
      labels: { style: { color: textColor() } },
    },
    series: [{ type: 'column', name: labels.titleGroups, data, color: primaryColor() }],
    tooltip: { formatter: releaseYearTooltip(labels) },
  }
}

/** A slice of a pie chart of an attribute, such as a content type or a source. */
export interface AttributePieSlice {
  name: string
  y: number
  /** Value of the extra measure of the slice, when the labels ask for one. */
  extraValue?: number
  /** Color overriding the by-index palette. */
  color?: string
}

export interface AttributePieLabels {
  /** Label of the count of the slices. */
  count: string
  /** Extra measure shown under the count in the tooltip, such as the total size of a torrent group. */
  extra?: { label: string; format: (value: number) => string }
}

const attributePieTooltip = (labels: AttributePieLabels) =>
  function (this: Highcharts.Point) {
    const point = this as unknown as AttributePieSlice
    const extra = labels.extra && point.extraValue !== undefined ? `<br/>${labels.extra.label}: ${labels.extra.format(point.extraValue)}` : ''
    return `<b>${point.name}</b><br/>${labels.count}: ${formatNumber(point.y ?? 0)}${extra}`
  }

/** Shared skeleton of the pie charts of an attribute. The slices are colored by index, so a caller
 * listing the attributes in a legend of its own can pick the same colors. */
const attributePieChart = (
  slices: AttributePieSlice[],
  labels: AttributePieLabels,
  dataLabels: Highcharts.SeriesPieOptions['dataLabels'],
  legend: Highcharts.LegendOptions | undefined,
): Highcharts.Options => ({
  ...baseChartOptions,
  ...(legend ? { legend } : {}),
  chart: { ...baseChartOptions.chart, type: 'pie' },
  plotOptions: {
    pie: {
      showInLegend: true,
    },
  },
  series: [
    {
      type: 'pie',
      data: slices.map((slice, index) => ({ ...slice, color: slice.color ?? CHART_COLORS[index % CHART_COLORS.length] })),
      dataLabels,
    },
  ],
  tooltip: { formatter: attributePieTooltip(labels) },
})

/** Pie chart of how many title groups fall into each value of an attribute, for the catalog
 * sidebars, which are narrow enough that the legend has to sit next to the pie. */
export const attributePieChartOptions = (dataPoints: TitleGroupAttributeCountDataPoint[], labels: AttributePieLabels): Highcharts.Options | null => {
  if (dataPoints.length === 0) return null

  const slices: AttributePieSlice[] = dataPoints.map((dataPoint) => ({ name: dataPoint.attribute_value, y: dataPoint.count }))
  const biggestSlice = slices.reduce((biggest, slice) => (slice.y > biggest.y ? slice : biggest), slices[0])
  biggestSlice.color = primaryColor()

  return attributePieChart(
    slices,
    labels,
    { enabled: true, format: '{point.percentage:.1f}%', style: { color: textColor(), textOutline: 'none', fontSize: '10px' } },
    sideLegend(),
  )
}

/** Pie chart of the slices of an attribute for the stats pages, which already list the attributes
 * in a legend above the chart and so name the slices instead of labelling their share. */
export const groupedAttributePieChartOptions = (slices: AttributePieSlice[], labels: AttributePieLabels): Highcharts.Options =>
  attributePieChart(
    slices,
    labels,
    {
      enabled: true,
      format: '{point.name}',
      connectorColor: textColor(),
      style: { color: textColor(), textOutline: 'none', fontSize: '11px' },
      distance: 25,
    },
    undefined,
  )
