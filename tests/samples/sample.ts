// 한 줄 주석
interface User {
  id: number;
  name: string;
  tags?: string[];
}

type Result<T> = { ok: true; value: T } | { ok: false; error: string };

enum Level { Low, High }

export class Repository<T extends { id: number }> {
  private items: Map<number, T> = new Map();

  add(item: T): void {
    this.items.set(item.id, item);
  }

  find(id: number): T | undefined {
    return this.items.get(id);
  }
}

function parse(input: string): Result<User> {
  try {
    const user = JSON.parse(input) as User;
    return { ok: true, value: user };
  } catch (e) {
    return { ok: false, error: String(e) };
  }
}

const repo = new Repository<User>();
repo.add({ id: 1, name: "홍길동" });
console.log(parse('{"id":2,"name":"Don"}'), Level.High);
