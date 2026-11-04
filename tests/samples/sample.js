// 한 줄 주석
import fs from "node:fs";

/** JSDoc 주석 */
export class Counter {
  #count = 0;

  increment(step = 1) {
    this.#count += step;
    return this.#count;
  }
}

const greet = async (name) => {
  const text = `안녕하세요, ${name}!`;
  await new Promise((resolve) => setTimeout(resolve, 10));
  return text;
};

const numbers = [1, 2, 3.5, 0xff];
const total = numbers.filter((n) => n > 1).reduce((a, b) => a + b, 0);
if (total > 10 && true) {
  console.log(greet("Don"), /ab+c/gi.test("abbc"), fs);
}
