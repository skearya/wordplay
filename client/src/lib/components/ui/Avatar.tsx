import { ComponentProps } from "solid-js";
import { tv, VariantProps } from "tailwind-variants";

const avatar = tv({
  base: "rounded-full",
});

type AvatarProps = ComponentProps<"img"> &
  VariantProps<typeof avatar> & {
    username: string;
    url?: string;
    size: number;
  };

export function Avatar(props: AvatarProps) {
  props["class"] = avatar(props);

  return (
    <img
      src={props.url ?? `https://avatar.vercel.sh/${props.username}`}
      alt="profile picture"
      title={props.username}
      width={props.size}
      height={props.size}
      {...props}
    />
  );
}
