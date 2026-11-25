// Runs before the page draws: puts the colour theme and style last shown on <html>, so that the first
// picture is already right. look.js checks them against config.json once it runs.
try {
  var saved = JSON.parse(localStorage.getItem('maillet.appearance') || 'null');
  if (saved) for (var name in saved) if (name.indexOf('data-sx-') === 0) document.documentElement.setAttribute(name, saved[name]);
} catch (e) {}
