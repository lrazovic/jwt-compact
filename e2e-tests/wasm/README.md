# Testing `jwt-compact` Crate in WASM

This simple crate tests that `jwt-compact` builds and can be used in WASM.

Note that `chrono` and `getrandom` crates need to be configured in [`Cargo.toml`](Cargo.toml)
in order to work with the WASM target:

```toml
[dependencies]
chrono = { version = "0.4.22", features = ["wasmbind"] }
getrandom = { version = "0.2", features = ["js"] }
```

## Usage

1. Install WASM target for Rust via `rustup`: `rustup target add wasm32-unknown-unknown`.
2. Install [`wasm-pack`](https://rustwasm.github.io/wasm-pack/installer/).
3. Install [Node](https://nodejs.org/).
4. Switch to the directory with this README and run `wasm-pack build --target web`.
5. Run the testing script: `node test.js`. 

:::note

If you are using `wasm-pack build --target web` the next step should NOT be needed. If you have to use ``wasm-pack build --target bundler`` then you will need to patch the JavaScript that it emits. This is because when you import a `wasm` file in Workers, you get a `WebAssembly.Module` instead of a `WebAssembly.Instance` for performance and security reasons.

:::

:::note

If you are using `wasm-bindgen` without `workers-rs` / `worker-build`, then you will need to patch the JavaScript that it emits. This is because when you import a `wasm` file in Workers, you get a `WebAssembly.Module` instead of a `WebAssembly.Instance` for performance and security reasons.

To patch the JavaScript that `wasm-bindgen` emits:

1. Run `wasm-pack build --target bundler` as you normally would.
2. Patch the JavaScript file that it produces (the following code block assumes the file is called `mywasmlib.js`):

```js
import * as imports from "./mywasmlib_bg.js";

// switch between both syntax for node and for workerd
import wkmod from "./mywasmlib_bg.wasm";
import * as nodemod from "./mywasmlib_bg.wasm";
if (typeof process !== "undefined" && process.release.name === "node") {
	imports.__wbg_set_wasm(nodemod);
} else {
	const instance = new WebAssembly.Instance(wkmod, {
		"./mywasmlib_bg.js": imports,
	});
	imports.__wbg_set_wasm(instance.exports);
}

export * from "./mywasmlib_bg.js";
```

3. In your Worker entrypoint, import the function and use it directly:

```js
import { myFunction } from "path/to/mylib.js";
```

:::

src: https://github.com/cloudflare/cloudflare-docs/blob/95aacf2eee782e3a89525b0f0855f02959367536/src/content/docs/workers/languages/rust/index.mdx?plain=1#L154