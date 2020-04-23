window.addEventListener('message', event => {
  const { id } = event.data;
  if (id === 'screenshot') {
    const canvas = document.getElementById('screen');
    if (!!canvas && !!canvas.toDataURL) {
      const data = canvas.toDataURL();
      if (!!data) {
        event.source.postMessage({
          id: 'screenshot',
          data,
          preview: event.data.preview
        }, '*');
      }
    }
  }
});

import("../pkg/index.js").catch(console.error);
