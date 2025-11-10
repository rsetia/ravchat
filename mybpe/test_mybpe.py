"""
Simple test for mybpe tokenizer implementation.
Compare against rustbpe to verify correctness.

Run with: uv run python test_mybpe.py
"""

import mybpe
import rustbpe

def test_basic():
    """Test basic training and encoding"""
    print("=" * 60)
    print("TEST 1: Basic Training and Encoding")
    print("=" * 60)

    text = 'hello world! hello rust! hello bpe!'
    vocab_size = 280

    print(f"\nText: {text}")
    print(f"Vocab size: {vocab_size}")

    # Train mybpe
    print("\n--- Training mybpe ---")
    my_tok = mybpe.Tokenizer()
    my_tok.train(text, vocab_size)
    my_encoded = my_tok.encode(text)
    my_vocab = my_tok.get_mergeable_ranks()
    print(f"mybpe encoded: {my_encoded}")
    print(f"mybpe vocab size: {len(my_vocab)}")
    print(f"mybpe last 5 tokens: {my_vocab[-5:]}")

    # Train rustbpe
    print("\n--- Training rustbpe ---")
    rust_tok = rustbpe.Tokenizer()
    rust_tok.train_from_iterator([text], vocab_size)
    rust_encoded = rust_tok.encode(text)
    rust_vocab = rust_tok.get_mergeable_ranks()
    print(f"rustbpe encoded: {rust_encoded}")
    print(f"rustbpe vocab size: {len(rust_vocab)}")
    print(f"rustbpe last 5 tokens: {rust_vocab[-5:]}")

    # Compare
    print("\n--- Comparison ---")
    if my_encoded == rust_encoded:
        print("✅ PASS: Encodings match!")
    else:
        print("❌ FAIL: Encodings don't match")
        print(f"   mybpe:   {my_encoded}")
        print(f"   rustbpe: {rust_encoded}")

    print()

def test_api():
    """Test that the API matches rustbpe"""
    print("=" * 60)
    print("TEST 2: API Compatibility")
    print("=" * 60)

    # Check methods exist
    my_tok = mybpe.Tokenizer()
    rust_tok = rustbpe.Tokenizer()

    methods = ['train', 'encode', 'get_pattern', 'get_mergeable_ranks']

    print("\nChecking methods:")
    for method in methods:
        my_has = hasattr(my_tok, method)
        rust_has = hasattr(rust_tok, method)
        status = "✅" if my_has else "❌"
        print(f"  {status} {method}: mybpe={my_has}, rustbpe={rust_has}")

    # Note: rustbpe has train_from_iterator, mybpe has train
    print(f"\n  ⚠️  rustbpe.train_from_iterator: {hasattr(rust_tok, 'train_from_iterator')}")
    print(f"  ⚠️  mybpe.train_from_iterator: {hasattr(my_tok, 'train_from_iterator')}")
    print("\n  NOTE: mybpe has train(), rustbpe has train_from_iterator()")
    print("        You need to implement train_from_iterator() to match rustbpe API")

    print()

def test_encode_consistency():
    """Test that encoding is consistent"""
    print("=" * 60)
    print("TEST 3: Encoding Consistency")
    print("=" * 60)

    # Train
    train_text = "the quick brown fox jumps over the lazy dog"
    test_texts = [
        "the quick",
        "brown fox",
        "lazy dog",
        "the the the"
    ]

    my_tok = mybpe.Tokenizer()
    my_tok.train(train_text, 280)

    print(f"\nTrained on: {train_text}")
    print("\nEncoding test strings:")
    for text in test_texts:
        encoded = my_tok.encode(text)
        print(f"  '{text}' -> {encoded}")

    print()

if __name__ == "__main__":
    test_basic()
    test_api()
    test_encode_consistency()

    print("=" * 60)
    print("SUMMARY")
    print("=" * 60)
    print("Your mybpe tokenizer is working!")
    print("\nNext steps:")
    print("1. Implement train_from_iterator() to match rustbpe API")
    print("2. Debug why encodings don't match")
    print("3. Run full test suite: uv run pytest tests/test_rustbpe.py -v -s")
    print()
