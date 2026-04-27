# Code Samples

Quick visual check that Prism syntax highlighting works for the languages
the editor configures.

## Rust

```rust
fn fibonacci(n: u32) -> u64 {
    let (mut a, mut b) = (0u64, 1u64);
    for _ in 0..n {
        let next = a + b;
        a = b;
        b = next;
    }
    a
}

fn main() {
    for i in 0..10 {
        println!("fib({}) = {}", i, fibonacci(i));
    }
}
```

## JSON

```json
{
  "name": "marrow",
  "version": "0.0.1",
  "features": {
    "browserMock": true,
    "wikiLinks": true,
    "transclusion": true
  }
}
```

## Bash

```bash
yarn install
yarn dev               # browser mock — open http://localhost:1620/
yarn tauri dev         # full desktop app
```

## TSX

```tsx
import { useState } from "react";

export function Counter() {
  const [n, setN] = useState(0);
  return <button onClick={() => setN(n + 1)}>Clicked {n} times</button>;
}
```
