export class Widget {
  private size: number;

  constructor(size: number) {
    this.size = size;
  }

  grow(amount: number): number {
    this.size += amount;
    return this.size;
  }

  shrink(amount: number): number {
    this.size -= amount;
    return this.size;
  }
}
