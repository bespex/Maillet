// Colour theme and style of every Maillet window, from the be spex kit (dist/kit, see
// tools/update-kit.sh): the same choice as in every be spex app. Stored as `appearance` in config.json,
// chosen in the settings (General); every window follows a change at once (`settings-changed`).

import { apply, normalize } from './kit/index.js';

const tauri = window.__TAURI__;
const invoke = tauri && tauri.core ? tauri.core.invoke : null;
const listen = tauri && tauri.event ? tauri.event.listen : null;

/** Before the kit there was light or dark, kept per window in localStorage ("theme"). */
const LEGACY = { light: { theme: 'paper', style: 'calm' }, dark: { theme: 'graphite', style: 'calm' } };
export const DEFAULT_APPEARANCE = { theme: 'auto', style: 'calm' };

/** The style axes that change something in Maillet; the picker offers only these. */
export const AXES_USED = ['shape', 'structure', 'chrome'];

/** Puts a choice on the page and remembers it for the next window's first picture. */
export function show(value) {
  const root = document.documentElement;
  apply(normalize(value), root);
  const attrs = {};
  for (const a of root.attributes) if (a.name.startsWith('data-sx-')) attrs[a.name] = a.value;
  try { localStorage.setItem('maillet.appearance', JSON.stringify(attrs)); } catch (e) { /* private mode */ }
}

/** The stored choice; a light/dark choice from before the kit is taken over once. */
export async function load() {
  let stored = null;
  try { stored = invoke ? await invoke('get_appearance') : null; } catch (e) { /* not in the app */ }
  if (!stored) {
    const legacy = LEGACY[localStorage.getItem('theme')];
    if (legacy && invoke) {
      try { await invoke('set_appearance', { appearance: legacy }); localStorage.removeItem('theme'); } catch (e) { /* keep it for next time */ }
    }
    stored = legacy || DEFAULT_APPEARANCE;
  }
  return normalize(stored);
}

const follow = async () => show(await load());
follow();
if (listen) listen('settings-changed', follow);
