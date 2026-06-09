import { useEffect, useRef, useCallback } from "react";

export function useChatScroll(deps: unknown[]) {
  const containerRef = useRef<HTMLDivElement>(null);
  const stickRef = useRef(true);

  const scrollToBottom = useCallback((smooth = false) => {
    const el = containerRef.current;
    if (!el) return;
    el.scrollTo({ top: el.scrollHeight, behavior: smooth ? "smooth" : "auto" });
  }, []);

  useEffect(() => {
    if (stickRef.current) scrollToBottom();
  }, deps);

  const onScroll = () => {
    const el = containerRef.current;
    if (!el) return;
    const dist = el.scrollHeight - el.scrollTop - el.clientHeight;
    stickRef.current = dist < 120;
  };

  const onFocus = () => {
    stickRef.current = true;
    scrollToBottom();
  };

  return { containerRef, scrollToBottom, onScroll, onFocus };
}
