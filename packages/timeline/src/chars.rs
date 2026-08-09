// Timeline character constants.
// CLI variants use pure ASCII; TUI variants use box-drawing Unicode.

// -- CLI (ASCII) --
pub const TL_HEADER: &str = "--";
pub const TL_BODY: &str = "  ";
pub const TL_CLOSE: &str = "--";
pub const TL_SEP: &str = "------";
pub const TL_SEP_CHAR: &str = "-";
pub const TL_TOOL_OPEN: &str = "  ";
pub const TL_TOOL_INNER: &str = "  ";
pub const TL_TOOL_CLOSE: &str = "  ";

pub const ARROW_UP: &str = "in";
pub const ARROW_DOWN: &str = "out";
pub const ARROW_SWAP: &str = "x";
pub const CHECK: &str = "ok";
pub const CROSS: &str = "!!";
pub const DOT_FILLED: &str = "*";
pub const DOT_EMPTY: &str = "o";
pub const DOT_ALT: &str = "@";
pub const HLINE: &str = "-";

// -- TUI (box-drawing Unicode) --
pub const BD_H: &str = "\u{2500}";
pub const BD_V: &str = "\u{2502}";
pub const BD_DR: &str = "\u{250C}";
pub const BD_DL: &str = "\u{2510}";
pub const BD_UR: &str = "\u{2514}";
pub const BD_UL: &str = "\u{2518}";
pub const BD_T_LEFT: &str = "\u{251C}";
pub const BD_RND_TL: &str = "\u{256D}";
pub const BD_RND_TR: &str = "\u{256E}";
pub const BD_RND_BL: &str = "\u{2570}";
pub const BD_RND_BR: &str = "\u{256F}";

// -- TUI tool block borders --
pub const TL_TOOL_OPEN_TUI: &str = "\u{256D}"; // ╭
pub const TL_TOOL_INNER_TUI: &str = "\u{2502}"; // │
pub const TL_TOOL_CLOSE_TUI: &str = "\u{2570}"; // ╰
pub const TL_SEP_TUI: &str = "\u{251C}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}"; // ├──────
