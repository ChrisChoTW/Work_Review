<script>
  import { createEventDispatcher } from 'svelte';
  import { t } from '$lib/i18n/index.js';
  
  export let config;
  export let runningApps = [];
  export let recentApps = [];
  
  const dispatch = createEventDispatcher();
  
  // 内联输入状态
  let showAppInput = false;
  let selectedApp = '';
  let selectedLevel = 'ignored';
  let showKeywordInput = false;
  let newKeyword = '';
  let showDomainInput = false;
  let newDomain = '';

  // 隐私级别定义 - 使用文字标签避免 emoji 渲染异常
  const privacyLevels = [
    { value: 'full', label: $t('settings.privacy.fullRecord'), textClass: 'settings-text-success', chipClass: 'settings-chip-success', activeClass: 'settings-segment-success', desc: $t('settings.privacy.fullRecordDesc') },
    { value: 'anonymized', label: $t('settings.privacy.durationOnly'), textClass: 'settings-text-warn', chipClass: 'settings-chip-warn', activeClass: 'settings-segment-warn', desc: $t('settings.privacy.durationOnlyDesc') },
    { value: 'ignored', label: $t('settings.privacy.ignore'), textClass: 'settings-text-danger', chipClass: 'settings-chip-danger', activeClass: 'settings-segment-danger', desc: $t('settings.privacy.ignoreDesc') },
  ];

  function addAppRule() {
    if (!selectedApp) return;
    // 检查是否已存在同名规则
    const existingIndex = config.privacy.app_rules.findIndex(r => r.app_name === selectedApp);
    if (existingIndex >= 0) {
      // 更新已有规则
      config.privacy.app_rules[existingIndex].level = selectedLevel;
      config.privacy.app_rules = [...config.privacy.app_rules];
    } else {
      config.privacy.app_rules = [
        ...config.privacy.app_rules,
        { app_name: selectedApp, level: selectedLevel }
      ];
    }
    showAppInput = false;
    selectedApp = '';
    dispatch('change', config);
  }

  function removeAppRule(index) {
    const rules = [...config.privacy.app_rules];
    rules.splice(index, 1);
    config.privacy.app_rules = rules;
    dispatch('change', config);
  }

  function addKeyword() {
    if (!newKeyword.trim()) return;
    // 避免重复添加
    if (config.privacy.excluded_keywords.includes(newKeyword.trim())) {
      newKeyword = '';
      return;
    }
    config.privacy.excluded_keywords = [
      ...config.privacy.excluded_keywords,
      newKeyword.trim()
    ];
    newKeyword = '';
    showKeywordInput = false;
    dispatch('change', config);
  }

  function removeKeyword(index) {
    const keywords = [...config.privacy.excluded_keywords];
    keywords.splice(index, 1);
    config.privacy.excluded_keywords = keywords;
    dispatch('change', config);
  }

  // 域名黑名单管理
  function addDomain() {
    if (!newDomain.trim()) return;
    const domains = config.privacy.excluded_domains || [];
    // 避免重复
    if (domains.includes(newDomain.trim())) {
      newDomain = '';
      return;
    }
    config.privacy.excluded_domains = [...domains, newDomain.trim()];
    newDomain = '';
    showDomainInput = false;
    dispatch('change', config);
  }

  function removeDomain(index) {
    const domains = [...(config.privacy.excluded_domains || [])];
    domains.splice(index, 1);
    config.privacy.excluded_domains = domains;
    dispatch('change', config);
  }

  // 快捷选择应用
  function selectApp(appName) {
    selectedApp = appName;
  }
</script>

<div class="settings-card mb-5">
  <h3 class="settings-card-title">{$t('settings.privacy.title')}</h3>
  <p class="settings-card-desc">{$t('settings.privacy.desc')}</p>
  
  <div class="settings-section">
    <!-- 应用规则 -->
    <div>
      <div class="flex items-center justify-between mb-1">
        <span class="settings-text">
          {$t('settings.privacy.appRules')}
        </span>
        <button
          on:click={() => showAppInput = !showAppInput}
          class="settings-link-action text-sm"
        >
          {showAppInput ? $t('overview.collapse') : $t('settings.privacy.addRule')}
        </button>
      </div>
      <p class="settings-muted mb-3">{$t('settings.privacy.ruleHint')}</p>

      {#if showAppInput}
        <div class="settings-panel mb-3 animate-fadeIn">
          <!-- 应用名称输入 -->
          <div class="settings-field mb-3">
            <label for="app-name-input" class="settings-label">{$t('settings.privacy.inputAppName')}</label>
            <input
              id="app-name-input"
              type="text"
              bind:value={selectedApp}
              class="control-input"
              placeholder={$t('settings.privacy.inputPlaceholder')}
            />
          </div>
          <!-- 策略选择：分段按钮 -->
          <div class="settings-field mb-3">
            <span class="settings-label">{$t('settings.privacy.recordPolicy')}</span>
            <div class="flex rounded-lg overflow-hidden border border-slate-200 dark:border-slate-600">
              {#each privacyLevels as level}
                <button
                  on:click={() => selectedLevel = level.value}
                  class="segment-btn
                         {selectedLevel === level.value
                           ? level.activeClass
                           : 'settings-segment-idle'}"
                >
                  {level.label}
                </button>
              {/each}
            </div>
            <p class="text-xs mt-1.5 {privacyLevels.find(l => l.value === selectedLevel)?.textClass || 'settings-subtle'}">
              {privacyLevels.find(l => l.value === selectedLevel)?.desc || ''}
            </p>
          </div>
          
          <!-- 快捷选择 -->
          {#if recentApps.length > 0 || runningApps.length > 0}
          <div class="settings-block">
            {#if recentApps.length > 0}
            <div>
              <span class="settings-subtle block mb-1.5">{$t('settings.privacy.recentApps')}</span>
              <div class="flex flex-wrap gap-1.5">
                {#each recentApps.slice(0, 12) as app}
                  <button
                    on:click={() => selectApp(app)}
                    class="settings-chip-button
                           {selectedApp === app 
                             ? 'settings-chip-button-active'
                             : ''}"
                  >
                    {app}
                  </button>
                {/each}
              </div>
            </div>
            {/if}
            
            {#if runningApps.length > 0}
            <div>
              <span class="settings-subtle block mb-1.5">{$t('settings.privacy.runningApps')}</span>
              <div class="flex flex-wrap gap-1.5">
                {#each runningApps.slice(0, 8) as app}
                  <button
                    on:click={() => selectApp(app)}
                    class="settings-chip-button
                           {selectedApp === app 
                             ? 'settings-chip-button-active'
                             : ''}"
                  >
                    {app}
                  </button>
                {/each}
              </div>
            </div>
            {/if}
          </div>
          {/if}

          <!-- 操作按钮 -->
          <div class="settings-actions mt-4">
            <button
              on:click={() => { showAppInput = false; selectedApp = ''; }}
              class="settings-action-secondary"
            >
              {$t('settings.privacy.cancel')}
            </button>
            <button
              on:click={addAppRule}
              class="settings-action-primary"
              disabled={!selectedApp}
            >
              {$t('settings.privacy.addRuleBtn')}
            </button>
          </div>
        </div>
      {/if}

      <!-- 已有规则列表 -->
      <div class="space-y-2">
        {#each config.privacy.app_rules as rule, i}
          <div class="flex items-center justify-between p-3 bg-slate-50 dark:bg-slate-700/30 rounded-lg group">
            <div class="flex items-center gap-3">
              <span class="text-sm font-medium text-slate-800 dark:text-white">{rule.app_name}</span>
              {#if rule.level === 'full'}
                <span class="settings-chip-success">{$t('settings.privacy.fullRecord')}</span>
              {:else if rule.level === 'anonymized'}
                <span class="settings-chip-warn">{$t('settings.privacy.durationOnly')}</span>
              {:else}
                <span class="settings-chip-danger">{$t('settings.privacy.ignore')}</span>
              {/if}
            </div>
            <button
              on:click={() => removeAppRule(i)}
              class="text-xs text-slate-400 hover:text-red-500 opacity-0 group-hover:opacity-100 transition-all"
            >
              {$t('settings.privacy.delete')}
            </button>
          </div>
        {/each}
        {#if config.privacy.app_rules.length === 0}
          <p class="settings-empty">{$t('settings.privacy.noRules')}</p>
        {/if}
      </div>
    </div>

    <hr class="border-slate-200 dark:border-slate-700" />

    <!-- 内容过滤（合并敏感词 + 域名黑名单） -->
    <div>
      <span class="settings-text block mb-1">{$t('settings.privacy.contentFilter')}</span>
      <p class="settings-muted mb-4">{$t('settings.privacy.contentFilterDesc')}</p>
      
      <!-- 敏感词 -->
      <div class="mb-4">
        <div class="flex items-center justify-between mb-2">
          <span class="settings-label">{$t('settings.privacy.sensitiveWords')}</span>
          <button
            on:click={() => showKeywordInput = !showKeywordInput}
            class="settings-link-action"
          >
            {showKeywordInput ? $t('overview.collapse') : $t('settings.privacy.addSensitive')}
          </button>
        </div>
        
        {#if showKeywordInput}
          <div class="flex gap-2 mb-2 animate-fadeIn">
            <input
              type="text"
              bind:value={newKeyword}
              class="control-input flex-1"
              placeholder={$t('settings.privacy.inputSensitive')}
              on:keydown={(e) => e.key === 'Enter' && addKeyword()}
            />
            <button
              on:click={addKeyword}
              class="settings-action-primary"
            >
              {$t('settings.privacy.add')}
            </button>
          </div>
        {/if}

        <div class="flex flex-wrap gap-1.5">
          {#each config.privacy.excluded_keywords as keyword, i}
            <div class="settings-chip-neutral group">
              <span>{keyword}</span>
              <button
                on:click={() => removeKeyword(i)}
                class="ml-1.5 text-slate-400 hover:text-red-500 opacity-50 group-hover:opacity-100 transition-opacity"
              >
                ×
              </button>
            </div>
          {/each}
          {#if config.privacy.excluded_keywords.length === 0}
            <span class="settings-subtle">{$t('settings.privacy.noSensitiveWords')}</span>
          {/if}
        </div>
        <!-- 敏感词匹配说明 -->
        <p class="settings-note">{$t('settings.privacy.sensitiveHint')}</p>
      </div>

      <!-- 域名黑名单 -->
      <div>
        <div class="flex items-center justify-between mb-2">
          <span class="settings-label">{$t('settings.privacy.domainBlacklist')}</span>
          <button
            on:click={() => showDomainInput = !showDomainInput}
            class="settings-link-action"
          >
            {showDomainInput ? $t('overview.collapse') : $t('settings.privacy.addSensitive')}
          </button>
        </div>
        
        {#if showDomainInput}
          <div class="flex gap-2 mb-2 animate-fadeIn">
            <input
              type="text"
              bind:value={newDomain}
              class="control-input flex-1"
              placeholder={$t('settings.privacy.domainPlaceholder')}
              on:keydown={(e) => e.key === 'Enter' && addDomain()}
            />
            <button
              on:click={addDomain}
              class="settings-action-primary"
            >
              {$t('settings.privacy.add')}
            </button>
          </div>
        {/if}

        <div class="flex flex-wrap gap-1.5">
          {#each (config.privacy.excluded_domains || []) as domain, i}
            <div class="settings-chip-danger group">
              <span>{domain}</span>
              <button
                on:click={() => removeDomain(i)}
                class="ml-1.5 text-red-400 hover:text-red-600 opacity-50 group-hover:opacity-100 transition-opacity"
              >
                ×
              </button>
            </div>
          {/each}
          {#if (config.privacy.excluded_domains || []).length === 0}
            <span class="settings-subtle">{$t('settings.privacy.noDomains')}</span>
          {/if}
        </div>
        <!-- 域名黑名单格式说明 -->
        <p class="settings-note">{$t('settings.privacy.domainHint')} <code class="settings-code">example.com</code>{$t('settings.privacy.domainHintSuffix')}</p>
      </div>
    </div>
  </div>
</div>
