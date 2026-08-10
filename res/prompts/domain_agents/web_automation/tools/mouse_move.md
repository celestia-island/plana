+++
name = "mouse_move"
agent = "web_automation"

[description]
en = "Simulate mouse movement operations within a browser page"
+++

# mouse_move

## Description

Moves the mouse cursor to the specified X and Y coordinates within the browser viewport. Dispatches mousemove events that trigger hover effects, tooltips, and other mouse-over behaviors. Use this to hover over elements or prepare the cursor position before a click.

## Parameters

- **`browser_id`** (string, required): Browser instance identifier (obtained from `create`)
- **x** (number, required): X coordinate in pixels, relative to the viewport top-left corner
- **y** (number, required): Y coordinate in pixels, relative to the viewport top-left corner

## Returns

### Success

```text
Mouse moved

Browser ID: browser_abc123
Position: (500, 300)
```

### Failure

```text
Mouse move failed

Browser ID: browser_abc123
Error: Coordinates out of viewport bounds
Position: (5000, 5000)
```

## Examples

### Example 1: Move to center of viewport

```text
browser_id: "browser_abc123"
x: 640
y: 360
```

### Example 2: Hover over a menu item area

```text
browser_id: "browser_abc123"
x: 200
y: 50
```

### Example 3: Move before clicking

```text
browser_id: "browser_abc123"
x: 960
y: 540
```

## Important Notes

- **Coordinate origin**: (0, 0) is the top-left corner of the viewport, not the page
- **Viewport bounds**: Coordinates beyond the viewport dimensions may not trigger visible effects
- **Hover effects**: Use this to trigger CSS hover states, tooltips, and dropdown menus that appear on mouse-over
- **Combined with click**: Move the mouse first, then use `mouse_click` with a selector to click at the hovered position
