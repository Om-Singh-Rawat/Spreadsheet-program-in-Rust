import init from './client.js';

init().then(() => {
  console.log("WASM loaded and Yew app mounted");
});