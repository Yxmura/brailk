// WASM VWC→LaTeX converter bridge
// Exports: allocate, deallocate, convert, free_string, memory

let wasm = null;

async function initWasmConverter(wasmUrl) {
  const response = await fetch(wasmUrl);
  const bytes = await response.arrayBuffer();
  const result = await WebAssembly.instantiate(bytes, {});
  wasm = result.instance.exports;
  return wasm;
}

function callVwcConverter(input, display) {
  if (!wasm) throw new Error("WASM not initialized");

  const encoder = new TextEncoder();
  const inputBytes = encoder.encode(input);
  const inputLen = inputBytes.length;

  // Allocate WASM memory for input string
  const inputPtr = wasm.allocate(inputLen + 1);

  // Write input bytes + null terminator
  const mem = new Uint8Array(wasm.memory.buffer);
  mem.set(inputBytes, inputPtr);
  mem[inputPtr + inputLen] = 0;

  // Call the converter
  const resultPtr = wasm.convert(inputPtr, inputLen, display);

  // Read null-terminated result string
  const resultBytes = [];
  let off = resultPtr;
  while (mem[off] !== 0) {
    resultBytes.push(mem[off]);
    off++;
  }
  const resultJson = new TextDecoder().decode(new Uint8Array(resultBytes));

  // Free WASM allocations
  wasm.free_string(resultPtr);
  wasm.deallocate(inputPtr, inputLen + 1);

  return JSON.parse(resultJson);
}
