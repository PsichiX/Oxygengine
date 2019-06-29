const path = require("path");
const HtmlWebpackPlugin = require("html-webpack-plugin");
const WasmPackPlugin = require("@wasm-tool/wasm-pack-plugin");
const CopyPlugin = require('copy-webpack-plugin');

const dist = path.resolve(__dirname, "dist");
const DEBUG = true;
console.log('BUILD MODE: ' + (DEBUG ? 'DEBUG' : 'RELEASE'));

module.exports = {
  mode: DEBUG ? 'development' : 'production',
  entry: "./js/index.js",
  output: {
    path: dist,
    filename: "bundle.js"
  },
  devServer: {
    contentBase: dist,
  },
  plugins: [
    new HtmlWebpackPlugin({
      template: 'index.html',
    }),
    new CopyPlugin([
      { from: 'static' },
    ]),
    new WasmPackPlugin({
      crateDirectory: path.resolve(__dirname, "crate"),
      forceMode: DEBUG ? undefined : 'release',
    }),
  ]
};
