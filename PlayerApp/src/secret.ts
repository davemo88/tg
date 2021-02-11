
const REDACTED = '[redacted]';

export class Secret<T> {
    private inner_secret: T;

    constructor(inner_secret: T) {
        this.inner_secret = inner_secret;
    }

    public expose_secret() : T {
        return this.inner_secret;
    }

    public toJSON(): string {
      return REDACTED;
    }

    public valueOf(): string {
      return REDACTED;
    }

//  This is the method used by `console.log` on objects
    [Symbol.for('nodejs.util.inspect.custom')]() {
      return REDACTED;
    }

    public toLocaleString(): string {
      return REDACTED;
    }

    public toString(): string {
      return REDACTED;
    }
}
