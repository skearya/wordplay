import { JoinGame } from "@game/Game";
import { useParams } from "@solidjs/router";

export default function Game() {
  const params = useParams();

  return <JoinGame room={params.id} />;
}
