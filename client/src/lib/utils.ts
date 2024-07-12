export function cloneElement(element: HTMLElement) {
  const clone = element.cloneNode(true) as HTMLElement;
  clone.style.position = "absolute";

  const rect = element.getBoundingClientRect();
  clone.style.top = rect.top + "px";
  clone.style.left = rect.left + "px";
  clone.style.width = rect.width + "px";

  return clone;
}
