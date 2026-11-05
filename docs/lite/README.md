# CoomiLite 开发说明

> 基线：`v1.3.2`（tag）
> 完整方案：[plan.md](plan.md)
> 回植清单：[backports.md](backports.md)

## 1. 当前进度

已完成：

- [x] 从 `v1.3.2` 导出干净源码，初始化独立 git 仓库。
- [x] 移除交互 TUI（`ui/src/terminal_ui`，约 6.6k 行），保留 `coomi serve` 与 `coomi exec`。
- [x] 移除匿名统计 `coomi-telemetry` crate 及前后端开关。
- [x] 子代理可选模型：`AgentProfile` 配置、`spawn_agent(task, agent_id?, fork_turns?)`、设置页 `/subagents`。
- [x] 复杂度预算脚本 `tools/lite_budget.py`。

下一步（P0）：

- [ ] 每会话独立模型（修复多会话串模型）。
- [ ] Provider 能力缓存 + 401/402/404/429 中文错误映射。
- [ ] 编辑可靠性 + 文件回撤（`file_journal`）。
- [ ] 压缩摘要保留任务进度，压缩失败降级截断。
- [ ] 每会话进程隔离，取消会话不误杀其他会话。
- [ ] 后台任务通知 / PiP。

## 2. 构建与验证

```bash
# Rust
cd apps/coomi-rs
cargo check --workspace
cargo test --workspace

# Web
cd apps/web
npm install
npm run build

# Android（需要 SDK + NDK 27；local.properties 不入库）
./gradlew :app:assembleDebug

# 复杂度预算
python tools/lite_budget.py
```

## 3. 子代理模型

配置文件是 `settings.json` 的两个字段：

```json
{
  "subagent_default": "reviewer",
  "subagents": [
    {
      "id": "reviewer",
      "name": "审查",
      "selector": "deepseek:deepseek-v4-pro",
      "instructions": "你是只读审查员，只报告问题，不改文件。",
      "tools": ["read_file", "grep_files", "list_dir", "view_image"],
      "max_parallel": 1
    }
  ]
}
```

约定：

- `selector` 使用 `provider_id` 或 `provider_id:model`，必须能在 `providers.json` 里解析。
- `tools` 为空 = 全部工具；非空 = 只允许列出的工具。
- `max_parallel` 1-5；全局子代理并发仍受 `AgentScheduler.max_agents`（默认 3）限制。
- `spawn_agent` 不传 `agent_id` 时使用 `subagent_default`；没有默认值时沿用主模型。
- 设置页只暴露“名称 / ID / 模型 / 附加指令 / 只读工具集 / 并发”，不引入新名词。
- 非法配置不会阻塞主会话：`run_turn` 会忽略并在日志里报告；设置页保存时会拒绝。

对应接口：

- `services/src/subagents.rs`：`SubAgentDocument`、`SubAgentProfile`、`ResolvedSubAgent`。
- `tools/src/agents.rs`：`AgentScheduler::with_profiles`、`spawn`、`run_agent`。
- `ui/src/web.rs`：`GET/PUT /api/settings/subagents`。
- `web/src/views/SubAgentsView.vue`：设置页。

## 4. 开发规则

1. **一个执行路径**：所有任务最终走 `Agent::run_turn`。子代理只是另一个 Profile 的同一个循环，不新增 runner。
2. **名词零新增**：新能力优先映射到已有名词（会话、任务、工具、技能、MCP、模型服务、子代理、记忆、身份、运行环境、文件、设置）。
3. **功能三种形态**：设置项、工具、Profile。不能落成这三种形态的需求不做页面。
4. **预算优先**：新增页面前先跑 `python tools/lite_budget.py`；超预算先删等量东西。
5. **不引入 Proot**：执行环境只保留 Termux 后端；需要 Linux 工具时走 `运行环境` 页的 `pkg install`。
6. **只 cherry-pick**：从 1.4/1.5 回植时，只搬逻辑，不整目录合并；每条改动记录到 `backports.md`。

## 5. 已知约束

- `targetSdkVersion` 必须保持 ≤ 28：Termux bootstrap 从应用私有目录执行，API 29+ 会 `EACCES`。
- `apps/coomi-app/app/src/main/cpp/bootstrap-aarch64.zip` 是 tracked 文件；`clean` 不再删除它，clean checkout 可直接构建。不要把它加入 `.gitignore`。
- 官方 Termux bootstrap 的 apt/dpkg 写死了 `com.termux`，`pkg` 安装可能受限；核心 bash 工具可用。
- 不内置 Proot，因此没有 Ubuntu userland、apt/pip 全量环境和进程隔离；安全边界是 Android 应用权限 + 路径/命令策略，不宣称沙箱。
- 旧版会话是整文件 JSON；迁移到 append-only 日志时保留原文件备份。

## 6. 包名、应用名与图标

- applicationId：`com.coomi.lite`；应用名：`CoomiLite`。
- 与 Coomi 1.5（`com.coomi.android`）并存，数据目录独立，不能覆盖升级；需要迁移时从旧 App 导出备份再导入。
- Termux 私有路径随包名变为 `/data/data/com.coomi.lite/files`；`TermuxConstants`、两处 `strings.xml`、快捷方式、Theme action 必须保持同步。
- 图标源：`assets/coomi-agent-lite.png`；生成命令 `python scripts/make_icons.py`。会同时更新 launcher、adaptive icon、原生 logo、TV banner、Web logo/favicon。
- 更新源：`https://updates.septemc.com/coomi/lite/android/latest.json`；不要指向全量 Coomi 的 feed，否则下载的 APK 无法覆盖安装。
