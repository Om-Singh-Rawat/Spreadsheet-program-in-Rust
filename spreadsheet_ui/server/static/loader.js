import init, { WasmSheet } from './client.js';
 
let sheet;
 
 async function run() {
     await init();
     
     // Create spreadsheet instance (adjust rows/cols as needed)
     sheet = new WasmSheet(20, 10);
     
     // Expose to browser console for debugging
     window.sheet = sheet;
     
     if (window.sheet instanceof WasmSheet) {
         console.log("WasmSheet instance is ready!");
     } else {
         console.error("WasmSheet instance NOT ready!");
     }
 }
 
 run().catch(console.error);
 export default run;