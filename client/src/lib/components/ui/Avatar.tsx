import { ComponentProps } from "solid-js";
import { tv, VariantProps } from "tailwind-variants";
import { generateGradient } from "~/lib/avatar";
import { removeNonAlphanumeric } from "~/lib/utils";

const avatar = tv({
  base: "rounded-full",
});

type AvatarProps = ComponentProps<"img"> &
  ComponentProps<"svg"> &
  VariantProps<typeof avatar> & {
    username: string;
    url?: string;
    size: number;
  };

export function Avatar(props: AvatarProps) {
  props.class = avatar(props);
  const filteredUsername = removeNonAlphanumeric(props.username);

  if (!props.url) {
    const { fromColor, toColor } = generateGradient(props.username);

    return (
      <svg
        width={props.size}
        height={props.size}
        viewBox={`0 0 ${props.size} ${props.size}`}
        version="1.1"
        xmlns="http://www.w3.org/2000/svg"
        {...props}
      >
        <g>
          <defs>
            <linearGradient id={filteredUsername} x1="0" y1="0" x2="1" y2="1">
              <stop offset="0%" stop-color={fromColor} />
              <stop offset="100%" stop-color={toColor} />
            </linearGradient>
          </defs>
          <rect
            fill={`url(#${filteredUsername})`}
            x="0"
            y="0"
            width={props.size}
            height={props.size}
          />
        </g>
      </svg>
    );
  }

  return (
    <img
      src={props.url}
      alt="profile picture"
      title={props.username}
      width={props.size}
      height={props.size}
      {...props}
    />
  );
}
