const path = require("path");
const CopyPlugin = require("copy-webpack-plugin");
const WasmPackPlugin = require("@wasm-tool/wasm-pack-plugin");

const dist = path.resolve(__dirname, "dist");
const DEBUG = !process.env.OXY_RELEASE;
console.log('BUILD MODE: ' + (DEBUG ? 'DEBUG' : 'RELEASE'));

module.exports = {
  mode: DEBUG ? 'development' : 'production',
  entry: {
    index: "./js/index.js"
  },
  output: {
    path: dist,
    filename: "oxygengine.js"
  },
  devServer: {
    contentBase: dist,
  },
  plugins: [
    new WasmPackPlugin({
      crateDirectory: __dirname,
      extraArgs: "--out-name oxygengine",
      forceMode: DEBUG ? undefined : 'release',
    }),
  ]
};
