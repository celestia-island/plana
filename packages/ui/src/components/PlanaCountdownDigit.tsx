import { defineComponent, ref, watch, type PropType } from "vue";

export const PCountdownDigit = defineComponent({
  name: "PlanaCountdownDigit",
  props: {
    value: { type: Number, default: 0 },
  },
  setup(props) {
    const prevValue = ref(props.value);
    const animating = ref(false);

    watch(() => props.value, (next, old) => {
      if (next !== old) {
        prevValue.value = old;
        animating.value = true;
        setTimeout(() => { animating.value = false; }, 350);
      }
    });

    return () => {
      const digits = String(props.value).padStart(props.value >= 100 ? 3 : 2, "0").split("");

      return (
        <span class="plana-countdown-digit" style={{
          display: "inline-flex",
          alignItems: "center",
          gap: "0",
          fontVariantNumeric: "tabular-nums",
          height: "1em",
          lineHeight: 1,
          fontWeight: 700,
        }}>
          {digits.map((d, i) => (
            <span
              key={i}
              class="plana-countdown-digit-slot"
              style={{
                position: "relative",
                display: "inline-flex",
                alignItems: "center",
                justifyContent: "center",
                width: `${0.65}em`,
                height: "1em",
                overflow: "hidden",
              }}
            >
              <span
                class={animating.value ? "plana-countdown-digit-flip" : ""}
                style={{
                  display: "inline-block",
                  lineHeight: 1,
                  transition: animating.value ? "transform 0.3s cubic-bezier(0.4, 0, 0.2, 1)" : "none",
                  transform: animating.value ? "translateY(-0.6em)" : "translateY(0)",
                }}
              >
                {d}
              </span>
            </span>
          ))}
          <span style={{
            fontSize: "0.75em",
            opacity: 0.6,
            fontWeight: 400,
            marginLeft: "1px",
          }}>s</span>
        </span>
      );
    };
  },
});
