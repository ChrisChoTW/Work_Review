<script>
  import { createEventDispatcher, onDestroy, onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { showToast } from '$lib/stores/toast.js';
  import { t, locale, setLocale, SUPPORTED_LOCALES } from '$lib/i18n/index.js';
  import {
    AVATAR_OPACITY_DEFAULT,
    AVATAR_SCALE_DEFAULT,
    clampAvatarOpacity,
    clampAvatarScale,
    formatAvatarOpacityLabel,
    formatAvatarScaleLabel,
    getAvatarToggleToast,
    getAvatarToggleUiState,
    toggleAvatarSetting,
    updateAvatarOpacitySetting,
    updateAvatarScaleSetting,
  } from '$lib/utils/avatarToggle.js';

  export let config;

  const dispatch = createEventDispatcher();

  let avatarSaving = false;
  let avatarScaleSaving = false;
  let avatarOpacitySaving = false;
  let avatarScaleTimer = null;
  let avatarOpacityTimer = null;

  // === 背景图片 ===
  let bgPreview = null;
  let bgUploading = false;
  let appearanceDestroyed = false;

  $: blurLabels = $t('settings.appearance.blurLabels');
  $: avatarToggleUi = getAvatarToggleUiState(Boolean(config.avatar_enabled), avatarSaving);
  $: avatarScale = clampAvatarScale(config.avatar_scale ?? AVATAR_SCALE_DEFAULT);
  $: avatarScaleLabel = formatAvatarScaleLabel(avatarScale);
  $: avatarOpacity = clampAvatarOpacity(config.avatar_opacity ?? AVATAR_OPACITY_DEFAULT);
  $: avatarOpacityLabel = formatAvatarOpacityLabel(avatarOpacity);

  onMount(async () => {
    try {
      const b64 = await invoke('get_background_image');
      if (b64) bgPreview = `data:image/jpeg;base64,${b64}`;
    } catch (e) { /* ignore */ }
  });

  onDestroy(() => {
    appearanceDestroyed = true;
    clearTimeout(avatarScaleTimer);
    clearTimeout(avatarOpacityTimer);
  });

  async function toggleAvatarMode() {
    if (avatarSaving) {
      return;
    }

    avatarSaving = true;

    try {
      const enabled = await toggleAvatarSetting(config, async (nextConfig) => {
        await invoke('save_config', { config: nextConfig });
      });

      dispatch('change', config);
      showToast(getAvatarToggleToast(enabled, $t), enabled ? 'success' : 'info');
    } catch (e) {
      console.error('设置桌宠失败:', e);
      showToast(`${$t('settings.appearance.avatarFailed')}: ${e}`, 'error');
    } finally {
      avatarSaving = false;
    }
  }

  function queueAvatarScaleSave(nextScale) {
    clearTimeout(avatarScaleTimer);
    avatarScaleTimer = setTimeout(async () => {
      avatarScaleSaving = true;

      try {
        const savedScale = await updateAvatarScaleSetting(config, nextScale, async (nextConfig) => {
          await invoke('save_config', { config: nextConfig });
        });
        config.avatar_scale = savedScale;
        dispatch('change', config);
      } catch (e) {
        console.error('保存桌宠缩放失败:', e);
        showToast(`${$t('settings.appearance.avatarScaleFailed')}: ${e}`, 'error');
      } finally {
        avatarScaleSaving = false;
      }
    }, 120);
  }

  function handleAvatarScaleInput(event) {
    const nextScale = clampAvatarScale(Number(event.currentTarget.value));
    config.avatar_scale = nextScale;
    dispatch('change', config);
    queueAvatarScaleSave(nextScale);
  }

  function queueAvatarOpacitySave(nextOpacity) {
    clearTimeout(avatarOpacityTimer);
    avatarOpacityTimer = setTimeout(async () => {
      avatarOpacitySaving = true;

      try {
        const savedOpacity = await updateAvatarOpacitySetting(
          config,
          nextOpacity,
          async (nextConfig) => {
            await invoke('save_config', { config: nextConfig });
          }
        );
        config.avatar_opacity = savedOpacity;
        dispatch('change', config);
      } catch (e) {
        console.error('保存桌宠透明度失败:', e);
        showToast(`${$t('settings.appearance.avatarOpacityFailed')}: ${e}`, 'error');
      } finally {
        avatarOpacitySaving = false;
      }
    }, 120);
  }

  function handleAvatarOpacityInput(event) {
    const nextOpacity = clampAvatarOpacity(Number(event.currentTarget.value));
    config.avatar_opacity = nextOpacity;
    dispatch('change', config);
    queueAvatarOpacitySave(nextOpacity);
  }

  function handleBgFileSelect(event) {
    const file = event.target.files?.[0];
    if (!file) return;
    if (!file.type.startsWith('image/')) return;
    if (file.size > 10 * 1024 * 1024) {
      showToast($t('settings.appearance.fileTooLarge'), 'warning');
      return;
    }

    bgUploading = true;
    const reader = new FileReader();
    reader.onload = async () => {
      if (appearanceDestroyed) return;

      try {
        const b64Data = typeof reader.result === 'string' ? reader.result.split(',')[1] : null;
        if (!b64Data) {
          throw new Error($t('settings.appearance.readFailed'));
        }
        await invoke('save_background_image', { data: b64Data });
        if (appearanceDestroyed) return;
        config.background_image = 'background.jpg';
        const freshB64 = await invoke('get_background_image');
        if (appearanceDestroyed) return;
        const imageUrl = freshB64 ? `data:image/jpeg;base64,${freshB64}` : null;
        bgPreview = imageUrl;
        dispatchBgEvent(imageUrl);
      } catch (e) {
        if (appearanceDestroyed) return;
        console.error('上传背景图失败:', e);
        showToast($t('settings.appearance.uploadFailed') + ': ' + e, 'error');
      } finally {
        if (!appearanceDestroyed) {
          bgUploading = false;
        }
      }
    };
    reader.readAsDataURL(file);
  }

  async function clearBg() {
    try {
      await invoke('clear_background_image');
      bgPreview = null;
      config.background_image = null;
      dispatchBgEvent(null);
    } catch (e) {
      console.error('清除背景图失败:', e);
      showToast($t('settings.appearance.clearFailed') + ': ' + e, 'error');
    }
  }

  function updateBgOpacity(val) {
    config.background_opacity = parseFloat(val);
    dispatch('change', config);
    dispatchBgEvent(bgPreview);
  }

  function updateBgBlur(val) {
    config.background_blur = parseInt(val);
    dispatch('change', config);
    dispatchBgEvent(bgPreview);
  }

  function dispatchBgEvent(image) {
    window.dispatchEvent(new CustomEvent('background-changed', {
      detail: {
        image,
        opacity: config.background_opacity ?? 0.25,
        blur: config.background_blur ?? 1,
      }
    }));
  }
</script>

<!-- 语言设置 -->
<div class="settings-card">
  <h3 class="settings-card-title">Language / 语言</h3>
  <div class="settings-section">
    <div class="flex gap-2">
      {#each SUPPORTED_LOCALES as loc}
        <button
          on:click={() => {
            setLocale(loc.code);
            config.locale = loc.code;
            dispatch('change', config);
          }}
          class="segment-btn {$locale === loc.code ? 'settings-segment-active' : 'settings-segment-base'}"
        >
          {loc.label}
        </button>
      {/each}
    </div>
  </div>
</div>

<div class="settings-card">
  <div class="settings-section">
    <div class="flex items-center justify-between gap-4">
      <div>
        <div class="flex items-center gap-2">
          <div class="settings-text">{$t("settings.appearance.avatar")}</div>
          <span class="rounded-full border border-amber-200 bg-amber-50 px-2 py-0.5 text-[10px] font-semibold uppercase tracking-[0.08em] text-amber-700 dark:border-amber-400/30 dark:bg-amber-500/10 dark:text-amber-200">
            Beta
          </span>
        </div>
        <div class="settings-muted mt-0.5">{$t("settings.appearance.avatarDesc")}</div>
        <div class="settings-muted mt-1 text-[12px]">{$t("settings.appearance.avatarExperimental")}</div>
      </div>
      <button
        type="button"
        on:click={toggleAvatarMode}
        class="switch-track {avatarToggleUi.trackClass} {avatarToggleUi.buttonClass}"
        disabled={avatarSaving}
        aria-label={avatarToggleUi.ariaLabel}
        aria-pressed={config.avatar_enabled}
      >
        <span class="switch-thumb {avatarToggleUi.thumbClass}"></span>
      </button>
    </div>

    <hr class="border-slate-200 dark:border-slate-700" />

    <div class="settings-block pt-1">
      <div class="flex items-center justify-between gap-3">
        <div>
          <div class="settings-text">{$t("settings.appearance.avatarSize")}</div>
          <div class="settings-muted mt-0.5">{$t("settings.appearance.avatarSizeDesc")}</div>
        </div>
        <div class="text-sm font-semibold text-slate-700 dark:text-slate-200">
          {avatarScaleLabel}
          {#if avatarScaleSaving}
            <span class="ml-2 text-xs font-normal text-slate-400 dark:text-slate-500">{$t("settings.appearance.syncing")}</span>
          {/if}
        </div>
      </div>

      <input
        type="range"
        min="0.7"
        max="1.3"
        step="0.05"
        value={avatarScale}
        on:input={handleAvatarScaleInput}
        class="mt-3 w-full accent-primary-500"
        aria-label={$t("settings.appearance.avatarSizeLabel")}
      />
      <div class="mt-2 flex justify-between text-[11px] text-slate-400 dark:text-slate-500">
        <span>{$t("settings.appearance.smaller")}</span>
        <span>90%</span>
        <span>{$t("settings.appearance.bigger")}</span>
      </div>
    </div>

    <div class="settings-block pt-1">
      <div class="flex items-center justify-between gap-3">
        <div>
          <div class="settings-text">{$t("settings.appearance.avatarOpacity")}</div>
          <div class="settings-muted mt-0.5">{$t("settings.appearance.avatarOpacityDesc")}</div>
        </div>
        <div class="text-sm font-semibold text-slate-700 dark:text-slate-200">
          {avatarOpacityLabel}
          {#if avatarOpacitySaving}
            <span class="ml-2 text-xs font-normal text-slate-400 dark:text-slate-500">{$t("settings.appearance.syncing")}</span>
          {/if}
        </div>
      </div>

      <input
        type="range"
        min="0.45"
        max="1"
        step="0.05"
        value={avatarOpacity}
        on:input={handleAvatarOpacityInput}
        class="mt-3 w-full accent-primary-500"
        aria-label={$t("settings.appearance.avatarOpacity")}
      />
      <div class="mt-2 flex justify-between text-[11px] text-slate-400 dark:text-slate-500">
        <span>{$t("settings.appearance.transparent")}</span>
        <span>82%</span>
        <span>{$t("settings.appearance.opaque")}</span>
      </div>
    </div>
  </div>
</div>

<!-- 背景图片 -->
<div class="settings-card">
  <h3 class="settings-card-title">{$t('settings.appearance.backgroundImage')}</h3>
  <p class="settings-card-desc">{$t('settings.appearance.uploadHint')}</p>

  <div class="settings-section">
    <!-- 预览 + 上传 -->
    <div class="flex items-start gap-4">
      {#if bgPreview}
        <div class="w-32 h-20 rounded-lg overflow-hidden border border-slate-200 dark:border-slate-700 flex-shrink-0">
          <img src={bgPreview} alt="background preview" class="w-full h-full object-cover" />
        </div>
      {:else}
        <div class="w-32 h-20 rounded-lg border-2 border-dashed border-slate-200 dark:border-slate-700 flex items-center justify-center flex-shrink-0">
          <span class="settings-subtle">{$t('settings.appearance.noBackground')}</span>
        </div>
      {/if}

      <div class="flex-1 settings-field">
        <label class="settings-action-secondary cursor-pointer">
          {#if bgUploading}
            <div class="animate-spin rounded-full h-3 w-3 border-2 border-slate-500 border-t-transparent"></div>
            {$t('settings.appearance.processing')}
          {:else}
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16l4.586-4.586a2 2 0 012.828 0L16 16m-2-2l1.586-1.586a2 2 0 012.828 0L20 14m-6-6h.01M6 20h12a2 2 0 002-2V6a2 2 0 00-2-2H6a2 2 0 00-2 2v12a2 2 0 002 2z" /></svg>
            {$t('settings.appearance.selectImage')}
          {/if}
          <input type="file" accept="image/*" class="hidden" on:change={handleBgFileSelect} disabled={bgUploading} />
        </label>
        {#if bgPreview}
          <button
            on:click={clearBg}
            class="settings-link-danger"
          >
            {$t('settings.appearance.clearBackground')}
          </button>
        {/if}
        <p class="settings-muted">{$t('settings.appearance.fileSizeLimit')}</p>
      </div>
    </div>

    {#if bgPreview || config.background_image}
      <hr class="border-slate-200 dark:border-slate-700" />

      <!-- 显示强度 -->
      <div class="settings-block">
        <div class="flex items-center justify-between">
          <span class="settings-text">{$t('settings.appearance.displayStrength')}</span>
          <span class="settings-value">{Math.round((config.background_opacity ?? 0.25) * 100)}%</span>
        </div>
        <input
          type="range"
          min="0.05"
          max="0.60"
          step="0.01"
          value={config.background_opacity ?? 0.25}
          on:input={(e) => updateBgOpacity(e.target.value)}
          class="range-input"
        />
        <div class="flex justify-between text-[10px] settings-subtle">
          <span>{$t('settings.appearance.light')}</span>
          <span>{$t('settings.appearance.strong')}</span>
        </div>
      </div>

      <!-- 模糊度 -->
      <div class="settings-block">
        <div class="flex items-center justify-between">
          <span class="settings-text">{$t('settings.appearance.blurLevel')}</span>
          <span class="settings-muted">{blurLabels[config.background_blur ?? 1]}</span>
        </div>
        <div class="flex gap-2">
          {#each [0, 1, 2] as level}
            <button
              on:click={() => updateBgBlur(level)}
              class="segment-btn
                {(config.background_blur ?? 1) === level
                  ? 'settings-segment-active'
                  : 'settings-segment-base'}"
            >
              {blurLabels[level]}
            </button>
          {/each}
        </div>
      </div>
    {/if}
  </div>
</div>
