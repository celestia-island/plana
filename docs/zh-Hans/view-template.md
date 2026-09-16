# 面板视图模板格式

> Status: active（格式随 `plana-celestia-types` 0.1.9 交付）
> 读者：模板作者与加载它们的宿主

工作区面板由**模板**配置：一份声明式文档，指明面板画在哪个基础视图上，并填该视图的 spec。
格式定义在 `plana-celestia-types`（`ws::ui::view_template`），并导出为 TypeScript 的
`@celestia-island/plana-types`，因此同一套形状同时抵达 Rust 宿主与 web UI。

格式中**没有任何东西会执行代码**：模板只能组合读取方已经能画的词表；宿主不认识的引擎会被
**显式**报为不支持，而不是被静默替换。

## 五个基础视图

每个模板只建立在其中一个之上，且只设置与之匹配的那一个 spec 分支：

| 基础视图 | 画什么 | spec |
|---|---|---|
| `waterfall` | 按桶分组的卡片流，独立列 | `WaterfallSpec` |
| `node-canvas` | 可平移缩放的节点/连线表面 | `NodeCanvasSpec` |
| `data-grid` | 带类型的表格 | `DataGridSpec` |
| `kanban` | 泳道 + 可拖拽卡片，每条泳道由卡片作表头 | `KanbanSpec` |
| `blank-canvas` | 整幅挂载，由宿主渲染器填充 | `BlankCanvasSpec` |

`kanban` **刻意不是**瀑布流变体：瀑布流把一条流枚举进列，而看板是一组泳道、卡片在泳道之间移动。
对话节点列表与多维表的看板视图都是看板的特化。

## 文档形态

```toml
version = 1

[[template]]
id = "reports"
title = "Reports"
render_engine = "celestia-waterfall"

[template.spec]
base = "waterfall"

[template.spec.waterfall]
columns = { kind = "adaptive", max = 2, min_width = 320 }
```

- `version` 是数字；**不认识该版本的读取方必须拒绝**该文件，而不是猜测。
- `id`、`render_engine` 与 i18n 键沿用 workspace-module 契约固定的形状。
- 一个 `[[template]]` 条目**恰好**设置其 `base` 指名的那个 spec 分支。

## 需要知道的字段

**瀑布流**：`columns`（single / fixed / adaptive）、可选 `grouping`（按日分桶）、可选
`windowing`（客户端或宿主侧）、可渲染的 `card_kinds`，以及可选的 `rail` / `search`。

**看板**：`axes`（`horizontal` / `vertical` / `both`——泳道轴与卡片轴相互独立）、`lanes`
（泳道键取自哪个字段、表头本身是否是一张卡片）、`card_kinds` 与 `dnd` 规则。

**节点画板**：`backend`（`svg` / `dom-overlay-svg` / `canvas-edges-dom` / `blank`）、
`camera` 边界，以及**默认即开**、位于右下角的小地图。

**Dock**：`DockPlacement` 覆盖全部八个锚点——四条边（`top` / `bottom` / `left` / `right`，居中）
与四个角，各自可选 `page` / `plane` 平面与 `glass` / `solid` 表面。部分宿主能画的更少：
**宿主必须拒绝它画不出的 placement，而不是忽略它。**

## 校验

加载是两步，第二步不可省：

```rust
let file: ViewTemplateFile = toml::from_str(&text)?;
file.validate()?; // id/键的形状、分支与 base 是否匹配、数值边界
```

`PanelTemplate::validate` 检查类型无法承载的一切：id 与 i18n 键形状、**是否恰好设置了 `base`
指名的那个分支**、以及数值边界（列数至少为 1、相机缩放区间有限且有序）。
`ViewTemplateFile::validate` 对文件里每个模板跑一遍——**一个坏模板会让整个文件失败**。

跳过校验的宿主会得到一个"画不出就忽略"的渲染器，而这正是本格式要避免的失败形态。
