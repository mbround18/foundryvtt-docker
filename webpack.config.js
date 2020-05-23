const path = require('path')
const webpack = require('webpack')
const nodeExternals = require('webpack-node-externals')
const HtmlWebPackPlugin = require("html-webpack-plugin")
NODE_ENV = process.env.NODE_ENV;
  
module.exports = {
    target: 'node',
    entry: {
        server: './src/uploader.js'
    },
    output: {
        path: path.join(__dirname, 'dist'),
        publicPath: '/',
        filename: '[name].js'
    },
    node: {
        // Need this when working with express, otherwise the build fails
        __dirname: false,   // if you don't put this is, __dirname
        __filename: false,  // and __filename return blank or /
    },
    externals: [nodeExternals()], 
    module: {
        rules: [
          {
            // Transpiles ES6-8 into ES5
            test: /\.js$/,
            exclude: /node_modules/,
            use: {
              loader: "babel-loader",
              options: {
                presets: ['@babel/env']
              }
            }
          },
          {
            test: /\.(jpe?g|png|ttf|eot|svg|woff(2)?)(\?[a-z0-9=&.]+)?$/,
            use: 'base64-inline-loader?limit=1000&name=[name].[ext]'
          },
          {
            // Loads the javacript into html template provided.
            // Entry point is set below in HtmlWebPackPlugin in Plugins 
            test: /\.html$/,
            use: [{loader: "html-loader"}]
          }
        ]
      },
    plugins: [
        new HtmlWebPackPlugin({
            template: "./public/index.html",
            filename: "./index.html",
            excludeChunks: [ 'server' ]
        })
    ],
    resolve: {
      extensions: ['.ts', '.js'],
    },
    optimization: {
        usedExports: true
    },
    devServer: {
        contentBase: './dist',
        hot: true
    }
}