import { useState } from "react";

export default function App() {
	const [count, setCount] = useState(0);

	return (
		<div>
			<button
				className="bg-white opacity-25 backdrop-blur-2xl hover:bg-red-50"
				onClick={() => setCount((count) => count + 1)}
			>
				count is {count}
			</button>
		</div>
	);
}
