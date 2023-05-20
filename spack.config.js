const { config } = require("@swc/core/spack");

module.exports = config({
  entry: {
    session: __dirname + "/templates/session.ts",
  },
  output: {
    path: __dirname + "/assets/js/",
  },
  module: {},
});
