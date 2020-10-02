// A dependency graph that contains any wasm must all be imported
// asynchronously. This `bootstrap.js` file does the single async import, so
// that no one else needs to worry about it again.
import("./index.js")
  .catch(e => {
      let p = document.createElement('p');
      p.style.color = 'red';
      p.style.whiteSpace = 'pre';
      if ('$_GAME_ERROR' in window) {
          p.innerText = 'Error:\n' + window.$_GAME_ERROR;
      } else {
          p.innerText = 'Error:\n' + e.toString();
      }
      document.body.textContent = '';
      document.body.appendChild(p);
  });
