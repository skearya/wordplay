import { ComponentProps } from "solid-js";
import { tv, VariantProps } from "tailwind-variants";

const input = tv({
  base: "rounded-lg border placeholder-white/50 transition-opacity disabled:opacity-50",
  variants: {
    color: {
      primary: "bg-light-background text-foreground",
    },
    size: {
      xs: "px-1.5 py-1 text-sm",
      sm: "px-2.5 py-2",
      md: "px-3 py-2.5",
      lg: "rounded-xl px-3.5 py-3 text-lg",
    },
  },
  defaultVariants: {
    color: "primary",
    size: "md",
  },
});

type InputProps = ComponentProps<"input"> & {
  onEnter?: (input: HTMLInputElement) => void;
} & VariantProps<typeof input>;

export function Input(props: InputProps) {
  props.class = input(props);

  return (
    <input
      type="text"
      {...(props.onEnter && {
        onKeyDown: (event) => {
          if (event.key === "Enter") {
            props.onEnter!(event.target as HTMLInputElement);
          }
        },
      })}
      {...props}
    />
  );
}
