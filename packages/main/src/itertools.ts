export function* everySecond<T>(iter: Iterable<T>) {
  let flag = true;
  for (const item of iter) {
    if (flag) {
      yield item;
    }
    flag = !flag;
  }
}

export function* map<T, U>(iter: Iterable<T>, fn: (item: T) => U) {
  for (const item of iter) {
    yield fn(item);
  }
}
