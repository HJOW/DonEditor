// ES 모듈 (.mjs)
import { readFile } from "node:fs/promises";

export const load = async (path) => {
  const text = await readFile(path, "utf8");
  return text.split("\n").length;
};

export default load;
