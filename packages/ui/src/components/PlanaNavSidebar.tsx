import { defineComponent, type Component, type PropType } from "vue";

import { NavItem, Sidebar } from "@celestia-island/hikari";
import { BarChart3, Bell, Bot, Box, Cable, Cpu, FileText, FolderOpen, Gauge, Key, Layers, LayoutDashboard, Mic, Monitor, Send, Settings, Share2, Shield, Webhook, Zap } from "lucide-vue-next";

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

export const NavSidebar = defineComponent({
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
      <Sidebar width="224px">
        {{
          default: () => (
            <nav>
              {props.navItems.map((item) => {
                const active = props.currentRoute === item.route;
                const IconComp = iconMap[item.icon];
                return (
                  <NavItem
                    key={item.key}
                    active={active}
                    disabled={item.disabled}
                    onClick={() => emit("navigate", item.route)}
                  >
                    {{
                      icon: () => (IconComp ? <IconComp size={16} /> : null),
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
                  </NavItem>
                );
              })}
            </nav>
          ),
        }}
      </Sidebar>
    );
  },
});
