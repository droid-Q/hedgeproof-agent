<script setup lang="ts">
import { computed, onMounted, reactive, ref } from 'vue'
import {
  Activity,
  BadgeCheck,
  Banknote,
  Clipboard,
  FileCheck2,
  Network,
  Play,
  ShieldCheck,
  Sparkles,
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

const apiBase = import.meta.env.VITE_API_BASE_URL || ''
const scenarios = ref<DemoScenario[]>([])
const quote = ref<QuoteResponse | null>(null)
const loading = ref(false)
const errorMessage = ref('')
const copied = ref(false)

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
      <div>
        <p class="product-line">HedgeProof Agent</p>
        <h1>Agentic RWA hedge quote desk</h1>
      </div>
      <div class="status-strip" aria-label="System status">
        <span><Activity :size="16" /> Quote-only</span>
        <span><Network :size="16" /> Robinhood Chain ready</span>
        <span><ShieldCheck :size="16" /> No custody</span>
      </div>
    </header>

    <section class="workspace-grid">
      <aside class="panel intake-panel" aria-label="Risk exposure intake">
        <div class="panel-heading">
          <Sparkles :size="19" />
          <div>
            <h2>Risk intake</h2>
            <p>Describe the exposure the agent should hedge.</p>
          </div>
        </div>

        <label class="field">
          <span>Exposure</span>
          <textarea v-model="form.exposure" rows="8" />
        </label>

        <div class="field-grid">
          <label class="field">
            <span>Asset type</span>
            <input v-model="form.assetType" />
          </label>
          <label class="field">
            <span>Chain</span>
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
          {{ loading ? 'Generating quote' : 'Generate hedge quote' }}
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
          <div class="market-table">
            <div class="table-row table-head">
              <span>Venue</span>
              <span>Market</span>
              <span>Prob.</span>
              <span>Liquidity</span>
            </div>
            <div v-for="market in quote.marketMatches" :key="market.market" class="table-row">
              <span>{{ market.venue }}</span>
              <span>
                {{ market.market }}
                <small>{{ market.basisRisk }}</small>
              </span>
              <span>{{ formatProbability(market.probability) }}</span>
              <span>{{ formatUsd(market.liquidityUsd) }}</span>
            </div>
          </div>
        </div>

        <div v-if="quote" class="data-section">
          <div class="section-heading">
            <h3>Hedge legs</h3>
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
              <h2>Quote receipt</h2>
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
            {{ copied ? 'Copied' : 'Copy receipt args' }}
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
  </main>
</template>
