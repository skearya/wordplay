/*
  ripped from https://github.com/vercel/avatar/blob/master/utils/gradient.ts
*/
import color from "tinycolor2";

function djb2(str: string) {
  let hash = 5381;
  for (let i = 0; i < str.length; i++) {
    hash = (hash << 5) + hash + str.charCodeAt(i);
  }
  return hash;
}

const avatarGradientCache: { [username: string]: ReturnType<typeof generateGradient> } = {};

export function generateGradient(username: string): { fromColor: string; toColor: string } {
  if (avatarGradientCache[username]) {
    return avatarGradientCache[username];
  }

  const first = color({ h: djb2(username) % 360, s: 0.95, l: 0.5 });
  const second = first.triad()[1];

  avatarGradientCache[username] = {
    fromColor: first.toHexString(),
    toColor: second.toHexString(),
  };

  return avatarGradientCache[username];
}
