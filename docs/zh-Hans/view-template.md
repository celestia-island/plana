# 面板视图模板格式

状态：**生效**（格式位于 `plana-celestia-types` 0.1.1 的 `ws::ui::view_template`
模块。英文权威版：[view-template.md](../../en/view-template.md)）

读者：模板作者与加载它们的宿主

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
| `kanban` | 泳道 + 可拖拽卡片，表头可选卡片样式 | `KanbanSpec` |
| `blank-canvas` | 整幅挂载，由宿主渲染器填充 | `BlankCanvasSpec` |

`kanban` **刻意不是**瀑布流变体：瀑布流把一条流枚举进列，而看板是一组泳道、卡片在泳道之间移动。
对话节点列表与多维表的看板视图都是看板的特化。

## 文档形态

```toml
version = 1

[[template]]
id = "reports"
title = "Reports"
title_key = "reports.title"
render_engine = "waterfall-stream"

[template.spec]
base = "waterfall"

[template.spec.waterfall]
columns = { kind = "adaptive", max = 2, min_width = 320 }

[template.spec.waterfall.grouping]
field = "timestamp"
newest_first = true

[template.spec.waterfall.windowing]
source = "client"
estimated_item_height = 150
overscan_screens = 1.0

[[template.spec.chrome.docks]]
widget = "chat-bar"
placement = "bottom"
anchor = "plane"
surface = "glass"
```

- `version` 是数字；**不认识该版本的读取方必须拒绝**该文件，而不是猜测——但这是**宿主**的
  检查：`validate()` 从不看 `version`。
- `title` 必填；`title_key` 是可选的 i18n 键，存在时宿主翻译它而不是用 `title`。
  `description` 与 `description_key` 是面板描述的同款可选组合。
- `id` 与 `render_engine` 各自必须匹配 `^[a-z][a-z0-9-]*$`。这就是全部规则：既没有引擎白名单，
  也没有 `celestia-` 前缀约定——`render_engine` 写的就是读取方宿主注册表能解析出的那个插件。
- 一个 `[[template]]` 条目**恰好**设置其 `base` 指名的那个 spec 分支。
- chrome 挂在 **spec** 上而不是模板上，且 dock 是一份*列表*：每一条都是一个
  `[[template.spec.chrome.docks]]` 表。写成 `[template.chrome.docks]` 不会报错——类型接受未知
  字段——它会被直接丢弃：模板照样通过校验，面板则不带任何 dock 渲染出来。

## 需要知道的字段

**瀑布流**：`columns`（single / fixed / adaptive）、可选 `grouping`（按日分桶）、可选
`windowing`（客户端或宿主侧；缺省 = 整条列表都渲染）、可渲染的 `card_kinds`，以及可选的
`rail` / `search`。

**数据网格**：`fields`（网格绑定的字段 schema id）、`editable`（是否就地编辑行，默认
`false`），以及可选、画在网格之上的 `overlay` 部件。

**看板**：`axes`（`horizontal` / `vertical` / `both`——泳道轴与卡片轴相互独立）、`lanes`
（泳道键取自哪个字段、表头本身是否画成一张卡片——`header_is_card` 默认 `false`）、
`card_kinds`、`dnd` 规则（`across_lanes` 与 `reorder` 均默认 `false`），以及长看板可选的
`windowing`。

**节点画板**：`backend`（`svg` / `dom-overlay-svg` / `canvas-edges-dom` / `blank`）、
`camera` 边界、可渲染的 `node_kinds`，以及可选的 `palette` / `inspector` 侧栏与
`auto_layout` 布局算法 id。小地图是一张**可选表**：不写 `minimap` 键就没有小地图；写了该键
时，其 `enabled` 与 `placement` 默认为 `true` 与 `bottom-right`。

**空白画布**：`mount`（宿主为整幅挂载解析的渲染器 id）与可选、画在其上的 `overlay`。

**Dock**：`DockPlacement` 覆盖全部八个锚点——四条边（`top` / `bottom` / `left` / `right`，
居中）与四个角，各自可选 `page` / `plane` 平面与 `glass` / `solid` 表面。一条贡献的
`anchor`、`surface` 与 `order` 默认为 `page`、`glass` 与 `0`。部分宿主能画的更少：
**宿主必须拒绝它画不出的 placement，而不是忽略它。**

## 校验

加载是两步，第二步不可省：

```rust
let file: ViewTemplateFile = toml::from_str(&text)?;
file.validate()?; // id/引擎/键的形状、分支与 base 是否匹配、数值边界
```

`PanelTemplate::validate` 检查类型无法承载的一切：`id` 与 `render_engine` 的形状
（`^[a-z][a-z0-9-]*$`）、i18n 键形状、**是否恰好设置了 `base` 指名的那个分支**，以及数值
边界（列数至少为 1、相机需要有限的 `0 < min_zoom <= max_zoom` 与正的 `zoom_step`）。
`ViewTemplateFile::validate` 对文件里每个模板跑一遍——**一个坏模板会让整个文件失败**。

它**不读** `version`：这个数字只被携带、从不被检查，因此"拒绝一个 `version` 不认识的
文件"是宿主自己必须做的检查。

跳过校验的宿主会得到一个"画不出就忽略"的渲染器，而这正是本格式要避免的失败形态。
