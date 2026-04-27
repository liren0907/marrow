// Mock for `@tauri-apps/api/event` `listen()`.
//
// The only event Marrow consumes is `fs-event`, which the real backend
// fires when the OS-level file watcher sees a change. There's no watcher
// in the browser, so we just register a no-op subscription and return an
// unsubscribe function. Phase 2 will add a DevPanel + an emit() helper so
// developers can manually trigger fake events to test reload / conflict /
// unresolved-link refresh flows.

type Listener<T> = (event: { payload: T }) => void;

const listeners = new Map<string, Set<Listener<unknown>>>();

export async function listen<T>(
  channel: string,
  cb: Listener<T>,
): Promise<() => void> {
  const set = listeners.get(channel) ?? new Set<Listener<unknown>>();
  set.add(cb as Listener<unknown>);
  listeners.set(channel, set);
  return () => {
    set.delete(cb as Listener<unknown>);
  };
}

/** Internal helper for the future DevPanel — fan out a payload to every
 *  registered listener on a channel. Not used yet. */
export function emitMock<T>(channel: string, payload: T): void {
  const set = listeners.get(channel);
  if (!set) return;
  for (const cb of set) cb({ payload });
}
