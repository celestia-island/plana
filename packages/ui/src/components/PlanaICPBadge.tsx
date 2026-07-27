import { defineComponent, ref, onMounted } from "vue";

function likelyChina(): boolean {
  try {
    const tz = Intl.DateTimeFormat().resolvedOptions().timeZone;
    if (tz === "Asia/Shanghai") return true;
    const lang = navigator.language;
    if (lang === "zh-CN" || lang === "zh-Hans" || lang === "zh") return true;
    return false;
  } catch { return false; }
}

export const PICPBadge = defineComponent({
  name: "PlanaICPBadge",
  props: {
    number: { type: String, default: "" },
  },
  setup(props) {
    const visible = ref(false);

    onMounted(() => {
      visible.value = likelyChina() && props.number.length > 0;
    });

    return () => {
      if (!visible.value) return null;
      return (
        <a
          href="https://beian.miit.gov.cn"
          target="_blank"
          rel="noopener noreferrer"
          style={{
            fontSize: "0.625rem",
            color: "rgb(var(--color-muted))",
            textDecoration: "none",
            opacity: 0.6,
          }}
        >
          {props.number}
        </a>
      );
    };
  },
});
