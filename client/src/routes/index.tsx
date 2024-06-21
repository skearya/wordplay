import { Match, Switch, createResource } from "solid-js";

type HomepageData = {
  clientsConnected: number;
  publicRooms: Array<{ name: string; players: number }>;
};

export default function Home() {
  const [data] = createResource<HomepageData>(async () => {
    const res = await fetch(`${import.meta.env.PUBLIC_SERVER}/api/info`);
    return res.json();
  });

  return (
    <main>
      <h1>wordplay</h1>
      <Switch>
        <Match when={data.loading}>
          <h1>loading data</h1>
        </Match>
        <Match when={data.error}>
          <h1>something went wrong</h1>
        </Match>
        <Match when={data()}>
          <pre>{JSON.stringify(data(), null, 2)}</pre>
        </Match>
      </Switch>
    </main>
  );
}
