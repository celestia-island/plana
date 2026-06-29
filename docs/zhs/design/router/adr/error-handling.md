+++
title = "Error Handling — thiserror with crate Result"
description = """架构决策记录 —— Error Handling — thiserror with crate Result。"""
lang = "zhs"
category = "design"
subcategory = "router"
+++

# Error Handling — thiserror with crate Result

- **Status**: 已采纳
- **Date**: 2026-06-09
- **Authors**: Evernight Core Team

## Context

Evernight 需要一个统一的错误类型，覆盖所有模块（screen、SSH、hardware、networking、tunneling）。库必须对外暴露清晰的错误 API，同时保持内部错误处理的易用性。

## Decision

使用 `thiserror` 派生 `EvernightError` 枚举，每个变体带有 `#[error(...)]` 展示实现。定义 `pub type Result<T> = std::result::Result<T, EvernightError>` 作为 crate 级别的结果类型。每个变体以 `String` 形式捕获领域上下文（ScreenCapture、Ssh、Tunnel 等），而非包装外部错误类型，以避免泄漏内部依赖的细节。

## Consequences

### Positive

- 稳定的公共错误 API；调用方可以根据枚举变体进行匹配，无需了解内部实现

### Negative

- String 转换会导致部分信息丢失
- 没有结构化的错误链
