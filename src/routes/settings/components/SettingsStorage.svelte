<script>
  import { createEventDispatcher } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { ask, open as openDialog } from '@tauri-apps/plugin-dialog';
  import { cache } from '../../../lib/stores/cache.js';
  import { showToast } from '$lib/stores/toast.js';
  import { t } from '$lib/i18n/index.js';
  
  export let config;
  export let storageStats = null;
  export let dataDir = '';
  export let defaultDataDir = '';
  
  const dispatch = createEventDispatcher();
  let isClearing = false;
  let isMigrating = false;
  let isCleaningPreviousDir = false;
  let cleanupCandidateDir = '';
  const screenshotModes = [
    {
      value: 'active_window',
      label: $t('settings.storage.activeWindowScreen'),
      description: $t('settings.storage.activeWindowScreenDesc'),
    },
    {
      value: 'all',
      label: $t('settings.storage.fullDesktop'),
      description: $t('settings.storage.fullDesktopDesc'),
    },
  ];

  function clearCache() {
    cache.clear();
    showToast($t('settings.storage.cacheCleared'));
    dispatch('clearCache');
  }

  async function clearOldData() {
    const confirmed = await ask($t('settings.storage.confirmDeleteAll'), {
      title: $t('settings.storage.confirmDeleteTitle'),
      kind: 'warning',
    });

    if (!confirmed) {
      return;
    }
    
    isClearing = true;
    try {
      const result = await invoke('clear_old_activities');
      showToast(result?.message || $t('settings.storage.cleanupDone'));
      cache.clear();
      dispatch('clearCache');
    } catch (e) {
      showToast($t('settings.storage.cleanupFailed') + ': ' + e, 'error');
    } finally {
      isClearing = false;
    }
  }

  async function migrateToDataDir(targetDir) {
    const nextDir = targetDir?.trim();
    if (!nextDir) {
      return;
    }

    if (nextDir === dataDir) {
      showToast($t('settings.storage.alreadyCurrent'));
      return;
    }

    const confirmed = await ask(
      $t('settings.storage.confirmMigrateMsg', { dir: nextDir }),
      {
        title: $t('settings.storage.confirmMigrateTitle'),
        kind: 'warning',
      },
    );

    if (!confirmed) {
      return;
    }

    isMigrating = true;
    try {
      const result = await invoke('change_data_dir', { targetDir: nextDir });
      cleanupCandidateDir = result?.oldDataDir || dataDir;
      showToast(result?.message || $t('settings.storage.dataDirUpdated'), 'success');
      dispatch('dataDirChanged', result);
    } catch (e) {
      showToast($t('settings.storage.migrateFailed') + ': ' + e, 'error');
    } finally {
      isMigrating = false;
    }
  }

  async function pickDataDir() {
    const selected = await openDialog({
      directory: true,
      multiple: false,
      defaultPath: dataDir || defaultDataDir || undefined,
    });

    if (!selected || Array.isArray(selected)) {
      return;
    }

    await migrateToDataDir(selected);
  }

  async function restoreDefaultDataDir() {
    await migrateToDataDir(defaultDataDir);
  }

  async function openCurrentDataDir() {
    try {
      await invoke('open_data_dir');
    } catch (e) {
      showToast($t('settings.storage.openFailed') + ': ' + e, 'error');
    }
  }

  async function cleanupPreviousDataDir() {
    const targetDir = cleanupCandidateDir?.trim();
    if (!targetDir || isCleaningPreviousDir) {
      return;
    }

    const confirmed = await ask(
      $t('settings.storage.confirmCleanOldDir', { dir: targetDir }),
      {
        title: $t('settings.storage.confirmCleanOldDirTitle'),
        kind: 'warning',
      },
    );

    if (!confirmed) {
      return;
    }

    isCleaningPreviousDir = true;
    try {
      const result = await invoke('cleanup_old_data_dir', { targetDir });
      cleanupCandidateDir = '';
      showToast(result?.message || $t('settings.storage.oldDirCleaned'), 'success');
    } catch (e) {
      showToast($t('settings.storage.cleanOldDirFailed') + ': ' + e, 'error');
    } finally {
      isCleaningPreviousDir = false;
    }
  }

  function handleChange() {
    dispatch('change', config);
  }

  async function pickDailyReportExportDir() {
    const selected = await openDialog({
      directory: true,
      multiple: false,
      defaultPath: config.daily_report_export_dir || dataDir || defaultDataDir || undefined,
    });

    if (!selected || Array.isArray(selected)) {
      return;
    }

    config.daily_report_export_dir = selected;
    handleChange();
  }

  function clearDailyReportExportDir() {
    config.daily_report_export_dir = null;
    handleChange();
  }

  // 计算存储使用百分比
  $: usagePercent = storageStats 
    ? Math.min(Math.round((storageStats.total_size_mb / storageStats.storage_limit_mb) * 100), 100) 
    : 0;

  // 使用量颜色
  $: usageColor = usagePercent > 80 ? 'bg-red-500' : usagePercent > 50 ? 'bg-amber-500' : 'bg-emerald-500';
  $: usingDefaultDataDir = dataDir && defaultDataDir && dataDir === defaultDataDir;
  $: if (cleanupCandidateDir && cleanupCandidateDir === dataDir) {
    cleanupCandidateDir = '';
  }
</script>

<!-- 截图与保留 -->
<div class="settings-card mb-5">
  <h3 class="settings-card-title">{$t('settings.storage.screenshotAndRetention')}</h3>
  <p class="settings-card-desc">{$t('settings.storage.screenshotAndRetentionDesc')}</p>
  
  <div class="settings-section">
    <!-- 轮询间隔 -->
    <div class="settings-block">
      <div class="flex items-center justify-between">
        <label for="screenshot-interval" class="settings-text">{$t('settings.storage.pollInterval')}</label>
        <span class="settings-value">{config.screenshot_interval}{$t('settings.storage.secondsUnit2')}</span>
      </div>
      <input
        id="screenshot-interval"
        type="range"
        bind:value={config.screenshot_interval}
        on:change={handleChange}
        min="10"
        max="120"
        step="5"
        class="range-input"
      />
      <div class="flex justify-between text-xs settings-subtle">
        <span>{$t('settings.storage.pollAccurate')}</span>
        <span>{$t('settings.storage.pollEfficient')}</span>
      </div>
      <p class="settings-note">{$t('settings.storage.pollHint')}</p>
    </div>

    <!-- 数据保留 -->
    <div class="settings-block">
      <div class="flex items-center justify-between">
        <label for="retention-days" class="settings-text">{$t('settings.storage.retentionDays')}</label>
        <span class="settings-value">{config.storage.screenshot_retention_days}{$t('settings.storage.daysUnit')}</span>
      </div>
      <input
        id="retention-days"
        type="range"
        bind:value={config.storage.screenshot_retention_days}
        on:change={() => {
          config.storage.metadata_retention_days = config.storage.screenshot_retention_days;
          handleChange();
        }}
        min="1"
        max="90"
        step="1"
        class="range-input"
      />
      <div class="flex justify-between text-xs settings-subtle">
        <span>{$t('settings.storage.oneDayLabel')}</span>
        <span>{$t('settings.storage.ninetyDayLabel')}</span>
      </div>
      <p class="settings-note">{$t('settings.storage.retentionHint')}</p>
    </div>

    <div class="settings-block">
      <p class="settings-text mb-2">{$t('settings.storage.screenshotRange')}</p>
      <div class="flex gap-2">
        {#each screenshotModes as mode}
          <button
            type="button"
            on:click={() => {
              config.storage.screenshot_display_mode = mode.value;
              handleChange();
            }}
            class="flex-1 min-h-16 px-3 py-2.5 rounded-lg text-sm font-medium leading-none transition-all duration-150
                   {config.storage.screenshot_display_mode === mode.value
                     ? 'settings-segment-active'
                     : 'settings-segment-base'}"
          >
            <div class="flex h-full flex-col items-center justify-center gap-1 text-center">
              <div class="leading-none">{mode.label}</div>
              <div class="text-[10px] leading-snug {config.storage.screenshot_display_mode === mode.value ? 'text-white/70' : 'settings-subtle'}">
                {mode.description}
              </div>
            </div>
          </button>
        {/each}
      </div>
      <p class="settings-note">
        {$t('settings.storage.screenshotRangeHint')}
      </p>
    </div>
  </div>
</div>

<!-- 日报导出 -->
<div class="settings-card mb-5">
  <h3 class="settings-card-title">{$t('settings.storage.reportExport')}</h3>
  <p class="settings-card-desc">{$t('settings.storage.reportExportDesc')}</p>

  <div class="settings-block">
    <div class="rounded-2xl border border-slate-200/80 bg-slate-50/90 p-4 dark:border-slate-700/80 dark:bg-slate-800/40">
      <p class="settings-text">{$t('settings.storage.reportExportDir')}</p>
      <p class="settings-muted mt-1 break-all">
        {config.daily_report_export_dir || $t('settings.storage.notSet')}
      </p>
      <p class="settings-note mt-3">{$t('settings.storage.reportExportHint')}</p>
      <div class="mt-4 flex flex-wrap gap-3">
        <button
          type="button"
          on:click={pickDailyReportExportDir}
          class="settings-action-secondary"
        >
          {$t('settings.storage.selectDir')}
        </button>
        {#if config.daily_report_export_dir}
          <button
            type="button"
            on:click={clearDailyReportExportDir}
            class="settings-action-secondary"
          >
            {$t('settings.storage.clearDir')}
          </button>
        {/if}
      </div>
    </div>
  </div>
</div>

<div class="settings-card mb-5">
  <h3 class="settings-card-title">{$t('settings.storage.dataDirAndCleanup')}</h3>
  <p class="settings-card-desc">{$t('settings.storage.dataDirAndCleanupDesc')}</p>

  <div class="settings-section">
    <div class="settings-block">
      <div class="rounded-2xl border border-slate-200/80 bg-slate-50/90 p-4 dark:border-slate-700/80 dark:bg-slate-800/40">
        <div class="grid gap-4 md:grid-cols-2">
          <div>
            <p class="settings-text">{$t('settings.storage.currentDir')}</p>
            <p class="settings-muted mt-1 break-all">{dataDir || $t('settings.storage.reading')}</p>
          </div>
          <div>
            <p class="settings-text">{$t('settings.storage.defaultDir')}</p>
            <p class="settings-muted mt-1 break-all">{defaultDataDir || $t('settings.storage.reading')}</p>
          </div>
        </div>

        <div class="mt-4 flex flex-wrap gap-3">
          <button
            on:click={pickDataDir}
            disabled={isMigrating}
            class="settings-action-secondary"
          >
            {#if isMigrating}
              {$t('settings.storage.migrating')}
            {:else}
              {$t('settings.storage.changeLocation')}
            {/if}
          </button>

          <button
            on:click={openCurrentDataDir}
            disabled={isMigrating}
            class="settings-action-secondary"
          >
            {$t('settings.storage.openDir')}
          </button>

          {#if !usingDefaultDataDir && defaultDataDir}
            <button
              on:click={restoreDefaultDataDir}
              disabled={isMigrating}
              class="settings-action-secondary"
            >
              {$t('settings.storage.restoreDefault')}
            </button>
          {/if}
        </div>

        <p class="settings-note mt-3">
          {$t('settings.storage.migrationHint')}
        </p>

        {#if cleanupCandidateDir}
          <div class="mt-4 rounded-xl border border-amber-200/70 bg-amber-50/90 p-3 dark:border-amber-500/30 dark:bg-amber-950/20">
            <p class="settings-text">{$t('settings.storage.oldDirPending')}</p>
            <p class="settings-muted mt-1 break-all">{cleanupCandidateDir}</p>
            <p class="settings-note mt-2">
              {$t('settings.storage.oldDirHint')}
            </p>
            <div class="mt-3 flex flex-wrap gap-3">
              <button
                on:click={cleanupPreviousDataDir}
                disabled={isCleaningPreviousDir || isMigrating}
                class="settings-action-secondary"
              >
                {#if isCleaningPreviousDir}
                  {$t('settings.storage.clearing')}
                {:else}
                  {$t('settings.storage.cleanOldDir')}
                {/if}
              </button>
              <button
                on:click={() => cleanupCandidateDir = ''}
                disabled={isCleaningPreviousDir}
                class="settings-action-secondary"
              >
                {$t('settings.storage.laterCleanup')}
              </button>
            </div>
          </div>
        {/if}
      </div>
    </div>

    {#if storageStats}
      <div class="settings-block">
        <div class="rounded-2xl border border-slate-200/80 bg-slate-50/90 p-4 dark:border-slate-700/80 dark:bg-slate-800/40">
          <div class="mb-5">
            <div class="mb-2 flex items-end justify-between">
              <div>
                <span class="text-2xl font-bold text-slate-800 dark:text-white">{storageStats.total_size_mb}</span>
                <span class="settings-muted"> / {storageStats.storage_limit_mb} MB</span>
              </div>
              <span class="text-sm font-medium {usagePercent > 80 ? 'settings-text-danger' : 'settings-muted'}">{usagePercent}%</span>
            </div>
            <div class="h-2.5 w-full overflow-hidden rounded-full bg-slate-100 dark:bg-slate-700">
              <div
                class="h-full rounded-full transition-all duration-500 {usageColor}"
                style="width: {usagePercent}%"
              ></div>
            </div>
          </div>

          <div class="grid grid-cols-3 gap-3">
            <div class="rounded-xl bg-white/70 p-3 text-center ring-1 ring-slate-200/70 dark:bg-slate-900/20 dark:ring-slate-700/70">
              <p class="text-xl font-bold text-slate-800 dark:text-white">{storageStats.total_files}</p>
              <p class="settings-muted mt-0.5">{$t('settings.storage.screenshots')}</p>
            </div>
            <div class="rounded-xl bg-white/70 p-3 text-center ring-1 ring-slate-200/70 dark:bg-slate-900/20 dark:ring-slate-700/70">
              <p class="text-xl font-bold text-slate-800 dark:text-white">{storageStats.total_size_mb} MB</p>
              <p class="settings-muted mt-0.5">{$t('settings.storage.usedSpace')}</p>
            </div>
            <div class="rounded-xl bg-white/70 p-3 text-center ring-1 ring-slate-200/70 dark:bg-slate-900/20 dark:ring-slate-700/70">
              <p class="text-xl font-bold text-slate-800 dark:text-white">{storageStats.retention_days} {$t('settings.storage.daysUnit')}</p>
              <p class="settings-muted mt-0.5">{$t('settings.storage.retention')}</p>
            </div>
          </div>
        </div>
      </div>
    {/if}

    <div class="settings-block">
      <div class="flex items-center justify-between rounded-xl bg-slate-50 p-3 dark:bg-slate-700/30">
        <div>
          <p class="settings-text">{$t('settings.storage.clearPageCache')}</p>
          <p class="settings-muted mt-0.5">{$t('settings.storage.storageFixHint')}</p>
        </div>
        <button
          on:click={clearCache}
          class="settings-action-secondary"
        >
          {$t('settings.storage.clearCacheBtn')}
        </button>
      </div>

      <div class="settings-panel-danger flex items-center justify-between">
        <div>
          <p class="settings-text-danger text-sm font-medium">{$t('settings.storage.clearHistory')}</p>
          <p class="settings-muted mt-0.5">{$t('settings.storage.storageDeleteHint')}</p>
        </div>
        <button
          on:click={clearOldData}
          disabled={isClearing}
          class="settings-action-danger"
        >
          {#if isClearing}
            {$t('settings.storage.clearing')}
          {:else}
            {$t('settings.storage.clearHistoryBtn')}
          {/if}
        </button>
      </div>
    </div>
  </div>
</div>
