import { defineComponent, h, type Component, type PropType } from "vue";

import { HNavItem, HSidebar } from "@celestia-island/hikari";
import BarChart3 from "lucide-vue-next/dist/esm/icons/chart-bar-big";
import Bell from "lucide-vue-next/dist/esm/icons/bell";
import Bot from "lucide-vue-next/dist/esm/icons/bot";
import Box from "lucide-vue-next/dist/esm/icons/box";
import Cable from "lucide-vue-next/dist/esm/icons/cable";
import Cpu from "lucide-vue-next/dist/esm/icons/cpu";
import FileText from "lucide-vue-next/dist/esm/icons/file-text";
import FolderOpen from "lucide-vue-next/dist/esm/icons/folder-open";
import Gauge from "lucide-vue-next/dist/esm/icons/gauge";
import Key from "lucide-vue-next/dist/esm/icons/key";
import Layers from "lucide-vue-next/dist/esm/icons/layers";
import LayoutDashboard from "lucide-vue-next/dist/esm/icons/layout-dashboard";
import Mic from "lucide-vue-next/dist/esm/icons/mic";
import Monitor from "lucide-vue-next/dist/esm/icons/monitor";
import Send from "lucide-vue-next/dist/esm/icons/send";
import Settings from "lucide-vue-next/dist/esm/icons/settings";
import Share2 from "lucide-vue-next/dist/esm/icons/share-2";
import Shield from "lucide-vue-next/dist/esm/icons/shield";
import Webhook from "lucide-vue-next/dist/esm/icons/webhook";
import Zap from "lucide-vue-next/dist/esm/icons/zap";

interface NavItem {
  key: string;
  icon: string;
  label: string;
  route: string;
  disabled?: boolean;
  badge?: string;
}

const iconMap: Record<string, Component> = {
  chart: LayoutDashboard,
  plug: Cable,
  bot: Bot,
  cpu: Cpu,
  zap: Zap,
  folder: FolderOpen,
  radio: Send,
  monitor: Monitor,
  box: Box,
  link: Webhook,
  key: Key,
  shield: Shield,
  bar: BarChart3,
  gauge: Gauge,
  mic: Mic,
  settings: Settings,
  panels: Layers,
  bell: Bell,
  share: Share2,
  manifest: FileText,
};

export const PNavSidebar = defineComponent({
  name: "PlanaNavSidebar",
  props: {
    navItems: {
      type: Array as PropType<NavItem[]>,
      required: true,
    },
    currentRoute: {
      type: String,
      required: true,
    },
    collapsed: {
      type: Boolean,
      default: false,
    },
  },
  emits: ["navigate"],
  setup(props, { emit }) {
    return () => (
      <HSidebar width="224px">
        {{
          default: () => (
            <nav>
              {props.navItems.map((item) => {
                const active = props.currentRoute === item.route;
                const IconComp = iconMap[item.icon];
                return (
                  <HNavItem
                    key={item.key}
                    active={active}
                    disabled={item.disabled}
                    onClick={() => emit("navigate", item.route)}
                  >
                    {{
                      // Render via h(): Vue's `Component` union includes
                      // non-callable option types, so `<IconComp />` is not a
                      // valid JSX element type (matches hikari's HkTree).
                      icon: () => (IconComp ? h(IconComp, { size: 16 }) : null),
                      default: () => (
                        <span class="s-nav-item-content">
                          <span class="truncate">{item.label}</span>
                          {item.badge && (
                            <span
                              class="s-nav-item-badge"
                              aria-label={`${item.badge} pending`}
                            >
                              {item.badge}
                            </span>
                          )}
                        </span>
                      ),
                    }}
                  </HNavItem>
                );
              })}
            </nav>
          ),
        }}
      </HSidebar>
    );
  },
});
