export function debounce<A extends unknown[]>(
  fn: (...args: A) => void,
  ms: number | (() => number),
): {
  (...args: A): void;
  flush(): void;
  cancel(): void;
} {
  let timer: ReturnType<typeof setTimeout> | null = null;
  let lastArgs: A | null = null;
  // `ms` may be a getter so callers can pull the current setting at fire
  // time (e.g. user-tunable autosave debounce). The function form means
  // each scheduling reads the live value rather than a stale closure.
  const getMs = typeof ms === "function" ? ms : () => ms;

  const debounced = (...args: A) => {
    lastArgs = args;
    if (timer) clearTimeout(timer);
    timer = setTimeout(() => {
      timer = null;
      if (lastArgs) {
        const a = lastArgs;
        lastArgs = null;
        fn(...a);
      }
    }, getMs());
  };

  debounced.flush = () => {
    if (timer) {
      clearTimeout(timer);
      timer = null;
    }
    if (lastArgs) {
      const a = lastArgs;
      lastArgs = null;
      fn(...a);
    }
  };

  debounced.cancel = () => {
    if (timer) clearTimeout(timer);
    timer = null;
    lastArgs = null;
  };

  return debounced;
}
