const TEXT_UPDATE_MS = 25;

export function animateText(el: HTMLElement, text: string) {
	let erasing = true;
	let timeoutId: number;

	const update = () => {
		if (erasing) {
			el.textContent = el.textContent.substring(0, el.textContent.length - 1);
			if (el.textContent === '') erasing = false;

			timeoutId = setTimeout(update, TEXT_UPDATE_MS);
		} else {
			el.textContent = text.substring(0, el.textContent.length + 1);
			if (el.textContent === text) return;

			timeoutId = setTimeout(update, TEXT_UPDATE_MS + el.textContent.length);
		}
	};

	update();

	return () => clearTimeout(timeoutId);
}
