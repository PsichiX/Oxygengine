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
    filename: "[name].js"
  },
  devServer: {
    contentBase: dist,
  },
  plugins: [
    new CopyPlugin([
      path.resolve(__dirname, "static/index.html"),
      path.resolve(__dirname, "static/assets.pack"),
    ]),
    new WasmPackPlugin({
      crateDirectory: __dirname,
      extraArgs: "--out-name index",
      forceMode: DEBUG ? undefined : 'release',
    }),
  ]
};
