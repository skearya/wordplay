import { useNavigate, useSearchParams } from "@solidjs/router";
import { createSignal, Show } from "solid-js";
import { Button } from "~/lib/components/ui/Button";
import { Input } from "~/lib/components/ui/Input";
import { url } from "~/lib/utils";

type ChooseUsernameResponse = {
  type: "success" | "error";
  message: string;
};

export default function ChooseUsername() {
  const navigate = useNavigate();
  const [searchParams] = useSearchParams();

  const [username, setUsername] = createSignal((searchParams.suggested ?? "").substring(0, 12));
  const [loading, setLoading] = createSignal(false);
  const [errorMessage, setErrorMessage] = createSignal("");

  async function createAccount() {
    setLoading(true);

    const res = await fetch(url("/api/auth/choose-username"), {
      method: "post",
      credentials: "include",
      body: new URLSearchParams({
        username: username(),
      }),
    });
    const { type, message }: ChooseUsernameResponse = await res.json();

    if (type === "success") {
      alert(message);
      navigate("/");
      return;
    } else {
      setErrorMessage(message);
    }

    setLoading(false);
  }

  return (
    <main class="flex h-screen items-center justify-center gap-x-8">
      <div class="flex min-w-64 flex-col gap-y-2.5">
        <Input
          minlength="3"
          maxlength="12"
          placeholder="username"
          disabled={loading()}
          value={username()}
          autofocus
          onInput={(event) => setUsername(event.target.value)}
          onEnter={createAccount}
        />
        <Button size="lg" disabled={loading()} onClick={createAccount}>
          Create Account
        </Button>
      </div>
      <Show when={errorMessage()}>
        <h1 class="text-center text-red-400">{errorMessage()}</h1>
      </Show>
    </main>
  );
}
