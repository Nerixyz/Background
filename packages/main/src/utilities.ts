export function stringifyTemp(tempInK: number): string {
  return stringifyNum(tempInK - 273.15);
}

export function stringifyNum(num: number): string {
  return Math.abs(num) > 10 ? Math.round(num).toString() : (Math.round(num * 10) / 10).toString();
}
