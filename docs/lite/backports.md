# 新版能力回植清单

> 来源：`Coomi-Android` 的 1.4.x / 1.5.x（HEAD）。
> 规则：只搬逻辑，不搬页面；每条改动可独立回滚；优先补 1.3.2 的 P0 缺陷。

## 状态说明

- `done`：已在 lite 实现并可编译。
- `planned`：已确定要做。
- `deferred`：明确不在首版。
- `cut`：不进入 lite 主线。

## P0：直接决定手感与稳定性

| 能力 | 来源 | 状态 | 落点 / 备注 |
|---|---|---|---|
| 子代理可选模型 | 1.4.0 子代理模型池（简化） | done | `services/src/subagents.rs`、`tools/src/agents.rs`、`/api/settings/subagents`、`SubAgentsView.vue` |
| 每会话独立模型 | 1.4.5 | planned | `Session.provider_id/model` 优先，去掉全局 active 影响 |
| Provider 能力缓存 | HEAD `services/src/capabilities.rs` | planned | `provider_id + model` 维度；400 只探测一次 |
| Provider 中文错误映射 | 反馈 B3/B4 | planned | 401/402/403/404/429/400 → 原因 + 下一步动作 |
| 编辑可靠性 | 反馈 A3 | planned | 读后写校验、内容锚点、最相似候选、参数类型守卫 |
| 文件回撤 | HEAD `services/src/file_journal.rs` | planned | 按会话/轮次记录写入，支持撤销 |
| 压缩保留进度 | 反馈 A1/A2 | planned | 摘要固定包含目标/已完成/未完成/下一步 |
| 压缩失败降级 | 反馈 A1 | planned | 摘要请求失败 → 截断保留最近 N 轮，不中断会话 |
| 每会话进程隔离 | 反馈 + HEAD `processes.rs` | planned | 进程注册表按 `session_id` 分区；取消只杀本会话进程组 |
| 后台任务通知 / PiP | 1.4.5 | planned | Android 侧；任务完成可回会话 |

## P1：高价值、可独立裁剪

| 能力 | 来源 | 状态 | 说明 |
|---|---|---|---|
| 计划进度 / 指令确认 / 推理关闭档 | 1.5.0-test.5 | planned | 复用现有计划模式与推理强度设置 |
| Anthropic / Gemini 流式 | HEAD | planned | SSE 解析；当前 1.3.2 只有非流式 |
| 上下文环 / 模型能力可视 | HEAD | planned | 只做展示，不新增页面 |
| Skill 路由 | HEAD `skill_router.rs` | planned | 内部按描述/关键词选择，控制提示词体积 |
| MCP 可视化配置 | 1.5.0-test.5 | planned | 现有 MCP 页面内表单，不新增名词 |
| 会话搜索 / 导出（简化） | HEAD `data_tools.rs` | planned | 只保留搜索与 Markdown 导出 |
| 启动自检 / Bootstrap 修复 | 反馈 D1 | planned | 可执行位、termux.properties、版本校验 |

## P2：首版不做

| 能力 | 状态 | 原因 |
|---|---|---|
| Git 面板 / AI Git | deferred | shell git 可用；需要时先做只读“变更”页 |
| 工作流 / Cron | deferred | Loop + 子代理 + 计划模式覆盖 |
| 悬浮窗 / Shizuku / 无障碍 | deferred | 移动端差异化，但非核心 |
| 本地模型 / OCR / TTS / UI 自动化 | deferred | 非核心 |
| 计费 / 余额 / 用量页 | deferred | 会话内显示 token 即可 |
| DeepSeek / 智谱账号登录 | cut | 保留 API Key 预设；登录 UI 在 HEAD 已是死代码 |
| 远程扩展市场 registry | cut | 内置目录 + 自定义配置足够 |
| 匿名统计 / 体验改进计划 / Experience | cut | 隐私与维护成本；已移除 telemetry crate |
| 交互 TUI | cut | Android 只用 WebView；保留 `coomi exec` |
| Proot / Runtime V2 / Ubuntu / Debian | cut | 用户明确要求；APK 减少 150MB+ |
| 工作室 / 群聊 / 协同 / 工单 | cut | 用多会话 + 子代理 Profile 替代 |
| 数字生命 / Life sidecar | cut | 用身份 + 记忆替代 |
| 提示词库 / 超载模型 | cut | 用身份、子代理 instructions、推理强度替代 |

## 回植记录模板

```md
### <能力名>

- 来源：<repo> <commit/文件:行>
- 改动：<lite 中的文件>
- 验证：<cargo test / 手测步骤>
- 回滚：<恢复哪几个文件 / revert 哪个 commit>
- 备注：<与 1.3.2 的差异、未搬运的部分>
```
