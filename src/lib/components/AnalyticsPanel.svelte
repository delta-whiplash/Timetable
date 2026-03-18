<script lang="ts">
  import { onMount, tick } from "svelte";
  import { appStore } from "$lib/stores/app";
  import type { AnalyticsDataView } from "$lib/types";
  import ApexCharts from "apexcharts";

  let analytics = $state<AnalyticsDataView | null>(null);
  let loading = $state(true);

  let dayChartEl: HTMLDivElement;
  let trendChartEl: HTMLDivElement;
  let dayChart: ApexCharts | null = null;
  let trendChart: ApexCharts | null = null;

  onMount(() => {
    void loadAnalytics();

    return () => {
      dayChart?.destroy();
      trendChart?.destroy();
    };
  });

  async function loadAnalytics() {
    loading = true;
    await appStore.loadAnalytics();
  }

  // Subscribe to store updates
  $effect(() => {
    const state = $appStore;
    analytics = state.analytics;
    loading = state.loading;
  });

  // Action for chart element binding
  function chartNode(node: HTMLDivElement, type: 'day' | 'trend') {
    if (type === 'day') {
      dayChartEl = node;
    } else {
      trendChartEl = node;
    }
    // Try to initialize charts when both elements are ready
    if (dayChartEl && trendChartEl && analytics && analytics.dayOfWeekStats.length > 0) {
      tick().then(() => {
        requestAnimationFrame(() => initCharts());
      });
    }
  }

  function initCharts() {
    if (!analytics || !dayChartEl || !trendChartEl) return;

    // Destroy existing charts
    if (dayChart) {
      dayChart.destroy();
      dayChart = null;
    }
    if (trendChart) {
      trendChart.destroy();
      trendChart = null;
    }

    // Day of Week Chart
    const dayOptions = {
      chart: {
        type: "bar" as const,
        height: 300,
        fontFamily: "Aptos, Bahnschrift, Segoe UI Variable, sans-serif",
        toolbar: { show: false },
        background: "transparent"
      },
      plotOptions: {
        bar: {
          borderRadius: 8,
          columnWidth: "60%"
        }
      },
      dataLabels: { enabled: false },
      stroke: { show: true, width: 2, colors: ["transparent"] },
      xaxis: {
        categories: analytics.dayOfWeekStats.map((d) => d.dayName),
        labels: { style: { colors: "#96a5bc" } }
      },
      yaxis: {
        labels: {
          style: { colors: "#96a5bc" },
          formatter: (val: number) => `${Math.round(val / 60)}h`
        }
      },
      fill: {
        opacity: 1,
        colors: ["#82b0ff"]
      },
      tooltip: {
        y: {
          formatter: (val: number) => {
            const hours = Math.floor(val / 60);
            const mins = val % 60;
            return `${hours}h ${mins.toString().padStart(2, "0")}min`;
          }
        }
      },
      theme: { mode: "dark" as const },
      series: [{
        name: "Heures moyennes",
        data: analytics.dayOfWeekStats.map((d) => d.averageMinutes)
      }]
    };

    dayChart = new ApexCharts(dayChartEl, dayOptions);
    dayChart.render();

    // Weekly Trend Chart
    const trendOptions = {
      chart: {
        type: "area" as const,
        height: 300,
        fontFamily: "Aptos, Bahnschrift, Segoe UI Variable, sans-serif",
        toolbar: { show: false },
        background: "transparent"
      },
      dataLabels: { enabled: false },
      stroke: {
        curve: "smooth" as const,
        width: 2
      },
      xaxis: {
        categories: analytics.weeklyTrends.map((d) => {
          const date = new Date(d.weekStart);
          return date.toLocaleDateString("fr-FR", { day: "2-digit", month: "short" });
        }),
        labels: { style: { colors: "#96a5bc" } }
      },
      yaxis: {
        labels: {
          style: { colors: "#96a5bc" },
          formatter: (val: number) => `${Math.round(val / 60)}h`
        }
      },
      fill: {
        type: "gradient" as const,
        gradient: {
          shadeIntensity: 1,
          opacityFrom: 0.4,
          opacityTo: 0.05,
          stops: [0, 90, 100]
        }
      },
      tooltip: {
        y: {
          formatter: (val: number) => {
            const hours = Math.floor(val / 60);
            const mins = val % 60;
            return `${hours}h ${mins.toString().padStart(2, "0")}min`;
          }
        }
      },
      colors: ["#82b0ff"],
      theme: { mode: "dark" as const },
      series: [{
        name: "Heures travaillées",
        data: analytics.weeklyTrends.map((d) => d.totalMinutes)
      }]
    };

    trendChart = new ApexCharts(trendChartEl, trendOptions);
    trendChart.render();
  }
</script>

<div class="analytics-panel">
  <div class="analytics-header">
    <h2>Analytiques</h2>
    <span class="analytics-meta">{analytics?.totalWeeks || 0} semaines de données</span>
  </div>

  {#if loading}
    <div class="analytics-loading">Chargement des analytiques...</div>
  {:else if analytics}
    <div class="analytics-grid">
      <!-- Day of Week Chart -->
      <div class="analytics-card">
        <h3>Moyenne par jour de la semaine</h3>
        <div use:chartNode={'day'}></div>
      </div>

      <!-- Weekly Trend Chart -->
      <div class="analytics-card">
        <h3>Tendance hebdomadaire (12 dernières semaines)</h3>
        <div use:chartNode={'trend'}></div>
      </div>
    </div>

    <!-- Monthly Stats -->
    <div class="analytics-card">
      <h3>Statistiques mensuelles</h3>
      <div class="monthly-stats-grid">
        {#each analytics.monthlyStats as month}
          <div class="monthly-stat-item">
            <span class="monthly-stat-month">{month.month}</span>
            <span class="monthly-stat-value">{month.totalLabel}</span>
            <span class="monthly-stat-avg">Moy. {month.weeklyAverageLabel}/sem</span>
          </div>
        {/each}
      </div>
    </div>
  {:else}
    <div class="analytics-empty">Aucune donnée analytique disponible</div>
  {/if}
</div>

<style>
  .analytics-panel {
    padding: var(--space-lg);
  }

  .analytics-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: var(--space-lg);
  }

  .analytics-header h2 {
    margin: 0;
    font-size: 1.5rem;
    letter-spacing: -0.04em;
  }

  .analytics-meta {
    color: var(--color-text-muted);
    font-size: 0.85rem;
  }

  .analytics-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(400px, 1fr));
    gap: var(--space-md);
    margin-bottom: var(--space-md);
  }

  .analytics-card {
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-lg);
    padding: var(--space-md);
  }

  .analytics-card h3 {
    margin: 0 0 var(--space-md);
    font-size: 1rem;
    color: var(--color-text-muted);
    font-weight: 600;
  }

  .analytics-loading,
  .analytics-empty {
    text-align: center;
    padding: 40px;
    color: var(--color-text-muted);
  }

  .monthly-stats-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(150px, 1fr));
    gap: 12px;
  }

  .monthly-stat-item {
    background: var(--color-bg-alt);
    border-radius: var(--radius-sm);
    padding: 12px;
    text-align: center;
  }

  .monthly-stat-month {
    display: block;
    font-size: 0.75rem;
    color: var(--color-text-muted);
    text-transform: uppercase;
    letter-spacing: 0.05em;
    margin-bottom: 4px;
  }

  .monthly-stat-value {
    display: block;
    font-size: 1.2rem;
    font-weight: 700;
    color: var(--color-primary);
  }

  .monthly-stat-avg {
    display: block;
    font-size: 0.75rem;
    color: var(--color-text-muted);
    margin-top: 2px;
  }
</style>
