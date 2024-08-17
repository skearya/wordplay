import { JSX } from "solid-js/jsx-runtime";
import { tv, VariantProps } from "tailwind-variants";

const button = tv({
  base: "rounded-lg border transition-opacity active:opacity-75 disabled:opacity-50",
  variants: {
    color: {
      primary: "bg-dark-green text-foreground",
      secondary: "bg-blue text-foreground",
      muted: "bg-light-background text-light-green",
    },
    size: {
      sm: "px-2 py-1.5",
      md: "px-2.5 py-2.5",
      lg: "px-3 py-4 font-medium",
    },
  },
  defaultVariants: {
    color: "primary",
    size: "md",
  },
});

type ButtonProps = JSX.HTMLElementTags["button"] & VariantProps<typeof button>;

export function Button(props: ButtonProps) {
  props["class"] = button(props);

  return <button {...props} />;
}
