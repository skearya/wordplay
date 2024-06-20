export function Errored(props: { errorMessage: string | null }) {
  return (
    <main class="flex min-h-screen flex-col items-center justify-center">
      <h1>we errored</h1>
      {props.errorMessage && <h1>{props.errorMessage}</h1>}
    </main>
  );
}
