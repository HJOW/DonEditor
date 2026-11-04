// CommonJS 모듈 (.cjs)
const path = require("node:path");

function join(...parts) {
  return path.join(...parts);
}

module.exports = { join, name: "Don Editor" };
