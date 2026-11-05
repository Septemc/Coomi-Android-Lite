# CoomiLite v2.0.0-lite.1（测试）

## 这一版做了什么

- 基于 `v1.3.2` 的精简线，移除 Proot/Ubuntu、交互 TUI、匿名统计。
- 会话空态保留最新版的「慎终如始，则无败事」格言与书法遮罩资源。
- 会话输入区支持图片/文件附件：
  - Android 文件选择器导入后显示附件 chip；
  - 图片缩略图、点击预览、移除；
  - 附件随草稿按会话保存；
  - 发送时以路径清单交给 Agent，由 `view_image` / `read_file` 读取；
  - 历史恢复后仍显示附件。
- 保留子代理可选模型：设置 → 子代理模型，支持独立 Provider/模型、只读工具集、并发。
- 包名 `com.coomi.lite`，应用名 `CoomiLite`，与全量 Coomi 并存。
- 应用图标、原生 logo、TV banner、Web logo/favicon 全部切换为 Lite 标识。

## 说明

- 这是测试包，使用调试签名；不要覆盖全量 Coomi 安装。
- 更新源需要部署到 `https://updates.septemc.com/coomi/lite/android/latest.json` 后，App 内检查更新才会生效。
- 无 Proot：Linux 工具链依赖 Termux 环境；需要时在「运行环境」页检查/安装常用工具。

## 校验

- APK：`CoomiLite-Android-arm64-v2.0.0-lite.1.apk`
- SHA-256：`d82b977d80fba582a098b745c0950fbb902208b22ce91ed2b6b56ba3e5e77aaa`
- Package：`com.coomi.lite`
- Version：`2.0.0-lite.1` / `200`
- 架构：`arm64-v8a`
- 最低系统：Android 7.0
