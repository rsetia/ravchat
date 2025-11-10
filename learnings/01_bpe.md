# Byte Pair Encoding (BPE) - Core Concepts

## Core BPE Algorithm

**Byte Pair Encoding fundamentals:**
- BPE builds vocabulary incrementally by merging the most frequent pair of adjacent tokens
- Starts with 256 base tokens (raw bytes 0-255)
- Iteratively finds most common pair, assigns it a new token ID, and applies merge
- Continues until reaching desired vocabulary size

**Token IDs vs Bytes:**
- Token IDs are `u32` (can represent any token in vocabulary)
- Bytes are `u8` (raw input data)
- Higher token IDs represent merged sequences of bytes
- Token 256 = first merge, 257 = second merge, etc.

## Training Process

**Regex-based chunking:**
- Text split into chunks using GPT-4 pattern regex before processing
- Pattern handles: contractions, letters, numbers, whitespace, special chars
- Prevents merges across chunk boundaries (important for linguistic coherence)
- Each chunk processed independently

**Merge algorithm:**
1. Convert chunks to byte-level tokens (bytes → u32)
2. Count all adjacent pairs across all chunks
3. Find most frequent pair
4. Assign new token ID to that pair
5. Apply merge across all chunks simultaneously
6. Repeat until vocab_size reached

**Key insight:** Merges are applied to all chunks before counting next iteration's pairs

## Encoding Process

**Applying learned merges:**
- Split input text into regex chunks
- Start with byte-level tokens for each chunk
- For each chunk, iteratively find and apply lowest-numbered valid merge
- Continue until no valid merges remain in chunk
- Lowest token ID has priority (deterministic tie-breaking)

**Loop-until-done pattern:**
```rust
loop {
    if let Some((pair, token_id)) = find_lowest_merge() {
        apply_merge(pair, token_id);
    } else {
        break;
    }
}
```

## Implementation Details

**Data structures:**
- `merges: HashMap<Pair, TokenId>` - learned merge rules
- `Pair = (TokenId, TokenId)` - adjacent token pair
- Chunks stored as `Vec<Vec<TokenId>>`

**Rust patterns learned:**
- `&mut self` for methods that modify tokenizer state (train)
- `&self` for read-only methods (encode, get_pattern)
- Slice types `&[TokenId]` more idiomatic than `&Vec<TokenId>`
- Separate public API (#[pymethods]) from internal helpers

## Key Gotchas

**Off-by-one errors:**
- Starting token ID must be 256 (not 255)
- Increment timing matters: use current_token, then increment

**Merge order determinism:**
- Tie-breaking in max_by_key can differ between implementations
- Both are "correct" if vocabulary size matches
- Explains why mybpe/rustbpe encodings differ but both work

**No cross-chunk merges:**
- Regex chunking prevents merges across boundaries
- Critical for maintaining linguistic structure
- "hello world" split into separate chunks, never merge across space

## API Design

**Matching tiktoken format:**
- `get_mergeable_ranks()` returns `Vec<(Vec<u8>, u32)>`
- Format: (byte_sequence, token_id)
- First 256 entries are single bytes
- Remaining entries reconstruct merged sequences from components

**Iterator pattern:**
- `train_from_iterator()` allows streaming large datasets
- Simple wrapper that concatenates all text, calls `train()`
- More complex version could process incrementally

## Summary

The hands-on implementation gave deep understanding of how modern tokenizers work - the same algorithm used in GPT models! The key is understanding that BPE is a greedy, iterative compression algorithm that learns frequent byte patterns from training data and builds a vocabulary by progressively merging the most common adjacent pairs.
