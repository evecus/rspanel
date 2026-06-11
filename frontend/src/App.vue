<template>
  <!-- Dashboard background -->
  <div id="dashboard">
    <div class="wp-bg" :style="wpStyle"></div>
    <div class="wp-overlay"></div>

    <div class="dash-content">
      <!-- Header -->
      <div class="dash-header" :style="headerStyle">
        <img v-if="panelInfo.logo" class="dash-logo" :src="panelInfo.logo" alt="" />
        <button class="icon-btn net-btn" @click="toggleNet" :title="netMode === 'lan' ? t('switchToWan') : t('switchToLan')">
          <svg v-if="netMode === 'lan'" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <rect x="2" y="3" width="20" height="14" rx="2"/><line x1="8" y1="21" x2="16" y2="21"/><line x1="12" y1="17" x2="12" y2="21"/>
          </svg>
          <svg v-else width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <circle cx="12" cy="12" r="10"/><line x1="2" y1="12" x2="22" y2="12"/>
            <path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"/>
          </svg>
        </button>
        <button class="icon-btn" @click="onSettingsClick">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <circle cx="12" cy="12" r="3"/>
            <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06-.06a2 2 0 0 1-2.83-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 2.83-2.83l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z"/>
          </svg>
        </button>
      </div>

      <!-- Hero -->
      <div class="hero">
        <div class="hero-hostname" :style="hostnameStyle">{{ panelInfo.hostname || 'RsPanel' }}</div>
        <div class="hero-clock" :style="clockStyle" v-html="clockHtml"></div>
      </div>

      <!-- Apps -->
      <div class="apps-outer" :style="outerStyle" @contextmenu.prevent="onPanelContextMenu">
        <div class="apps-inner" :style="innerStyle">

          <!-- Sort bar -->
          <div v-if="sortMode" class="sort-bar">
            <button class="sort-btn" @click="exitSort(true)">💾 {{ t('saveSortBtn') }}</button>
            <button class="sort-btn" style="background:rgba(255,255,255,.6)" @click="exitSort(false)">✕ {{ t('cancelBtn') }}</button>
          </div>

          <!-- App grid -->
        <div class="apps-grid" :style="{ gap: dispSet.iconGap + 'px' }">
            <div
              v-for="app in apps" :key="app.id"
              class="app-card"
              :class="{ 'sort-mode': sortMode, 'drag-over': dragOverId === app.id }"
              :style="{ width: (dispSet.iconSize + 14) + 'px' }"
              :draggable="sortMode"
              @click="onAppClick(app)"
              @contextmenu.prevent.stop="onAppContextMenu($event, app.id)"
              @dragstart="onDragStart($event, app.id)"
              @dragover.prevent="dragOverId = app.id"
              @dragleave="dragOverId = null"
              @drop="onDrop($event, app.id)"
            >
              <div class="app-icon-wrap" :style="iconWrapStyle">
                <template v-if="app.icon_type === 'image' && app.icon_image">
                  <img class="app-icon-img" :src="app.icon_image" :alt="app.title"
                    :style="{ borderRadius: iconBorderRadius }"
                    @error="e => { e.target.style.display='none'; e.target.nextElementSibling.style.display='flex' }" />
                  <div class="app-icon-txt" style="display:none" :style="iconTxtStyle">{{ (app.title || '?').substring(0, 2) }}</div>
                </template>
                <div v-else class="app-icon-txt" :style="iconTxtStyle">
                  {{ (app.icon_text && app.icon_text.trim()) ? app.icon_text.trim() : (app.title || '?').substring(0, 2) }}
                </div>
              </div>
              <div v-show="showAppName" class="app-name" :style="appNameStyle">{{ app.title }}</div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>

  <!-- Components -->
  <LoginModal ref="loginModal" />
  <AppModal ref="appModal" @saved="loadApps" @deleted="loadApps" @toast="showToast" />
  <ContextMenu ref="ctxMenu" @edit="onCtxEdit" @delete="onCtxDelete" @add="onCtxAdd" @sort="onCtxSort" />
  <SettingsPanel
    ref="settingsPanel"
    :user="curUser"
    :panel-info="panelInfo"
    :pub-mode-value="pubMode"
    :net-mode-value="netMode"
    @net-mode-changed="v => netMode = v"
    :show-app-name="showAppName"
    @show-app-name-changed="v => showAppName = v"
    :desktop-disp="desktopDisp"
    :mobile-disp="mobileDisp"
    :disp-set="dispSet"
    :font-set="fontSet"
    :clk-cfg="clkCfg"
    :apps="apps"
    @toast="showToast"
    @logout="doLogout"
    @panel-updated="loadPanel"
    @clear-data="clearAllData"
  />
  <AppToast ref="toast" />
</template>

<script setup>
import { ref, reactive, computed, onMounted, onUnmounted } from 'vue'
import { useI18n, lang } from './composables/useI18n.js'
import { apiCall } from './composables/useApi.js'
import { useTheme, resolveFont, curThemeId, curTheme, WPS } from './composables/useTheme.js'
import { getLunar } from './composables/useLunar.js'
import LoginModal from './components/LoginModal.vue'
import AppModal from './components/AppModal.vue'
import ContextMenu from './components/ContextMenu.vue'
import SettingsPanel from './components/SettingsPanel.vue'
import AppToast from './components/AppToast.vue'

const { t } = useI18n()
useTheme()

// ── Refs ────────────────────────────────────────────────────────
const loginModal   = ref(null)
const appModal     = ref(null)
const ctxMenu      = ref(null)
const settingsPanel  = ref(null)
const toast        = ref(null)

// ── State ───────────────────────────────────────────────────────
const apps      = ref([])
const panelInfo = ref({})
const curUser   = ref(null)
const pubMode   = ref(false)
const netMode   = ref('lan')  // 'lan' | 'wan'
const showAppName    = ref(true)
const desktopDisp    = ref(null)  // 桌面端完整样式数据
const mobileDisp     = ref(null)  // 移动端完整样式数据
const clkCfg    = reactive({ show_time: true, show_date: true, show_weekday: true, show_lunar: false, show_seconds: false, show_year: false })
const clockHtml = ref('')
const dispSet   = reactive({ hostnameSize: 76, clockSize: 24, iconSize: 64, appNameSize: 14, iconRadius: 25, iconGap: 22, sidePadding: 52 })
const fontSet   = reactive({ hostname: 'system', clock: 'system', appname: 'system', ui: 'system' })
const sortMode  = ref(false)
const dragOverId = ref(null)

let clkTimer = null

// ── Computed ────────────────────────────────────────────────────
const fallbackGrad = computed(() => curTheme.value.grad || 'linear-gradient(135deg,#6b21a8 0%,#a855f7 40%,#ec4899 100%)')
const wallpaperUrl  = computed(() => panelInfo.value.wallpaper || curTheme.value.wp || '/default-wallpaper')
const wpStyle = computed(() => ({
  background: fallbackGrad.value,
  backgroundImage: wallpaperUrl.value ? `url(${wallpaperUrl.value})` : 'none',
  backgroundSize: 'cover',
  backgroundPosition: 'center',
}))

const iconBorderRadius = computed(() => Math.round((dispSet.iconRadius || 26) / 100 * dispSet.iconSize) + 'px')
const iconWrapStyle = computed(() => ({ width: dispSet.iconSize + 'px', height: dispSet.iconSize + 'px', borderRadius: iconBorderRadius.value }))
const iconTxtStyle  = computed(() => ({ fontSize: Math.round(dispSet.iconSize * 0.33) + 'px', borderRadius: iconBorderRadius.value }))
const appNameStyle  = computed(() => ({ fontSize: dispSet.appNameSize + 'px', maxWidth: (dispSet.iconSize + 10) + 'px', fontFamily: resolveFont(fontSet.appname) }))
const hostnameStyle = computed(() => ({ fontSize: dispSet.hostnameSize + 'px', fontFamily: resolveFont(fontSet.hostname) }))
const clockStyle    = computed(() => ({ fontSize: dispSet.clockSize + 'px', fontFamily: resolveFont(fontSet.clock) }))
const outerStyle    = computed(() => ({ padding: `0 0 52px` }))
const headerStyle   = computed(() => ({ padding: `22px ${dispSet.sidePadding}px 0` }))
const innerStyle    = computed(() => ({ padding: `0 ${dispSet.sidePadding}px` }))

// ── Clock ───────────────────────────────────────────────────────
function tick() {
  const now = new Date(), pad = n => String(n).padStart(2, '0')
  const parts = []

  // 左侧：年 / 农历 / 日期 / 星期
  const leftItems = []
  if (clkCfg.show_year) leftItems.push(`${now.getFullYear()}年`)
  if (clkCfg.show_lunar && lang.value === 'zh') {
    const l = getLunar(now)
    if (l) leftItems.push(`农历 ${l}`)
  }
  if (clkCfg.show_date) leftItems.push(`${now.getMonth() + 1}月${now.getDate()}日`)
  const days = t('weekdays')
  if (clkCfg.show_weekday) leftItems.push(days[now.getDay()])
  if (leftItems.length) parts.push(`<span>${leftItems.join(' &nbsp;')}</span>`)

  // 分隔符 + 右侧：时间
  if (clkCfg.show_time) {
    let tv = `${pad(now.getHours())}:${pad(now.getMinutes())}`
    if (clkCfg.show_seconds) tv += `:${pad(now.getSeconds())}`
    if (leftItems.length) parts.push(`<span class="div">|</span>`)
    parts.push(`<span>${tv}</span>`)
  }

  clockHtml.value = parts.join('')
}

function startClock() {
  if (clkTimer) clearInterval(clkTimer)
  tick(); clkTimer = setInterval(tick, 1000)
}

// ── Data ────────────────────────────────────────────────────────
async function loadPanel() {
  try {
    const info = await apiCall('/api/panel')
    panelInfo.value = info
    pubMode.value = info.public_mode || false
    netMode.value = info.network_mode || 'lan'
    showAppName.value    = info.show_app_name !== false // 默认true
    desktopDisp.value    = info.desktop || null
    mobileDisp.value     = info.mobile  || null
    if (info.clock) Object.assign(clkCfg, info.clock)
    const sl = info.language || localStorage.getItem('ep_lang') || 'zh'
    lang.value = sl; localStorage.setItem('ep_lang', sl)
    const th = (await import('./composables/useTheme.js')).THEMES.find(x => x.id === info.theme)
    if (th) { curThemeId.value = th.id; (await import('./composables/useTheme.js')).applyThemeCss(th) }
    Object.assign(dispSet, {
      hostnameSize: info.hostname_size || 56, clockSize: info.clock_size || 16,
      iconSize: info.icon_size || 78, appNameSize: info.app_name_size || 12,
      iconRadius: info.icon_radius || 26, iconGap: info.icon_gap || 22, sidePadding: info.side_padding || 52,
    })
    Object.assign(fontSet, {
      hostname: info.font_hostname || 'system', clock: info.font_clock || 'system',
      appname: info.font_appname || 'system', ui: info.font_ui || 'system',
    })
    tick()
  } catch (e) { console.error(e) }
}

async function loadApps() {
  try { apps.value = await apiCall('/api/apps') } catch (e) { console.error(e) }
}

// ── Auth ────────────────────────────────────────────────────────
function showToast(msg) { toast.value?.show(msg) }

async function requireAuth(cb) {
  // 内存里已有用户，直接放行
  if (curUser.value) { await cb(); return }
  // 内存没有，但 cookie 里可能有有效 token，先验证一次
  try {
    const auth = await apiCall('/api/checkauth')
    if (auth.logged_in) {
      curUser.value = auth
      await cb()
      return
    }
  } catch (e) { console.error('checkauth error:', e) }
  // cookie 也无效，才弹登录框
  loginModal.value?.open(t('loginRequired'), async (user) => { curUser.value = user; await cb() })
}

async function doLogout() {
  await apiCall('/api/logout', { method: 'POST' }).catch(() => {})
  curUser.value = null
  settingsPanel.value?.close()
  if (!pubMode.value) loginModal.value?.open(t('pleaseLogin'), (user) => { curUser.value = user })
  showToast(t('tLoggedOut'))
}

// ── Settings ────────────────────────────────────────────────────
async function onSettingsClick() {
  await requireAuth(async () => { await settingsPanel.value?.open() })
}

// ── App actions ─────────────────────────────────────────────────
function onAddApp() { requireAuth(() => appModal.value?.openAdd()) }
function onAppClick(app) {
  if (sortMode.value) return
  // 根据网络模式选地址，优先用对应模式地址，没有则 fallback 到另一个，再 fallback 旧 url 字段
  let url = ''
  if (netMode.value === 'lan') {
    url = app.url_lan || app.url_wan || app.url || ''
  } else {
    url = app.url_wan || app.url_lan || app.url || ''
  }
  if (!url) return
  app.open_type === 'current' ? (window.location.href = url) : window.open(url, '_blank')
}
function toggleNet() {
  netMode.value = netMode.value === 'lan' ? 'wan' : 'lan'
}
function onAppContextMenu(e, id) {
  requireAuth(() => ctxMenu.value?.show(e.clientX, e.clientY, id))
}
function onPanelContextMenu(e) {
  requireAuth(() => ctxMenu.value?.showPanel(e.clientX, e.clientY))
}

function onCtxEdit(id) {
  const app = apps.value.find(x => x.id === id)
  if (app) appModal.value?.openEdit(app)
}
async function onCtxDelete(id) {
  if (!confirm(t('confirmDelete'))) return
  try { await apiCall(`/api/apps/${id}`, { method: 'DELETE' }); await loadApps(); showToast(t('tDeleted')) }
  catch { showToast(t('tDeleteFailed')) }
}
function onCtxAdd()  { onAddApp() }
function onCtxSort() { onEnterSort() }

// ── Sort ────────────────────────────────────────────────────────
function onEnterSort() { requireAuth(() => { sortMode.value = true }) }

async function exitSort(save) {
  sortMode.value = false; dragOverId.value = null
  if (save) {
    try { await apiCall('/api/apps/reorder', { method: 'POST', body: JSON.stringify({ ids: apps.value.map(a => a.id) }) }); showToast(t('tSortSaved')) }
    catch { showToast('failed') }
  } else await loadApps()
}

function onDragStart(e, id) {
  if (!sortMode.value) { e.preventDefault(); return }
  e.dataTransfer.effectAllowed = 'move'
  e.dataTransfer.setData('text/plain', id)
}
function onDrop(e, targetId) {
  dragOverId.value = null
  const srcId = e.dataTransfer.getData('text/plain')
  if (!srcId || srcId === targetId) return
  const a = apps.value.findIndex(x => x.id === srcId), b = apps.value.findIndex(x => x.id === targetId)
  if (a < 0 || b < 0) return
  const [item] = apps.value.splice(a, 1); apps.value.splice(b, 0, item)
}

// ── Backup clear ─────────────────────────────────────────────────
async function clearAllData() {
  if (!confirm(t('confirmClear'))) return
  try {
    for (const app of apps.value) await apiCall(`/api/apps/${app.id}`, { method: 'DELETE' }).catch(() => {})
    await loadApps(); showToast(t('tCleared'))
  } catch { showToast(t('tFailed')) }
}

// ── Lifecycle ───────────────────────────────────────────────────
onMounted(async () => {
  await loadPanel()
  await loadApps()
  startClock()
  const auth = await apiCall('/api/checkauth').catch(() => ({ logged_in: false }))
  if (auth.logged_in) curUser.value = auth
  if (!pubMode.value && !curUser.value) {
    loginModal.value?.open(t('pleaseLogin'), (user) => { curUser.value = user })
  }
  document.addEventListener('click', (e) => {
    const ctx = document.getElementById('ctx-menu')
    if (ctx && !ctx.contains(e.target)) ctxMenu.value?.hide()
  })
})

onUnmounted(() => { if (clkTimer) clearInterval(clkTimer) })
</script>

<style scoped>
#dashboard { min-height: 100vh; position: relative; }
.wp-bg { position: fixed; inset: 0; z-index: 0; background-size: cover; background-position: center; transition: background .6s; }
.wp-overlay { position: fixed; inset: 0; z-index: 0; background: rgba(0,0,0,.22); }
.dash-content { position: relative; z-index: 1; min-height: 100vh; display: flex; flex-direction: column; }
.dash-header { display: flex; justify-content: flex-start; align-items: flex-start; }
.dash-logo { height: 34px; width: auto; border-radius: 8px; object-fit: contain; }
.net-btn { position: fixed; top: 22px; right: 100px; }
.sys-ico-fill { width:100%; height:100%; display:flex; align-items:center; justify-content:center; border-radius:inherit; }
.icon-btn { position: fixed; top: 22px; right: 52px; z-index: 10; width: 40px; height: 40px; border-radius: 12px; cursor: pointer; display: flex; align-items: center; justify-content: center; color: white; font-size: 17px; transition: all var(--tr); border: 1px solid rgba(255,255,255,.25); background: rgba(255,255,255,.15); backdrop-filter: blur(12px); }
.icon-btn:hover { background: rgba(255,255,255,.28); transform: translateY(-1px); }
.hero { text-align: center; padding: 32px 20px 40px; color: white; flex-shrink: 0; }
.hero-hostname { font-size: 56px; font-weight: 900; letter-spacing: -1.5px; line-height: 1.05; margin-bottom: 20px; text-shadow: 0 2px 24px rgba(0,0,0,.28); }
.hero-clock { font-size: 16px; display: flex; align-items: center; justify-content: center; gap: 8px; flex-wrap: wrap; opacity: .9; }
:deep(.hero-clock .div) { opacity: .35; }
.apps-outer { flex: 1; display: flex; justify-content: center; }
.apps-inner { width: 100%; max-width: 1100px; }
.group-row { display: flex; align-items: center; gap: 10px; margin-bottom: 18px; }
.group-lbl { color: rgba(255,255,255,.92); font-size: 15px; font-weight: 800; text-transform: uppercase; letter-spacing: 2px; text-shadow: 0 1px 6px rgba(0,0,0,.3); }
.group-tools { display: none; align-items: center; gap: 6px; }
.group-row:hover .group-tools { display: flex; }
.tool-btn { width: 30px; height: 30px; border-radius: 9px; border: 1.5px solid rgba(255,255,255,.4); background: rgba(255,255,255,.12); backdrop-filter: blur(8px); cursor: pointer; display: flex; align-items: center; justify-content: center; color: rgba(255,255,255,.9); transition: all var(--tr); }
.tool-btn:hover, .tool-btn.active { background: rgba(255,255,255,.28); border-color: white; }
.sort-bar { display: flex; margin-bottom: 14px; gap: 8px; flex-wrap: wrap; }
.sort-btn { padding: 8px 18px; background: rgba(255,255,255,.88); color: #1e1b2e; border: none; border-radius: 10px; font-size: 13px; font-weight: 700; cursor: pointer; display: flex; align-items: center; gap: 5px; box-shadow: 0 4px 14px rgba(0,0,0,.14); transition: all var(--tr); }
.sort-btn:hover { background: white; transform: translateY(-1px); }
.apps-grid { display: flex; flex-wrap: wrap; justify-content: flex-start; }
.app-card { display: flex; flex-direction: column; align-items: center; gap: 9px; cursor: pointer; user-select: none; }
.app-icon-wrap { overflow: hidden; transition: transform var(--tr), box-shadow var(--tr); box-shadow: 0 6px 20px rgba(0,0,0,.24), 0 1px 0 rgba(255,255,255,.2) inset; }
.app-card:not(.sort-mode):hover .app-icon-wrap { transform: translateY(-6px) scale(1.07); box-shadow: 0 16px 36px rgba(0,0,0,.32), 0 1px 0 rgba(255,255,255,.2) inset; }
.app-icon-img { width: 100%; height: 100%; object-fit: cover; display: block; }
.app-icon-txt { width: 100%; height: 100%; display: flex; align-items: center; justify-content: center; background: var(--grad); font-weight: 800; color: white; text-shadow: 0 1px 4px rgba(0,0,0,.25); }
.app-name { font-size: 12px; color: rgba(255,255,255,.96); text-align: center; font-weight: 600; text-shadow: 0 1px 4px rgba(0,0,0,.4); overflow: hidden; text-overflow: ellipsis; white-space: normal; display: -webkit-box; -webkit-line-clamp: 2; -webkit-box-orient: vertical; word-break: break-word; line-height: 1.35; min-height: 1.35em; }
.app-card.sort-mode { animation: wiggle .4s ease infinite alternate; }
.app-card.sort-mode .app-icon-wrap { border: 2.5px dashed rgba(255,255,255,.65); transform: none !important; box-shadow: none !important; }
.app-card.drag-over .app-icon-wrap { border-color: white; background: rgba(255,255,255,.12); }
@media(max-width:700px){
  .hero { padding: 22px 16px 30px; }
  .apps-grid { justify-content: center; }
  .icon-btn { right: 18px; }
}
</style>
