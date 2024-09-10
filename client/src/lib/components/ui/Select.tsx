import { ComponentProps } from "solid-js";
import { tv, VariantProps } from "tailwind-variants";

const select = tv({
  base: "rounded-lg border bg-transparent transition-opacity disabled:opacity-50",
  variants: {
    color: {
      primary: "bg-dark-green text-foreground",
      secondary: "bg-blue text-foreground",
      muted: "bg-light-background text-light-green",
    },
    size: {
      xs: "rounded px-1.5 py-1 text-sm",
      md: "px-2.5 py-2",
    },
  },
  defaultVariants: {
    color: "primary",
    size: "md",
  },
});

type SelectProps = ComponentProps<"select"> & VariantProps<typeof select>;

export function Select(props: SelectProps) {
  props.class = select(props);

  return <select {...props}>{props.children}</select>;
}
