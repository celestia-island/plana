import { defineComponent, ref } from "vue";

export const Footer = defineComponent({
  name: "PlanaFooter",
  props: {
    height: { type: String, default: "var(--s-footer-height)" },
  },
  setup(props, { slots }) {
    return () => (
      <footer class="s-status-bar" style={{ position: "fixed", bottom: 0, left: 0, right: 0, zIndex: 40 }}>
        <div class="s-status-bar-left">
          {slots.left?.()}
        </div>
        <div class="s-status-bar-center">
          {slots.center?.()}
        </div>
        <div class="s-status-bar-right">
          {slots.right?.()}
        </div>
      </footer>
    );
  },
});
