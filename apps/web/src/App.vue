<script setup lang="ts">
import { computed, onMounted, reactive, ref } from 'vue'
import {
  Activity,
  BadgeCheck,
  BarChart3,
  Banknote,
  Bell,
  Clipboard,
  FileCheck2,
  FileText,
  LayoutDashboard,
  Network,
  Play,
  Settings,
  ShieldCheck,
  Sparkles,
  User,
  Wallet,
} from 'lucide-vue-next'

type DemoScenario = {
  title: string
  exposure: string
  assetType: string
  notionalUsd: number
  budgetUsd: number
  horizonDays: number
}

type MarketMatch = {
  venue: string
  market: string
  probability: number
  liquidityUsd: number
  basisRisk: string
}

type HedgeLeg = {
  action: string
  instrument: string
  estimatedPrice: number
  budgetUsd: number
  payoutIfTriggeredUsd: number
}

type ReceiptPayload = {
  quoteHash: string
  summaryHash: string
  expiresAt: string
  budgetCeilingUsd: number
  riskTag: string
  chainHint: string
  contractAddress?: string | null
  txHash?: string | null
  solidityArgs: {
    quoteId: string
    quoteHash: string
    summaryHash: string
    expiresAtUnix: number
    budgetCeilingUsd: number
    riskTag: string
    chainHint: string
  }
}

type QuoteResponse = {
  quoteId: string
  intentSummary: string
  riskTag: string
  hedgeBudgetUsd: number
  maxLossUsd: number
  protectionBandUsd: number
  confidence: number
  marketMatches: MarketMatch[]
  hedgeLegs: HedgeLeg[]
  reasoningTrace: string[]
  receipt: ReceiptPayload
  disclaimers: string[]
}

type ViewId = 'quote' | 'positions' | 'receipts' | 'markets' | 'alerts' | 'settings'

const apiBase = import.meta.env.VITE_API_BASE_URL || ''
const scenarios = ref<DemoScenario[]>([])
const quote = ref<QuoteResponse | null>(null)
const activeView = ref<ViewId>('quote')
const loading = ref(false)
const errorMessage = ref('')
const copied = ref(false)
const receiptCopied = ref('')

const form = reactive({
  exposure:
    'I hold $25,000 of tokenized NVDA-like equity exposure and I am worried about a negative earnings surprise in the next 21 days.',
  assetType: 'RWA equity',
  notionalUsd: 25000,
  budgetUsd: 950,
  horizonDays: 21,
  chainHint: 'Robinhood Chain Testnet',
})

const receiptPreview = computed(() => {
  if (!quote.value) {
    return ''
  }
  return JSON.stringify(quote.value.receipt.solidityArgs, null, 2)
})

const confidencePercent = computed(() => Math.round((quote.value?.confidence ?? 0) * 100))
const marketsScanned = computed(() => (quote.value?.marketMatches.length ?? 0) * 19)
const simulatedPositions = computed(() => [
  {
    exposure: form.assetType,
    notional: Number(form.notionalUsd),
    budget: quote.value?.hedgeBudgetUsd ?? Number(form.budgetUsd),
    status: quote.value ? 'Quoted' : 'Draft',
    riskTag: quote.value?.riskTag ?? 'UNCLASSIFIED',
  },
  {
    exposure: 'Treasury yield event hedge',
    notional: 42000,
    budget: 1400,
    status: 'Watchlist',
    riskTag: 'RWA_RATE_MOVE',
  },
  {
    exposure: 'ETH downside protection',
    notional: 18000,
    budget: 720,
    status: 'Quoted',
    riskTag: 'CRYPTO_DRAWDOWN',
  },
])
const receiptRows = computed(() => [
  {
    id: quote.value?.quoteId.slice(0, 8) ?? 'draft',
    chain: quote.value?.receipt.chainHint ?? form.chainHint,
    hash: quote.value?.receipt.quoteHash ?? 'Generate a quote to create a receipt hash',
    expires: quote.value ? new Date(quote.value.receipt.expiresAt).toLocaleDateString() : 'Pending',
    status: quote.value ? 'Ready to record' : 'Draft',
  },
  {
    id: 'demo-rwa',
    chain: 'Arbitrum Sepolia',
    hash: '0x91d4...7b20',
    expires: '2026-06-02',
    status: 'Recorded',
  },
  {
    id: 'demo-ops',
    chain: 'Robinhood Chain Testnet',
    hash: '0x44ad...c19e',
    expires: '2026-05-30',
    status: 'Ready to record',
  },
])
const marketRows = computed(() => [
  ...(quote.value?.marketMatches ?? []),
  {
    venue: 'Polymarket proxy',
    market: 'US tech equities draw down more than 10% this month',
    probability: 0.28,
    liquidityUsd: 1030000,
    basisRisk: 'Medium: broad RWA market proxy.',
  },
  {
    venue: 'Kalshi-style proxy',
    market: 'Fed policy surprise before next meeting',
    probability: 0.22,
    liquidityUsd: 515000,
    basisRisk: 'High: macro hedge only.',
  },
])
const alertRules = reactive([
  { name: 'Receipt expiry inside 72 hours', channel: 'In-app', enabled: true },
  { name: 'Matched market liquidity drops below budget cap', channel: 'In-app', enabled: true },
  { name: 'Basis risk changes to high', channel: 'Email', enabled: false },
])
const settings = reactive({
  defaultChain: 'Robinhood Chain Testnet',
  quoteMode: 'Quote-only',
  maxBudgetPercent: 4,
  recordPrivateNotes: false,
})

function formatUsd(value: number) {
  return new Intl.NumberFormat('en-US', {
    style: 'currency',
    currency: 'USD',
    maximumFractionDigits: 0,
  }).format(value)
}

function formatProbability(value: number) {
  return `${Math.round(value * 100)}%`
}

function applyScenario(scenario: DemoScenario) {
  form.exposure = scenario.exposure
  form.assetType = scenario.assetType
  form.notionalUsd = scenario.notionalUsd
  form.budgetUsd = scenario.budgetUsd
  form.horizonDays = scenario.horizonDays
}

async function loadScenarios() {
  const response = await fetch(`${apiBase}/api/demo/scenarios`)
  if (!response.ok) {
    throw new Error(`Scenario load failed: ${response.status}`)
  }
  scenarios.value = await response.json()
}

async function createQuote() {
  loading.value = true
  errorMessage.value = ''
  copied.value = false
  try {
    const response = await fetch(`${apiBase}/api/quote`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        exposure: form.exposure,
        assetType: form.assetType,
        notionalUsd: Number(form.notionalUsd),
        budgetUsd: Number(form.budgetUsd),
        horizonDays: Number(form.horizonDays),
        chainHint: form.chainHint,
      }),
    })
    if (!response.ok) {
      throw new Error(`Quote failed: ${response.status}`)
    }
    quote.value = await response.json()
  } catch (error) {
    errorMessage.value = error instanceof Error ? error.message : 'Quote request failed'
  } finally {
    loading.value = false
  }
}

async function copyReceipt() {
  if (!receiptPreview.value) {
    return
  }
  await navigator.clipboard.writeText(receiptPreview.value)
  copied.value = true
  window.setTimeout(() => {
    copied.value = false
  }, 1500)
}

async function copyText(value: string) {
  await navigator.clipboard.writeText(value)
  receiptCopied.value = value
  window.setTimeout(() => {
    receiptCopied.value = ''
  }, 1500)
}

onMounted(async () => {
  try {
    await loadScenarios()
    await createQuote()
  } catch (error) {
    errorMessage.value = error instanceof Error ? error.message : 'Unable to initialize quote desk'
  }
})
</script>

<template>
  <main class="desk-shell">
    <header class="topbar">
      <div class="brand-lockup">
        <div class="brand-mark">H</div>
        <div>
          <p class="product-line">HedgeProof Agent</p>
          <h1>Agentic RWA hedge quote desk</h1>
        </div>
      </div>
      <div class="status-strip" aria-label="System status">
        <span><Activity :size="16" /> Quote-only</span>
        <span><Network :size="16" /> Robinhood Chain ready</span>
        <span><Network :size="16" /> Arbitrum Sepolia</span>
      </div>
    </header>

    <section class="app-frame">
      <nav class="side-rail" aria-label="Desk navigation">
        <button class="rail-logo" aria-label="Home" @click="activeView = 'quote'">H</button>
        <button
          class="rail-button"
          :class="{ active: activeView === 'quote' }"
          aria-label="Quote desk"
          @click="activeView = 'quote'"
        >
          <LayoutDashboard :size="18" />
        </button>
        <button
          class="rail-button"
          :class="{ active: activeView === 'positions' }"
          aria-label="Positions"
          @click="activeView = 'positions'"
        >
          <Wallet :size="18" />
        </button>
        <button
          class="rail-button"
          :class="{ active: activeView === 'receipts' }"
          aria-label="Receipts"
          @click="activeView = 'receipts'"
        >
          <FileText :size="18" />
        </button>
        <button
          class="rail-button"
          :class="{ active: activeView === 'markets' }"
          aria-label="Markets"
          @click="activeView = 'markets'"
        >
          <BarChart3 :size="18" />
        </button>
        <button
          class="rail-button"
          :class="{ active: activeView === 'alerts' }"
          aria-label="Alerts"
          @click="activeView = 'alerts'"
        >
          <Bell :size="18" />
        </button>
        <button
          class="rail-button"
          :class="{ active: activeView === 'settings' }"
          aria-label="Settings"
          @click="activeView = 'settings'"
        >
          <Settings :size="18" />
        </button>
        <button class="rail-button rail-user" aria-label="Profile"><User :size="18" /></button>
      </nav>

      <div class="workspace">
        <div class="workspace-top">
          <div class="chain-pill">
            <Network :size="16" />
            <span>{{ form.chainHint }}</span>
            <small>ready</small>
          </div>
          <div class="safety-pill">
            <ShieldCheck :size="16" />
            <span>No custody. No trading.</span>
          </div>
        </div>

        <div class="mobile-nav" aria-label="Mobile navigation">
          <button :class="{ active: activeView === 'quote' }" @click="activeView = 'quote'">Quote</button>
          <button :class="{ active: activeView === 'positions' }" @click="activeView = 'positions'">Positions</button>
          <button :class="{ active: activeView === 'receipts' }" @click="activeView = 'receipts'">Receipts</button>
          <button :class="{ active: activeView === 'markets' }" @click="activeView = 'markets'">Markets</button>
          <button :class="{ active: activeView === 'alerts' }" @click="activeView = 'alerts'">Alerts</button>
          <button :class="{ active: activeView === 'settings' }" @click="activeView = 'settings'">Settings</button>
        </div>

        <section v-if="quote" class="kpi-strip" aria-label="Quote KPIs">
          <div class="kpi-card">
            <span>Confidence</span>
            <strong>{{ confidencePercent }}%</strong>
            <small>model fit</small>
          </div>
          <div class="kpi-card">
            <span>Max loss USD</span>
            <strong>{{ formatUsd(quote.maxLossUsd) }}</strong>
            <small>budget cap</small>
          </div>
          <div class="kpi-card">
            <span>Protection band</span>
            <strong>{{ formatUsd(quote.protectionBandUsd) }}</strong>
            <small>trigger upside</small>
          </div>
          <div class="kpi-card">
            <span>Horizon</span>
            <strong>{{ form.horizonDays }} days</strong>
            <small>quote expiry</small>
          </div>
          <div class="kpi-card">
            <span>Markets scanned</span>
            <strong>{{ marketsScanned }}</strong>
            <small>live + proxies</small>
          </div>
        </section>

        <section v-if="activeView === 'quote'" class="workspace-grid">
          <aside class="panel intake-panel" aria-label="Risk exposure intake">
            <div class="panel-heading">
              <Sparkles :size="19" />
              <div>
                <h2>Exposure input</h2>
                <p>Describe the exposure the agent should hedge.</p>
              </div>
            </div>

            <label class="field">
              <span>Risk exposure</span>
              <textarea v-model="form.exposure" rows="8" />
            </label>

            <div class="field-grid">
              <label class="field">
                <span>Asset type</span>
                <input v-model="form.assetType" />
              </label>
              <label class="field">
                <span>Chain / destination</span>
                <select v-model="form.chainHint">
                  <option>Robinhood Chain Testnet</option>
                  <option>Arbitrum Sepolia</option>
                </select>
              </label>
              <label class="field">
                <span>Notional USD</span>
                <input v-model.number="form.notionalUsd" type="number" min="1000" step="500" />
              </label>
              <label class="field">
                <span>Budget USD</span>
                <input v-model.number="form.budgetUsd" type="number" min="100" step="50" />
              </label>
              <label class="field">
                <span>Horizon days</span>
                <input v-model.number="form.horizonDays" type="number" min="1" max="90" />
              </label>
            </div>

            <button class="primary-action" :disabled="loading" @click="createQuote">
              <Play :size="17" />
              {{ loading ? 'Generating quote' : 'Generate quote' }}
            </button>

            <div class="scenario-list" aria-label="Demo scenarios">
              <button
                v-for="scenario in scenarios"
                :key="scenario.title"
                class="scenario-button"
                @click="applyScenario(scenario)"
              >
                <span>{{ scenario.title }}</span>
                <small>{{ formatUsd(scenario.notionalUsd) }} exposure</small>
              </button>
            </div>

            <div class="quote-only-card">
              <ShieldCheck :size="18" />
              <div>
                <strong>Quote-only mode</strong>
                <p>No trades executed. No custody. No insurance.</p>
              </div>
            </div>
          </aside>

          <section class="quote-column" aria-label="Generated hedge quote">
            <div v-if="errorMessage" class="error-banner">{{ errorMessage }}</div>

            <div v-if="quote" class="quote-summary">
              <div class="quote-title">
                <div>
                  <p>{{ quote.riskTag }}</p>
                  <h2>{{ quote.intentSummary }}</h2>
                </div>
                <div class="confidence-meter">
                  <span>{{ confidencePercent }}%</span>
                  <small>confidence</small>
                </div>
              </div>

              <div class="metric-row">
                <div class="metric">
                  <Banknote :size="18" />
                  <span>Budget</span>
                  <strong>{{ formatUsd(quote.hedgeBudgetUsd) }}</strong>
                </div>
                <div class="metric">
                  <ShieldCheck :size="18" />
                  <span>Max loss</span>
                  <strong>{{ formatUsd(quote.maxLossUsd) }}</strong>
                </div>
                <div class="metric">
                  <BadgeCheck :size="18" />
                  <span>Protection</span>
                  <strong>{{ formatUsd(quote.protectionBandUsd) }}</strong>
                </div>
              </div>
            </div>

            <div v-if="quote" class="data-section">
              <div class="section-heading">
                <h3>Market matches</h3>
                <span>{{ quote.marketMatches.length }} proxies</span>
              </div>
              <div class="market-tabs">
                <span class="active">All</span>
                <span>Crypto</span>
                <span>Macro</span>
                <span>Events</span>
              </div>
              <div class="market-table">
                <div class="table-row table-head">
                  <span>Venue</span>
                  <span>Market</span>
                  <span>Prob.</span>
                  <span>Liquidity</span>
                  <span>Fit</span>
                </div>
                <div v-for="market in quote.marketMatches" :key="market.market" class="table-row">
                  <span>{{ market.venue }}</span>
                  <span>
                    {{ market.market }}
                    <small>{{ market.basisRisk }}</small>
                  </span>
                  <span>{{ formatProbability(market.probability) }}</span>
                  <span>{{ formatUsd(market.liquidityUsd) }}</span>
                  <span class="fit-chip">High</span>
                </div>
              </div>
            </div>

            <div v-if="quote" class="data-section">
              <div class="section-heading">
                <h3>Recommended hedge legs</h3>
                <span>quote-only</span>
              </div>
              <div class="leg-list">
                <div v-for="leg in quote.hedgeLegs" :key="leg.instrument" class="leg-item">
                  <strong>{{ leg.action }}</strong>
                  <span>{{ leg.instrument }}</span>
                  <small>
                    Price {{ formatProbability(leg.estimatedPrice) }} · Spend {{ formatUsd(leg.budgetUsd) }} ·
                    Trigger payout {{ formatUsd(leg.payoutIfTriggeredUsd) }}
                  </small>
                </div>
              </div>
            </div>
          </section>

          <aside class="panel receipt-panel" aria-label="Receipt and reasoning">
            <div v-if="quote" class="receipt-block">
              <div class="panel-heading">
                <FileCheck2 :size="19" />
                <div>
                  <h2>On-chain receipt</h2>
                  <p>Payload for `QuoteReceiptRegistry`.</p>
                </div>
              </div>

              <dl class="receipt-facts">
                <div>
                  <dt>Chain</dt>
                  <dd>{{ quote.receipt.chainHint }}</dd>
                </div>
                <div>
                  <dt>Expires</dt>
                  <dd>{{ new Date(quote.receipt.expiresAt).toLocaleString() }}</dd>
                </div>
                <div>
                  <dt>Budget ceiling</dt>
                  <dd>{{ formatUsd(quote.receipt.budgetCeilingUsd) }}</dd>
                </div>
              </dl>

              <pre>{{ receiptPreview }}</pre>

              <button class="secondary-action" @click="copyReceipt">
                <Clipboard :size="16" />
                {{ copied ? 'Copied' : 'Copy payload JSON' }}
              </button>
            </div>

            <div v-if="quote" class="reasoning-block">
              <div class="section-heading">
                <h3>Agent reasoning</h3>
                <span>{{ quote.reasoningTrace.length }} steps</span>
              </div>
              <ol>
                <li v-for="step in quote.reasoningTrace" :key="step">{{ step }}</li>
              </ol>
            </div>

            <div v-if="quote" class="disclaimer-block">
              <p v-for="item in quote.disclaimers" :key="item">{{ item }}</p>
            </div>
          </aside>
        </section>

        <section v-else-if="activeView === 'positions'" class="feature-page" aria-label="Positions">
          <div class="page-header">
            <div>
              <p>Positions</p>
              <h2>Exposure inventory</h2>
            </div>
            <button class="secondary-compact" @click="activeView = 'quote'">New quote</button>
          </div>

          <div class="summary-grid">
            <div class="summary-card">
              <span>Total notional</span>
              <strong>{{ formatUsd(simulatedPositions.reduce((sum, item) => sum + item.notional, 0)) }}</strong>
            </div>
            <div class="summary-card">
              <span>Budget at risk</span>
              <strong>{{ formatUsd(simulatedPositions.reduce((sum, item) => sum + item.budget, 0)) }}</strong>
            </div>
            <div class="summary-card">
              <span>Open exposures</span>
              <strong>{{ simulatedPositions.length }}</strong>
            </div>
          </div>

          <div class="dense-table">
            <div class="dense-row dense-head">
              <span>Exposure</span>
              <span>Risk tag</span>
              <span>Notional</span>
              <span>Budget</span>
              <span>Status</span>
            </div>
            <div v-for="position in simulatedPositions" :key="position.exposure" class="dense-row">
              <span>{{ position.exposure }}</span>
              <span>{{ position.riskTag }}</span>
              <span>{{ formatUsd(position.notional) }}</span>
              <span>{{ formatUsd(position.budget) }}</span>
              <span><em>{{ position.status }}</em></span>
            </div>
          </div>
        </section>

        <section v-else-if="activeView === 'receipts'" class="feature-page" aria-label="Receipts">
          <div class="page-header">
            <div>
              <p>Receipts</p>
              <h2>Quote receipt registry</h2>
            </div>
            <button class="secondary-compact" :disabled="!quote" @click="quote && copyText(quote.receipt.quoteHash)">
              {{ receiptCopied ? 'Copied hash' : 'Copy current hash' }}
            </button>
          </div>

          <div class="receipt-ledger">
            <article v-for="receipt in receiptRows" :key="receipt.id" class="ledger-card">
              <div>
                <span>{{ receipt.id }}</span>
                <strong>{{ receipt.status }}</strong>
              </div>
              <p>{{ receipt.hash }}</p>
              <dl>
                <div>
                  <dt>Chain</dt>
                  <dd>{{ receipt.chain }}</dd>
                </div>
                <div>
                  <dt>Expires</dt>
                  <dd>{{ receipt.expires }}</dd>
                </div>
              </dl>
              <button class="secondary-action" @click="copyText(receipt.hash)">
                <Clipboard :size="16" />
                {{ receiptCopied === receipt.hash ? 'Copied' : 'Copy hash' }}
              </button>
            </article>
          </div>
        </section>

        <section v-else-if="activeView === 'markets'" class="feature-page" aria-label="Markets">
          <div class="page-header">
            <div>
              <p>Markets</p>
              <h2>Prediction-market watchlist</h2>
            </div>
            <button class="secondary-compact" @click="createQuote">Refresh matches</button>
          </div>

          <div class="dense-table market-watchlist">
            <div class="dense-row dense-head">
              <span>Venue</span>
              <span>Market</span>
              <span>Probability</span>
              <span>Liquidity</span>
              <span>Basis risk</span>
            </div>
            <div v-for="market in marketRows" :key="`${market.venue}-${market.market}`" class="dense-row">
              <span>{{ market.venue }}</span>
              <span>{{ market.market }}</span>
              <span>{{ formatProbability(market.probability) }}</span>
              <span>{{ formatUsd(market.liquidityUsd) }}</span>
              <span>{{ market.basisRisk }}</span>
            </div>
          </div>
        </section>

        <section v-else-if="activeView === 'alerts'" class="feature-page" aria-label="Alerts">
          <div class="page-header">
            <div>
              <p>Alerts</p>
              <h2>Risk monitoring rules</h2>
            </div>
            <button class="secondary-compact">Test alerts</button>
          </div>

          <div class="rule-list">
            <label v-for="rule in alertRules" :key="rule.name" class="rule-card">
              <input v-model="rule.enabled" type="checkbox" />
              <div>
                <strong>{{ rule.name }}</strong>
                <span>{{ rule.channel }} channel · {{ rule.enabled ? 'enabled' : 'paused' }}</span>
              </div>
            </label>
          </div>
        </section>

        <section v-else class="feature-page" aria-label="Settings">
          <div class="page-header">
            <div>
              <p>Settings</p>
              <h2>Quote desk controls</h2>
            </div>
            <button class="secondary-compact" @click="activeView = 'quote'">Back to desk</button>
          </div>

          <div class="settings-grid">
            <label class="field">
              <span>Default chain</span>
              <select v-model="settings.defaultChain">
                <option>Robinhood Chain Testnet</option>
                <option>Arbitrum Sepolia</option>
              </select>
            </label>
            <label class="field">
              <span>Quote mode</span>
              <select v-model="settings.quoteMode">
                <option>Quote-only</option>
                <option>Receipt preparation only</option>
              </select>
            </label>
            <label class="field">
              <span>Max budget percent</span>
              <input v-model.number="settings.maxBudgetPercent" type="number" min="1" max="12" />
            </label>
            <label class="rule-card setting-toggle">
              <input v-model="settings.recordPrivateNotes" type="checkbox" />
              <div>
                <strong>Record private operator notes</strong>
                <span>Off by default for hackathon demos.</span>
              </div>
            </label>
          </div>
        </section>

        <footer v-if="quote" class="desk-footer">
          <div>
            <strong>Networks</strong>
            <span>Robinhood Chain Testnet</span>
            <span>Arbitrum Sepolia</span>
          </div>
          <div>
            <strong>Risk notice</strong>
            <span>Prediction markets carry liquidity and resolution risk. Nothing here is financial advice.</span>
          </div>
          <div>
            <strong>Receipt hash</strong>
            <code>{{ quote.receipt.quoteHash }}</code>
          </div>
        </footer>
      </div>
    </section>
  </main>
</template>
