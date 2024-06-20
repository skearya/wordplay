import { useSearchParams } from "@solidjs/router";

export default function ChooseUsername() {
  const [searchParams] = useSearchParams();

  return (
    <main>
      <form action={`${import.meta.env.PUBLIC_SERVER}/auth/choose-username`} method="post">
        <input type="text" name="username" value={searchParams.suggested} />
        <button>submit</button>
      </form>
    </main>
  );
}
