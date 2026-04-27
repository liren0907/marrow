# Quick Start

A whirlwind tour of what the editor can render and how to interact with it.

## Headings

Levels 1-6, all show up in the outline panel on the right (when the pane is
wide enough).

### Level 3

#### Level 4

##### Level 5

###### Level 6

## Inline formatting

This paragraph has **bold**, *italic*, ***both***, ~~strikethrough~~,
`inline code`, and a [link to the Marrow repo](https://github.com).

## Lists

- Unordered item one
- Unordered item two
  - Nested item
  - Another nested
- Back to top level

1. Ordered first
2. Ordered second
3. Ordered third

- [ ] Unchecked task
- [x] Checked task
- [ ] Another todo

## Blockquote

> "Local-first means your files stay yours." — every Marrow user, probably

> Multi-paragraph quotes work too.
>
> Just leave a blank `>` line between them.

## Code block

```typescript
function greet(name: string): string {
  return `Hello, ${name}!`;
}

const msg = greet("Marrow");
console.log(msg);
```

```python
def fib(n: int) -> int:
    if n < 2:
        return n
    return fib(n - 1) + fib(n - 2)

for i in range(10):
    print(fib(i))
```

## Table

| Feature | Status | Notes |
|---------|--------|-------|
| Pretty mode | ✅ | Milkdown WYSIWYG |
| Raw mode | ✅ | CodeMirror, `⌘/` to toggle |
| Wiki links | ✅ | Type `[[` to autocomplete |
| Transclusion | ✅ | Type `![[` to embed |
| Image paste | ⚠️ | Browser mock: degraded |

## Math

Inline math: $E = mc^2$

Block math:

$$
\int_0^\infty e^{-x^2} dx = \frac{\sqrt{\pi}}{2}
$$

## Horizontal rule

---

That's it — go explore!
