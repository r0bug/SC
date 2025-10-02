

export const index = 10;
let component_cache;
export const component = async () => component_cache ??= (await import('../entries/pages/dashboard/_page.svelte.js')).default;
export const imports = ["_app/immutable/nodes/10.BYG2eTQg.js","_app/immutable/chunks/D_6ItM0N.js","_app/immutable/chunks/DoEj3q9v.js","_app/immutable/chunks/IHki7fMi.js","_app/immutable/chunks/BZp5haeM.js"];
export const stylesheets = ["_app/immutable/assets/10.88yHHFMu.css"];
export const fonts = [];
