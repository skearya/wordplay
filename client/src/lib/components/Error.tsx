import { createSignal } from "solid-js";
import { Button } from "~/lib/components/ui/Button";
import { Copy } from "~/lib/components/ui/Copy";
import { ErrorType, GameError } from "~/lib/utils";

const errorTranslations: Record<ErrorType, string> = {
  "socket closed": "the server is likely offline",
};

export const [error, setError] = createSignal<unknown | undefined>(undefined);

export function ErrorDisplay({ error, reset }: { error: unknown; reset?: () => void }) {
  const message = error instanceof GameError ? errorTranslations[error.type] : undefined;

  return (
    <main class="flex h-screen items-center justify-center">
      <div class="flex flex-col gap-y-4 rounded-lg border bg-light-background p-4">
        <div class="flex items-center gap-x-3">
          <h1 class="text-lg">oh, we errored</h1>
          <div class="w-[1px] scale-y-90 self-stretch bg-dark-green/30"></div>
          <pre class="text-light-green">
            {JSON.stringify(
              error instanceof Error
                ? {
                    name: error.name,
                    ...(error instanceof GameError && { type: error.type }),
                    message: error.message,
                  }
                : error,
              null,
              1,
            )}
          </pre>
        </div>
        {message && <h1 class="text-center text-lightest-green">the server might be offline</h1>}
        <div class="flex gap-x-2.5">
          <Button size="lg" class="flex-1" onClick={reset ?? (() => setError(undefined))}>
            rejoin
          </Button>
          {error instanceof Error && error.stack && (
            <Copy color="muted" size="lg" class="flex-1" content={error.stack}>
              copy stack trace
            </Copy>
          )}
        </div>
      </div>
    </main>
  );
}
