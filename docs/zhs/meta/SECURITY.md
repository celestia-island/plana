+++
title = "安全政策"
description = """请勿为安全漏洞公开发布 issue。"""
lang = "zhs"
category = "meta"
+++

# 安全政策

## 报告漏洞

**请勿为安全漏洞公开发布 issue。**

请通过
[GitHub Security Advisories](https://github.com/celestia-island/arona/security/advisories/new)
私下报告。如果您无法使用 GitHub Security Advisories，请发送邮件
至 security@celestia.world，并附上清晰的描述和复现步骤。

## 范围

在范围内：

- 认证绕过、JWT/OAuth 弱点、会话处理缺陷
- API 密钥/凭证泄露或不当存储
- 授权和 RBAC 执行缺陷
- 注入漏洞（SQL、命令注入、SSRF、XSS）
- 不安全的反序列化、路径遍历、SSRF
- 导致权限提升或跨租户访问的问题

不在范围内：

- 无法通过本项目利用的上游依赖中的漏洞
- 不符合文档指南的不安全配置的自托管部署
- 针对公共 LLM 提供商端点的拒绝服务攻击

## 响应

| 阶段 | 目标 |
| --- | --- |
| 机器人确认 | 10 分钟 |
| 人工确认 | 1 个日历日 |
| 初步评估 | 3 个日历日 |
| 修复或缓解 | 30 个日历日（视严重程度而定） |

请包含：(1) 受影响的组件和版本，(2) 攻击向量和影响，(3) 复现步骤，
以及 (4) 建议的缓解措施。

## 支持的版本

仅 `main` / `dev` 分支上的最新发布线接收安全修复。
