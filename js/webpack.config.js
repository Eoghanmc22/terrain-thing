const webpack = require('webpack')
const path = require('path')
const CopyPlugin = require('copy-webpack-plugin')

const config = {
  mode: 'production',
  entry: path.resolve(__dirname, './bootstrap.js'),
  output: {
    path: path.resolve(__dirname, './public'),
    filename: './bootstrap.js'
  },
  resolve: {
    fallback: {
      zlib: require.resolve('browserify-zlib'),
      stream: require.resolve('stream-browserify'),
      buffer: require.resolve('buffer/'),
      events: require.resolve('events/'),
      assert: require.resolve('assert/')
    }
  },
  plugins: [
    // fix "process is not defined" error:
    new webpack.ProvidePlugin({
      process: 'process/browser'
    }),
    new webpack.ProvidePlugin({
      Buffer: ['buffer', 'Buffer']
    }),
    new webpack.NormalModuleReplacementPlugin(
      /prismarine-viewer[/|\\]viewer[/|\\]lib[/|\\]utils/,
      './utils.web.js'
    ),
    new CopyPlugin({
      patterns: [
        { from: './public/blocksStates/', to: './blocksStates/' },
        { from: './public/textures/', to: './textures/' },
        { from: './public/worker.js', to: './' },
        { from: './public/supportedVersions.json', to: './' }
      ]
    })
  ],
  devServer: {
    contentBase: path.resolve(__dirname, './public'),
    compress: true,
    inline: true,
    // open: true,
    hot: true,
    watchOptions: {
      ignored: /node_modules/
    }
  },
  experiments: {
    asyncWebAssembly: true
  },
  optimization: {
    minimize: false,
  }
}

const workerConfig = {
  entry: './lib/worker.js',
  mode: 'production',
  output: {
    path: path.join(__dirname, '/public'),
    filename: './worker.js'
  },
  resolve: {
    fallback: {
      zlib: false
    }
  },
  plugins: [
    // fix "process is not defined" error:
    new webpack.ProvidePlugin({
      process: 'process/browser'
    }),
    new webpack.ProvidePlugin({
      Buffer: ['buffer', 'Buffer']
    })
  ],
  optimization: {
    minimize: false,
  }
}

module.exports = [config, workerConfig]
