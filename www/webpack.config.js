const CopyWebpackPlugin = require('copy-webpack-plugin');
const path = require('path');

module.exports = {
  entry: './src/bootstrap.js',
  output: {
    path: path.resolve(__dirname, 'dist'),
    filename: 'bootstrap.js',
  },
  mode: 'production',
  plugins: [
    new CopyWebpackPlugin({
      patterns: [
        './src/index.html',
        './src/style.css',
      ],
    }),
  ],
  module: {
    rules: [
      {
        test: /\.svg/,
        type: 'asset/resource',
      },
    ],
  },
  experiments: {
    syncWebAssembly: true,
  },
};
