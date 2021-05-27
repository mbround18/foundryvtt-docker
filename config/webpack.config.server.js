const path = require("path");
const webpack = require("webpack");
const nodeExternals = require("webpack-node-externals");
const HtmlWebpackPlugin = require('html-webpack-plugin')
NODE_ENV = process.env.NODE_ENV;

module.exports = {
  target: "node",
  entry: {
    server:   path.join(__dirname, "../src/server/main.js")
  },
  output: {
    path:  path.join(__dirname, "../dist"),
    filename: "[name]/[name].js",
  },
  node: {
    // Need this when working with express, otherwise the build fails
    __dirname: false, // if you don't put this is, __dirname
    __filename: false, // and __filename return blank or /
  },
  externals: [nodeExternals({
    allowlist: ['webpack/hot/dev-server', /^lodash/, /^axios/]
  })],
  module: {
    rules: [
      {
        test: /\.html$/i,
        loader: 'html-loader',
        options: {
          esModule: false,
        },
      },
      {
        // Transpiles ES6-8 into ES5
        test: /\.js$/,
        exclude: /node_modules/,
        use: {
          loader: "babel-loader",
          options: {
            presets: ["@babel/env"],
          },
        },
      },
    ],
  },
  plugins: [
  ],
  resolve: {
    extensions: [".ts", ".js", ".html"],
  },
  optimization: {
    // usedExports: true,
  },
  devServer: {
    contentBase:  path.join(__dirname, "../dist"),
    hot: true,
  },
};
