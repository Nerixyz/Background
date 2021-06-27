export function lastNotUndefined<T>(data: Array<T>, selector: (el: T) => number | null) {
  for (let i = data.length - 1; i >= 0; i--) {
    const v = selector(data[i]);
    if(typeof v === 'number') return v;
  }
  return 0;
}

export function toPrettyString(value: number): string {
  return (Math.round(value * 10) / 10).toString();
}
