// Polyfill for `ReadableStream.prototype[Symbol.asyncIterator]`.
// WebKit (WKWebView) shipped this only from Safari 17.4 and some bundled
// versions still lack it at runtime. pdfjs-dist 5.x uses
// `for await (const value of readableStream)` inside `getTextContent`,
// so without this shim the native PDF → Markdown path throws
// "undefined is not a function (near '...value of readableStream...')".

if (
  typeof ReadableStream !== "undefined" &&
  !(Symbol.asyncIterator in ReadableStream.prototype)
) {
  // @ts-expect-error augmenting built-in prototype
  ReadableStream.prototype[Symbol.asyncIterator] = function () {
    const reader = this.getReader();
    return {
      next: () => reader.read(),
      return: () => {
        reader.releaseLock();
        return Promise.resolve({ value: undefined, done: true });
      },
      [Symbol.asyncIterator]() {
        return this;
      },
    };
  };
}
