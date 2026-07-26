import { defineComponent, ref, onMounted } from "vue";
import { Cookie } from "lucide-vue-next";

const STORAGE_KEY = "plana-cookies-accepted";

export const PCookieConsent = defineComponent({
  name: "PlanaCookieConsent",
  setup() {
    const accepted = ref(false);
    const show = ref(false);

    onMounted(() => {
      accepted.value = localStorage.getItem(STORAGE_KEY) === "1";
      if (!accepted.value) {
        const tz = Intl.DateTimeFormat().resolvedOptions().timeZone;
        const eu = tz?.startsWith("Europe/") ?? false;
        show.value = eu;
      }
    });

    function accept() {
      localStorage.setItem(STORAGE_KEY, "1");
      accepted.value = true;
      show.value = false;
    }

    return () => {
      if (!show.value && !accepted.value) return null;
      if (accepted.value) return (
        <Cookie size={12} style={{ color: "rgb(var(--color-success))", opacity: 0.5, cursor: "default" }} />
      );
      return (
        <span style={{ display: "flex", alignItems: "center", gap: "4px", fontSize: "0.625rem", color: "rgb(var(--color-muted))" }}>
          This site uses cookies.
          <button onClick={accept} style={{ background: "transparent", border: "none", color: "rgb(var(--color-primary))", cursor: "pointer", fontSize: "0.625rem", fontWeight: 600 }}>OK</button>
        </span>
      );
    };
  },
});
