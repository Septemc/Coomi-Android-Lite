<script setup lang="ts">
/**
 * 子代理：给审查、编码等任务指定不同模型。
 * 只保留一个配置页，所有任务仍走同一个 Agent 循环。
 */
import { computed, onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import { apiGet, apiSend } from '@/bridge/http'
import { useConfigStore } from '@/stores/config'
import PageHead from '@/components/PageHead.vue'
import CoomiIcon from '@/components/CoomiIcon.vue'

interface SubAgentProfile {
  id: string
  name: string
  selector: string
  instructions: string
  tools: string[]
  max_parallel: number
}

const MAX_AGENTS = 8
const MAX_PARALLEL = 5
const READ_ONLY_TOOLS = [
  'read_file',
  'list_dir',
  'grep_files',
  'view_image',
  'show_image',
  'web_search',
  'fetch',
  'list_skills',
  'read_skill',
  'list_mcp',
  'memory_list',
  'memory_read',
  'memory_search',
  'get_loop',
  'wait_agent',
]

const router = useRouter()
const config = useConfigStore()
const agents = ref<SubAgentProfile[]>([])
const defaultId = ref('')
const loading = ref(false)
const saving = ref(false)
const error = ref('')
const saved = ref(false)

const selectorOptions = computed(() => {
  const options: { selector: string; label: string }[] = []
  const seen = new Set<string>()
  const push = (selector: string, label: string) => {
    if (!selector || seen.has(selector)) return
    seen.add(selector)
    options.push({ selector, label })
  }
  for (const provider of config.providers) {
    push(provider.id, `${provider.name} · 默认模型`)
    const models = provider.models.length > 0 ? provider.models : (provider.model ? [provider.model] : [])
    for (const model of models) push(`${provider.id}:${model}`, `${provider.name} · ${model}`)
  }
  for (const agent of agents.value) push(agent.selector, agent.selector)
  return options
})

function isReadOnly(agent: SubAgentProfile): boolean {
  return agent.tools.length > 0
}

function onReadOnlyChange(agent: SubAgentProfile, event: Event) {
  const checked = (event.target as HTMLInputElement).checked
  agent.tools = checked ? [...READ_ONLY_TOOLS] : []
}

function addAgent() {
  if (agents.value.length >= MAX_AGENTS) return
  let index = agents.value.length + 1
  while (agents.value.some(agent => agent.id === `agent${index}`)) index += 1
  const selector = config.currentProviderId && config.currentModel
    ? `${config.currentProviderId}:${config.currentModel}`
    : (selectorOptions.value[0]?.selector ?? '')
  agents.value.push({
    id: `agent${index}`,
    name: '',
    selector,
    instructions: '',
    tools: [],
    max_parallel: 2,
  })
}

function removeAgent(index: number) {
  const [removed] = agents.value.splice(index, 1)
  if (removed && defaultId.value === removed.id) defaultId.value = ''
}

async function load() {
  loading.value = true
  error.value = ''
  try {
    await config.fetchProviders()
    const data = await apiGet<{ default: string; agents: SubAgentProfile[] }>('/api/settings/subagents')
    defaultId.value = data.default ?? ''
    agents.value = (data.agents ?? []).map(agent => ({
      id: agent.id ?? '',
      name: agent.name ?? '',
      selector: agent.selector ?? '',
      instructions: agent.instructions ?? '',
      tools: Array.isArray(agent.tools) ? agent.tools : [],
      max_parallel: Number(agent.max_parallel) || 2,
    }))
  } catch (e) {
    error.value = `读取子代理配置失败：${e instanceof Error ? e.message : String(e)}`
  } finally {
    loading.value = false
  }
}

async function save() {
  error.value = ''
  saved.value = false
  const seen = new Set<string>()
  for (const agent of agents.value) {
    const id = agent.id.trim()
    if (!id) {
      error.value = '子代理 ID 不能为空'
      return
    }
    if (seen.has(id.toLowerCase())) {
      error.value = `子代理 ID 重复：${id}`
      return
    }
    seen.add(id.toLowerCase())
    if (!agent.selector) {
      error.value = `子代理 ${id} 还没有选择模型`
      return
    }
    if (!Number.isInteger(agent.max_parallel) || agent.max_parallel < 1 || agent.max_parallel > MAX_PARALLEL) {
      error.value = `子代理 ${id} 的并发需为 1 - ${MAX_PARALLEL}`
      return
    }
  }
  saving.value = true
  try {
    await apiSend('/api/settings/subagents', 'PUT', {
      default: defaultId.value,
      agents: agents.value.map(agent => ({
        id: agent.id.trim(),
        name: agent.name.trim(),
        selector: agent.selector,
        instructions: agent.instructions.trim() || null,
        tools: agent.tools,
        max_parallel: agent.max_parallel,
      })),
    })
    saved.value = true
  } catch (e) {
    error.value = `保存失败：${e instanceof Error ? e.message : String(e)}`
  } finally {
    saving.value = false
  }
}

onMounted(load)
</script>

<template>
  <div class="page">
    <PageHead title="子代理" @back="router.back()" />
    <main class="body">
      <p class="intro">
        把任务交给不同模型的子代理。不配置时，子代理沿用当前会话的主模型；
        配置后，主 Agent 可以用 <code>agent_id</code> 指定审查、编码等专用模型。
      </p>

      <p class="sec-label">默认子代理</p>
      <div class="group">
        <label class="field">
          <span class="field-label">未指定 agent_id 时使用</span>
          <select v-model="defaultId">
            <option value="">跟随主模型</option>
            <option v-for="agent in agents" :key="agent.id" :value="agent.id">
              {{ agent.name || agent.id }}
            </option>
          </select>
        </label>
      </div>

      <p class="sec-label">已配置 {{ agents.length }} / {{ MAX_AGENTS }}</p>
      <p v-if="loading" class="empty">正在读取…</p>
      <p v-else-if="agents.length === 0" class="empty">
        还没有子代理。新增一个“审查”子代理，选择只读工具和另一个模型，就能做 coder / reviewer 协作。
      </p>

      <div class="agent-list">
        <section v-for="(agent, index) in agents" :key="index" class="agent-card">
          <div class="agent-head">
            <span class="agent-index">{{ index + 1 }}</span>
            <input v-model="agent.name" class="name-input" placeholder="名称，如 审查 / 编码" />
            <button class="icon-btn danger" aria-label="删除子代理" @click="removeAgent(index)">
              <CoomiIcon name="trash" :size="16" />
            </button>
          </div>

          <label class="field">
            <span class="field-label">ID（工具调用时传给 agent_id）</span>
            <input v-model="agent.id" class="mono" placeholder="reviewer" autocapitalize="off" autocorrect="off" />
          </label>

          <label class="field">
            <span class="field-label">模型</span>
            <select v-model="agent.selector">
              <option value="" disabled>选择模型</option>
              <option v-for="option in selectorOptions" :key="option.selector" :value="option.selector">
                {{ option.label }}
              </option>
            </select>
          </label>

          <label class="field">
            <span class="field-label">附加指令（可选）</span>
            <textarea
              v-model="agent.instructions"
              rows="2"
              placeholder="例如：只读审查，只报告问题，不改文件。"
            />
          </label>

          <div class="field-row">
            <label class="check">
              <input
                type="checkbox"
                :checked="isReadOnly(agent)"
                @change="onReadOnlyChange(agent, $event)"
              />
              <span>只读工具集</span>
            </label>
            <label class="parallel">
              <span>并发</span>
              <input v-model.number="agent.max_parallel" type="number" min="1" :max="MAX_PARALLEL" step="1" />
            </label>
          </div>
        </section>
      </div>

      <button v-if="agents.length < MAX_AGENTS" class="add-btn" type="button" @click="addAgent">
        <CoomiIcon name="plus" :size="16" /> 新增子代理
      </button>

      <p v-if="error" class="msg err">{{ error }}</p>
      <p v-else-if="saved" class="msg ok">已保存</p>

      <div class="actions">
        <button class="primary-action" type="button" :disabled="saving || loading" @click="save">
          {{ saving ? '保存中…' : '保存' }}
        </button>
      </div>
    </main>
  </div>
</template>

<style scoped>
.page { display: flex; flex-direction: column; height: 100%; background: var(--page); }
.body { flex: 1; overflow-y: auto; padding: 14px 12px calc(var(--safe-bottom) + 24px); }
.intro { margin: 0 4px 14px; font-size: 12.5px; line-height: 1.65; color: var(--text-3); }
.intro code { padding: 1px 4px; border-radius: 4px; background: var(--fill); color: var(--text-2); }
.sec-label { margin: 16px 0 0; }
.sec-label:first-of-type { margin-top: 0; }
.group { border-radius: var(--r-card); background: var(--bg); box-shadow: var(--shadow-1); overflow: hidden; }
.field { display: flex; flex-direction: column; gap: 6px; padding: 11px 13px; }
.field + .field { border-top: 1px solid var(--border); }
.field-label { font-size: 11.5px; color: var(--text-3); }
.field input, .field select, .field textarea {
  width: 100%; min-height: 38px; padding: 8px 10px;
  border: 1px solid var(--border-strong); border-radius: 7px;
  background: var(--page); color: var(--text); font-size: 13.5px;
}
.field textarea { resize: vertical; line-height: 1.5; }
.field .mono { font-family: var(--mono, ui-monospace, SFMono-Regular, Menlo, monospace); }
.agent-list { display: flex; flex-direction: column; gap: 10px; margin-top: 8px; }
.agent-card { border-radius: var(--r-card); background: var(--bg); box-shadow: var(--shadow-1); overflow: hidden; }
.agent-head { display: flex; align-items: center; gap: 9px; padding: 10px 12px; border-bottom: 1px solid var(--border); }
.agent-index {
  display: grid; place-items: center;
  width: 22px; height: 22px; border-radius: 50%;
  background: var(--blue-soft); color: var(--blue); font-size: 11.5px; font-weight: 700;
}
.name-input {
  flex: 1; min-width: 0; height: 34px; padding: 0 8px;
  border: 0; border-radius: 6px; background: transparent;
  color: var(--text); font-size: 14.5px; font-weight: 650;
}
.name-input:focus { background: var(--fill); outline: none; }
.icon-btn { display: grid; place-items: center; width: 32px; height: 32px; border-radius: 7px; background: transparent; color: var(--text-3); }
.icon-btn.danger { color: var(--red, #d4574e); }
.icon-btn:active { background: var(--fill); }
.field-row { display: flex; align-items: center; justify-content: space-between; gap: 12px; padding: 10px 13px; border-top: 1px solid var(--border); }
.check { display: flex; align-items: center; gap: 8px; font-size: 13px; color: var(--text-2); }
.check input { width: 16px; height: 16px; accent-color: var(--blue); }
.parallel { display: flex; align-items: center; gap: 8px; font-size: 12.5px; color: var(--text-3); }
.parallel input { width: 64px; height: 34px; padding: 0 8px; border: 1px solid var(--border-strong); border-radius: 6px; background: var(--page); color: var(--text); text-align: right; }
.empty { margin: 8px 4px 0; padding: 14px; border-radius: var(--r-card); background: var(--bg); color: var(--text-3); font-size: 12.5px; line-height: 1.6; }
.add-btn {
  display: flex; align-items: center; justify-content: center; gap: 6px;
  width: 100%; min-height: 44px; margin-top: 10px;
  border: 1px dashed var(--border-strong); border-radius: var(--r-card);
  background: transparent; color: var(--blue); font-size: 13.5px; font-weight: 600;
}
.add-btn:active { background: var(--blue-soft); }
.msg { margin: 12px 4px 0; font-size: 12.5px; }
.msg.err { color: var(--red, #d4574e); }
.msg.ok { color: var(--green, #2f9e6b); }
.actions { margin-top: 16px; }
.primary-action {
  width: 100%; min-height: 46px; border-radius: var(--r-card);
  background: var(--blue); color: #fff; font-size: 14.5px; font-weight: 650;
}
.primary-action:disabled { opacity: .55; }
</style>
