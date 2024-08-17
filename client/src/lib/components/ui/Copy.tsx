import { createSignal } from "solid-js";
import { Button } from "./Button";

type CopyProps = Omit<Parameters<typeof Button>[0], "children"> & {
  content: string;
  children: string;
};

export function Copy(props: CopyProps) {
  let copiedTimeout: ReturnType<typeof setTimeout> | undefined;
  const [text, setText] = createSignal(props.children);

  return (
    <Button
      onClick={() => {
        navigator.clipboard.writeText(props.content);
        setText("copied!");

        clearTimeout(copiedTimeout);
        copiedTimeout = setTimeout(() => setText(props.children), 1000);
      }}
      {...props}
    >
      {text()}
    </Button>
  );
}
