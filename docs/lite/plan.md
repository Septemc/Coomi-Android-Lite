# CoomiLite：基于 v1.3.2 的减法重构方案

> 状态：方案稿，未开始实施
> 基线：`v1.3.2`（commit `cd7a70ee`）
> 参考：`F:\_WorkSpace\Projects\AILab\AgentLab\reference_code\pi`

## 0. 一句话结论

**不要从 1.5 做减法，从 1.3.2 做加法。**
保留 1.3.2 的“单会话 + 单 Agent + 一套工具”心智模型；把 1.4/1.5 的新能力按“运行时接口”而不是“产品页面”回植；用硬预算阻止二次膨胀。

## 1. 诊断

### 1.1 规模对比

| 指标 | v1.3.2 | 当前 HEAD | 变化 |
|---|---:|---:|---:|
| Rust 源码 | 26,488 行 | 56,108 行 | +112% |
| Web 源码 | 10,153 行 | 24,581 行 | +142% |
| 源码文件 | 84 | 205 | +144% |
| Web 路由 / 页面 | 13 / 12 | 45+ / 39 | +246% |
| 内置工具 | 35 | 46+ | +31% |
| 引擎 API | 较少 | 约 190 条 | — |
| `web.rs` | 5,334 行 | 12,395 行 | +132% |
| APK | ≈191 MB | ≈338 MB | +77% |

### 1.2 根因

1.3.2 的“顺手”不是因为功能少，而是因为**只有一个心智模型**：

> 一个会话里，一个 Agent 用一套工具，模型可换，历史可恢复。

1.5 的臃肿不是因为功能多，而是因为**每个功能都变成了一等产品面**：

- 8 条执行路径：聊天、工作室、群聊、协同、工作流、数字生命、子代理、TUI。
- 45+ 个路由、39 个页面、190 条 API。
- 12,395 行的 `web.rs` 同时承载 8 类业务。
- 概念名词爆炸：工作室、成员、工单、群聊、协同、工作流、生命体、提示词库、超载、体验计划……
- 每个名词都带来独立的数据模型、存储、页面、store、API、提示词分支。

**结论：要砍的不是功能，是“功能的产品面”。**

## 2. 总策略

### 2.1 从 tag 拉分支做加法

- 从 `v1.3.2` 拉长期分支 `lite/2.0`，`main` 保持 1.5 Full 不动。
- 只 cherry-pick 单点提交/文件，不做整目录合并。
- 维护 `docs/lite/backports.md`：每条回植记录来源 commit、改动文件、测试、回滚方式。
- 1.3.2 作为“手感基线”：任何改动都要过一组固定验收任务。

### 2.2 三条原则

1. **一个执行路径**：所有能力最终都走 `Agent::run_turn`。子代理只是“另一个 profile 的同一个循环”。不新增 runner。
2. **名词零新增**：不创造新的一级产品名词。需要新能力时，优先映射到已有名词（会话、任务、工具、技能、MCP、模型服务、子代理、记忆、身份、运行环境、文件、设置）。
3. **功能只有三种形态**：设置项、工具、配置（Profile）。如果一个需求不能落成这三种形态，就不做页面。

### 2.3 复杂度预算（硬约束）

| 项目 | 预算 | 1.3.2 | HEAD |
|---|---:|---:|---:|
| Rust 源码 | ≤ 20,000 行 | 26,488 | 56,108 |
| Web 源码 | ≤ 8,000 行 | 10,153 | 24,581 |
| 一级入口 | 5 个 | — | — |
| 页面总数 | ≤ 14 个 | 12 | 39 |
| 内置工具 | ≤ 24 个 | 35 | 46+ |
| API 路由 | ≤ 70 条 | — | ~190 |
| 执行路径 | 2 条（会话 / 子代理） | 3 条 | 8 条 |
| 新用户名词 | 0 个 | — | — |
| APK | ≤ 210 MB | ~191 MB | ~338 MB |

任何 PR 想加页面/工具/名词，必须先在预算内移除等量东西，或在 PR 里写明例外理由。

## 3. 功能取舍

### 3.1 保留：1.3.2 的核心

- 对话：流式文本、推理、工具卡、审批、问询、图片预览。
- 会话：历史、置顶、标题、草稿、断线重连、后台继续、5 会话并行。
- 模型服务：OpenAI 兼容、OpenAI Responses、Anthropic、Gemini；能力开关与降级。
- 工具：文件读写/编辑/补丁/搜索、shell、持久进程、网页搜索/抓取、图片、问询、文件导入导出、计划、Loop、子代理、Skill、MCP、记忆。
- 记忆：global / project / local 三层，手动管理。
- 上下文压缩：自动阈值 + 手动 `/compact`。
- 技能与 MCP：内置目录 + 自定义配置；按需加载。
- 身份定位（Persona）：保留，作为“数字生命体”的替代。
- 文件管理、外观主题、备份恢复、更新。
- 运行环境页：改成 Termux 环境状态 + 常用工具检查/安装，不再安装 proot。
- Hooks：保留后端能力，降级到“高级设置”，不占一级入口。

### 3.2 回植：新版特性（按优先级）

| 优先级 | 特性 | 来源 | 价值 | 大致成本 |
|---|---|---|---|---|
| P0 | 每会话独立模型 + 按 Provider 分组的模型选择 | 1.4.5 | 修复多会话互相串模型 | 小 |
| P0 | 子代理可选模型（命名 Profile） | 1.4.0 简化 | 解决“子代理只能用主模型”的核心痛点 | 中 |
| P0 | Provider 能力缓存 + 中文错误映射 | HEAD `capabilities.rs` + 反馈 B3/B4 | 减少 400/401/402/429 支持成本 | 中 |
| P0 | 编辑可靠性 + 文件回撤 | HEAD `file_journal.rs` + 反馈 A3 | edit 失败是 P0 痛点 | 中 |
| P0 | 压缩摘要保留进度 + 失败降级截断 | 反馈 A1/A2 | 压缩后不“重头开始” | 小 |
| P0 | 每会话进程隔离 + 取消不误杀 | 反馈 + HEAD `processes.rs` | 5 会话并行的稳定性 | 中 |
| P0 | 后台任务通知 / PiP | 1.4.5 | 移动端必需 | 中 |
| P1 | 计划进度、指令确认、推理关闭档 | 1.5.0-test.5 | 手感与可控性 | 小 |
| P1 | Anthropic / Gemini 流式 | HEAD | 多 Provider 体验 | 中 |
| P1 | 模型能力可视 + 上下文环 | HEAD | 降低理解成本 | 小 |
| P1 | Skill 路由（内部） | HEAD `skill_router.rs` | 控制系统提示词体积 | 中 |
| P1 | MCP 可视化配置 | 1.5.0-test.5 | 降低 MCP 安装门槛 | 小 |
| P1 | 会话搜索/导出（简化） | HEAD `data_tools.rs` | 长会话可用性 | 小 |
| P2 | 悬浮窗 / Shizuku / 无障碍 | HEAD | 差异化，但非核心 | 大 |
| P2 | Git 面板 / AI Git | HEAD | 可用 shell git 替代 | 大 |
| P2 | 本地模型 / OCR / TTS | HEAD | 端侧增强，非核心 | 大 |
| P2 | 计费/用量/余额 | HEAD | 商业化，非核心 | 中 |

回植原则：**只搬逻辑，不搬页面**。例如 Git 先不做页面，保留 shell git；用量先不做计费页，只在会话里显示 token。

### 3.3 砍掉：不再进入 lite

| 功能 | 约 LOC（Rust + 前端） | 理由 |
|---|---:|---|
| Proot / Runtime V2 / Ubuntu / Debian rootfs | ~2,100 Rust + ~300 Java + 448 MB 资产 | 用户明确要求；1.3.2 无 proot 仍顺手；APK 直接减 150 MB+ |
| 自定义迭代 / CoomiDev Build Kit | ~300 Rust + 142 Vue + 资产 | proot 的附属品 |
| 数字生命体 / Life sidecar | ~2,000 Rust + 2,100 Python + 747 Vue | 高耦合；用“身份 + 记忆”替代 |
| 工作室 / Studio | ~790 Rust + 955 Vue | 用“多会话 + 子代理”替代 |
| 群聊 / AI 工作台 | ~195 Rust + 39 Vue | 同上 |
| 协同工作台 / Collab | ~250 Rust + 265 Vue | 同上 |
| 工作流 / Workflow / Cron | ~3,700 Rust + 537 Vue | 用 Loop + 子代理 + 计划模式替代 |
| 提示词库 | ~330 | 用身份、子代理 instructions、技能替代 |
| 超载模型 | ~500 | 用会话模型 + 推理强度替代 |
| TUI / terminal_ui | ~7,400 | Android 只用 WebView；保留 `coomi exec` 无头 CLI |
| Git 面板 / AI Git / Ops / Data / Restore / Usage / Maintenance / Browser | ~10,700 Rust + ~4,700 Vue | 首版不做；shell git + 文件管理覆盖 |
| Experience / UX Program / Telemetry / 自动反馈上传 | ~1,800 | 隐私与维护成本；保留本地崩溃日志与手动反馈入口 |
| 远程扩展市场 registry | ~1,000 | 内置目录 + 自定义配置足够 |
| DeepSeek / 智谱账号登录 | ~1,700 Rust + 589 Vue | 保留 API Key 预设即可；登录 UI 在 HEAD 已是死代码 |
| 本地模型 / OCR / TTS / UI 自动化 | ~1,000 | 非核心 |
| 悬浮窗 / 无障碍 / Shizuku | ~2,500 Java | 第二阶段可选模块 |
| Demo 模式 | ~400 | 生产构建排除即可 |

> 注意：上表是“不进入 lite 主线”，不是“永久删除”。Git、悬浮窗等可以作为独立插件/后续版本，但必须先过预算与准入规则。

### 3.4 概念映射：旧产品名词 → 运行时接口

| 1.5 的产品概念 | Lite 的运行时实现 | 用户看到什么 |
|---|---|---|
| 工作室 / 成员 / 工单 | `AgentProfile` + 多会话 | 会话列表；设置里的“子代理” |
| 群聊 / 工作台 | 多会话 + 不同 Profile | 不需要专门页面 |
| 协同 / 改码审查 | coder / reviewer 两个 `AgentProfile` | 子代理设置；任务里看结果 |
| 提示词库 | `PromptSection` / `AgentProfile.instructions` / Skill | 身份、子代理、技能 |
| 数字生命体 | `AgentProfile.instructions` + Memory | 身份、记忆 |
| 超载模型 | 会话模型 + `reasoning_effort` | 模型选择、推理强度 |
| 工作流 | Loop + 子代理 + 计划模式 | 任务 |
| 体验改进计划 | 本地日志 + 手动反馈 | 关于/反馈 |
| 远程市场 | 内置目录 + 自定义配置 | 技能与 MCP |

**关键：这些概念不再有独立的数据模型、页面、API、store。**

## 4. 目标架构

### 4.1 分层

```
Android 壳（Termux bootstrap + WebView）
        │ HTTP / WS
        ▼
server（axum，按域拆模块）
  ├─ sessions / chat / tasks
  ├─ providers / models / subagents
  ├─ files / skills / mcp / memory
  ├─ settings / backup / update
  └─ ws（事件协议）
        │
        ▼
harness（PromptBuilder + AgentProfile + Skills/MCP/Memory）
        │
        ▼
engine（Agent loop / Session log / Compaction / Events）
        │
        ├─ providers（OpenAI / Responses / Anthropic / Gemini）
        ├─ tools（小工具集）
        └─ exec（FileSystem / Shell / Process，Termux 后端）
```

单向依赖：`server → harness → engine → providers/exec`。engine 不认识任何产品概念。

### 4.2 四个核心接口

1. **`AgentProfile`**

   ```rust
   struct AgentProfile {
       id: String,
       name: String,
       provider_id: String,
       model: String,
       instructions: Option<String>,
       skills: Vec<String>,        // 允许的技能；空 = 全部
       tools: ToolLoadout,         // 允许的工具集
       memory_scope: MemoryScope,  // none / project / global
       max_parallel: usize,
   }
   ```

   用途：主会话模型、身份、子代理、审查员，全部用同一个结构。
   UI 上只暴露“身份”和“子代理”两个已有名词。

2. **`PromptSection`**

   ```rust
   struct PromptSection {
       id: &'static str,
       priority: u8,
       content: String,
   }
   ```

   固定 6 段：身份 → 运行环境 → 项目指令 → 记忆 → 技能索引 → MCP 清单。
   每段有 token 上限；不再有隐藏的 strict / life / studio 注入。

3. **`ToolLoadout`**

   ```rust
   enum ToolLoadout {
       Full,
       ReadOnly,
       Plan,
       Custom(Vec<String>),
   }
   ```

   计划模式 = `Plan`（只读 + `update_plan`）；审查子代理 = `ReadOnly`；主会话 = `Full`。
   不再为“计划模式 / 审查模式”写独立状态机。

4. **`RunContext`**

   ```rust
   struct RunContext {
       session_id: String,
       cwd: PathBuf,
       permission: PermissionMode,
       cancel: CancellationToken,
       observer: Arc<dyn AgentObserver>,
       approval: Arc<dyn ApprovalHandler>,
       processes: Arc<ProcessRegistry>,
       file_journal: Option<Arc<FileJournal>>,
   }
   ```

   所有工具只依赖它，不直接碰 Android/Termux。

### 4.3 会话：append-only 日志

现状：v1.3.2 每次 checkpoint 克隆整个 session、序列化、写临时文件、rename，全局写锁。长会话/多会话会卡。

Lite：

- 每会话一个 JSONL 文件，append-only。
- entry 类型：`message`、`model_change`、`compaction`、`custom`（UI 状态）、`usage`。
- 投影：读取时从最新 compaction 的 `first_kept_entry_id` 开始拼上下文。
- 压缩：结构化摘要（目标 / 已完成 / 未完成 / 下一步 / 关键上下文）+ 保留最近 N tokens；压缩请求失败 → 直接截断，不中断会话。
- 迁移：启动时把 1.3.2 / 1.5 的 `<uuid>.json` 一次性转换为 JSONL，保留原文件备份。
- 写入：单会话串行 append，不阻塞其他会话。

### 4.4 执行环境：没有 proot

- `FileSystem` / `Shell` / `Process` 三个 trait。
- 后端：`TermuxBackend`（默认）、`HostShellBackend`（桌面调试）、`ShizukuBackend`（可选，P2）。
- 路径：只保留 workspace / home / tmp 三个根，不再有 `/workspace`、`/home/coomi`、`/opt/coomi-dev` guest 别名。
- 进程：按 `session_id` 分区注册；取消会话只杀本会话的进程组。
- 环境页：显示 Termux bootstrap、shell 路径、git/python/node 可用性；提供一键 `pkg install` 常用工具（这是 proot 的替代，不是新功能）。
- 安全：不再声称沙箱；保留路径边界、危险命令检查、审批模式；明确“应用权限 = 工具权限”。

### 4.5 子代理：模型选择是核心

v1.3.2 的 `AgentScheduler::new` 只接收一个 `ProviderConfig`，`run_agent` 固定用 `self.provider`，所以所有子代理只能用主模型。这是要修的第一优先级。

Lite 设计：

- `~/.coomi/config/agents.json`（或 settings 的 `agents` 段）：

  ```json
  {
    "default": "main",
    "agents": [
      {
        "id": "reviewer",
        "name": "审查",
        "provider_id": "deepseek",
        "model": "deepseek-v4-pro",
        "instructions": "你是只读审查员，只报告问题，不改文件。",
        "tools": ["read_file", "grep_files", "list_dir", "view_image"],
        "max_parallel": 2
      },
      {
        "id": "coder",
        "name": "编码",
        "provider_id": "zhipu",
        "model": "glm-5",
        "tools": ["read_file", "write_file", "edit_file", "shell", "grep_files"],
        "max_parallel": 2
      }
    ]
  }
  ```

- 工具 schema：`spawn_agent(task, agent_id?, fork_turns?)`。
  - `agent_id` 省略 → 用 `default`（主模型），保持 1.3.2 行为。
  - 指定 → 用该 Profile 的 provider/model/tools/instructions。
- 子代理有独立上下文、独立进程注册、独立 usage；`wait_agent` 返回状态 + 输出 + usage。
- 权限：继承主会话权限模式，但可被 `ToolLoadout` 收窄（reviewer 只读）。
- 并发：全局上限 5，Profile 可设 `max_parallel`。
- 成本：任务卡显示 Profile 名称和模型；会话页汇总 token。
- 这同时替代 1.4.5 的“改码审查协作模式”：不建新模式，只建 coder / reviewer 两个子代理。

### 4.6 Provider

- 统一 trait：`complete_stream(request) -> EventStream`。
- 适配器：OpenAI chat/completions（SSE）、OpenAI Responses（SSE）、Anthropic messages（SSE）、Gemini streamGenerateContent（SSE）。
- 能力缓存：`provider_id + model` 维度的 `ModelCapabilityProfile`，持久化到配置；400 时只探测一次，之后不再发不支持字段。
- 错误映射：401 / 402 / 403 / 404 / 429 / 400 → 中文原因 + 下一步动作（检查 Key、充值、等待、改 Base URL、换模型）。
- 每会话模型：`Session.provider_id/model` 优先，不再改全局 active。

## 5. 信息架构

一级入口 5 个：

```
对话（默认） / 会话 / 任务 / 文件 / 设置
```

设置二级页 8 个：

```
模型服务 / 子代理 / 技能与 MCP / 记忆 / 身份 /
运行环境 / 外观 / 数据与备份 / 关于与更新
```

（“高级”折叠：Hooks、诊断、日志。）

对话页只保留：

- 消息流、工具卡、审批、问询、计划进度、上下文环。
- 输入框：模型选择、权限模式、计划模式、推理强度、附件。
- 不再有“超载”按钮、模式切换条、工作室入口。

## 6. 1.3.2 必须一起修的老问题

| 优先级 | 问题 | 修法 |
|---|---|---|
| P0 | 取消一个会话会杀掉所有会话的 shell | 进程注册表按 `session_id` 分区 |
| P0 | `auto` 与 `full` 权限实际等价 | 真正的三档：Ask / Auto（写文件自动、shell 询问）/ Full |
| P0 | `shell` 超时只杀直接子进程 | 进程组 + `kill -9 -pgid`，Android 用 native kill |
| P0 | 会话整文件写入 + 全局锁 | append-only JSONL |
| P0 | Provider 能力降级不持久 | capability cache |
| P0 | 模型选择是全局的，多会话串模型 | 每会话模型 |
| P0 | `edit_file` 失败率高 | 读后写校验、内容锚点、最相似候选、同参重试放行“参数已改” |
| P0 | 压缩后丢进度 / 压缩失败中断 | 结构化摘要 + 截断降级 |
| P1 | Anthropic / Gemini 无流式 | SSE |
| P1 | 图片降级不可恢复 | 按 provider + model 能力缓存 |
| P1 | token 估算 `len/4` | 字符类别加权 + provider usage 校准 |
| P1 | MCP 工具全部按 Mutating 串行 | `readOnlyHint` / `destructiveHint` 映射 |
| P1 | 更新源分裂（Rust TUI / Android / Web） | 只保留 Android 更新源 |
| P1 | targetSdk 28 + Termux bootstrap 脆弱 | 启动自检、可执行位修复、bootstrap 版本校验 |
| P2 | API Key 明文 | Android Keystore 加密（可选） |
| P2 | WS token 在 URL | Header / 子协议（可选） |

## 7. 分阶段计划

| 阶段 | 内容 | 出口条件 | 预估 |
|---|---|---|---|
| P0 基线 | 拉 `lite/2.0`；真机跑通 1.3.2；写预算/准入/验收清单 | 1.3.2 APK 可构建、可安装、核心任务通过 | 2-3 天 |
| P1 去臃肿 | 砍 TUI / demo / telemetry / 远程市场；拆 `web.rs`；设置信息架构 | Rust ≤ 22k，Web ≤ 9k，页面 ≤ 14 | 1 周 |
| P2 执行环境 | 无 proot；ExecutionEnv；每会话进程；Termux 环境页；修 4 个 P0 执行问题 | 5 会话并行、取消不误杀、长任务稳定 | 1 周 |
| P3 Agent 内核 | JSONL 会话；压缩；Provider 能力缓存/错误映射；流式；工具契约 | 长会话压缩后可继续；400 不再重复 | 2-3 周 |
| P4 特性回植 | 每会话模型；子代理 Profile；编辑可靠性/回撤；计划进度；通知/PiP | coder/reviewer 工作流跑通；子代理可选模型 | 2-3 周 |
| P5 UI 收敛 | 5 一级入口；设置 9 页；复用新版好组件；删多余 store | 页面 ≤ 14，无死路由 | 1-2 周 |
| P6 稳定化 | 真机矩阵、长会话、断线、升级、迁移、性能 | 验收清单全绿 | 1-2 周 |

单人约 8-10 周；2-3 人并行约 4-6 周。P2/P3 是主要风险。

## 8. 验收

功能：

- 全新安装 → 配 Provider → 对话 → 读写文件 → shell → 装 Skill/MCP → 压缩 → 恢复会话 → 升级安装，全链路通过。
- 子代理：主模型 + reviewer / coder 两个不同模型，`wait_agent` 返回结果，任务卡显示模型。
- 5 会话并行：取消其中一个，其他会话的 shell 不受影响。
- Provider 400 / 401 / 402 / 429：中文提示 + 可执行动作，不重复踩同一错误。
- 压缩后继续同一任务：能引用压缩前进度。

性能：

- 冷启动 ≤ 2s（引擎就绪）。
- 首 token 延迟不高于 1.3.2。
- 长会话（200 轮）滚动不卡；会话文件 append 不阻塞其他会话。
- APK ≤ 210 MB；内存峰值 ≤ 350 MB。

预算：

- 每次发布前跑 `tools/lite_budget.sh`：LOC、页面数、路由数、工具数、新名词数。
- 超预算必须写例外说明。

真机矩阵：

- Android 7 / 11 / 16；ARM64；HyperOS / 原生；低内存设备。

## 9. 风险

| 风险 | 影响 | 缓解 |
|---|---|---|
| 无 proot 后 Linux 工具链缺失 | Python/Node/编译工具不可用 | Termux 环境页 + 一键 pkg install + 预置常用工具 |
| targetSdk 28 / Termux bootstrap 限制 | 新系统安装/执行失败 | 启动自检、bootstrap 修复、真机矩阵；保留 1.5 Full APK |
| 1.3.2 → lite 数据迁移 | 会话丢失 | 一次性迁移 + 原文件备份 + 回滚开关 |
| 回植补丁与 1.3.2 分叉 | 合并冲突 | 只 cherry-pick；每条补丁有测试和端口日志 |
| 用户从 1.5 降级 | 无法降级/数据不兼容 | 独立 `.lite` 包先发测试；稳定后再决定并包 |
| 子代理配置变复杂 | 理解成本上升 | 最多 8 个 Profile；默认主模型；UI 只暴露“名称/模型/只读” |
| 再次膨胀 | 6 个月后回到 1.5 | 预算 + 准入规则 + 发布检查 |

## 10. 默认决策（可改）

1. **版本 / 包名**：先发独立包 `com.coomi.lite`（2.0.0-lite），与 1.5 Full 并存；稳定后再决定是否并入主包。这样不会把 proot 用户直接断掉，也能随时回滚。
2. **Git**：首版不做 Git 面板，shell git 可用；若反馈强烈，再加一个只读“变更”页。
3. **Hooks**：保留后端，收进“高级设置”，默认关闭。
4. **反馈**：不做自动上传 / 体验计划；保留本地崩溃日志 + 手动反馈入口（GitHub / QQ）。
5. **TUI**：砍掉交互 TUI；保留 `coomi exec` 无头 CLI，方便脚本与测试。
6. **悬浮窗 / Shizuku / 无障碍**：不进首版；作为 P2 可选模块。
7. **远程市场**：只保留内置目录 + 自定义配置。
8. **DeepSeek 登录**：砍掉；保留 API Key 预设。
9. **工作流**：砍掉；Loop + 子代理 + 计划模式覆盖自动化。
10. **命名**：只允许已有名词；新增能力必须先写“映射到哪个已有名词”。

## 11. 仓库与构建产物卫生（P0 顺手做）

当前工作区本体已经膨胀到约 45-50 GB，和产品臃肿是同一个问题的另一面：

| 目录/文件 | 体积 | 处理 |
|---|---:|---|
| `apps/`（主要是 `coomi-rs/target`） | 33 GB | 清理 target；确认 `.gitignore`；CI 缓存 |
| `build/` | 5.0 GB | 删除；加入 `.gitignore` |
| `video-demo/` | 2.7 GB | 移出仓库；放 Release / 对象存储 |
| `runtime-v2-dist/` | 1.5 GB | Lite 直接删；1.5 Full 分支保留 |
| `deploy/` | 815 MB | 只保留 `latest.json` 与必要 APK；历史版本移走 |
| `release-dist/` | 680 MB | 同上 |
| 7 个 `java_pid*.hprof` | ≈3.9 GB | 删除；加入 `.gitignore` |
| 根目录测试 APK | ≈2.5 GB | 移出仓库；改放 Release 附件 |
| `references/` | 236 MB | 移出仓库或使用 LFS |
| `node_modules/` | 114 MB | 加入 `.gitignore` |

Lite 分支第一条提交就做仓库清理；否则后面每次 clone / 构建都在为历史付成本。
